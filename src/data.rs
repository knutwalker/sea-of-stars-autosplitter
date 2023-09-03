#[cfg(debugger)]
use core::fmt;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use asr::{
    arrayvec::ArrayString,
    game_engine::unity::il2cpp::{Class, Image, Module, Version},
    Address, Address64, Process,
};
use bytemuck::AnyBitPattern;

#[cfg(debugger)]
use csharp_mem::{CSString, List, Map, Set};
use csharp_mem::{MemReader, Pointer};

pub struct Data<'a> {
    process: &'a Process,
    module: Module,
    image: Image,
    char_select: CharacterSelectionScreenBinding,
    combat: Singleton<CombatManagerBinding>,
    encounter: EncounterBinding,
    title_screen: LateInit<Singleton<TitleSequenceManagerBinding>>,
    progression: LateInit<Progression>,
    #[cfg(debugger)]
    combat2: LateInit<Combat>,
    #[cfg(debugger)]
    inventory: LateInit<inventory::Inventory>,
}

pub enum GameState {
    NotStarted,
    JustStarted,
    AlreadyRunning,
}

impl Data<'_> {
    pub async fn game_start(&mut self) -> GameState {
        let title_screen = self
            .title_screen
            .try_get(
                self.process,
                &self.module,
                &self.image,
                TitleSequenceManagerBinding::new,
            )
            .await;

        let title_screen =
            title_screen.and_then(|o| TitleSequenceManagerBinding::resolve(o, self.process));

        let char_select = title_screen.and_then(|o| {
            self.char_select
                .read(self.process, o.selection_screen.into())
                .ok()
        });

        if let Some(char_select) = char_select {
            if char_select.selected {
                GameState::JustStarted
            } else {
                GameState::NotStarted
            }
        } else {
            GameState::AlreadyRunning
        }
    }

    pub async fn progress(&mut self) -> Option<CurrentProgress> {
        let progression = self
            .progression
            .try_get(self.process, &self.module, &self.image, Progression::new)
            .await?;
        progression.get_progress(self.process)
    }

    pub fn encounter(&self) -> Option<(Address64, Encounter)> {
        let combat = CombatManagerBinding::resolve(&self.combat, self.process)?;
        let encounter = combat
            .encounter
            .resolve_with((self.process, &self.encounter))?;
        Some((combat.encounter.address_value().into(), encounter))
    }

    #[cfg(debugger)]
    pub async fn dump_current_encounter(&mut self) {
        let encounter = self
            .combat2
            .try_get(self.process, &self.module, &self.image, Combat::new)
            .await
            .and_then(|o| o.resolve(self.process));
        if let Some(encounter) = encounter {
            encounter.enemies().for_each(|e| {
                log!("{e:?}");
            });
        }
    }

    #[cfg(debugger)]
    pub async fn dump_current_hp_levels(&mut self) {
        const KEYS: [&str; 6] = [
            "enemy_1", "enemy_2", "enemy_3", "enemy_4", "enemy_5", "enemy_6",
        ];

        const HP_KEYS: [&str; 6] = [
            "enemy_1_hp",
            "enemy_2_hp",
            "enemy_3_hp",
            "enemy_4_hp",
            "enemy_5_hp",
            "enemy_6_hp",
        ];

        if let Some(enc) = self
            .combat2
            .try_get(self.process, &self.module, &self.image, Combat::new)
            .await
            .and_then(|o| o.resolve(self.process))
        {
            for (e, (key, hp_key)) in enc
                .enemies()
                .filter_map(|e| match e {
                    combat::EnemyEncounter::Enemy(e) => Some(e),
                    _ => None,
                })
                .map(Some)
                .chain(core::iter::repeat(None))
                .zip(KEYS.into_iter().zip(HP_KEYS))
            {
                let hp = e.map_or(0, |o| o.current_hp);
                let id = e.as_ref().map_or("", |o| o.id.as_str());
                asr::timer::set_variable(key, id);
                asr::timer::set_variable_int(hp_key, hp);
            }
        }
    }

    pub fn resolve_encounter(&self, address: Address64) -> Option<Encounter> {
        self.encounter.read(self.process, address.into()).ok()
    }

    #[cfg(debugger)]
    pub async fn check_for_new_key_items(&mut self) -> impl Iterator<Item = ArrayString<32>> + '_ {
        self.inventory
            .try_get(
                self.process,
                &self.module,
                &self.image,
                inventory::Inventory::new,
            )
            .await
            .into_iter()
            .flat_map(|o| o.refresh(self.process, &self.module))
    }
}

