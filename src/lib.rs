#![no_std]

use crate::{
    data::{Data, SpeedrunRelic, StartScreen},
    splits::{EventHandler, Running, Split},
    utils::EnumSet,
};
use asr::{
    Process,
    future::next_tick,
    settings::{Gui, gui::Title},
    timer::{self, TimerState},
    watcher::Watcher,
};
use num_enum::TryFromPrimitive;

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

#[derive(Gui)]
pub struct Settings {
    /// Report 'Game Time' according to the Speedrun Relic, also acts as Load Remover
    #[default = true]
    remove_loads: bool,

    /// Start after confirming the relics
    #[default = true]
    relic_start: bool,

    /// Split on finished boss encounters (previous behavioroverrides specific encounter splits)
    #[default = false]
    split: bool,

    /// LEGACY: Start after selecting the character
    #[default = false]
    start: bool,

    /// LEGACY: Use a custom load remover, not synced to the Speedrun Relic
    #[default = false]
    custom_load_remover: bool,

    /// Individual splits, enable those that match your splits
    _splits_heading: Title,

    /// Split when starting the battle against Wyrd
    #[default = false]
    tutorial: bool,

    /// Split when Wyrd is defeated
    #[default = true]
    wyrd: bool,

    /// Split when entering the forbidden cavern during training
    #[default = false]
    training: bool,

    /// Split when starting the battle against Bossslug
    #[default = false]
    forbidden_cavern: bool,

    /// Split when Bossslug is defeated
    #[default = true]
    bossslug: bool,

    /// Split when entering the Elder Mist trials
    #[default = false]
    mountain_trails: bool,

    /// Split when starting the battle against Elder Mist
    #[default = false]
    elder_mist_trials: bool,

    /// Split when Elder Mist is defeated
    #[default = true]
    elder_mist: bool,

    /// Split when being yeeted the first time
    #[default = false]
    yeet: bool,

    /// Split when entering Moorland
    #[default = false]
    xtols_landing: bool,

    /// Splits when the cutscene with Teaks starts (not implemented)
    #[default = false]
    moorland: bool,

    /// Split when leaving Moorland
    #[default = false]
    solar_rain: bool,

    /// Split when entering the first battle against Salamander
    #[default = false]
    wind_mine_tunnels: bool,

    /// Split when Salamander is defeated
    #[default = false]
    rockie: bool,

    /// Split when Malkomud is defeated
    #[default = true]
    malkomud: bool,

    /// Split when leaving the Choral Cascades
    #[default = false]
    choral_cascades: bool,

    /// Split when entering the Wizard Lab
    #[default = false]
    brisk: bool,

    /// Split when starting the battle against Chromatic Apparition
    #[default = false]
    demo_wizard_lab: bool,

    /// Split when Chromatic Apparition is defeated
    #[default = true]
    chromatic: bool,

    /// Split when first sailing the world with "boat"
    #[default = false]
    boat: bool,

    /// Split when leaving the Wraith Island Docks
    #[default = false]
    wraith_island_docks: bool,

    /// Split when leaving the Cursed Woods
    #[default = false]
    cursed_woods: bool,

    /// Splits when the battle against the Duke starts
    #[default = false]
    flooded_graveyard: bool,

    /// Split when the Duke is defeated
    #[default = true]
    duke: bool,

    /// Split when obtaining the Graplou
    #[default = false]
    necromancers_lair: bool,

    /// Split when the battle against Romaya starts
    #[default = false]
    rope_dart: bool,

    /// Split when Romaya is defeated
    #[default = true]
    romaya: bool,

    /// Split when going back to Lucent from the Ferry
    #[default = false]
    enchanted_scarf: bool,

    /// Split when the Master Ghost Sandwhich is handed over
    #[default = false]
    cooking: bool,

    /// Split when the battle against the Botanic Horror starts
    #[default = false]
    garden: bool,

    /// Split when the Botanic Horror is defeated
    #[default = true]
    big_plant: bool,

