#![no_std]

use crate::{
    data::{Data, SpeedrunRelic, StartScreen},
    splits::{EventHandler, Running, Split},
};
use asr::{
    Process,
    future::next_tick,
    settings::Gui,
    timer::{self, TimerState},
    watcher::Watcher,
};

#[macro_export]
macro_rules! log {
    ($format:expr$(, $($arg:tt)*)?) => {{
        let mut buf = ::asr::arrayvec::ArrayString::<1024>::new();
        let _ = ::core::fmt::Write::write_fmt(
            &mut buf,
            ::core::format_args!(concat!("[SoS]: ", $format) $(, $($arg)*)?),
        );
        ::asr::print_message(&buf);
    }};
}

#[cfg(debug_assertions)]
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
                $crate::log!(
                    "[{}:{}] {} = {:#?}",
                    ::core::file!(),
                    ::core::line!(),
                    ::core::stringify!($val),
                    &tmp
                );
                tmp
            }
        }
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! dbg {
    () => {{}};
    ($val:expr $(,)?) => {
        $val
    };
}

mod data;
mod mapping;
mod memory;
mod splits;
mod utils;

// const TICK_RATE: f64 = 2.0;
const TICK_RATE: f64 = 60.0;

#[derive(Debug, Gui)]
pub struct Settings {
    /// Report 'Game Time' according to the Speedrun Relic, also acts as Load Remover
    #[default = true]
    remove_loads: bool,

    /// Start after confirming the relics
    #[default = true]
    relic_start: bool,

    /// Split on finished boss encounters
    #[default = true]
    split: bool,

    /// LEGACY: Start after selecting the character
    #[default = false]
    start: bool,

    /// LEGACY: Use a custom load remover, not synced to the Speedrun Relic
    #[default = false]
    custom_load_remover: bool,
}

#[derive(Debug)]
enum Action {
    StartCharacter,
    StartRelic,
    SplitBoss,
    Split(Split),
    Pause,
    Resume,
}

struct NotRunning {
    start_screen: StartScreen,
    initial_relic_time: Watcher<f64>,
    start_at: f64,
}

impl Default for NotRunning {
    fn default() -> Self {
        Self {
            start_screen: StartScreen::Undecided,
            initial_relic_time: Watcher::new(),
            start_at: 0.0,
        }
    }
}

enum Timer {
    NotRunning(NotRunning),
    Running(Running),
}

impl Timer {
    fn new() -> Self {
        return Self::NotRunning(NotRunning::default());
    }

    fn get_or_start(&mut self) -> &mut Running {
        match self {
            Self::Running(running) => running,
            Self::NotRunning(nr) => {
                let running = Running::new(nr.start_at);
                *self = Self::Running(running);
                let Self::Running(running) = self else {
                    unreachable!()
                };
                running
            }
        }
    }

    fn stop(&mut self) -> &mut NotRunning {
        match self {
            Self::NotRunning(not_running) => not_running,
            Self::Running(_) => {
                *self = Self::new();
                let Self::NotRunning(not_running) = self else {
                    unreachable!()
                };
                not_running
            }
        }
    }
}

struct Game {
    process: Process,
    data: Data,
}

struct State<'s> {
    handler: EventHandler<'s>,
    timer: Timer,
}

impl<'s> State<'s> {
    fn new(settings: &'s mut Settings) -> Self {
        Self {
            handler: EventHandler::new(settings),
            timer: Timer::new(),
        }
    }

    async fn connect(&mut self) -> Game {
        let process = Process::wait_attach("SeaOfStars.exe").await;
        log!("Attached to process");
        let data = Data::wait_new(&process).await;
        return Game { process, data };
    }

    async fn connected_loop(&mut self, Game { process, data }: &Game) {
        process
            .until_closes(self.main_loop(process, data))
            .await
            .unwrap_or_default();

        // always resume game timer after a disconnect
        // in case a crash happened during a load
        if let Timer::Running(ref mut r) = self.timer {
            r.act(self.handler.settings, Action::Resume);
        }
    }

    async fn main_loop(&mut self, process: &Process, data: &Data) {
        loop {
            self.tick(process, data);
            next_tick().await;
            self.handler.settings.update();
        }
    }

    fn tick(&mut self, process: &Process, data: &Data) {
        let timer_state = timer::state();
        match timer_state {
            TimerState::Running | TimerState::Paused => {
                let running = self.timer.get_or_start();
                running.tick(&mut self.handler, process, data);
            }
            TimerState::NotRunning | TimerState::Ended => {
                let not_running = self.timer.stop();
                not_running.tick(self.handler.settings, process, data);
            }
            _otherwise => {
                log!("Unexpected timer state: {:?}", _otherwise);
            }
        }
    }
}

impl NotRunning {
    fn tick(&mut self, settings: &Settings, process: &Process, data: &Data) {
        if let Some(action) = self.check_start(settings, process, data) {
            self.act(settings, action);
        }
    }

    fn check_start(
        &mut self,
        settings: &Settings,
        process: &Process,
        data: &Data,
    ) -> Option<Action> {
        if settings.relic_start {
            let current = self
                .start_screen(process, data)
                .unwrap_or(self.start_screen);
            if current >= StartScreen::DifficultyScreen {
                match data.speedrun_time(process) {
                    Some(SpeedrunRelic::Active(time)) => {
                        let relic_time = self.initial_relic_time.update_infallible(time);
                        if relic_time.bytes_changed() {
                            self.start_at = relic_time.current;
                            return Some(Action::StartRelic);
                        }
                    }
                    Some(SpeedrunRelic::Inactive) | None => {
                        let _irt = self.initial_relic_time.update(None);
                    }
                }
            }
        } else if settings.start {
            if let Some(StartScreen::CharSelected) = self.start_screen(process, data) {
                return Some(Action::StartCharacter);
            }
        }

        return None;
    }

    fn start_screen(&mut self, process: &Process, data: &Data) -> Option<StartScreen> {
        if let Some(next) = data.start_screen(process, self.start_screen) {
            log!(
                "Start screen changed from {:?} to {:?}",
                self.start_screen,
                next
            );
            self.start_screen = next;
            return Some(next);
        }
        return None;
    }

    fn act(&self, settings: &Settings, action: Action) {
        match action {
            Action::StartCharacter if settings.start => {
                log!("Starting timer on char select");
                timer::start();
                timer::pause_game_time();
            }
            Action::StartRelic if settings.relic_start => {
                log!("Starting timer");
                timer::start();
                timer::pause_game_time();
            }
            _ => {}
        }
    }
}

asr::async_main!(stable);
asr::panic_handler!();

async fn main() {
    asr::set_tick_rate(TICK_RATE);

    let mut settings = {
        let mut s = Settings::register();
        s.update();
        log!("Loaded settings: {:?}", s);
        s
    };

    let mut state = State::new(&mut settings);

    loop {
        let game = state.connect().await;
        state.connected_loop(&game).await;
    }
}