macro_rules! binds {
    ($process:expr, $module:expr, $image:expr, ($($cls:ty),+ $(,)?)) => {{
        (
            $({
                let binding = <$cls>::bind($process, $module, $image).await;
                log!(concat!("Created binding for class ", stringify!($cls)));
                binding
            }),+
        )
    }};
}

macro_rules! singleton {
    ($cls:ty) => {
        async fn new(process: &Process, module: &Module, image: &Image) -> Singleton<Self> {
            let binding = <$cls>::bind(process, module, image).await;
            let address = binding
                .class()
                .wait_get_static_instance(process, module, "instance")
                .await;

            log!(
                concat!("found ", stringify!($cls), " instance at {}"),
                address
            );

            Singleton { binding, address }
        }

        fn resolve(this: &Singleton<Self>, process: &Process) -> Option<$cls> {
            this.binding.read(process, this.address).ok()
        }
    };
}

pin_project_lite::pin_project! {
    struct UnRetry<F> {
        #[pin]
        fut: F
    }
}

impl<F: Future> Future for UnRetry<F> {
    type Output = Option<F::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let res = match this.fut.poll(cx) {
            Poll::Ready(res) => Some(res),
            Poll::Pending => None,
        };
        Poll::Ready(res)
    }
}

impl<F: Future> UnRetry<F> {
    fn new(fut: F) -> Self {
        Self { fut }
    }
}

struct LateInit<T> {
    res: Option<T>,
}

impl<T> LateInit<T> {
    fn new() -> Self {
        Self { res: None }
    }

    async fn try_get<'a, 'b, F, Fut>(
        &mut self,
        process: &'a Process,
        module: &'b Module,
        image: &'b Image,
        ctor: F,
    ) -> Option<&mut T>
    where
        F: FnOnce(&'a Process, &'b Module, &'b Image) -> Fut,
        Fut: Future<Output = T>,
    {
        if self.res.is_none() {
            let fut = ctor(process, module, image);
            let fut = UnRetry::new(fut);
            let res = fut.await;
            if let Some(res) = res {
                self.res = Some(res);
            }
        }
        self.res.as_mut()
    }
}

#[derive(Class)]
struct TitleSequenceManager {
    #[rename = "characterSelectionScreen"]
    selection_screen: Address64,
}

#[derive(Class)]
struct CharacterSelectionScreen {
    #[rename = "characterSelected"]
    selected: bool,
}

#[cfg(debugger)]
struct Combat {
    manager: Singleton<CombatManagerBinding>,
    encounter: EncounterBinding,
    loot: EncounterLootBinding,
    actor: EnemyCombatActorBinding,
    char_data: EnemyCharacterDataBinding,
    by_damage: FloatByEDamageTypeBinding,
    xp: XPDataBinding,
    e_target: EnemyCombatTargetBinding,
}

#[cfg(debugger)]
impl fmt::Debug for Combat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Combat").finish_non_exhaustive()
    }
}

#[cfg(debugger)]
impl Combat {
    pub async fn new(process: &Process, module: &Module, image: &Image) -> Self {
        let (encounter, loot, actor, char_data, by_damage, xp, e_target) = binds!(
            process,
            module,
            image,
            (
                Encounter,
                EncounterLoot,
                EnemyCombatActor,
                EnemyCharacterData,
                FloatByEDamageType,
                XPData,
                EnemyCombatTarget,
            )
        );

        let manager = CombatManagerBinding::new(process, module, image).await;
        Self {
            manager,
            encounter,
            loot,
            actor,
            char_data,
            by_damage,
            xp,
            e_target,
        }
    }
}

#[derive(Class)]
struct CombatManager {
    #[rename = "currentEncounter"]
    encounter: Pointer<Encounter>,
}

#[cfg(not(debugger))]
#[derive(Class, Debug)]
pub struct Encounter {
    #[rename = "encounterDone"]
    pub done: bool,
    #[rename = "bossEncounter"]
    pub boss: bool,
}

#[cfg(debugger)]
#[derive(Class, Debug)]
pub struct Encounter {
    #[rename = "encounterDone"]
    pub done: bool,
    #[rename = "bossEncounter"]
    pub boss: bool,

    #[rename = "fullHealPartyAfterEncounter"]
    full_heal_after: bool,
    #[rename = "isUnderwater"]
    underwater: bool,
    #[rename = "allEnemiesKilled"]
    all_enemies_killed: bool,
    #[rename = "isRunning"]
    running: bool,

