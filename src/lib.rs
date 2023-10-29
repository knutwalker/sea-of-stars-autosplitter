#![no_std]

use crate::{data::Data, splits::Progress};
use asr::{
    future::next_tick,
    msg,
    settings::Gui,
    timer::{self, TimerState},
    Process,
};

#[cfg(any(debug_assertions, debugger))]
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        ::asr::msg!($($arg)*);
    }};
}

#[cfg(not(any(debug_assertions, debugger)))]
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{}};
}

mod data;
mod game;
mod memory;
mod splits;

asr::async_main!(stable);
asr::panic_handler!();

async fn main() {
    asr::set_tick_rate(60.0);
    let mut settings = Settings::register();
    log!("Loaded settings: {settings:?}");

    loop {
        let process = Process::wait_attach("SeaOfStars.exe").await;
        log!("attached to process");
        process
            .until_closes(async {
                let mut data = Data::new(&process).await;
                let mut progress = Progress::new();

                loop {
                    settings.update();
                    match timer::state() {
                        TimerState::NotRunning => progress.not_running(&mut data),
                        TimerState::Running => progress.running(&mut data),
                        _ => {}
                    }

                    for action in progress.actions() {
                        act(action, &settings);
                    }

                    next_tick().await;
                }
            })
            .await;
    }
}

#[derive(Copy, Clone, Debug)]
enum Action {
    Start,
    Split(Split),
    Pause,
    Resume,
    GamePause,
    GameResume,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Split {
    _Start,
    Tutorial,
    Wyrd,
    Training,
    ForbiddenCavern,
    Bossslug,
    MountainTrails,
    ElderMistTrials,
    ElderMist,
    Yeet,
    XtolsLanding,
    _Moorland,
    SolarRain,
    WindMineTunnels,
    Rockie,
    Malkomud,
    ChoralCascades,
    Brisk,
    DemoWizardLab,
    Chromatic,
    Boat,
    WraithIslandDocks,
    CursedWoods,
    FloodedGraveyard,
    Duke,
    NecromancerssLair,
    RopeDart,
    Romaya,
    EnchantedScarf,
    Cooking,
    Garden,
    BigPlant,
    DwellerOfWoeP1,
    DwellerOfWoe,
    BattleOfBrisk,
    Map,
    ThreeTowers,
    Stormcaller,
    Ship,
    BuildMirth,
    Mirth,
    JunglePath,
    OneThree,
    SacredGrove,
    ShoppingConches,
    Antsudlo,
    GlacialPeak,
    TwoFour,
    SignetOfclarity,
    TormentPeak,
    DwellerOfTorment,
    BackToMirth,
    MesaHike,
    AutumnHills,
    LeafMonster,
    BambooCreek,
    SongshroomMarsh,
    ErlynaAndBrugaves,
    _ClockworkCastle,
    Watchmaker,
    OneTwoThreeFour,
    DwellerOfStrifeP1,
    DwellerOfStrifeP2,
    SkywardShrine,
    Council,
    AirElemental,
    Hydralion,
    Volcano,
    Toadcano,
    RIPGarl,
    SeaofStars,
    Guardian,
    DerelictFactory,
    Repine,
    CeruleanExpanse,
    LostOnesHamlet,
    Meduso,
    LeavingforSpires,
    HuntingFields,
    Triumvirate,
    JustKickIt,
    SkyBase,
    Catalyst,
    InfiniteAbyss,
    DwellerOfDread,
    FleshmancersLair,
    LeJugg,
    NolanSimulator,
    Reaper,
    FFVIISimulator,
    ElysandarelleP1,
    ElysandarelleP2,
    WorldEater,
}

#[derive(Debug, Gui)]
pub struct Settings {
    /// Stop game timer during loads (load remover)
    #[default = true]
    remove_loads: bool,

    /// Start splitting on character select
    #[default = true]
    start: bool,

