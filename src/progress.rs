#[cfg(debugger)]
use crate::data::{Activity, EnemyEncounter, Level, PlayTime};
use crate::data::{Data, GameStart};
use crate::Action;
#[cfg(debugger)]
use ahash::{HashSet, HashSetExt};
#[cfg(debugger)]
use asr::timer;
use asr::watcher::Watcher;

pub struct Progress {
    loading: Watcher<bool>,
    in_encounter: bool,
    #[cfg(debugger)]
    debug: DebugProgress,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            loading: Watcher::new(),
            in_encounter: false,
            #[cfg(debugger)]
            debug: DebugProgress::new(),
        }
    }

    pub(crate) fn start(&mut self, data: &mut Data<'_>) -> Option<Action> {
        matches!(data.game_start(), GameStart::JustStarted).then_some(Action::Start)
    }

    pub(crate) async fn act(&mut self, data: &mut Data<'_>) -> Option<Action> {
        let loading;

        #[cfg(debugger)]
        {
            loading = self.debug.check_progression(data).await;
        }
        #[cfg(not(debugger))]
        {
            loading = data.progress().await.map(|o| o.is_loading)
        }

        match self.loading.update(loading) {
            Some(l) if l.changed_to(&false) => Some(Action::Resume),
            Some(l) if l.changed_to(&true) => Some(Action::Pause),
            _ => self
                .check_encounter(data)
                .await
                .and_then(|o| o.then_some(Action::Split)),
        }
    }

    async fn check_encounter(&mut self, data: &mut Data<'_>) -> Option<bool> {
        #[cfg(debugger)]
        self.debug.log_enemies(data).await;

        if self.in_encounter {
            match data.encounter().await {
                Some(enc) if enc.done => {
                    // #[cfg(debugger)]
                    // data.dump_current_encounter().await;
                    self.in_encounter = false;
                    return Some(enc.boss);
                }
                Some(_) =>
                {
                    #[cfg(debugger)]
                    self.debug.dump_current_hp_levels(data).await
                }
                None => {
                    self.in_encounter = false;
                }
            }
        } else {
            match data.encounter().await {
                Some(enc) if !enc.done => {
                    // #[cfg(debugger)]
                    // data.dump_current_encounter().await;
                    self.in_encounter = true;
                }
                _ => {}
            }
        }
        Some(false)
    }
}

#[cfg(debugger)]
struct DebugProgress {
    play_time: Watcher<PlayTime>,
    activity: Watcher<Activity>,
    in_cutscene: Watcher<bool>,
    current_level: Watcher<Level>,
    previous_level: Watcher<Level>,
    seen_enemies: HashSet<Enemy>,
}

#[cfg(debugger)]
impl DebugProgress {
    fn new() -> Self {
        Self {
            play_time: Watcher::new(),
            activity: Watcher::new(),
            in_cutscene: Watcher::new(),
            current_level: Watcher::new(),
            previous_level: Watcher::new(),
            seen_enemies: HashSet::new(),
        }
    }

    async fn check_progression(&mut self, data: &mut Data<'_>) -> Option<bool> {
        if let Some(progress) = data.progress().await {
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
                log!("{:?}", activity.current);
                timer::set_variable("activity", &activity.current.name);
                timer::set_variable("activity_id", activity.current.id.as_str());
            }

            let first = self.previous_level.pair.is_none();
            let level = self.previous_level.update_infallible(progress.prev_level());
            if first || level.changed() {
                log!("Previous {:?}", level.current);
            }

            let first = self.current_level.pair.is_none();
            let level = self
                .current_level
                .update_infallible(progress.current_level());
            if first || level.changed() {
                log!("Current {:?}", level.current);
                timer::set_variable("level", &level.name);
                timer::set_variable("level_id", level.id.as_str());
            }

            let in_cutscene = self.in_cutscene.update_infallible(progress.is_in_cutscene);
            if in_cutscene.changed_from_to(&false, &true) {
                log!("Cutscene started");
            } else if in_cutscene.changed_from_to(&true, &false) {
                log!("Cutscene stopped");
            }

            Some(progress.is_loading)
        } else {
            None
        }
    }

    async fn log_enemies(&mut self, data: &mut Data<'_>) {
        if let Some(_) = data.encounter().await {
            if let Some(encounter) = data.deep_resolve_encounter().await {
                let mut enc = Encounter::default();
                let mut nmy = Enemy::default();
                for enemy in encounter.enemies() {
                    match enemy {
                        EnemyEncounter::General(encounter) => {
                            enc.boss = encounter.boss;
                            enc.achievement = encounter.has_achievement;
                            nmy.encounter = enc;
                        }
                        EnemyEncounter::Enemy(enemy) => {
                            nmy.id = enemy.id.to_owned();
                            nmy.name = enemy.name.to_owned()
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

    async fn dump_current_hp_levels(&mut self, data: &mut Data<'_>) {
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

        if let Some(enc) = data.deep_resolve_encounter().await {
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
                .filter_map(|o| o)
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
struct Enemy {
    id: String,
    name: String,
    hp: i32,
    level: i32,
    at: i32,
    mat: i32,
    de: i32,
    mde: i32,
    dmg: String,
    encounter: Encounter,
}

impl PartialEq for Enemy {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Ord for Enemy {
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

impl PartialOrd for Enemy {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::hash::Hash for Enemy {
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

impl std::fmt::Debug for Enemy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("")
            .field("boss", &self.encounter.boss)
            .field(
                "name",
                &if self.name.is_empty() {
                    "TODO"
                } else {
                    self.name.as_str()
                },
            )
            .field("id", &self.id)
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