    #[rename = "xpGained"]
    xp_gain: u32,

    #[rename = "encounterDoneAchievement"]
    has_achievement: Address64,

    #[rename = "encounterLoot"]
    loot: Pointer<EncounterLoot>,

    #[rename = "enemyActors"]
    enemy_actors: Pointer<List<Pointer<EnemyCombatActor>>>,

    #[rename = "enemyTargets"]
    enemy_targets: Pointer<List<Pointer<EnemyCombatTarget>>>,
}

#[cfg(debugger)]
#[derive(Class, Debug)]
struct EncounterLoot {
    #[rename = "goldToAward"]
    gold: u32,
}

#[cfg(debugger)]
#[derive(Class, Debug)]
struct EnemyCombatActor {
    #[rename = "hideHP"]
    hide_hp: bool,
    #[rename = "awardXP"]
    xp: bool,
    #[rename = "enemyData"]
    data: Pointer<EnemyCharacterData>,
}

#[cfg(debugger)]
#[derive(Class, Debug)]
struct EnemyCharacterData {
    guid: Pointer<CSString>,

    hp: u32,
    speed: u32,
    #[rename = "basePhysicalDefense"]
    base_physical_defense: u32,
    #[rename = "basePhysicalAttack"]
    base_physical_attack: u32,
    #[rename = "baseMagicAttack"]
    base_magic_attack: u32,
    #[rename = "baseMagicDefense"]
    base_magic_defense: u32,
    #[rename = "damageTypeModifiers"]
    damage_type_modifiers: Pointer<FloatByEDamageType>,
    #[rename = "damageTypeModifiersOverride"]
    damage_type_override: Pointer<FloatByEDamageType>,

    #[rename = "enemyLevel"]
    level: u32,
    #[rename = "xpData"]
    xp: Pointer<XPData>,
}

#[cfg(debugger)]
#[derive(Class, Debug)]
struct FloatByEDamageType {
    dictionary: Pointer<Map<EDamageType, f32>>,
}

#[cfg(debugger)]
#[derive(Class, Debug)]
struct XPData {
    #[rename = "goldReward"]
    gold: u32,
}

#[cfg(debugger)]
#[derive(Class, Debug)]
struct EnemyCombatTarget {
    #[rename = "currentHP"]
    current_hp: u32,
}

#[derive(Copy, Clone, Debug, AnyBitPattern)]
#[repr(C)]
struct EDamageType {
    value: u32,
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
enum DamageType {
    None = 0x0000,
    Any = 0x0001,
    Sword = 0x0002,
    Sun = 0x0004,
    Moon = 0x0008,
    Eclipse = 0x0010,
    Poison = 0x0020,
    Arcane = 0x0040,
    Stun = 0x0080,
    Magical = 0x00fc, // Stun | Arcane | Poison | Eclipse | Sun | Moon
    Blunt = 0x0100,
}

impl core::ops::BitAnd for DamageType {
    type Output = bool;

    fn bitand(self, rhs: Self) -> Self::Output {
        (self as u32) & (rhs as u32) != 0
    }
}

macro_rules! binding {
    ($binding:ty => $cls:ty) => {
        impl<'a> ::csharp_mem::Binding<$cls> for ($crate::data::Proc<'a>, &'a $binding) {
            fn read(self, addr: u64) -> Option<$cls> {
                self.1.read(self.0 .0, addr.into()).ok()
            }
        }

        impl<'a> ::csharp_mem::Binding<$cls> for (&'a ::asr::Process, &'a $binding) {
            fn read(self, addr: u64) -> Option<$cls> {
                self.1.read(self.0, addr.into()).ok()
            }
        }
    };
}

binding!(EncounterBinding => Encounter);

#[cfg(debugger)]
binding!(EncounterLootBinding => EncounterLoot);
#[cfg(debugger)]
binding!(EnemyCombatActorBinding => EnemyCombatActor);
#[cfg(debugger)]
binding!(EnemyCharacterDataBinding => EnemyCharacterData);
#[cfg(debugger)]
binding!(FloatByEDamageTypeBinding => FloatByEDamageType);
#[cfg(debugger)]
binding!(XPDataBinding => XPData);
#[cfg(debugger)]
binding!(EnemyCombatTargetBinding => EnemyCombatTarget);

