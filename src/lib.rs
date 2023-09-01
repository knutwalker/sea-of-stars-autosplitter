#![no_std]

use crate::data::{Data, GameState};
use asr::{
    future::next_tick,
    timer::{self, TimerState},
    watcher::Watcher,
    Address64, Process,
};

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        let mut buf = ::asr::arrayvec::ArrayString::<1024>::new();
        let _ = ::core::fmt::Write::write_fmt(
            &mut buf,
            ::core::format_args!($($arg)*),
        );
        ::asr::print_message(&buf);
    }};
}

#[cfg(not(debug_assertions))]
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
                $crate::log!("[{}:{}] {} = {:#?}",
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
asr::panic_handler!();

async fn main() {
    asr::set_tick_rate(60.0);
    let settings = Settings::register();
    log!("Loaded settings: {settings:?}");

    loop {
        let process = Process::wait_attach("SeaOfStars.exe").await;
        log!("attached to process");
        process
            .until_closes(async {
                let data = Data::new(&process).await;
                let mut progress = Progress::new();

                loop {
                    match timer::state() {
                        TimerState::NotRunning => {
                            let start = progress.start(&data);
                            act(start, &settings);
                        }
                        TimerState::Running => {
                            let action = progress.act(&data);
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
}

impl Progress {
    pub fn new() -> Self {
        Self {
            loading: Watcher::new(),
            encounter: None,
        }
    }

    pub fn start(&mut self, data: &Data<'_>) -> Option<Action> {
        matches!(data.game_start(), GameState::JustStarted).then_some(Action::Start)
    }

    pub fn act(&mut self, data: &Data<'_>) -> Option<Action> {
        match self.loading.update(data.is_loading()) {
            Some(l) if l.changed_to(&false) => Some(Action::Resume),
            Some(l) if l.changed_to(&true) => Some(Action::Pause),
            _ => self
                .check_encounter(data)
                .and_then(|o| o.then_some(Action::Split)),
        }
    }

    fn check_encounter(&mut self, data: &Data<'_>) -> Option<bool> {
        match self.encounter {
            Some(enc) => match data.resolve_encounter(enc) {
                Some(enc) if enc.done => {
                    self.encounter = None;
                    return Some(true);
                }
                Some(_) => {}
                None => {
                    self.encounter = None;
                }
            },
            None => {
                let (address, encounter) = data.encounter()?;
                if encounter.boss && !encounter.done {
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