    /// Split when Phase 2 against the Dweller of Woe starts
    #[default = false]
    dweller_of_woe_p1: bool,

    /// Split when the Dweller of Woe is defeated
    #[default = true]
    dweller_of_woe: bool,

    /// Split when leaving the destroyed Brisk
    #[default = false]
    battle_of_brisk: bool,

    /// Split when obtaining the Map item
    #[default = false]
    map: bool,

    /// Split when the battle against Stormcaller starts
    #[default = false]
    three_towers: bool,

    /// Split when Stormcaller is defeated
    #[default = true]
    stormcaller: bool,

    /// Split when leaving Brisk to head for Mirth
    #[default = false]
    ship: bool,

    /// Split when Mirth is built
    #[default = false]
    build_mirth: bool,

    /// Split when leaving Mirth
    #[default = false]
    mirth: bool,

    /// Split when starting the battle against One and Three
    #[default = false]
    jungle_path: bool,

    /// Split when One and Three are defeated
    #[default = true]
    _13: bool,

    /// Split when handing in the Seashell item
    #[default = false]
    sacred_grove: bool,

    /// Split when entering Antsudlo
    #[default = false]
    shopping_conches: bool,

    /// Split when entering Glacial Peak
    #[default = false]
    antsudlo: bool,

    /// Split when the battle against Two and Four starts
    #[default = false]
    glacial_peak: bool,

    /// Split when Two and Four are defeated
    #[default = true]
    _24: bool,

    /// Split when re-entering the Great Archives
    #[default = false]
    signet_of_clarity: bool,

    /// Split when the battle against the Dweller of Torment starts
    #[default = false]
    torment_peak: bool,

    /// Split when the Dweller of Torment is defeated
    #[default = true]
    dweller_of_torment: bool,

    /// Split when leaving for Mesa Island
    #[default = false]
    back_to_mirth: bool,

    /// Split when leaving the Mesa Hike
    #[default = false]
    mesa_hike: bool,

    /// Split when entering the battle against Leaf Monster
    #[default = false]
    autumn_hills: bool,

    /// Split when Leaf Monster is defeated
    #[default = true]
    leaf_monster: bool,

    /// Split when leaving Bamboo Creek
    #[default = false]
    bamboo_creek: bool,

    /// Split when leaving Songshroom Marsh
    #[default = false]
    songshroom_marsh: bool,

    /// Split when Erlyna and Brugaves are defeated
    #[default = true]
    erlyna_and_brugaves: bool,

    /// Split when talking to ??? in the clock tower (not implemented)
    #[default = false]
    clockwork_castle: bool,

    /// Split when the battle against One, Two, Three, and Four starts
    #[default = false]
    watchmaker: bool,

    /// Split when One, Two, Three, and Four are defeated
    #[default = true]
    _1234: bool,

    /// Split when the the Dweller of Strife is defeated the first time
    #[default = true]
    dweller_of_strife_p1: bool,

    /// Split when the Dweller of Strife is defeated the second time (scripted battle)
    #[default = false]
    dweller_of_strife_p2: bool,

    /// Split when leavin Skyward Shrine after all the cutscenes
    #[default = false]
    skyward_shrine: bool,

    /// Split when leaving the Air Counil
    #[default = false]
    council: bool,

    /// Split after using the Coral Hammer on Skyland
    #[default = false]
    air_elemental: bool,

    /// Split when Hydralion is defeated
    #[default = true]
    hydralion: bool,

    /// Split when the battle against Toadcano starts
    #[default = false]
    volcano: bool,

    /// Split when Toadcano is defeated
    #[default = true]
    toadcano: bool,

    /// Split after the cutscenes after the Garl event
    #[default = false]
    rip_garl: bool,

    /// Split when entering the battle against the Guardian
    #[default = false]
    sea_of_stars: bool,

    /// Split when the Guardian is defeated
    #[default = true]
    guardian: bool,

