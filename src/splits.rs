use core::num::NonZeroU32;

use crate::{
    Action, Settings,
    data::{Data, EncounterData, SpeedrunRelic},
    mapping::{AnyEnemy, Enemy, KeyItem, Level, Unknown},
    utils::{EnumSet, EnumSetMember},
};
use asr::{Process, arrayvec::ArrayVec, timer, watcher::Watcher};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Split {
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

impl Split {
    pub fn is_enabled(self, settings: &Settings) -> bool {
        if settings.split && self.compat() {
            return true;
        }
        return self.filter(settings);
    }

    fn compat(&self) -> bool {
        matches!(
            self,
            Split::Bossslug
                | Split::ElderMist
                | Split::Malkomud
                | Split::Chromatic
                | Split::Romaya
                | Split::BigPlant
                | Split::DwellerOfWoe
                | Split::Stormcaller
                | Split::DwellerOfTorment
                | Split::LeafMonster
                | Split::ErlynaAndBrugaves
                | Split::OneTwoThreeFour
                | Split::DwellerOfStrifeP1
                | Split::Hydralion
                | Split::Toadcano
                | Split::Guardian
                | Split::Triumvirate
                | Split::Catalyst
                | Split::DwellerOfDread
                | Split::LeJugg
                | Split::Reaper
                | Split::ElysandarelleP2
                | Split::WorldEater
        )
    }

    pub fn filter(&self, settings: &Settings) -> bool {
        match self {
            Split::Tutorial => settings.tutorial,
            Split::Wyrd => settings.wyrd,
            Split::Training => settings.training,
            Split::ForbiddenCavern => settings.forbidden_cavern,
            Split::Bossslug => settings.bossslug,
            Split::MountainTrails => settings.mountain_trails,
            Split::ElderMistTrials => settings.elder_mist_trials,
            Split::ElderMist => settings.elder_mist,
            Split::Yeet => settings.yeet,
            Split::XtolsLanding => settings.xtols_landing,
            Split::_Moorland => settings.moorland,
            Split::SolarRain => settings.solar_rain,
            Split::WindMineTunnels => settings.wind_mine_tunnels,
            Split::Rockie => settings.rockie,
            Split::Malkomud => settings.malkomud,
            Split::ChoralCascades => settings.choral_cascades,
            Split::Brisk => settings.brisk,
            Split::DemoWizardLab => settings.demo_wizard_lab,
            Split::Chromatic => settings.chromatic,
            Split::Boat => settings.boat,
            Split::WraithIslandDocks => settings.wraith_island_docks,
            Split::CursedWoods => settings.cursed_woods,
            Split::FloodedGraveyard => settings.flooded_graveyard,
            Split::Duke => settings.duke,
            Split::NecromancerssLair => settings.necromancers_lair,
            Split::RopeDart => settings.rope_dart,
            Split::Romaya => settings.romaya,
            Split::EnchantedScarf => settings.enchanted_scarf,
            Split::Cooking => settings.cooking,
            Split::Garden => settings.garden,
            Split::BigPlant => settings.big_plant,
            Split::DwellerOfWoeP1 => settings.dweller_of_woe_p1,
            Split::DwellerOfWoe => settings.dweller_of_woe,
            Split::BattleOfBrisk => settings.battle_of_brisk,
            Split::Map => settings.map,
            Split::ThreeTowers => settings.three_towers,
            Split::Stormcaller => settings.stormcaller,
            Split::Ship => settings.ship,
            Split::BuildMirth => settings.build_mirth,
            Split::Mirth => settings.mirth,
            Split::JunglePath => settings.jungle_path,
            Split::OneThree => settings._13,
            Split::SacredGrove => settings.sacred_grove,
            Split::ShoppingConches => settings.shopping_conches,
            Split::Antsudlo => settings.antsudlo,
            Split::GlacialPeak => settings.glacial_peak,
            Split::TwoFour => settings._24,
            Split::SignetOfclarity => settings.signet_of_clarity,
            Split::TormentPeak => settings.torment_peak,
            Split::DwellerOfTorment => settings.dweller_of_torment,
            Split::BackToMirth => settings.back_to_mirth,
            Split::MesaHike => settings.mesa_hike,
            Split::AutumnHills => settings.autumn_hills,
            Split::LeafMonster => settings.leaf_monster,
            Split::BambooCreek => settings.bamboo_creek,
            Split::SongshroomMarsh => settings.songshroom_marsh,
            Split::ErlynaAndBrugaves => settings.erlyna_and_brugaves,
            Split::_ClockworkCastle => settings.clockwork_castle,
            Split::Watchmaker => settings.watchmaker,
            Split::OneTwoThreeFour => settings._1234,
            Split::DwellerOfStrifeP1 => settings.dweller_of_strife_p1,
            Split::DwellerOfStrifeP2 => settings.dweller_of_strife_p2,
            Split::SkywardShrine => settings.skyward_shrine,
            Split::Council => settings.council,
            Split::AirElemental => settings.air_elemental,
            Split::Hydralion => settings.hydralion,
            Split::Volcano => settings.volcano,
            Split::Toadcano => settings.toadcano,
            Split::RIPGarl => settings.rip_garl,
            Split::SeaofStars => settings.sea_of_stars,
            Split::Guardian => settings.guardian,
            Split::DerelictFactory => settings.derelict_factory,
            Split::Repine => settings.repine,
            Split::CeruleanExpanse => settings.cerulean_expanse,
            Split::LostOnesHamlet => settings.lost_ones_hamlet,
            Split::Meduso => settings.meduso,
            Split::LeavingforSpires => settings.leaving_for_spires,
            Split::HuntingFields => settings.hunting_fields,
            Split::Triumvirate => settings.triumvirate,
            Split::JustKickIt => settings.just_kick_it,
            Split::SkyBase => settings.sky_base,
            Split::Catalyst => settings.catalyst,
            Split::InfiniteAbyss => settings.infinite_abyss,
            Split::DwellerOfDread => settings.dweller_of_dread,
            Split::FleshmancersLair => settings.fleshmancers_lair,
            Split::LeJugg => settings.le_jugg,
            Split::NolanSimulator => settings.nolan_simulator,
            Split::Reaper => settings.reaper,
            Split::FFVIISimulator => settings.ffvii_simulator,
            Split::ElysandarelleP1 => settings.elysandarelle_p1,
            Split::ElysandarelleP2 => settings.elysandarelle_p2,
            Split::WorldEater => settings.world_eater,
        }
    }
}

impl EnumSetMember for Split {
    fn ordinal(&self) -> Option<u8> {
        Some(u8::from(*self))
    }
}

pub struct Running {
    seen: EnumSet<Split>,
    relic_time: Watcher<f64>,
    paused: bool,
    loading: Watcher<bool>,
    cutscene: Watcher<bool>,
    level: Watcher<Level>,
    number_of_items: Watcher<u32>,
    inventory_generation: u32,
    key_items: [(u32, u32); 4],
    encounter: Option<EncounterData>,
}

impl Running {
    pub fn new(time: f64) -> Self {
        let mut running = Self {
            seen: EnumSet::empty(),
            relic_time: Watcher::new(),
            paused: false,
            loading: Watcher::new(),
            cutscene: Watcher::new(),
            level: Watcher::new(),
            number_of_items: Watcher::new(),
            inventory_generation: 0,
            key_items: [(u32::MAX, u32::MAX); 4],
            encounter: None,
        };
        let _ = running.relic_time.update_infallible(time);
        running
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Event {
    LoadStart,
    LoadEnd,
    LevelChange { from: Level, to: Level },
    CutsceneStart,
    CutsceneEnd,
    EncounterStart(Enemy, bool),
    EncountersStart(ArrayVec<Enemy, 6>, bool),
    EncounterEnd(Enemy, bool),
    EncountersEnd(ArrayVec<Enemy, 6>, bool),
    UnknownEnemyStart(ArrayVec<Unknown, 6>, bool),
    UnknownEnemyEnd(ArrayVec<Unknown, 6>, bool),
    PickedUpKeyItem(KeyItem),
    LostKeyItem(KeyItem),
}

pub struct EventHandler<'s> {
    pub settings: &'s mut Settings,
    actions: ArrayVec<Action, 8>,
    cutscenes: Delayed,
}

impl<'s> EventHandler<'s> {
    pub fn new(settings: &'s mut Settings) -> Self {
        Self {
            settings,
            actions: ArrayVec::new(),
            cutscenes: Delayed::default(),
        }
    }

    fn accept(&mut self, ev: Event) {
        match ev {
            Event::LoadStart => {}
            Event::LoadEnd => {}
            _ => log!("Event: {:?}", ev),
        }

        match ev {
            Event::LoadStart => self.act(Action::Pause),
            Event::LoadEnd => self.act(Action::Resume),
            Event::LevelChange { from, to } => {
                use Level::*;
                let split = match (from, to) {
                    (HomeWorld, ForbiddenCavern) => Split::Training,
                    (MountainTrail, ElderMistTrials) => Split::MountainTrails,
                    (HomeWorld, ArchivistRoom) => Split::Yeet,
                    (HomeWorld, Moorland) => Split::XtolsLanding,
                    (Moorland, HomeWorld) => Split::SolarRain,
                    (CoralCascade, HomeWorld) => Split::ChoralCascades,
                    (BriskOriginal, HomeWorld) => Split::Boat,
                    (Docks, HomeWorld) => Split::WraithIslandDocks,
                    (CursedWood, Lucent) => Split::CursedWoods,
                    (FloodedGraveyard, Lucent) => Split::EnchantedScarf,
                    (BriskDestroyed, Peninsula) => Split::BattleOfBrisk,
                    (BriskRebuilt, HomeWorld) => Split::Ship,
                    (HomeWorld, Mirth) => Split::BuildMirth,
                    (Mirth, ArchivistRoom) => Split::Mirth,
                    (HomeWorld, WaterTemple) => Split::ShoppingConches,
                    (ArchivistRoom, GlacialPeak) => Split::Antsudlo,
                    (GlacialPeak, ArchivistRoom) => Split::SignetOfclarity,
                    (Vespertine, HomeWorld) => Split::BackToMirth,
                    (MesaHike, HomeWorld) => Split::MesaHike,
                    (BambooCreek, HomeWorld) => Split::BambooCreek,
                    (SongShroomMarsh, HomeWorld) => Split::SongshroomMarsh,
                    (SkywardShrine, HomeWorld) => Split::SkywardShrine,
                    (SkyGiantsVillage, HomeWorld) => Split::Council,
                    (Skyland, StormCallerIsland) => Split::AirElemental,
                    (Mooncradle, SkyGiantsVillage) => Split::RIPGarl,
                    (SeraisWorld, Repine) => Split::DerelictFactory,
                    (Repine, SeraisWorld) => Split::Repine,
                    (CeruleanExpanse, LostOnesHamlet) => Split::CeruleanExpanse,
                    (SeraisWorld, SacrosanctSpires) => Split::LeavingforSpires,
                    (EstristaesLookout, SeraisWorld) => Split::JustKickIt,
                    (HomeWorld, WizardLab) => {
                        self.cutscenes.set(2, Split::Brisk);
                        return;
                    }
                    (FleshmancersLair, WorldEeater) => {
                        self.cutscenes.set(1, Split::WorldEater);
                        return;
                    }
                    _ => return,
                };
                self.act(Action::Split(split));
            }
            Event::CutsceneStart => {
                let split = match self.cutscenes.tick() {
                    Some(split) => split,
                    None => return,
                };
                self.act(Action::Split(split));
            }
            Event::CutsceneEnd => {}
            Event::EncounterStart(enemy, _boss) => {
                use Enemy::*;
                let split = match enemy {
                    Wyrd => Split::Tutorial,
                    Bossslug => Split::ForbiddenCavern,
                    ElderMist => Split::ElderMistTrials,
                    Salamander => Split::WindMineTunnels,
                    ChromaticApparition => Split::DemoWizardLab,
                    Duke => Split::FloodedGraveyard,
                    Romaya => Split::RopeDart,
                    BotanicalHorror => Split::Garden,
                    DwellerOfWoe => Split::DwellerOfWoeP1,
                    Stormcaller => Split::ThreeTowers,
                    DwellerOfTorment => Split::TormentPeak,
                    LeafMonster => Split::AutumnHills,
                    Toadcano => Split::Volcano,
                    Guardian => Split::SeaofStars,
                    Meduso => Split::LostOnesHamlet,
                    LeJugg => Split::FleshmancersLair,
                    PhaseReaper => Split::NolanSimulator,
                    Elysandarelle1 => Split::FFVIISimulator,
                    Malkomud | Malkomount | BonePile | FleshPile | BottomFlower | TopFlower
                    | BrugavesAlly | ErlynaAlly | One | Two | Three | Four | Erlina | Brugaves
                    | DwellerOfStrife1 | DwellerOfStrife2 | Tail | Hydralion | Casugin
                    | Abstarak | Rachater | Repeater | Catalyst | Tentacle | DwellerOfDread
                    | Elysandarelle2 => return,
                };
                self.act(Action::Split(split));
            }
            Event::EncountersStart(enemies, boss) => {
                for enemy in enemies.clone() {
                    self.accept(Event::EncounterStart(enemy, boss));
                }
                use Enemy::*;
                let split = match enemies.as_slice() {
                    [One, Three] => Split::JunglePath,
                    [Two, Four] => Split::GlacialPeak,
                    [One, Two, Three, Four] => Split::Watchmaker,
                    [Casugin, Abstarak, Rachater] => Split::HuntingFields,
                    [Repeater, Repeater] => Split::SkyBase,
                    [Tentacle, Tentacle] => Split::InfiniteAbyss,
                    _ => return,
                };
                self.act(Action::Split(split));
            }
            Event::EncounterEnd(enemy, boss) => {
                if boss {
                    self.act(Action::SplitBoss);
                }
                use Enemy::*;
                let split = match enemy {
                    Wyrd => Split::Wyrd,
                    Bossslug => Split::Bossslug,
                    ElderMist => Split::ElderMist,
                    Salamander => Split::Rockie,
                    Malkomud => Split::Malkomud,
                    ChromaticApparition => Split::Chromatic,
                    Duke => Split::Duke,
                    Romaya => Split::Romaya,
                    BotanicalHorror => Split::BigPlant,
                    DwellerOfWoe => Split::DwellerOfWoe,
                    Stormcaller => Split::Stormcaller,
                    DwellerOfTorment => Split::DwellerOfTorment,
                    LeafMonster => Split::LeafMonster,
                    DwellerOfStrife1 => Split::DwellerOfStrifeP1,
                    DwellerOfStrife2 => Split::DwellerOfStrifeP2,
                    Hydralion => Split::Hydralion,
                    Toadcano => Split::Toadcano,
                    Guardian => Split::Guardian,
                    Meduso => Split::Meduso,
                    Catalyst => Split::Catalyst,
                    DwellerOfDread => Split::DwellerOfDread,
                    LeJugg => Split::LeJugg,
                    PhaseReaper => Split::Reaper,
                    Elysandarelle1 => Split::ElysandarelleP1,
                    Elysandarelle2 => Split::ElysandarelleP2,
                    Malkomount | BonePile | FleshPile | BottomFlower | TopFlower | BrugavesAlly
                    | ErlynaAlly | One | Two | Three | Four | Erlina | Brugaves | Tail
                    | Casugin | Abstarak | Rachater | Repeater | Tentacle => return,
                };
                self.act(Action::Split(split));
            }
            Event::EncountersEnd(enemies, boss) => {
                if boss {
                    self.act(Action::SplitBoss);
                }
                for enemy in enemies.clone() {
                    self.accept(Event::EncounterEnd(enemy, false));
                }
                use Enemy::*;
                let split = match enemies.as_slice() {
                    [One, Three] => Split::OneThree,
                    [Two, Four] => Split::TwoFour,
                    [One, Two, Three, Four] => Split::OneTwoThreeFour,
                    [Erlina, Brugaves] => Split::ErlynaAndBrugaves,
                    [Casugin, Abstarak, Rachater] => Split::Triumvirate,
                    _ => return,
                };
                self.act(Action::Split(split));
            }
            Event::UnknownEnemyStart(..) => {}
            Event::UnknownEnemyEnd(..) => {}
            Event::PickedUpKeyItem(key_item) => {
                use KeyItem::*;
                let split = match key_item {
                    Graplou => Split::NecromancerssLair,
                    Map => {
                        self.cutscenes.set(1, Split::Map);
                        return;
                    }
                    MasterGhostSandwich | Seashell => return,
                };
                self.act(Action::Split(split));
            }
            Event::LostKeyItem(key_item) => {
                use KeyItem::*;
                let split = match key_item {
                    MasterGhostSandwich => Split::Cooking,
                    Seashell => Split::SacredGrove,
                    Graplou | Map => return,
                };
                self.act(Action::Split(split));
            }
        }
    }

    fn act(&mut self, action: Action) {
        if self.filter(&action) {
            let _ = self.actions.try_push(action);
        }
    }

    fn filter(&mut self, action: &Action) -> bool {
        match action {
            Action::SplitBoss => {
                if self.settings.split == false {
                    log!("Skipping encounter_boss: Disabled in settings");
                    return false;
                }
            }
            Action::Split(split) => {
                if split.is_enabled(self.settings) == false {
                    log!("Skipping {:?}: Disabled in settings", split);
                    return false;
                }
            }
            Action::Pause => {
                if self.settings.remove_loads == false {
                    return false;
                }
            }
            Action::Resume => {
                if self.settings.remove_loads == false {
                    return false;
                }
            }
            Action::StartCharacter | Action::StartRelic => {}
        };
        return true;
    }
}

impl Running {
    pub fn act(&mut self, handler: &mut EventHandler<'_>, action: Action) {
        if handler.filter(&action) {
            self.act_internal(action);
        }
    }

    fn act_internal(&mut self, action: Action) {
        match action {
            Action::SplitBoss => {
                log!("Splitting: encounter_boss");
                if cfg!(not(dummy)) {
                    timer::split();
                }
            }
            Action::Split(split) => {
                if self.seen.insert(&split) {
                    log!("Splitting: {:?}", split);
                    if cfg!(not(dummy)) {
                        timer::split();
                    }
                } else {
                    log!("Skipping {:?}: Duplicate split", split);
                }
            }
            Action::Pause => {
                self.paused = true;
                if cfg!(not(dummy)) {
                    timer::pause_game_time();
                }
            }
            Action::Resume => {
                self.paused = false;
                if cfg!(not(dummy)) {
                    timer::resume_game_time();
                }
            }
            Action::StartCharacter | Action::StartRelic => {}
        }
    }

    pub fn tick(&mut self, handler: &mut EventHandler<'_>, process: &Process, data: &Data) {
        self.load_removal(handler, process, data);
        self.check_encounter(handler, process, data);
        self.check_level(handler, process, data);
        self.check_key_items(handler, process, data);

        for action in handler.actions.drain(..) {
            self.act_internal(action);
        }
    }

    fn load_removal(&mut self, handler: &mut EventHandler<'_>, process: &Process, data: &Data) {
        if handler.settings.remove_loads == false {
            return;
        }
        if handler.settings.custom_load_remover == false
            && let Some(SpeedrunRelic::Active(time)) = data.speedrun_time(process)
        {
            let relic_time = self.relic_time.update_infallible(time);
            match (relic_time.increased(), self.paused) {
                // relic and lrt are both running
                (true, false) => {}
                // relic and lrt are both paused
                (false, true) => {}
                // relic paused, lrt is running, need to pause lrt
                (false, false) => handler.accept(Event::LoadStart),
                // relic is running, lrt is paused, need to unpause lrt
                (true, true) => handler.accept(Event::LoadEnd),
            }
        } else if handler.settings.custom_load_remover == true {
            match self.loading.update(data.is_loading(process)) {
                Some(l) if l.changed_to(&false) => handler.accept(Event::LoadEnd),
                Some(l) if l.changed_to(&true) => handler.accept(Event::LoadStart),
                _ => {}
            }
        }
    }

    fn check_encounter(&mut self, handler: &mut EventHandler<'_>, process: &Process, data: &Data) {
        match (&mut self.encounter, data.encounter_done(process)) {
            // we were in an encounter, and now it's done
            (Some(start), Some(true)) => {
                Self::split_enemies(
                    &start.enemies,
                    handler,
                    start.boss,
                    Event::EncounterEnd,
                    Event::EncountersEnd,
                    Event::UnknownEnemyEnd,
                );
                self.encounter = None;
            }
            // we weren't in an encounter, and now we are
            (None, Some(false)) => {
                let Some(mut enemies) = data.encounter_data(process) else {
                    return;
                };
                enemies.enemies.sort_unstable();
                Self::split_enemies(
                    &enemies.enemies,
                    handler,
                    enemies.boss,
                    Event::EncounterStart,
                    Event::EncountersStart,
                    Event::UnknownEnemyStart,
                );
                self.encounter = Some(enemies)
            }
            // we thought we were in an encounter, but we're not
            (Some(_), None) => self.encounter = None,
            // we are in an encounter and it's still going
            (Some(_), Some(false)) => {}
            // we aren't in an encounter and there isn't one or it just finished
            (None, None | Some(true)) => {}
        }
    }

    fn check_level(&mut self, handler: &mut EventHandler<'_>, process: &Process, data: &Data) {
        let progression = data.progression(process);

        let cutscene = self.cutscene.update_infallible(progression.in_cutscene);
        if cutscene.changed_to(&true) {
            handler.accept(Event::CutsceneStart);
        } else if cutscene.changed_to(&false) {
            handler.accept(Event::CutsceneEnd);
        }

        let Some(level) = self.level.update(progression.level).filter(|o| o.changed()) else {
            return;
        };
        handler.accept(Event::LevelChange {
            from: level.old,
            to: level.current,
        });
    }

    fn check_key_items(&mut self, handler: &mut EventHandler<'_>, process: &Process, data: &Data) {
        let Some(owned_items_ptr) = data.owned_items(process) else {
            return;
        };
        let Some(owned_items) = owned_items_ptr.read(process) else {
            return;
        };
        let first = self.number_of_items.pair.is_none();
        let owned = self.number_of_items.update_infallible(owned_items.size);

        if first == false && owned.changed() == false {
            return;
        }

        let generation = self.inventory_generation.saturating_add(1);
        self.inventory_generation = generation;

        let Some(owned_items) = owned_items_ptr.iter(process) else {
            return;
        };
        for (item, _amount) in owned_items {
            if let Some(item) = item.guid.chars(process).and_then(KeyItem::resolve) {
                let idx = usize::from(u8::from(item));
                match self.key_items[idx] {
                    (u32::MAX, u32::MAX) => self.key_items[idx] = (generation, generation),
                    (_, ref mut current) => *current = generation,
                }
            }
        }

        for (item, &(insert, current)) in self.key_items.iter().enumerate() {
            if insert == u32::MAX {
                continue;
            }
            let Ok(item) = u8::try_from(item) else {
                unreachable!();
            };
            let Ok(item) = KeyItem::try_from_primitive(item) else {
                continue;
            };

            if current == generation {
                if insert == current {
                    handler.accept(Event::PickedUpKeyItem(item));
                }
            } else {
                handler.accept(Event::LostKeyItem(item));
            }
        }
    }

    fn split_enemies(
        enemies: &[AnyEnemy],
        handler: &mut EventHandler<'_>,
        boss_encounter: bool,
        from_single: impl FnOnce(Enemy, bool) -> Event,
        from_multiple: impl FnOnce(ArrayVec<Enemy, 6>, bool) -> Event,
        from_unknown: impl FnOnce(ArrayVec<Unknown, 6>, bool) -> Event,
    ) {
        let mut known = ArrayVec::new();
        let mut unknown = ArrayVec::new();

        for enemy in enemies {
            match enemy {
                AnyEnemy::Known(enemy) => known.push(*enemy),
                #[allow(clippy::unit_arg)]
                AnyEnemy::Unknown(name) => unknown.push(*name),
            }
        }

        match known.as_slice() {
            [] => {}
            [known] => {
                handler.accept(from_single(*known, boss_encounter));
            }
            _ => {
                handler.accept(from_multiple(known, boss_encounter));
            }
        }

        if unknown.is_empty() == false {
            handler.accept(from_unknown(unknown, boss_encounter));
        }
    }
}

#[derive(Debug, Default)]
struct Delayed {
    delay: Option<NonZeroU32>,
    split: Option<Split>,
}

impl Delayed {
    fn set(&mut self, amount: u32, split: Split) {
        self.delay = NonZeroU32::new(amount);
        self.split = Some(split);
    }

    fn tick(&mut self) -> Option<Split> {
        let n = self.delay?;
        self.delay = NonZeroU32::new(n.get() - 1);
        self.delay
            .is_none()
            .then(|| self.split.take().expect("double tick"))
    }
}
