use crate::{data::Data, progress::Progress};
use asr::{
    future::next_tick,
    timer::{self, TimerState},
    Process,
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
mod progress;

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
                        log!("picked up a new key item: {} ({})", item.name, item.id);
                    }
                    #[cfg(debugger)]
                    for item in data.check_for_lost_key_items().await {
                        log!("lost a key item: {} ({})", item.name, item.id);
                    }

                    match timer::state() {
                        TimerState::NotRunning => {
                            let start = progress.start(&mut data);
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
