use std::num::NonZeroU32;

#[cfg(debugger)]
use crate::data::{Activity, EnemyEncounter, LevelData, PlayTime};
use crate::{
    data::Data,
    game::{Enemy, Event, Game, KeyItem, Level},
};
use crate::{Action, Split};
#[cfg(debugger)]
use ahash::{HashSet, HashSetExt};
use asr::{arrayvec::ArrayVec, msg};
#[cfg(debugger)]
use asr::{timer, watcher::Watcher};

pub struct Progress {
    game: Game,
    actions: ArrayVec<Action, 4>,
    cutscenes: Delayed,
    #[cfg(debugger)]
    debug: DebugProgress,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            game: Game::new(),
            actions: ArrayVec::new(),
            cutscenes: Delayed::default(),
            #[cfg(debugger)]
            debug: DebugProgress::new(),
        }
    }

    pub(crate) fn actions(&mut self) -> impl Iterator<Item = Action> + '_ {
        self.handle_events();
        self.actions.drain(..)
    }

    pub fn not_running(&mut self, data: &mut Data<'_>) {
        #[cfg(debugger)]
        self.debug.check_progression(data);

        self.game.not_running(data);
    }

    pub fn running(&mut self, data: &mut Data<'_>) {
        #[cfg(debugger)]
        self.debug.check_progression(data);

        // #[cfg(debugger)]
        // for item in data.check_for_changed_key_items() {
        //     match item {
        //         crate::data::Change::PickedUp(item) => {
        //             log!("picked up a new key item: {} ({})", item.name, item.id);
        //         }
        //         crate::data::Change::Lost(item) => {
        //             log!("lost a key item: {} ({})", item.name, item.id);
        //         }
        //     }
        // }

        #[cfg(debugger)]
        self.debug.log_enemies(data);

        #[cfg(debugger)]
        self.debug.log_encounter_data(data);

        self.game.running(data);
    }

    fn handle_events(&mut self) {
        let cutscenes = &mut self.cutscenes;
        for event in self.game.events().flat_map(Event::expand) {
            macro_rules! unhandled {
                () => {{
                    msg!("Unhandled event: {:?}", event);
                    continue;
                }};
            }

            let action = match event {
                Event::GameStarted => Action::Start,
                Event::LoadStart => Action::Pause,
                Event::LoadEnd => Action::Resume,
                Event::PauseStart => Action::GamePause,
                Event::PauseEnd => Action::GameResume,
                Event::LevelChange { from, to } => {
                    use Level::*;
                    match (from, to) {
                        (HomeWorld, ForbiddenCavern) => Action::Split(Split::Training),
                        (MountainTrail, ElderMistTrials) => Action::Split(Split::MountainTrails),
                        (HomeWorld, ArchivistRoom) => Action::Split(Split::Yeet),
                        (HomeWorld, Moorland) => Action::Split(Split::XtolsLanding),
                        (Moorland, HomeWorld) => Action::Split(Split::SolarRain),
                        (CoralCascade, HomeWorld) => Action::Split(Split::ChoralCascades),
                        (BriskOriginal, HomeWorld) => Action::Split(Split::Boat),
                        (Docks, HomeWorld) => Action::Split(Split::WraitIslandDocks),
                        (CursedWood, Lucent) => Action::Split(Split::CursedWoods),
                        (FloodedGraveyard, Lucent) => Action::Split(Split::EnchantedScarf),
                        (BriskDestroyed, Peninsula) => Action::Split(Split::BattleOfBrisk),
                        (BriskRebuilt, HomeWorld) => Action::Split(Split::Ship),
                        (HomeWorld, Mirth) => Action::Split(Split::BuildMirth),
                        (Mirth, ArchivistRoom) => Action::Split(Split::Mirth),
                        (HomeWorld, WaterTemple) => Action::Split(Split::ShoppingConches),
                        (ArchivistRoom, GlacialPeak) => Action::Split(Split::Antsudlo),
                        (GlacialPeak, ArchivistRoom) => Action::Split(Split::SignetOfclarity),
                        (Vespertine, HomeWorld) => Action::Split(Split::BackToMirth),
                        (MesaHike, HomeWorld) => Action::Split(Split::MesaHike),
                        (BambooCreek, HomeWorld) => Action::Split(Split::BambooCreek),
                        (SongShroomMarsh, HomeWorld) => Action::Split(Split::SongshroomMarsh),
                        (SkywardShrine, HomeWorld) => Action::Split(Split::SkywardShrine),
                        (SkyGiantsVillage, HomeWorld) => Action::Split(Split::Council),
                        (Skyland, StormCallerIsland) => Action::Split(Split::AirElemental),
                        (Mooncradle, SkyGiantsVillage) => Action::Split(Split::RIPGarl),
                        (SeraisWorld, Repine) => Action::Split(Split::DerelictFactory),
                        (Repine, SeraisWorld) => Action::Split(Split::Repine),
                        (CeruleanExpanse, LostOnesHamlet) => Action::Split(Split::CeruleanExpense),
                        (SeraisWorld, SacrosanctSpires) => Action::Split(Split::LeavingforSpires),
                        (EstristaesLookout, SeraisWorld) => Action::Split(Split::JustKickIt),
                        (HomeWorld, WizardLab) => {
                            cutscenes.set(2, Action::Split(Split::Brisk));
                            continue;
                        }
                        (_, WorldEeater) => {
                            cutscenes.set(1, Action::Split(Split::WorldEater));
                            continue;
                        }
                        _ => unhandled!(),
                    }
                }
                Event::CutsceneStart => match cutscenes.tick() {
                    Some(action) => action,
                    None => continue,
                },
                Event::EncounterStart(enemy) => {
                    use Enemy::*;
                    match enemy {
                        Wyrd => Action::Split(Split::Tutorial),
                        Bossslug => Action::Split(Split::ForbiddenCavern),
                        ElderMist => Action::Split(Split::ElderMistTrials),
                        Salamander => Action::Split(Split::WindMineTunnels),
                        ChromaticApparition => Action::Split(Split::DemoWizardLab),
                        Duke => Action::Split(Split::FloodedGraveyard),
                        Romaya => Action::Split(Split::RopeDart),
                        BotanicalHorror => Action::Split(Split::Garden),
                        DwellerOfWoe => Action::Split(Split::DwellerOfWoeP1),
                        Stormcaller => Action::Split(Split::ThreeTowers),
                        DwellerOfTorment => Action::Split(Split::TormentPeak),
                        LeafMonster => Action::Split(Split::AutumnHills),
                        Toadcano => Action::Split(Split::Volcano),
                        Guardian => Action::Split(Split::SeaofStars),
                        Meduso => Action::Split(Split::LostOnesHamlet),
                        LeJugg => Action::Split(Split::FleshmancersLair),
                        PhaseReaper => Action::Split(Split::NolanSimulator),
                        Elysandarelle1 => Action::Split(Split::FFVIISimulator),
                        Malkomud | Malkomount | BonePile | FleshPile | BottomFlower | TopFlower
                        | BrugavesAlly | ErlynaAlly | One | Two | Three | Four | Erlina
                        | Brugaves | DwellerOfStrife1 | DwellerOfStrife2 | Tail | Hydralion
                        | Casugin | Abstarak | Rachater | Repeater | Catalyst | Tentacle
                        | DwellerOfDread | Elysandarelle2 => {
                            unhandled!()
                        }
                    }
                }
                Event::EncountersStart(ref es) => {
                    use Enemy::*;
                    match es.as_slice() {
                        [One, Three] => Action::Split(Split::JunglePath),
                        [Two, Four] => Action::Split(Split::GlacialPeak),
                        [One, Two, Three, Four] => Action::Split(Split::Watchmaker),
                        [Casugin, Abstarak, Rachater] => Action::Split(Split::HuntingFields),
                        [Repeater, Repeater] => Action::Split(Split::SkyBase),
                        [Tentacle, Tentacle] => Action::Split(Split::InfiniteAbyss),
                        _ => unhandled!(),
                    }
                }
                Event::EncounterEnd(enemy) => {
                    use Enemy::*;
                    match enemy {
                        Wyrd => Action::Split(Split::Wyrd),
                        Bossslug => Action::Split(Split::Bossslug),
                        ElderMist => Action::Split(Split::ElderMist),
                        Salamander => Action::Split(Split::Rockie),
                        Malkomud => Action::Split(Split::Malkomud),
                        ChromaticApparition => Action::Split(Split::Chromatic),
                        Duke => Action::Split(Split::Duke),
                        Romaya => Action::Split(Split::Romaya),
                        BotanicalHorror => Action::Split(Split::BigPlant),
                        DwellerOfWoe => Action::Split(Split::DwellerOfWoe),
                        Stormcaller => Action::Split(Split::Stormcaller),
                        DwellerOfTorment => Action::Split(Split::DwellerOfTorment),
                        LeafMonster => Action::Split(Split::LeafMonster),
                        DwellerOfStrife1 => Action::Split(Split::DwellerOfStrifeP1),
                        DwellerOfStrife2 => Action::Split(Split::DwellerOfStrifeP2),
                        Hydralion => Action::Split(Split::Hydralion),
                        Toadcano => Action::Split(Split::Toadcano),
                        Guardian => Action::Split(Split::Guardian),
                        Meduso => Action::Split(Split::Meduso),
                        Catalyst => Action::Split(Split::Catalyst),
                        DwellerOfDread => Action::Split(Split::DwellerOfDread),
                        LeJugg => Action::Split(Split::LeJugg),
                        PhaseReaper => Action::Split(Split::Reaper),
                        Elysandarelle1 => Action::Split(Split::ElysandarelleP1),
                        Elysandarelle2 => Action::Split(Split::ElysandarelleP2),
                        Malkomount | BonePile | FleshPile | BottomFlower | TopFlower
                        | BrugavesAlly | ErlynaAlly | One | Two | Three | Four | Erlina
                        | Brugaves | Tail | Casugin | Abstarak | Rachater | Repeater | Tentacle => {
                            unhandled!()
                        }
                    }
                }
                Event::EncountersEnd(ref es) => {
                    use Enemy::*;
                    match es.as_slice() {
                        [One, Three] => Action::Split(Split::OneThree),
                        [Two, Four] => Action::Split(Split::TwoFour),
                        [One, Two, Three, Four] => Action::Split(Split::OneTwoThreeFour),
                        [Erlina, Brugaves] => Action::Split(Split::ErlynaAndBrugaves),
                        [Casugin, Abstarak, Rachater] => Action::Split(Split::Triumvirate),
                        _ => unhandled!(),
                    }
                }
                Event::PickedUpKeyItem(KeyItem::Graplou) => Action::Split(Split::NecromancerssLair),
                Event::PickedUpKeyItem(KeyItem::Map) => {
                    cutscenes.set(1, Action::Split(Split::Map));
                    continue;
                }
                Event::LostKeyItem(KeyItem::MasterGhostSandwich) => Action::Split(Split::Cooking),
                Event::LostKeyItem(KeyItem::Seashell) => Action::Split(Split::SacredGrove),
                _ => unhandled!(),
            };
            self.actions.push(action);
        }
    }
}

