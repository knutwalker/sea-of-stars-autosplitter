#![no_std]

use crate::{data::Data, splits::Progress};
use asr::{
    future::next_tick,
    settings::Gui,
    timer::{self, TimerState},
    Process,
};

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        ::asr::msg!($($arg)*);
    }};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {};
}

mod data;
mod game;
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
    _Moorlands,
    SolarRain,
    WindMineTunnels,
    Rockie,
    Malkomud,
    ChoralCascades,
    Brisk,
    DemoWizardLab,
    Chromatic,
    Boat,
    WraitIslandDocks,
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
    CeruleanExpense,
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

    /// Split on finished boss encounters
    #[default = true]
    split: bool,

    /// until battle against wyrd
    #[default = true]
    tutorial: bool,

    /// defeated
    #[default = true]
    wyrd: bool,

    /// entering forbidden cavern
    #[default = true]
    training: bool,

    /// split when cutscene against slug starts
    #[default = true]
    forbidden_cavern: bool,

    /// defeated
    #[default = true]
    bossslug: bool,

    /// loading into the elder mist trials level
    #[default = true]
    mountain_trails: bool,

    /// cutscene before boss
    #[default = true]
    elder_mist_trials: bool,

    /// defeated
    #[default = true]
    elder_mist: bool,

    /// being thrown / world map load
    #[default = true]
    yeet: bool,

    /// mountain descend (demo)
    #[default = true]
    xtols_landing: bool,

    /// when teaks cutscene starts
    #[default = true]
    moorlands: bool,

    /// Loading into wordlmap from moorlands
    #[default = true]
    solar_rain: bool,

    /// cutscene with first sala starts
    #[default = true]
    wind_mine_tunnels: bool,

    /// first beatup
    #[default = true]
    rockie: bool,

    /// defeated
    #[default = true]
    malkomud: bool,

    /// load from it into worldmap
    #[default = true]
    choral_cascades: bool,

    /// when entering the lab
    #[default = true]
    brisk: bool,

    /// when entering the boss room
    #[default = true]
    demo_wizard_lab: bool,

    /// defeated
    #[default = true]
    chromatic: bool,

    /// obtain the "boat", loaded into worldmap
    #[default = true]
    boat: bool,

    /// loaded into world map from boat docks
    #[default = true]
    wrait_island_docks: bool,

    /// Leaving the area, back into town
    #[default = true]
    cursed_woods: bool,

    /// cutscene against duke starts
    #[default = true]
    flooded_graveyard: bool,

    /// defeated
    #[default = true]
    duke: bool,

    /// obtaining graplou, then leaving
    #[default = true]
    necromancers_lair: bool,

    /// enter cutscene for romaya
    #[default = true]
    rope_dart: bool,

    /// defeated
    #[default = true]
    romaya: bool,

    /// travel from ferry to town
    #[default = true]
    enchanted_scarf: bool,

    /// Master Ghost Sandwhich
    #[default = true]
    cooking: bool,

    /// cutscene for big plant
    #[default = true]
    garden: bool,

    /// defeated Botanical Horror
    #[default = true]
    big_plant: bool,

    /// after Garl cutscene, spawn phase 2
    #[default = true]
    dweller_of_woe_p1: bool,

    /// defeated
    #[default = true]
    dweller_of_woe: bool,

    /// load from brisk into peninsula
    #[default = true]
    battle_of_brisk: bool,

    /// obtaining the worldmap item
    #[default = true]
    map: bool,

    /// cutscene before Stormcaller
    #[default = true]
    three_towers: bool,

    /// defeated
    #[default = true]
    stormcaller: bool,

    /// load from brisk to worldmap
    #[default = true]
    ship: bool,

    /// Mirth is build and loads in
    #[default = true]
    build_mirth: bool,

    /// Leave Mirth to World map with boat
    #[default = true]
    mirth: bool,

    /// enter cutscene for 1,3
    #[default = true]
    jungle_path: bool,

    /// defeated
    #[default = true]
    _13: bool,

    /// lose Seashell KI, later enter worldmap
    #[default = true]
    sacred_grove: bool,

    /// enter Antsudlo
    #[default = true]
    shopping_conches: bool,

    /// Antsudlo -> Glacial Peak
    #[default = true]
    antsudlo: bool,

    /// cutscene into 2,4
    #[default = true]
    glacial_peak: bool,

    /// defeated
    #[default = true]
    _24: bool,

    /// Glacial -> Great Archives
    #[default = true]
    signet_of_clarity: bool,

    /// enter cutscene against Dweller of Torment
    #[default = true]
    torment_peak: bool,

    /// defeated
    #[default = true]
    dweller_of_torment: bool,

    /// load back into worldmap
    #[default = true]
    back_to_mirth: bool,

    /// load back into worldmap
    #[default = true]
    mesa_hike: bool,

    /// cutscene to leaf monster
    #[default = true]
    autumn_hills: bool,

    /// defeated
    #[default = true]
    leaf_monster: bool,

    /// Bamboo creek -> loading into worldmap
    #[default = true]
    bamboo_creek: bool,

    /// Songshroom Marsh -> loading into worldmap
    #[default = true]
    songshroom_marsh: bool,

    /// defeated
    #[default = true]
    erlyna_and_brugaves: bool,

    /// enter a cutscene / getting the quest
    #[default = true]
    clockwork_castle: bool,

    /// cutscene before 1, 2, 3, 4
    #[default = true]
    watchmaker: bool,

    /// defeated
    #[default = true]
    _1234: bool,

    /// defeated
    #[default = true]
    dweller_of_strife_p1: bool,

    /// defeated
    #[default = true]
    dweller_of_strife_p2: bool,

    /// Skyward Shrine -> loading into worldmap
    #[default = true]
    skyward_shrine: bool,

    /// Counil -> loading into worldmap
    #[default = true]
    council: bool,

    /// Use the coral hammer on skypedia
    #[default = true]
    air_elemental: bool,

    /// defeated
    #[default = true]
    hydralion: bool,

    /// cutscene before Toadcano
    #[default = true]
    volcano: bool,

    /// defeated
    #[default = true]
    toadcano: bool,

    /// Cutscenes into load Cloud Kingdom
    #[default = true]
    rip_garl: bool,

    /// cutscene into boss
    #[default = true]
    sea_of_stars: bool,

    /// defeated
    #[default = true]
    guardian: bool,

    /// Worldmap -> Repine
    #[default = true]
    derelict_factory: bool,

    /// Repine -> Worldmap
    #[default = true]
    repine: bool,

    /// Load into Lost Ones Hamlet
    #[default = true]
    cerulean_expense: bool,

    /// cutscene before Meduso
    #[default = true]
    lost_ones_hamlet: bool,

    /// defeated
    #[default = true]
    meduso: bool,

    /// load worldmap -> Sacrosanct Spires
    #[default = true]
    leaving_for_spires: bool,

    /// cutscene before Triumvirate
    #[default = true]
    hunting_fields: bool,

    /// defeated
    #[default = true]
    triumvirate: bool,

    /// load from Lookout into worldmap
    #[default = true]
    just_kick_it: bool,

    /// cutscene before Catalyst
    #[default = true]
    sky_base: bool,

    /// defeated
    #[default = true]
    catalyst: bool,

    /// cutscene before Dweller of Dread
    #[default = true]
    infinite_abyss: bool,

    /// defeated
    #[default = true]
    dweller_of_dread: bool,

    /// cutscene before LeJugg
    #[default = true]
    fleshmancers_lair: bool,

    /// defeated
    #[default = true]
    le_jugg: bool,

    /// cutscene before Reaper
    #[default = true]
    nolan_simulator: bool,

    /// defeated
    #[default = true]
    reaper: bool,

    /// cutscene before Elysan
    #[default = true]
    ffvii_simulator: bool,

    /// defeated
    #[default = true]
    elysandarelle_p1: bool,

    /// defeated
    #[default = true]
    elysandarelle_p2: bool,

    /// defeated
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
                Split::_Moorlands => self.moorlands,
                Split::SolarRain => self.solar_rain,
                Split::WindMineTunnels => self.wind_mine_tunnels,
                Split::Rockie => self.rockie,
                Split::Malkomud => self.malkomud || self.split,
                Split::ChoralCascades => self.choral_cascades,
                Split::Brisk => self.brisk,
                Split::DemoWizardLab => self.demo_wizard_lab,
                Split::Chromatic => self.chromatic || self.split,
                Split::Boat => self.boat,
                Split::WraitIslandDocks => self.wrait_island_docks,
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
                Split::CeruleanExpense => self.cerulean_expense,
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
        asr::msg!("Decided on an action: {action:?}");
        match (action, timer::state() == TimerState::Running) {
            (Action::Start, false) => {
                log!("Starting timer");
                timer::start();
            }
            (Action::Split(_split), true) => {
                log!("Splitting: {_split:?}");
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