impl<'a> Data<'a> {
    pub async fn new(process: &'a Process) -> Data<'a> {
        let module = Module::wait_attach(process, Version::V2020).await;
        let image = module.wait_get_default_image(process).await;
        log!("Attaching to the game");

        let (char_select, encounter) = binds!(
            process,
            &module,
            &image,
            (CharacterSelectionScreen, Encounter)
        );

        let combat = CombatManagerBinding::new(process, &module, &image).await;

        log!("Attached to the game");

        Self {
            process,
            module,
            image,
            char_select,
            combat,
            encounter,
            title_screen: LateInit::new(),
            progression: LateInit::new(),
            #[cfg(debugger)]
            combat2: LateInit::new(),
            #[cfg(debugger)]
            inventory: LateInit::new(),
        }
    }
}

#[derive(Debug)]
struct Singleton<T> {
    binding: T,
    address: Address,
}

impl TitleSequenceManagerBinding {
    singleton!(TitleSequenceManager);
}

impl CombatManagerBinding {
    singleton!(CombatManager);
}

#[cfg(debugger)]
#[derive(Class, Debug)]
struct ProgressionManager {
    timestamp: f64,
    #[rename = "playTime"]
    play_time: f64,
    #[rename = "defeatedPermaDeathEnemies"]
    defeated_perma_death_enemies: Pointer<Set<Pointer<CSString>>>,
}

#[cfg(debugger)]
#[derive(Class, Debug)]
struct ActivityManager {
    #[rename = "mainActivity"]
    main_activity: Pointer<CSString>,
    #[rename = "subActivityIndex"]
    sub_activity_index: u32,
}

#[cfg(not(debugger))]
#[derive(Class, Debug)]
struct LevelManager {
    #[rename = "loadingLevel"]
    is_loading: bool,
}

#[cfg(debugger)]
#[derive(Class, Debug)]
struct LevelManager {
    #[rename = "loadingLevel"]
    is_loading: bool,

    #[rename = "currentLevel"]
    current_level: LevelReference,

    #[rename = "previousLevelInfo"]
    previous_level_info: Pointer<LoadedLevelInfo>,
}

#[cfg(debugger)]
#[derive(Class, Debug)]
struct LoadedLevelInfo {
    level: LevelReference,
}

#[cfg(debugger)]
#[derive(Copy, Clone, Debug, AnyBitPattern)]
struct LevelReference {
    guid: Pointer<CSString>,
}

#[cfg(debugger)]
impl ProgressionManagerBinding {
    singleton!(ProgressionManager);
}

#[cfg(debugger)]
impl ActivityManagerBinding {
    singleton!(ActivityManager);
}
impl LevelManagerBinding {
    singleton!(LevelManager);
}

#[cfg(debugger)]
binding!(LoadedLevelInfoBinding => LoadedLevelInfo);

struct Progression {
    level_manager: Singleton<LevelManagerBinding>,
    #[cfg(debugger)]
    progression_manager: Singleton<ProgressionManagerBinding>,
    #[cfg(debugger)]
    activity_manager: Singleton<ActivityManagerBinding>,
    #[cfg(debugger)]
    loaded_level_info: LoadedLevelInfoBinding,
}

impl Progression {
    pub async fn new(process: &Process, module: &Module, image: &Image) -> Self {
        let level_manager = LevelManagerBinding::new(process, module, image).await;

        #[cfg(debugger)]
        {
            let loaded_level_info = binds!(process, module, image, (LoadedLevelInfo));
            let progression_manager = ProgressionManagerBinding::new(process, module, image).await;
            let activity_manager = ActivityManagerBinding::new(process, module, image).await;

            Self {
                level_manager,
                progression_manager,
                activity_manager,
                loaded_level_info,
            }
        }

        #[cfg(not(debugger))]
        {
            Self { level_manager }
        }
    }

