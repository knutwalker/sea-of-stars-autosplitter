#![no_std]

use crate::data::{Data, GameStart, SpeedrunRelic};
use asr::{
    future::next_tick,
    settings::Gui,
    time::Duration,
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
    let mut settings = LiveSettings::new(settings);

    loop {
        let process = Process::wait_attach("SeaOfStars.exe").await;
        log!("attached to process");
        process
            .until_closes(async {
                let data = Data::new(&process).await;
                let mut progress = Progress::new();
                let mut game_start = GameStart::Unknown;

                loop {
                    settings.update();
                    match timer::state() {
                        TimerState::NotRunning => {
                            let (new_state, start) = progress.start(&data, game_start);
                            if new_state != game_start {
                                log!("Game state changed from {game_start:?} to {new_state:?}");
                                game_start = new_state;
                            }
                            act(start, &settings);
                        }
                        TimerState::Running => {
                            game_start = GameStart::Unknown;

                            match data.speedrun_time() {
                                Some(SpeedrunRelic::Inactive) => {
                                    settings.disable_speedrun_relic();
                                }
                                Some(SpeedrunRelic::Active(time)) => {
                                    settings.enable_speedrun_relic();
                                    act(Some(Action::SetGameTime(time)), &settings)
                                }
                                None => {}
                            }

                            let action = progress.act(&data);
                            act(action, &settings);
                        }
                        TimerState::Ended => {
                            log!("Timer ended");
                            game_start = GameStart::Unknown;
                        }
                        _ => {}
                    }
                    next_tick().await;
                }
            })
            .await;
    }
}

#[derive(Debug, Gui)]
pub struct Settings {
    /// Stop game timer during loads (load remover)
    #[default = true]
    remove_loads: bool,

    /// Start after selecting the character
    #[default = false]
    start: bool,

    /// Start after confirming the relics
    #[default = true]
    relic_start: bool,

    /// Split on finished boss encounters
    #[default = true]
    split: bool,

    /// Use the speedrun relic timer instead of the load remover
    #[default = false]
    speedrun_relic: bool,
}

struct LiveSettings {
    settings: Settings,
    speedrun_relic: bool,
}

#[derive(Debug)]
enum Action {
    StartCharacter,
    StartRelic,
    Split,
    Pause,
    Resume,
    SetGameTime(f64),
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

    pub fn start(&mut self, data: &Data<'_>, current: GameStart) -> (GameStart, Option<Action>) {
        let current = data.game_start(current);
        let action = match current {
            GameStart::CharSelected => Some(Action::StartCharacter),
            GameStart::JustStarted => Some(Action::StartRelic),
            _ => None,
        };
        (current, action)
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

impl LiveSettings {
    fn new(settings: Settings) -> Self {
        let speedrun_relic = settings.speedrun_relic;
        Self {
            settings,
            speedrun_relic,
        }
    }

    fn update(&mut self) {
        self.settings.update();
    }

    fn disable_speedrun_relic(&mut self) {
        self.speedrun_relic = false;
    }

    fn enable_speedrun_relic(&mut self) {
        self.speedrun_relic = self.settings.speedrun_relic;
    }

    fn filter(&self, action: &Action) -> bool {
        match action {
            Action::Pause | Action::Resume => self.settings.remove_loads && !self.speedrun_relic,
            Action::StartCharacter => self.settings.start,
            Action::StartRelic => self.settings.relic_start,
            Action::Split => self.settings.split,
            Action::SetGameTime(_) => self.speedrun_relic,
        }
    }
}

fn act(action: Option<Action>, settings: &LiveSettings) {
    if let Some(action) = action.filter(|o| settings.filter(o)) {
        log!("Decided on an action: {action:?}");
        match (action, timer::state() == TimerState::Running) {
            (Action::StartCharacter | Action::StartRelic, false) => {
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
            (Action::SetGameTime(time), true) => {
                let time = Duration::seconds_f64(time);
                timer::set_game_time(time);
            }

            (Action::StartCharacter | Action::StartRelic, true) => {}
            (Action::Split | Action::Pause | Action::Resume | Action::SetGameTime(_), false) => {}
        }
    }
}
