#![no_std]

use crate::data::{Data, GameStart, SpeedrunRelic};
use asr::{
    Address64, Process,
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

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! dbg {
    () => {};
    ($val:expr $(,)?) => {};
    ($($val:expr),+ $(,)?) => {};
}

mod data;

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
    Split,
    Pause,
    Resume,
    SplitAndPause,
    SplitAndResume,
}

struct NotRunning {
    game_start: GameStart,
    start_at: f64,
}

impl Default for NotRunning {
    fn default() -> Self {
        Self {
            game_start: GameStart::Undecided,
            start_at: 0.0,
        }
    }
}

struct Running {
    loading: Watcher<bool>,
    encounter: Option<Address64>,
    relic_time: Watcher<f64>,
    paused: bool,
}

impl Running {
    fn new(time: f64) -> Self {
        let mut running = Self {
            loading: Watcher::new(),
            encounter: None,
            relic_time: Watcher::new(),
            paused: false,
        };
        let _ = running.relic_time.update_infallible(time);
        running
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
    // option because of borrowck shenanigans
    process: Option<Process>,
    data: Data,
}

struct State<'s> {
    settings: &'s mut Settings,
    timer: Timer,
    game: Option<Game>,
}

impl State<'_> {
    async fn connect(&mut self) {
        if self.game.is_none() {
            let process = Process::wait_attach("SeaOfStars.exe").await;
            log!("Attached to process");
            let data = Data::wait_new(&process).await;
            self.game = Some(Game {
                process: Some(process),
                data,
            });
        }
    }

    async fn connected_loop(&mut self) {
        let Some(mut game) = self.game.take() else {
            unreachable!()
        };
        let Some(process) = game.process.take() else {
            unreachable!()
        };

        process
            .until_closes(self.main_loop(&process, &game.data))
            .await
            .unwrap_or_default();

        // always resume game timer after a disconnect
        // in case a crash happened during a load
        if let Timer::Running(ref mut r) = self.timer {
            r.act(self.settings, Action::Resume);
        }
    }

    async fn main_loop(&mut self, process: &Process, data: &Data) {
        loop {
            self.tick(process, data);
            next_tick().await;
            self.settings.update();
        }
    }

    fn tick(&mut self, process: &Process, data: &Data) {
        let timer_state = timer::state();
        match timer_state {
            TimerState::Running | TimerState::Paused => {
                let running = self.timer.get_or_start();
                running.tick(self.settings, process, data);
            }
            TimerState::NotRunning | TimerState::Ended => {
                let not_running = self.timer.stop();
                not_running.tick(self.settings, process, data);
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
            let _ = self.game_start(process, data);
            if self.game_start >= GameStart::DifficultyScreen
                && let Some(SpeedrunRelic::Active(time)) = data.speedrun_time(process)
                && time > 0.0
            {
                self.start_at = time;
                return Some(Action::StartRelic);
            }
        } else if settings.start {
            if let Some(GameStart::CharSelected) = self.game_start(process, data) {
                return Some(Action::StartCharacter);
            }
        }

        return None;
    }

    fn game_start(&mut self, process: &Process, data: &Data) -> Option<GameStart> {
        let current = data.game_start(process, self.game_start);
        if current != self.game_start {
            log!(
                "Game Start changed from {:?} to {:?}",
                self.game_start,
                current
            );
            self.game_start = current;
            return Some(current);
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

#[derive(Debug, Clone, Copy)]
enum LoadRemoval {
    Pause,
    Resume,
}

impl Running {
    fn tick(&mut self, settings: &Settings, process: &Process, data: &Data) {
        let load_removal = self.load_removal(settings, process, data);
        let split = settings.split && self.check_encounter_done(process, data);

        let action = match (load_removal, split) {
            (None, false) => return,
            (None, true) => Action::Split,
            (Some(LoadRemoval::Pause), false) => Action::Pause,
            (Some(LoadRemoval::Pause), true) => Action::SplitAndPause,
            (Some(LoadRemoval::Resume), false) => Action::Resume,
            (Some(LoadRemoval::Resume), true) => Action::SplitAndResume,
        };
        self.act(settings, action);
    }

    fn load_removal(
        &mut self,
        settings: &Settings,
        process: &Process,
        data: &Data,
    ) -> Option<LoadRemoval> {
        if settings.remove_loads == false {
            return None;
        }
        if settings.custom_load_remover == false
            && let Some(SpeedrunRelic::Active(time)) = data.speedrun_time(process)
        {
            let relic_time = self.relic_time.update_infallible(time);
            match (relic_time.increased(), self.paused) {
                // relic and lrt are both running
                (true, false) => {}
                // relic and lrt are both paused
                (false, true) => {}
                // relic paused, lrt is running, need to pause lrt
                (false, false) => return Some(LoadRemoval::Pause),
                // relic is running, lrt is paused, need to unpause lrt
                (true, true) => return Some(LoadRemoval::Resume),
            }
        } else if settings.custom_load_remover == true {
            match self.loading.update(data.is_loading(process)) {
                Some(l) if l.changed_to(&false) => return Some(LoadRemoval::Resume),
                Some(l) if l.changed_to(&true) => return Some(LoadRemoval::Pause),
                _ => {}
            }
        }

        return None;
    }

    fn check_encounter_done(&mut self, process: &Process, data: &Data) -> bool {
        match self.encounter {
            Some(enc) => match data.resolve_encounter(process, enc) {
                Some(enc) if enc.done => {
                    self.encounter = None;
                    return true;
                }
                Some(_) => {}
                None => {
                    self.encounter = None;
                }
            },
            None => {
                let Some((address, encounter)) = data.encounter(process) else {
                    return false;
                };
                if encounter.boss && !encounter.done {
                    self.encounter = Some(address);
                }
            }
        };
        return false;
    }

    fn act(&mut self, settings: &Settings, action: Action) {
        match action {
            Action::Split if settings.split => {
                log!("Splitting");
                timer::split();
            }
            Action::Pause if settings.remove_loads => {
                self.paused = true;
                timer::pause_game_time();
            }
            Action::Resume if settings.remove_loads => {
                self.paused = false;
                timer::resume_game_time();
            }
            Action::SplitAndPause => {
                self.act(settings, Action::Pause);
                self.act(settings, Action::Split);
            }
            Action::SplitAndResume => {
                self.act(settings, Action::Resume);
                self.act(settings, Action::Split);
            }
            _otherwise => {}
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

    let mut state = State {
        settings: &mut settings,
        timer: Timer::new(),
        game: None,
    };

    loop {
        state.connect().await;
        state.connected_loop().await;
    }
}