    pub fn get_progress(&self, process: &Process) -> Option<CurrentProgress> {
        let process = Proc(process);

        let level = LevelManagerBinding::resolve(&self.level_manager, process.0)?;

        #[cfg(debugger)]
        {
            let progression =
                ProgressionManagerBinding::resolve(&self.progression_manager, process.0)?;

            let number_of_defeated_perma_death_enemies = progression
                .defeated_perma_death_enemies
                .resolve(process)?
                .size();

            let activity = ActivityManagerBinding::resolve(&self.activity_manager, process.0)?;
            let main_activity = activity.main_activity.resolve(process)?.to_string(process);

            let current_level = level
                .current_level
                .guid
                .resolve(process)?
                .to_string(process);

            let previous_level = level
                .previous_level_info
                .resolve_with((process, &self.loaded_level_info))
                .and_then(|o| o.level.guid.resolve(process).map(|o| o.to_string(process)))
                .unwrap_or_default();

            Some(CurrentProgress {
                is_loading: level.is_loading,
                timestamp: progression.timestamp,
                play_time: progression.play_time,
                main_activity,
                sub_activity_index: activity.sub_activity_index,
                current_level,
                previous_level,
                number_of_defeated_perma_death_enemies,
            })
        }

        #[cfg(not(debugger))]
        {
            Some(CurrentProgress {
                is_loading: level.is_loading,
            })
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct CurrentProgress {
    pub is_loading: bool,
    #[cfg(debugger)]
    pub timestamp: f64,
    #[cfg(debugger)]
    pub play_time: f64,
    #[cfg(debugger)]
    pub main_activity: ArrayString<36>,
    #[cfg(debugger)]
    pub sub_activity_index: u32,
    #[cfg(debugger)]
    pub current_level: ArrayString<36>,
    #[cfg(debugger)]
    pub previous_level: ArrayString<36>,
    #[cfg(debugger)]
    pub number_of_defeated_perma_death_enemies: u32,
}

#[cfg(debugger)]
impl CurrentProgress {
    pub fn play_time(&self) -> PlayTime {
        PlayTime {
            session: self.timestamp as u64,
            total: self.play_time as u64,
        }
    }

    pub fn activity(&self) -> Activity {
        Activity {
            id: self.main_activity,
            sub_index: self.sub_activity_index,
            defeated_perma_death_enemies: self.number_of_defeated_perma_death_enemies,
        }
    }

    pub fn level(&self) -> Level {
        Level {
            is_loading: self.is_loading,
            current_level: self.current_level,
            previous_level: self.previous_level,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayTime {
    pub session: u64,
    pub total: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub id: ArrayString<36>,
    pub sub_index: u32,
    pub defeated_perma_death_enemies: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level {
    pub is_loading: bool,
    pub current_level: ArrayString<36>,
    pub previous_level: ArrayString<36>,
}

#[cfg(debugger)]
mod inventory {
    use ahash::HashSet;
    use asr::{
        game_engine::unity::il2cpp::{Class, Image, Module},
        string::ArrayString,
        watcher::Watcher,
        Process,
    };
    use bytemuck::AnyBitPattern;

    use csharp_mem::{CSString, Map, Pointer};

    use super::Singleton;

    #[derive(Class, Debug)]
    struct InventoryManager {
        #[rename = "allInventoryItemData"]
        all_items: Pointer<Map<InventoryItemReference, Pointer<InventoryItem>>>,
        #[rename = "ownedInventoryItems"]
        owned_items: Pointer<QuantityByInventoryItemReference>,
    }

    #[derive(Class, Debug)]
    struct InventoryItem {
        guid: Pointer<CSString>,
    }

    #[derive(Class, Debug)]
    struct KeyItem {}

    #[derive(Debug, Copy, Clone, AnyBitPattern)]
    struct InventoryItemReference {
        guid: Pointer<CSString>,
    }

    #[derive(Class, Debug)]
    struct QuantityByInventoryItemReference {
        dictionary: Pointer<Map<InventoryItemReference, u32>>,
    }

    impl InventoryManagerBinding {
        singleton!(InventoryManager);
    }

    binding!(QuantityByInventoryItemReferenceBinding => QuantityByInventoryItemReference);
    binding!(InventoryItemBinding => InventoryItem);

    pub struct Inventory {
        inventory_item: InventoryItemBinding,
        key_item: KeyItemBinding,
        quantity: QuantityByInventoryItemReferenceBinding,
        manager: Singleton<InventoryManagerBinding>,
        all_key_items: HashSet<ArrayString<32>>,
        number_of_owned_items: Watcher<u32>,
        owned_key_items: HashSet<ArrayString<32>>,
    }

    impl Inventory {
        pub async fn new(process: &Process, module: &Module, image: &Image) -> Self {
            let (inventory_item, key_item, quantity) = binds!(
                process,
                module,
                image,
                (InventoryItem, KeyItem, QuantityByInventoryItemReference)
            );
            let manager = InventoryManagerBinding::new(process, module, image).await;
            Self {
                inventory_item,
                key_item,
                quantity,
                manager,
                number_of_owned_items: Watcher::new(),
                all_key_items: HashSet::with_hasher(ahash::RandomState::new()),
                owned_key_items: HashSet::with_hasher(ahash::RandomState::new()),
            }
        }

        pub fn refresh<'a>(
            &'a mut self,
            process: &'a Process,
            module: &'a Module,
        ) -> impl Iterator<Item = ArrayString<32>> + '_ {
            self.cache_available_items(process, module);
            self.new_owned_key_items(process)
        }

        pub fn cache_available_items(&mut self, process: &Process, module: &Module) -> bool {
            if !self.all_key_items.is_empty() {
                return false;
            }

            (|| {
                let process = super::Proc(process);
                let manager = InventoryManagerBinding::resolve(&self.manager, process.0)?;
                let all_items = manager.all_items.resolve(process)?;

                for (_, v) in all_items.iter(process) {
                    let is_key_item = self
                        .key_item
                        .class()
                        .is_instance(process.0, module, v.address_value())
                        .ok()?;

                    if is_key_item {
                        let item = v.resolve_with((process, &self.inventory_item))?;
                        let item = item.guid.resolve(process)?;
                        let item = item.to_string(process);
                        self.all_key_items.insert(item);
                    }
                }

                Some(())
            })();

            !self.all_key_items.is_empty()
        }

        pub fn new_owned_key_items<'a>(
            &'a mut self,
            process: &'a Process,
        ) -> impl Iterator<Item = ArrayString<32>> + 'a {
            Some(&self.all_key_items)
                .and_then(|key_items| {
                    let process = super::Proc(process);
                    let manager = InventoryManagerBinding::resolve(&self.manager, process.0)?;
                    let owned = manager
                        .owned_items
                        .resolve_with((process, &self.quantity))?;
                    let owned = owned.dictionary.resolve(process)?;

                    let amount = owned.size();
                    let owned_items = &mut self.owned_key_items;
                    self.number_of_owned_items
                        .update_infallible(amount)
                        .changed()
                        .then(move || {
                            owned.iter(process).filter_map(move |(item, _amount)| {
                                let item = item.guid.resolve(process)?;
                                let item = item.to_string(process);
                                (key_items.contains(&item) && owned_items.insert(item))
                                    .then_some(item)
                            })
                        })
                })
                .into_iter()
                .flatten()
        }
    }
}

#[cfg(debugger)]
mod combat {
    use core::fmt;

    use asr::{
        arrayvec::{ArrayString, ArrayVec},
        Process,
    };

    use super::DamageType;

    impl super::Combat {
        pub fn resolve(&self, process: &Process) -> Option<Encounter> {
            let process = super::Proc(process);

            let combat = super::CombatManagerBinding::resolve(&self.manager, process.0)?;
            let encounter = combat.encounter.resolve_with((process, &self.encounter))?;
            let loot = encounter.loot.resolve_with((process, &self.loot))?;

            let actors = encounter.enemy_actors.resolve(process);
            let actors = actors
                .into_iter()
                .flat_map(|o| {
                    o.iter(process).filter_map(|o| {
                        let actor = o.resolve_with((process, &self.actor))?;
                        let char_data = actor.data.resolve_with((process, &self.char_data))?;
                        let xp = char_data.xp.resolve_with((process, &self.xp))?;

                        let damage_type_modifiers = char_data
                            .damage_type_modifiers
                            .resolve_with((process, &self.by_damage));
                        let damage_type_modifiers = damage_type_modifiers
                            .and_then(|o| o.dictionary.resolve(process))
                            .map(|o| {
                                o.iter(process)
                                    .map(|(k, v)| (unsafe { core::mem::transmute(k) }, v))
                                    .collect()
                            });

                        let damage_type_override = char_data
                            .damage_type_override
                            .resolve_with((process, &self.by_damage));
                        let damage_type_override = damage_type_override
                            .and_then(|o| o.dictionary.resolve(process))
                            .map(|o| {
                                o.iter(process)
                                    .map(|(k, v)| (unsafe { core::mem::transmute(k) }, v))
                                    .collect()
                            });

                        let e_guid = char_data.guid.resolve(process)?;

                        Some(EnemyCombatActor {
                            hide_hp: actor.hide_hp,
                            xp: actor.xp,
                            data: EnemyCharacterData {
                                id: e_guid.to_string(process),
                                level: char_data.level,
                                xp,
                                hp: char_data.hp,
                                speed: char_data.speed,
                                base_physical_defense: char_data.base_physical_defense,
                                base_physical_attack: char_data.base_physical_attack,
                                base_magic_attack: char_data.base_magic_attack,
                                base_magic_defense: char_data.base_magic_defense,
                                damage_type_modifiers,
                                damage_type_override,
                            },
                        })
                    })
                })
                .take(6)
                .collect();

            let targets = encounter.enemy_targets.resolve(process);
            let targets = targets
                .into_iter()
                .flat_map(|o| {
                    o.iter(process).filter_map(|o| {
                        let e_target = o.resolve_with((process, &self.e_target))?;

                        Some(EnemyCombatTarget {
                            current_hp: e_target.current_hp,
                        })
                    })
                })
                .take(6)
                .collect();

            Some(Encounter {
                done: encounter.done,
                boss: encounter.boss,
                full_heal_after: encounter.full_heal_after,
                underwater: encounter.underwater,
                all_enemies_killed: encounter.all_enemies_killed,
                running: encounter.running,
                xp_gain: encounter.xp_gain,
                has_achievement: !encounter.has_achievement.is_null(),
                loot,
                enemy_actors: actors,
                enemy_targets: targets,
            })
        }
    }

    #[derive(Debug)]
    pub struct Encounter {
        pub done: bool,
        pub boss: bool,
        full_heal_after: bool,
        underwater: bool,
        all_enemies_killed: bool,
        running: bool,
        xp_gain: u32,
        has_achievement: bool,
        loot: super::EncounterLoot,
        enemy_actors: ArrayVec<EnemyCombatActor, 6>,
        enemy_targets: ArrayVec<EnemyCombatTarget, 6>,
    }

    impl Encounter {
        pub fn enemies(&self) -> impl Iterator<Item = EnemyEncounter> + '_ {
            Some(EnemyEncounter::General(General {
                boss: self.boss,
                has_achievement: self.has_achievement,
                done: self.done,
                is_running: self.running,
                all_enemies_killed: self.all_enemies_killed,
                full_heal_after: self.full_heal_after,
                underwater: self.underwater,
                xp_gained: self.xp_gain,
                gold_gained: self.loot.gold,
            }))
            .into_iter()
            .chain(
                self.enemy_actors
                    .iter()
                    .zip(self.enemy_targets.iter())
                    .flat_map(move |(a, t)| {
                        let mut any_modifier = 1.0;
                        let mut sword_modifier = 1.0;
                        let mut sun_modifier = 1.0;
                        let mut moon_modifier = 1.0;
                        let mut eclipse_modifier = 1.0;
                        let mut poison_modifier = 1.0;
                        let mut arcane_modifier = 1.0;
                        let mut stun_modifier = 1.0;
                        let mut blunt_modifier = 1.0;
                        for (dmg, modifier) in a
                            .data
                            .damage_type_modifiers
                            .iter()
                            .flatten()
                            .chain(a.data.damage_type_override.iter().flatten())
                            .copied()
                        {
                            if dmg & DamageType::Any {
                                any_modifier = modifier;
                            }
                            if dmg & DamageType::Sword {
                                sword_modifier = modifier;
                            }
                            if dmg & DamageType::Sun {
                                sun_modifier = modifier;
                            }
                            if dmg & DamageType::Moon {
                                moon_modifier = modifier;
                            }
                            if dmg & DamageType::Eclipse {
                                eclipse_modifier = modifier;
                            }
                            if dmg & DamageType::Poison {
                                poison_modifier = modifier;
                            }
                            if dmg & DamageType::Arcane {
                                arcane_modifier = modifier;
                            }
                            if dmg & DamageType::Stun {
                                stun_modifier = modifier;
                            }
                            if dmg & DamageType::Blunt {
                                blunt_modifier = modifier;
                            }
                        }

                        [
                            EnemyEncounter::Enemy(Enemy {
                                id: a.data.id,
                                current_hp: t.current_hp,
                                max_hp: a.data.hp,
                                hide_hp: a.hide_hp,
                                award_xp: a.xp,
                                gold_drop: a.data.xp.gold,
                            }),
                            EnemyEncounter::EnemyStats(EnemyStats {
                                level: a.data.level,
                                speed: a.data.speed,
                                attack: a.data.base_physical_attack,
                                defense: a.data.base_physical_defense,
                                magic_attack: a.data.base_magic_attack,
                                magic_defense: a.data.base_magic_defense,
                            }),
                            EnemyEncounter::EnemyMods(EnemyMods {
                                any: any_modifier,
                                sword: sword_modifier,
                                sun: sun_modifier,
                                moon: moon_modifier,
                                eclipse: eclipse_modifier,
                                poison: poison_modifier,
                                arcane: arcane_modifier,
                                stun: stun_modifier,
                                blunt: blunt_modifier,
                            }),
                        ]
                    }),
            )
        }
    }

    #[derive(Debug)]
    struct EnemyCombatActor {
        hide_hp: bool,
        xp: bool,
        data: EnemyCharacterData,
    }

    #[derive(Debug)]
    struct EnemyCharacterData {
        id: ArrayString<36>,
        level: u32,
        xp: super::XPData,
        hp: u32,
        speed: u32,
        base_physical_defense: u32,
        base_physical_attack: u32,
        base_magic_attack: u32,
        base_magic_defense: u32,
        damage_type_modifiers: Option<ArrayVec<(super::DamageType, f32), 11>>,
        damage_type_override: Option<ArrayVec<(super::DamageType, f32), 11>>,
    }

    #[derive(Debug)]
    struct EnemyCombatTarget {
        current_hp: u32,
    }

    #[derive(Copy, Clone, Debug)]
    pub struct General {
        pub boss: bool,
        pub has_achievement: bool,
        pub done: bool,
        pub is_running: bool,
        pub all_enemies_killed: bool,

        pub full_heal_after: bool,
        pub underwater: bool,

        pub xp_gained: u32,
        pub gold_gained: u32,
    }

    #[derive(Copy, Clone, Debug)]
    pub struct Enemy {
        pub id: ArrayString<36>,
        pub current_hp: u32,
        pub max_hp: u32,
        pub hide_hp: bool,
        pub award_xp: bool,
        pub gold_drop: u32,
    }

    #[derive(Copy, Clone, Debug)]
    pub struct EnemyStats {
        pub level: u32,
        pub speed: u32,
        pub attack: u32,
        pub defense: u32,
        pub magic_attack: u32,
        pub magic_defense: u32,
    }

    #[derive(Copy, Clone, Debug)]
    pub struct EnemyMods {
        pub any: f32,
        pub sword: f32,
        pub blunt: f32,
        pub sun: f32,
        pub moon: f32,
        pub poison: f32,
        pub arcane: f32,
        pub eclipse: f32,
        pub stun: f32,
    }

    pub enum EnemyEncounter {
        General(General),
        Enemy(Enemy),
        EnemyStats(EnemyStats),
        EnemyMods(EnemyMods),
    }

    impl fmt::Debug for EnemyEncounter {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if f.alternate() {
                match self {
                    EnemyEncounter::General(e) => e.fmt(f),
                    EnemyEncounter::Enemy(e) => e.fmt(f),
                    EnemyEncounter::EnemyStats(e) => e.fmt(f),
                    EnemyEncounter::EnemyMods(e) => e.fmt(f),
                }
            } else {
                match self {
                    EnemyEncounter::General(General {
                        boss,
                        has_achievement,
                        done,
                        is_running,
                        all_enemies_killed,
                        ..
                    }) => {
                        write!(
                            f,
                            "Encounter: boss={} achievement={} done={} running={} killed={}",
                            boss, has_achievement, done, is_running, all_enemies_killed
                        )
                    }
                    EnemyEncounter::Enemy(Enemy {
                        id,
                        current_hp,
                        max_hp,
                        hide_hp,
                        ..
                    }) => {
                        write!(
                            f,
                            "Enemy: id={} hp={}/{} hide_hp={}",
                            id, current_hp, max_hp, hide_hp
                        )
                    }
                    EnemyEncounter::EnemyStats(EnemyStats {
                        level,
                        speed,
                        attack,
                        defense,
                        magic_attack,
                        magic_defense,
                    }) => {
                        write!(
                            f,
                            "|--stats: level={} speed={} A/MA={}/{}, D/MD={}/{}",
                            level, speed, attack, magic_attack, defense, magic_defense
                        )
                    }
                    EnemyEncounter::EnemyMods(EnemyMods {
                        any,
                        sword,
                        blunt,
                        sun,
                        moon,
                        poison,
                        arcane,
                        eclipse,
                        stun,
                    }) => {
                        write!(
                            f,
                            concat!(
                                "|--mods: any={} sword={} blunt={} sun={} ",
                                "moon={} poison={}, arcane={} eclipse={} sun={}"
                            ),
                            any, sword, blunt, sun, moon, poison, arcane, eclipse, stun
                        )
                    }
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
struct Proc<'a>(&'a Process);

impl<'a> MemReader for Proc<'a> {
    fn read<T: AnyBitPattern>(&self, addr: u64) -> Option<T> {
        self.0.read(Address64::from(addr)).ok()
    }
}