    /// Split when entering Repine for the first time
    #[default = false]
    derelict_factory: bool,

    /// Split when leaving Repine for the first time (shopping)
    #[default = false]
    repine: bool,

    /// Split when leaving the Cerulean Expanse
    #[default = false]
    cerulean_expanse: bool,

    /// Split when the battle against Meduso starts
    #[default = false]
    lost_ones_hamlet: bool,

    /// Split when Meduso is defeated
    #[default = true]
    meduso: bool,

    /// Split when entering the Sacrosanct Spires
    #[default = false]
    leaving_for_spires: bool,

    /// Split when the battle against the Triumvirate starts
    #[default = false]
    hunting_fields: bool,

    /// Split when the Triumvirate is defeated
    #[default = true]
    triumvirate: bool,

    /// Split when leaving the Lookout
    #[default = false]
    just_kick_it: bool,

    /// Split when the battle against the Catalyst starts
    #[default = false]
    sky_base: bool,

    /// Split when the Catalyst is defeated
    #[default = true]
    catalyst: bool,

    /// Split when the battle against the Dweller of Dread starts
    #[default = false]
    infinite_abyss: bool,

    /// Split when the Dweller of Dread is defeated
    #[default = true]
    dweller_of_dread: bool,

    /// Split when the battle against LeJugg starts
    #[default = false]
    fleshmancers_lair: bool,

    /// Split when LeJugg is defeated
    #[default = true]
    le_jugg: bool,

    /// Split when the battle against Phase Reaper starts
    #[default = false]
    nolan_simulator: bool,

    /// Split when Phase Reaper is defeated
    #[default = true]
    reaper: bool,

    /// Split when the battle against Elysan'darëlle starts
    #[default = false]
    ffvii_simulator: bool,

    /// Split when Elysan'darëlle Phase 1 is defeated
    #[default = false]
    elysandarelle_p1: bool,

    /// Split when Elysan'darëlle Phase 2 is defeated
    #[default = true]
    elysandarelle_p2: bool,

    /// Split on the final damage on the World Eater (Any% end)
    #[default = true]
    world_eater: bool,
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
            r.act(&mut self.handler, Action::Resume);
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
                log!("Starting timer on legacy char select");
                if cfg!(not(dummy)) {
                    timer::start();
                    timer::pause_game_time();
                }
            }
            Action::StartRelic if settings.relic_start => {
                log!("Starting timer");
                if cfg!(not(dummy)) {
                    timer::start();
                    timer::pause_game_time();
                }
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

        let mut splits = EnumSet::<Split>::empty();
        for idx in u8::MIN..u8::MAX {
            let Ok(split) = Split::try_from_primitive(idx) else {
                break;
            };
            if split.filter(&s) {
                let _ = splits.insert(&split);
            }
        }
        let splits = SplitsDebug(splits.into_bits());
        let splits = SettingsDebug {
            remove_loads: s.remove_loads,
            relic_start: s.relic_start,
            split: s.split,
            start: s.start,
            custom_load_remover: s.custom_load_remover,
            splits,
        };

        log!("Loaded settings: {:?}", splits);
        s
    };

    let mut state = State::new(&mut settings);

    loop {
        let game = state.connect().await;
        state.connected_loop(&game).await;
    }
}

struct SettingsDebug {
    remove_loads: bool,
    relic_start: bool,
    split: bool,
    start: bool,
    custom_load_remover: bool,
    splits: SplitsDebug,
}

impl core::fmt::Debug for SettingsDebug {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SettingsDebug")
            .field("load_removal", &self.remove_loads)
            .field("start", &self.relic_start)
            .field("boss_split", &self.split)
            .field("legacy_start", &self.start)
            .field("legacy_load_removal", &self.custom_load_remover)
            .field("splits", &self.splits)
            .finish()
    }
}

struct SplitsDebug(u128);

impl core::fmt::Debug for SplitsDebug {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}
