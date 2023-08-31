#![no_std]

use crate::data::{Data, GameState};
use asr::{
    future::next_tick,
    timer::{self, TimerState},
    watcher::Watcher,
    Address64, Process,
};

#[cfg(debug_assertions)]
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
macro_rules! log {
    ($($arg:tt)*) => {};
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
                            log!("start: {:?}", start);
                            if let Some(action) = start {
                                act(action);
                            }
                        }
                        TimerState::Running => {
                            while let Some(action) = progress.act(&data) {
                                if let Some(action) = settings.filter(action) {
                                    log!("Decided on an action: {action:?}");
                                    act(action);
                                }
                            }
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
    /// Stop game timer during loads
    remove_loads: bool,
}

#[derive(Debug)]
enum Action {
    Reset,
    Start,
    Split,
    Pause,
    Resume,
}

struct Progress {
    next: Option<Action>,
    loading: Watcher<bool>,
    encounter: Option<Address64>,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            next: None,
            loading: Watcher::new(),
            encounter: None,
        }
    }

    pub fn start(&mut self, data: &Data<'_>) -> Option<Action> {
        matches!(data.game_start(), GameState::JustStarted).then_some(Action::Start)
    }

    pub fn act(&mut self, data: &Data<'_>) -> Option<Action> {
        if let Some(next) = self.next.take() {
            return Some(next);
        }

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
            Some(enc) => {
                let done = match data.resolve_encounter(enc) {
                    Some(enc) if enc.done => true,
                    Some(_) => false,
                    None => true,
                };
                if done {
                    self.encounter = None;
                    return Some(true);
                }
            }
            None => {
                let (address, encounter) = data.encounter()?;
                if encounter.boss {
                    self.encounter = Some(address);
                }
            }
        };
        Some(false)
    }
}

impl Settings {
    fn filter(&self, action: Action) -> Option<Action> {
        Some(action).filter(|action| match action {
            Action::Pause | Action::Resume => self.remove_loads,
            _ => true,
        })
    }
}

fn act(action: Action) {
    match action {
        Action::Reset if timer::state() == TimerState::Ended => {
            log!("Resetting timer");
            timer::reset();
        }
        Action::Start if timer::state() != TimerState::Running => {
            log!("Starting timer");
            timer::start();
        }
        Action::Split => {
            log!("Splitting");
            timer::split();
        }
        Action::Pause => {
            log!("Pause game time");
            timer::pause_game_time();
        }
        Action::Resume => {
            log!("Resume game time");
            timer::resume_game_time();
        }
        _ => {}
    }
}
