#![no_std]

use crate::data::{Data, GameStart, SpeedrunRelic};
use asr::{
    Address64, Process,
    future::{next_tick, retry},
    settings::Gui,
    time::Duration,
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
    SetGameTime(f64),
    SplitAndGameTime(f64),
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
    time: f64,
}

impl Running {
    fn new(time: f64) -> Self {
        let mut running = Self {
            loading: Watcher::new(),
            encounter: None,
            relic_time: Watcher::new(),
            time,
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
            let process = retry(|| self.connect_process()).await;
            let data = Data::wait_new(&process).await;
            self.game = Some(Game {
                process: Some(process),
                data,
            });
        }
    }

    fn connect_process(&mut self) -> Option<Process> {
        self.tick_lrt();
        let process = Process::attach("SeaOfStars.exe")?;
        log!("Attached to process");
        return Some(process);
    }

    fn tick_lrt(&mut self) {
        if let Timer::Running(ref mut r) = self.timer {
            r.tick_lrt();
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
    }

    async fn main_loop(&mut self, process: &Process, data: &Data) {
        loop {
            let action = self.tick(process, data);
            act(action, self.settings);
            next_tick().await;
            self.settings.update();
        }
    }

    fn tick(&mut self, process: &Process, data: &Data) -> Option<Action> {
        let timer_state = timer::state();
        match timer_state {
            TimerState::Running | TimerState::Paused => {
                let running = self.timer.get_or_start();
                return running.tick(self.settings, process, data);
            }
            TimerState::NotRunning | TimerState::Ended => {
                let not_running = self.timer.stop();
                return not_running.tick(self.settings, process, data);
            }
            _otherwise => {
                log!("Unexpected timer state: {:?}", _otherwise);
                return None;
            }
        }
    }
}

impl NotRunning {
    fn tick(&mut self, settings: &Settings, process: &Process, data: &Data) -> Option<Action> {
        if settings.relic_start {
            let _ = self.game_start(process, data);
            if self.game_start >= GameStart::DifficultyScreen
                && let Some(SpeedrunRelic::Active(time)) = data.speedrun_time(process)
                && time > 0.0
            {
                self.start_at = time;
                return Some(Action::StartRelic);
            }
        }
        if settings.start {
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
}

impl Running {
    fn tick_lrt(&mut self) {
        self.time += 1.0 / TICK_RATE;
        set_game_time(self.time);
    }

    fn tick(&mut self, settings: &Settings, process: &Process, data: &Data) -> Option<Action> {
        let relic_time = settings
            .use_relic()
            .then(|| self.check_relic_timer(process, data))
            .flatten();

        if relic_time.is_none() && settings.use_load_manager() {
            match self.loading.update(data.is_loading(process)) {
                Some(l) if l.changed_to(&false) => return Some(Action::Resume),
                Some(l) if l.changed_to(&true) => return Some(Action::Pause),
                _ => {}
            }
        }

        let split = settings.is_split() && self.check_encounter(process, data);

        match (relic_time, split) {
            (None, false) => None,
            (None, true) => Some(Action::Split),
            (Some(time), false) => Some(Action::SetGameTime(time)),
            (Some(time), true) => Some(Action::SplitAndGameTime(time)),
        }
    }

    fn check_relic_timer(&mut self, process: &Process, data: &Data) -> Option<f64> {
        match data.speedrun_time(process) {
            Some(SpeedrunRelic::Active(time)) => {
                let relic_time = self.relic_time.update_infallible(time);
                if relic_time.increased() {
                    let time_delta = relic_time.current - relic_time.old;
                    self.time += time_delta;
                    return Some(self.time);
                    // return Some(relic_time.current);
                }
            }
            Some(SpeedrunRelic::Inactive) => self.tick_lrt(),
            None => {}
        }
        return None;
    }

    fn check_encounter(&mut self, process: &Process, data: &Data) -> bool {
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
        state.tick_lrt();
        state.connect().await;
        state.connected_loop().await;
    }
}

impl Settings {
    fn is_lrt(&self) -> bool {
        self.remove_loads
    }

    fn use_relic(&self) -> bool {
        self.is_lrt() && self.custom_load_remover == false
    }

    fn use_load_manager(&self) -> bool {
        self.is_lrt() && self.custom_load_remover == true
    }

    fn is_split(&self) -> bool {
        self.split
    }

    fn filter(&self, action: &Action) -> bool {
        match action {
            Action::Pause | Action::Resume => self.use_load_manager(),
            Action::StartCharacter => self.start,
            Action::StartRelic => self.relic_start,
            Action::Split => self.is_split(),
            Action::SetGameTime(_) => self.use_relic(),
            Action::SplitAndGameTime(_) => self.is_split() || self.use_relic(),
        }
    }
}

fn act(action: Option<Action>, settings: &Settings) {
    let Some(action) = action else {
        return;
    };
    match action {
        Action::StartCharacter
        | Action::StartRelic
        | Action::Split
        | Action::SplitAndGameTime(_) => {
            log!("Possible action: {:?}", action)
        }
        Action::Pause | Action::Resume => {}
        Action::SetGameTime(_) => {}
    }
    if settings.filter(&action) {
        match action {
            Action::StartCharacter => {
                log!("Starting timer on char select");
                timer::start();
            }
            Action::StartRelic => {
                log!("Starting timer");
                timer::start();
                timer::pause_game_time();
            }
            Action::Split => {
                log!("Splitting");
                timer::split();
            }
            Action::Pause => {
                timer::pause_game_time();
            }
            Action::Resume => {
                timer::resume_game_time();
            }
            Action::SetGameTime(time) => {
                set_game_time(time);
            }
            Action::SplitAndGameTime(time) => {
                log!("Splitting");
                timer::split();
                set_game_time(time);
            }
        }
    }
}

fn set_game_time(time: f64) {
    let time = Duration::seconds_f64(time);
    timer::set_game_time(time);
}
