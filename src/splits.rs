use core::num::NonZeroU32;

use crate::{
    Action, Settings,
    data::{Data, SpeedrunRelic},
    mapping::{AnyEnemy, Enemy, KeyItem, Level, Unknown},
    memory::EncounterData,
};
use asr::{Process, arrayvec::ArrayVec, timer, watcher::Watcher};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Split {
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

pub struct Running {
    relic_time: Watcher<f64>,
    paused: bool,
    loading: Watcher<bool>,
    encounter: Option<EncounterData>,
}

impl Running {
    pub fn new(time: f64) -> Self {
        let mut running = Self {
            relic_time: Watcher::new(),
            paused: false,
            loading: Watcher::new(),
            encounter: None,
        };
        let _ = running.relic_time.update_infallible(time);
        running
    }
}

// #[derive(Debug, Clone, Copy)]
// enum LoadRemoval {
//     Pause,
//     Resume,
// }

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
                if boss && self.settings.split {
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
                if boss && self.settings.split {
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
        // TODO: handle overflow, somehow
        let _ = self.actions.try_push(action);
    }
}

impl Running {
    pub fn tick(&mut self, handler: &mut EventHandler<'_>, process: &Process, data: &Data) {
        self.load_removal(handler, process, data);
        self.encounter_changes(handler, process, data);

        for action in handler.actions.drain(..) {
            self.act(handler.settings, action);
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

    fn encounter_changes(
        &mut self,
        handler: &mut EventHandler<'_>,
        process: &Process,
        data: &Data,
    ) {
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

    // fn check_encounter_done(&mut self, process: &Process, data: &Data) -> bool {
    //     match self.encounter {
    //         Some(enc) => match data.resolve_encounter(process, enc) {
    //             Some(enc) if enc.done => {
    //                 self.encounter = None;
    //                 return true;
    //             }
    //             Some(_) => {}
    //             None => {
    //                 self.encounter = None;
    //             }
    //         },
    //         None => {
    //             let Some((address, encounter)) = data.encounter(process) else {
    //                 return false;
    //             };
    //             if encounter.boss && !encounter.done {
    //                 self.encounter = Some(address);
    //             }
    //         }
    //     };
    //     return false;
    // }

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

    // fn level_changes(&mut self, data: &mut Data<'_>) -> Option<()> {
    //     let progression = data.current_progression()?;

    //     let loading = self.loading.update_infallible(progression.is_loading);
    //     if loading.changed_to(&true) {
    //         self.events.push(Event::LoadStart);
    //     } else if loading.changed_to(&false) {
    //         self.events.push(Event::LoadEnd);
    //     }

    //     let cutscene = self.cutscene.update_infallible(progression.is_in_cutscene);
    //     if cutscene.changed_to(&true) {
    //         self.events.push(Event::CutsceneStart);
    //     } else if cutscene.changed_to(&false) {
    //         self.events.push(Event::CutsceneEnd);
    //     }

    //     let level = self
    //         .level
    //         .update(progression.level)
    //         .filter(|o| o.changed())?;

    //     self.events.push(Event::LevelChange {
    //         from: level.old,
    //         to: level.current,
    //     });

    //     Some(())
    // }

    // fn key_item_changes(&mut self, process: &Process, data: &Data) {
    //     for item in data.key_item_changes() {
    //         let event = match item {
    //             Change::PickedUp(item) => Event::PickedUpKeyItem(item),
    //             Change::Lost(item) => Event::LostKeyItem(item),
    //         };
    //         self.events.push(event);
    //     }
    // }

    pub fn act(&mut self, settings: &Settings, action: Action) {
        match action {
            Action::SplitBoss if settings.split => {
                log!("Splitting");
                timer::split();
            }
            Action::Split(split) => {
                log!("Splitting: {:?}", split);
                // timer::split();
            }
            Action::Pause if settings.remove_loads => {
                self.paused = true;
                timer::pause_game_time();
            }
            Action::Resume if settings.remove_loads => {
                self.paused = false;
                timer::resume_game_time();
            }
            _otherwise => {}
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