impl Event {
    fn expand(self) -> ArrayVec<Self, 7> {
        match self {
            Event::EncountersStart(es) => es
                .clone()
                .into_iter()
                .map(Event::EncounterStart)
                .chain(Some(Event::EncountersStart(es)))
                .collect(),
            Event::EncountersEnd(es) => es
                .clone()
                .into_iter()
                .map(Event::EncounterEnd)
                .chain(Some(Event::EncountersEnd(es)))
                .collect(),
            e => [e].into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Delayed {
    delay: Option<NonZeroU32>,
    action: Option<Action>,
}

impl Delayed {
    fn set(&mut self, amount: u32, action: Action) {
        self.delay = NonZeroU32::new(amount);
        self.action = Some(action);
    }

    fn tick(&mut self) -> Option<Action> {
        let n = self.delay?;
        self.delay = NonZeroU32::new(n.get() - 1);
        self.delay
            .is_none()
            .then(|| self.action.take().expect("double tick"))
    }
}

#[cfg(debugger)]
struct DebugProgress {
    in_encounter: bool,
    play_time: Watcher<PlayTime>,
    activity: Watcher<Activity>,
    in_cutscene: Watcher<bool>,
    current_level: Watcher<LevelData>,
    previous_level: Watcher<LevelData>,
    seen_enemies: HashSet<DebugEnemy>,
}

#[cfg(debugger)]
impl DebugProgress {
    fn new() -> Self {
        Self {
            in_encounter: false,
            play_time: Watcher::new(),
            activity: Watcher::new(),
            in_cutscene: Watcher::new(),
            current_level: Watcher::new(),
            previous_level: Watcher::new(),
            seen_enemies: HashSet::new(),
        }
    }

    fn check_progression(
        &mut self,
        data: &mut Data<'_>,
    ) -> Option<(bool, Option<crate::data::Level>)> {
        if let Some(progress) = data.progress() {
            let play_time = self.play_time.update_infallible(progress.play_time());
            if play_time.changed() {
                log!(
                    "this session={}, total={}",
                    play_time.current.session,
                    play_time.current.total
                );
                timer::set_game_time(play_time.current.total);
            }

            let first = self.activity.pair.is_none();
            let activity = self.activity.update_infallible(progress.activity());
            if first || activity.changed() {
                // log!("{:?}", activity.current);
                timer::set_variable("activity", &activity.current.name);
                timer::set_variable("activity_id", activity.current.id.as_str());
            }

            // let first = self.previous_level.pair.is_none();
            // let level = self.previous_level.update_infallible(progress.prev_level());
            // if first || level.changed() {
            //     log!("Previous {:?}", level.current);
            // }

            let first = self.current_level.pair.is_none();
            let level = self
                .current_level
                .update_infallible(progress.current_level());
            if first || level.changed() {
                // log!("Current {:?}", level.current);
                timer::set_variable("level", &level.name);
                timer::set_variable("level_id", level.id.as_str());
            }

            // let in_cutscene = self.in_cutscene.update_infallible(progress.is_in_cutscene);
            // if in_cutscene.changed_from_to(&false, &true) {
            //     log!("Cutscene started");
            // } else if in_cutscene.changed_from_to(&true, &false) {
            //     log!("Cutscene stopped");
            // }

            Some((progress.is_loading, progress.level))
        } else {
            None
        }
    }

    fn log_enemies(&mut self, data: &mut Data<'_>) {
        if data.encounter().is_some() {
            if let Some(encounter) = data.deep_resolve_encounter() {
                let mut enc = Encounter::default();
                let mut nmy = DebugEnemy::default();
                for enemy in encounter.enemies() {
                    match enemy {
                        EnemyEncounter::General(encounter) => {
                            enc.boss = encounter.boss;
                            enc.achievement = encounter.has_achievement;
                            nmy.encounter = enc;
                        }
                        EnemyEncounter::Enemy(enemy) => {
                            nmy.id = enemy.id.to_owned();
                            nmy.name = enemy.name.to_owned();
                            nmy.enemey = enemy.enemy;
                        }
                        EnemyEncounter::EnemyStats(enemy) => {
                            nmy.hp = enemy.max_hp as _;
                            nmy.level = enemy.level as _;
                            nmy.at = enemy.attack as _;
                            nmy.mat = enemy.magic_attack as _;
                            nmy.de = enemy.defense as _;
                            nmy.mde = enemy.magic_defense as _;
                        }
                        EnemyEncounter::EnemyMods(enemy) => {
                            nmy.dmg = format!("{:?}", EnemyEncounter::EnemyMods(enemy))
                                .trim_start_matches("|--mods: ")
                                .to_owned();
                            let enemy = std::mem::take(&mut nmy);
                            nmy.encounter = enc;
                            if !self.seen_enemies.contains(&enemy) {
                                log!("{:?}", enemy);
                                self.seen_enemies.insert(enemy);
                            }
                        }
                    }
                }
            }
        }
    }

    fn log_encounter_data(&mut self, data: &mut Data<'_>) -> Option<()> {
        if self.in_encounter {
            match data.encounter() {
                Some(enc) if enc.done => {
                    self.in_encounter = false;
                    return Some(());
                }
                Some(_) =>
                {
                    #[cfg(debugger)]
                    self.dump_current_hp_levels(data)
                }
                None => {
                    self.in_encounter = false;
                }
            }
        } else {
            match data.encounter() {
                Some(enc) if !enc.done => {
                    self.in_encounter = true;
                }
                _ => {}
            }
        }
        Some(())
    }

    fn dump_current_hp_levels(&mut self, data: &mut Data<'_>) {
        const KEYS: [(&str, &str, &str); 6] = [
            ("enemy_1", "enemy_1_id", "enemy_1_hp"),
            ("enemy_2", "enemy_2_id", "enemy_2_hp"),
            ("enemy_3", "enemy_3_id", "enemy_3_hp"),
            ("enemy_4", "enemy_4_id", "enemy_4_hp"),
            ("enemy_5", "enemy_5_id", "enemy_5_hp"),
            ("enemy_6", "enemy_6_id", "enemy_6_hp"),
        ];

        #[derive(Copy, Clone, Debug, Default)]
        struct Data<'a> {
            id: &'a str,
            name: &'a str,
            hp: u32,
        }

        if let Some(enc) = data.deep_resolve_encounter() {
            for (e, (name_key, id_key, hp_key)) in enc
                .enemies()
                .scan(Data::default(), |acc, e| {
                    Some(match e {
                        EnemyEncounter::Enemy(e) => {
                            acc.id = e.id;
                            acc.name = e.name;
                            None
                        }
                        EnemyEncounter::EnemyStats(e) => {
                            acc.hp = e.current_hp;
                            Some(core::mem::take(acc))
                        }
                        _ => None,
                    })
                })
                .flatten()
                .map(Some)
                .chain(core::iter::repeat(None))
                .zip(KEYS)
            {
                let name = e.map_or("", |o| o.name);
                let id = e.map_or("", |o| o.id);
                let hp = e.map_or(0, |o| o.hp);

                timer::set_variable(name_key, name);
                timer::set_variable(id_key, id);
                timer::set_variable_int(hp_key, hp);
            }
        }
    }
}

#[derive(Clone, Default, Eq)]
struct DebugEnemy {
    id: String,
    name: String,
    enemey: Option<crate::data::Enemy>,
    hp: i32,
    level: i32,
    at: i32,
    mat: i32,
    de: i32,
    mde: i32,
    dmg: String,
    encounter: Encounter,
}

impl PartialEq for DebugEnemy {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Ord for DebugEnemy {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.encounter
            .boss
            .cmp(&other.encounter.boss)
            .reverse()
            .then(self.name.cmp(&other.name))
            .then(self.id.cmp(&other.id))
            .then(self.hp.cmp(&other.hp))
            .then(self.encounter.achievement.cmp(&other.encounter.achievement))
            .then(self.level.cmp(&other.level))
            .then(self.at.cmp(&other.at))
            .then(self.mat.cmp(&other.mat))
            .then(self.de.cmp(&other.de))
            .then(self.mde.cmp(&other.mde))
            .then(self.dmg.cmp(&other.dmg))
    }
}

impl PartialOrd for DebugEnemy {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::hash::Hash for DebugEnemy {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.encounter.boss.hash(state);
        self.name.hash(state);
        self.id.hash(state);
        self.hp.hash(state);
        self.encounter.achievement.hash(state);
        self.level.hash(state);
        self.at.hash(state);
        self.mat.hash(state);
        self.de.hash(state);
        self.mde.hash(state);
        self.dmg.hash(state);
    }
}

impl std::fmt::Debug for DebugEnemy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("");

        d.field("boss", &self.encounter.boss).field(
            "name",
            &if self.name.is_empty() {
                "TODO"
            } else {
                self.name.as_str()
            },
        );

        if let Some(enemy) = &self.enemey {
            d.field("enemy", enemy);
        }

        d.field("id", &self.id)
            .field("hp", &self.hp)
            .field("achievement", &self.encounter.achievement)
            .field("level", &self.level)
            .field("at", &self.at)
            .field("mat", &self.mat)
            .field("de", &self.de)
            .field("mde", &self.mde)
            .field("dmg", &self.dmg)
            .finish()
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
struct Encounter {
    boss: bool,
    achievement: bool,
}

impl std::fmt::Debug for Encounter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Encounter")
            .field("boss", &self.boss)
            .field("achievement", &self.achievement)
            .finish()
    }
}
