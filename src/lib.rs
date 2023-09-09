#[cfg(debugger)]
use crate::data::{Activity, Level, PlayTime};
use crate::data::{Data, GameStart};
use asr::{
    future::next_tick,
    timer::{self, TimerState},
    watcher::Watcher,
    Address64, Process,
};

#[cfg(any(debug_assertions, debugger))]
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        let mut buf = ::asr::arrayvec::ArrayString::<8192>::new();
        let _ = ::core::fmt::Write::write_fmt(
            &mut buf,
            ::core::format_args!($($arg)*),
        );
        ::asr::print_message(&buf);
    }};
}

#[cfg(not(any(debug_assertions, debugger)))]
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {};
}

#[macro_export]
macro_rules! dbg {
    // Copy of ::std::dbg! but for no_std with redirection to log!
    () => {
        $crate::log!("[{}:{}]", ::core::file!(), ::core::line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::log!("[{}:{}] {} = {:?}",
                    ::core::file!(), ::core::line!(), ::core::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

mod data;

asr::async_main!(stable);

async fn main() {
    asr::set_tick_rate(60.0);
    let settings = Settings::register();
    log!("Loaded settings: {settings:?}");

    loop {
        let process = Process::wait_attach("SeaOfStars.exe").await;
        log!("attached to process");
        process
            .until_closes(async {
                let mut data = Data::new(&process).await;
                let mut progress = Progress::new();

                loop {
                    #[cfg(debugger)]
                    for item in data.check_for_new_key_items().await {
                        log!("picked up a new key item: {item}");
                    }

                    match timer::state() {
                        TimerState::NotRunning => {
                            let start = progress.start(&mut data).await;
                            act(start, &settings);
                        }
                        TimerState::Running => {
                            let action = progress.act(&mut data).await;
                            act(action, &settings);
                        }
                        _ => {}
                    }
                    next_tick().await;
                }
            })
            .await;
    }
}

#[derive(Debug, asr::user_settings::Settings)]
pub struct Settings {
    /// Stop game timer during loads (load remover)
    #[default = true]
    remove_loads: bool,

    /// Start splitting on character select
    #[default = true]
    start: bool,

    /// Split on finished boss encounters
    #[default = true]
    split: bool,
}

#[derive(Debug)]
enum Action {
    Start,
    Split,
    Pause,
    Resume,
}

struct Progress {
    loading: Watcher<bool>,
    encounter: Option<Address64>,
    #[cfg(debugger)]
    play_time: Watcher<PlayTime>,
    #[cfg(debugger)]
    activity: Watcher<Activity>,
    #[cfg(debugger)]
    current_level: Watcher<Level>,
    #[cfg(debugger)]
    previous_level: Watcher<Level>,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            loading: Watcher::new(),
            encounter: None,
            #[cfg(debugger)]
            play_time: Watcher::new(),
            #[cfg(debugger)]
            activity: Watcher::new(),
            #[cfg(debugger)]
            current_level: Watcher::new(),
            #[cfg(debugger)]
            previous_level: Watcher::new(),
        }
    }

    pub async fn start(&mut self, data: &mut Data<'_>) -> Option<Action> {
        matches!(data.game_start().await, GameStart::JustStarted).then_some(Action::Start)
    }

    pub async fn act(&mut self, data: &mut Data<'_>) -> Option<Action> {
        let loading = if let Some(progress) = data.progress().await {
            #[cfg(debugger)]
            {
                let play_time = self.play_time.update_infallible(progress.play_time());
                if play_time.changed() {
                    log!(
                        "this session={}, total={}",
                        play_time.current.session,
                        play_time.current.total
                    );
                    timer::set_game_time(play_time.current.total);
                }

                let first = self.activity.pair.is_none();
                let activity = self.activity.update_infallible(progress.activity());
                if first || activity.changed() {
                    log!("{:?}", activity.current);
                    timer::set_variable("activity", &activity.current.name);
                    timer::set_variable("activity_id", activity.current.id.as_str());
                }

                let first = self.previous_level.pair.is_none();
                let level = self.previous_level.update_infallible(progress.prev_level());
                if first || level.changed() {
                    log!("Previous {:?}", level.current);
                }

                let first = self.current_level.pair.is_none();
                let level = self
                    .current_level
                    .update_infallible(progress.current_level());
                if first || level.changed() {
                    log!("Current {:?}", level.current);
                    timer::set_variable("level", &level.name);
                    timer::set_variable("level_id", level.id.as_str());
                }
            }

            Some(progress.is_loading)
        } else {
            None
        };

        match self.loading.update(loading) {
            Some(l) if l.changed_to(&false) => Some(Action::Resume),
            Some(l) if l.changed_to(&true) => Some(Action::Pause),
            _ => self
                .check_encounter(data)
                .await
                .and_then(|o| o.then_some(Action::Split)),
        }
    }

    async fn check_encounter(&mut self, data: &mut Data<'_>) -> Option<bool> {
        match self.encounter {
            Some(enc) => match data.encounter(Some(enc)).await {
                Some((_, enc)) if enc.done => {
                    #[cfg(debugger)]
                    data.dump_current_encounter().await;
                    self.encounter = None;
                    return Some(enc.boss);
                }
                Some(_) =>
                {
                    #[cfg(debugger)]
                    data.dump_current_hp_levels().await
                }
                None => {
                    self.encounter = None;
                }
            },
            None => {
                let (address, encounter) = data.encounter(None).await?;
                if !encounter.done {
                    #[cfg(debugger)]
                    data.dump_current_encounter().await;
                    self.encounter = Some(address);
                }
            }
        };
        Some(false)
    }
}

impl Settings {
    fn filter(&self, action: &Action) -> bool {
        match action {
            Action::Pause | Action::Resume => self.remove_loads,
            Action::Start => self.start,
            Action::Split => self.split,
        }
    }
}

fn act(action: Option<Action>, settings: &Settings) {
    if let Some(action) = action.filter(|o| settings.filter(o)) {
        log!("Decided on an action: {action:?}");
        match (action, timer::state() == TimerState::Running) {
            (Action::Start, false) => {
                log!("Starting timer");
                timer::start();
            }
            (Action::Split, true) => {
                log!("Splitting");
                timer::split();
            }
            (Action::Pause, true) => {
                log!("Pause game time");
                timer::pause_game_time();
            }
            (Action::Resume, true) => {
                log!("Resume game time");
                timer::resume_game_time();
            }
            _ => {}
        }
    }
}