    /// Split on finished boss encounters (overrides specific encounter splits)
    #[default = false]
    split: bool,

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

impl Settings {
    fn filter(&self, action: &Action) -> bool {
        match action {
            Action::Pause | Action::Resume => self.remove_loads,
            Action::GamePause | Action::GameResume => false,
            Action::Start => self.start,
            Action::Split(s) => match s {
                Split::_Start => false,
                Split::Tutorial => self.tutorial,
                Split::Wyrd => self.wyrd || self.split,
                Split::Training => self.training,
                Split::ForbiddenCavern => self.forbidden_cavern,
                Split::Bossslug => self.bossslug || self.split,
                Split::MountainTrails => self.mountain_trails,
                Split::ElderMistTrials => self.elder_mist_trials,
                Split::ElderMist => self.elder_mist || self.split,
                Split::Yeet => self.yeet,
                Split::XtolsLanding => self.xtols_landing,
                Split::_Moorland => self.moorland,
                Split::SolarRain => self.solar_rain,
                Split::WindMineTunnels => self.wind_mine_tunnels,
                Split::Rockie => self.rockie,
                Split::Malkomud => self.malkomud || self.split,
                Split::ChoralCascades => self.choral_cascades,
                Split::Brisk => self.brisk,
                Split::DemoWizardLab => self.demo_wizard_lab,
                Split::Chromatic => self.chromatic || self.split,
                Split::Boat => self.boat,
                Split::WraithIslandDocks => self.wraith_island_docks,
                Split::CursedWoods => self.cursed_woods,
                Split::FloodedGraveyard => self.flooded_graveyard,
                Split::Duke => self.duke || self.split,
                Split::NecromancerssLair => self.necromancers_lair,
                Split::RopeDart => self.rope_dart,
                Split::Romaya => self.romaya || self.split,
                Split::EnchantedScarf => self.enchanted_scarf,
                Split::Cooking => self.cooking,
                Split::Garden => self.garden,
                Split::BigPlant => self.big_plant || self.split,
                Split::DwellerOfWoeP1 => self.dweller_of_woe_p1,
                Split::DwellerOfWoe => self.dweller_of_woe || self.split,
                Split::BattleOfBrisk => self.battle_of_brisk,
                Split::Map => self.map,
                Split::ThreeTowers => self.three_towers,
                Split::Stormcaller => self.stormcaller || self.split,
                Split::Ship => self.ship,
                Split::BuildMirth => self.build_mirth,
                Split::Mirth => self.mirth,
                Split::JunglePath => self.jungle_path,
                Split::OneThree => self._13 || self.split,
                Split::SacredGrove => self.sacred_grove,
                Split::ShoppingConches => self.shopping_conches,
                Split::Antsudlo => self.antsudlo,
                Split::GlacialPeak => self.glacial_peak,
                Split::TwoFour => self._24 || self.split,
                Split::SignetOfclarity => self.signet_of_clarity,
                Split::TormentPeak => self.torment_peak,
                Split::DwellerOfTorment => self.dweller_of_torment || self.split,
                Split::BackToMirth => self.back_to_mirth,
                Split::MesaHike => self.mesa_hike,
                Split::AutumnHills => self.autumn_hills,
                Split::LeafMonster => self.leaf_monster || self.split,
                Split::BambooCreek => self.bamboo_creek,
                Split::SongshroomMarsh => self.songshroom_marsh,
                Split::ErlynaAndBrugaves => self.erlyna_and_brugaves || self.split,
                Split::_ClockworkCastle => self.clockwork_castle,
                Split::Watchmaker => self.watchmaker,
                Split::OneTwoThreeFour => self._1234 || self.split,
                Split::DwellerOfStrifeP1 => self.dweller_of_strife_p1 || self.split,
                Split::DwellerOfStrifeP2 => self.dweller_of_strife_p2,
                Split::SkywardShrine => self.skyward_shrine,
                Split::Council => self.council,
                Split::AirElemental => self.air_elemental,
                Split::Hydralion => self.hydralion || self.split,
                Split::Volcano => self.volcano,
                Split::Toadcano => self.toadcano || self.split,
                Split::RIPGarl => self.rip_garl,
                Split::SeaofStars => self.sea_of_stars,
                Split::Guardian => self.guardian || self.split,
                Split::DerelictFactory => self.derelict_factory,
                Split::Repine => self.repine,
                Split::CeruleanExpanse => self.cerulean_expanse,
                Split::LostOnesHamlet => self.lost_ones_hamlet,
                Split::Meduso => self.meduso || self.split,
                Split::LeavingforSpires => self.leaving_for_spires,
                Split::HuntingFields => self.hunting_fields,
                Split::Triumvirate => self.triumvirate || self.split,
                Split::JustKickIt => self.just_kick_it,
                Split::SkyBase => self.sky_base,
                Split::Catalyst => self.catalyst || self.split,
                Split::InfiniteAbyss => self.infinite_abyss,
                Split::DwellerOfDread => self.dweller_of_dread || self.split,
                Split::FleshmancersLair => self.fleshmancers_lair,
                Split::LeJugg => self.le_jugg || self.split,
                Split::NolanSimulator => self.nolan_simulator,
                Split::Reaper => self.reaper || self.split,
                Split::FFVIISimulator => self.ffvii_simulator,
                Split::ElysandarelleP1 => self.elysandarelle_p1,
                Split::ElysandarelleP2 => self.elysandarelle_p2 || self.split,
                Split::WorldEater => self.world_eater || self.split,
            },
        }
    }
}

fn act(action: Action, settings: &Settings) {
    if settings.filter(&action) {
        log!("Decided on an action: {action:?}");
        match (action, timer::state() == TimerState::Running) {
            (Action::Start, false) => {
                msg!("Starting timer");
                timer::start();
            }
            (Action::Split(split), true) => {
                msg!("Splitting: {split:?}");
                timer::split();
            }
            (Action::Pause, true) => {
                msg!("Pause game time");
                timer::pause_game_time();
            }
            (Action::Resume, true) => {
                msg!("Resume game time");
                timer::resume_game_time();
            }
            _ => {}
        }
    }
}
