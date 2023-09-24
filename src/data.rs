use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use asr::{
    game_engine::unity::il2cpp::{Image, Module, Version},
    Address, Process,
};
use bytemuck::AnyBitPattern;
use csharp_mem::MemReader;

use self::{
    combat::{Combat, Encounter},
    progress::Progression,
    title_screen::TitleScreen,
};
#[cfg(debugger)]
pub use self::{
    combat::{Enemy, EnemyEncounter, EnemyMods, EnemyStats, General},
    inventory::{Inventory, NamedKeyItem},
    loc::Loc,
};

pub use self::progress::CurrentProgress;
#[cfg(debugger)]
pub use self::progress::{Activity, Level, PlayTime};
pub use self::title_screen::GameStart;

pub struct Data<'a> {
    process: &'a Process,
    module: Module,
    image: Image,
    title_screen: TitleScreen,
    combat: LateInit<Combat>,
    progression: LateInit<Progression>,
    #[cfg(debugger)]
    loc: LateInit<Option<Loc>>,
    #[cfg(debugger)]
    inventory: LateInit<Inventory>,
}

impl Data<'_> {
    pub fn game_start(&mut self) -> GameStart {
        self.title_screen
            .game_start(self.process, &self.module, &self.image)
    }

    #[cfg(not(debugger))]
    pub async fn progress(&mut self) -> Option<CurrentProgress> {
        let progression = self
            .progression
            .try_get(self.process, &self.module, &self.image, Progression::new)
            .await?;

        progression.get_progress(self.process)
    }

    #[cfg(debugger)]
    pub async fn progress(&mut self) -> Option<CurrentProgress> {
        let progression = self
            .progression
            .try_get(self.process, &self.module, &self.image, Progression::new)
            .await?;

        let loc = self
            .loc
            .try_get(self.process, &self.module, &self.image, Loc::new)
            .await?
            .as_ref()?;

        progression.get_progress(self.process, loc)
    }

    pub async fn encounter(&mut self) -> Option<Encounter> {
        let combat = self
            .combat
            .try_get(self.process, &self.module, &self.image, Combat::new)
            .await?;
        combat.current_encounter(self.process)
    }

    // #[cfg(debugger)]
    // pub async fn dump_current_encounter(&mut self) {
    //     if let Some(enc) = self.deep_resolve_encounter().await {
    //         enc.enemies().for_each(|e| {
    //             log!("{e:?}");
    //         });
    //     }
    // }

    #[cfg(debugger)]
    pub async fn deep_resolve_encounter(&mut self) -> Option<combat::BattleEncounter> {
        let combat = self
            .combat
            .try_get(self.process, &self.module, &self.image, Combat::new)
            .await?;

        let loc = self
            .loc
            .try_get(self.process, &self.module, &self.image, Loc::new)
            .await?
            .as_ref()?;

        combat.resolve(self.process, loc)
    }

    #[cfg(debugger)]
    pub async fn check_for_new_key_items<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = &'a NamedKeyItem> + 'a {
        (|| async move {
            let inventory = self
                .inventory
                .try_get(self.process, &self.module, &self.image, Inventory::new)
                .await?;

            let loc = self
                .loc
                .try_get(self.process, &self.module, &self.image, Loc::new)
                .await?
                .as_ref()?;

            Some(inventory.check_new(self.process, &self.module, loc))
        })()
        .await
        .into_iter()
        .flatten()
    }

    #[cfg(debugger)]
    pub async fn check_for_lost_key_items<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = NamedKeyItem> + 'a {
        (|| async move {
            let inventory = self
                .inventory
                .try_get(self.process, &self.module, &self.image, Inventory::new)
                .await?;

            let loc = self
                .loc
                .try_get(self.process, &self.module, &self.image, Loc::new)
                .await?
                .as_ref()?;

            Some(inventory.check_lost(self.process, &self.module, loc))
        })()
        .await
        .into_iter()
        .flatten()
    }
}

impl<'a> Data<'a> {
    pub async fn new(process: &'a Process) -> Data<'a> {
        let module = Module::wait_attach(process, Version::V2020).await;
        let image = module.wait_get_default_image(process).await;
        log!("Attached to the game");

        Self {
            process,
            module,
            image,
            title_screen: TitleScreen::default(),
            combat: LateInit::new(),
            progression: LateInit::new(),
            #[cfg(debugger)]
            inventory: LateInit::new(),
            #[cfg(debugger)]
            loc: LateInit::new(),
        }
    }
}

macro_rules! binds {
    ($process:expr, $module:expr, $image:expr, ($($cls:ty),+ $(,)?)) => {{
        let res = (
            $(<$cls>::bind($process, $module, $image).await),+
        );

        $({
            log!(concat!("Created binding for class ", stringify!($cls)));
        })+

        res
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

#[derive(Debug)]
struct Singleton<T> {
    binding: T,
    address: Address,
}

#[derive(Copy, Clone)]
struct Proc<'a>(&'a Process);

impl<'a> MemReader for Proc<'a> {
    fn read<T: AnyBitPattern>(&self, addr: u64) -> Option<T> {
        self.0.read(asr::Address64::from(addr)).ok()
    }
}

mod title_screen {
    use asr::{
        game_engine::unity::il2cpp::{Class, Image, Module},
        Address, Address64, Process,
    };

    pub enum GameStart {
        NotStarted,
        JustStarted,
        AlreadyRunning,
    }

    #[derive(Default)]
    pub struct TitleScreen {
        title_screen_class: Option<Class>,
        selection_screen_offset: Option<u32>,
        title_screen: Option<Address>,
        char_select_class: Option<Class>,
        char_select_offset: Option<u32>,
    }

    impl TitleScreen {
        pub fn selected(
            &mut self,
            process: &Process,
            module: &Module,
            image: &Image,
        ) -> Option<bool> {
            let title_screen_class = match self.title_screen_class {
                Some(ref cls) => cls,
                None => {
                    let title_screen_class =
                        image.get_class(process, module, "TitleSequenceManager")?;
                    self.title_screen_class = Some(title_screen_class);
                    self.title_screen_class.as_ref().unwrap()
                }
            };

            let selection_screen_offset = match self.selection_screen_offset {
                Some(screen) => screen,
                None => {
                    let selection_screen_offset = title_screen_class.get_field(
                        process,
                        module,
                        "characterSelectionScreen",
                    )?;
                    self.selection_screen_offset = Some(selection_screen_offset);
                    self.selection_screen_offset.unwrap()
                }
            };

            let title_screen = match self.title_screen {
                Some(title_screen) => title_screen,
                None => {
                    let address =
                        title_screen_class.get_static_instance(process, module, "instance")?;
                    log!("found TitleSequenceManager instance at {}", address);
                    self.title_screen = Some(address);
                    self.title_screen.unwrap()
                }
            };

            let char_select_class = match self.char_select_class {
                Some(ref cls) => cls,
                None => {
                    let char_select_class =
                        image.get_class(process, module, "CharacterSelectionScreen")?;
                    self.char_select_class = Some(char_select_class);
                    self.char_select_class.as_ref().unwrap()
                }
            };

            let char_select_offset = match self.char_select_offset {
                Some(screen) => screen,
                None => {
                    let char_select_offset =
                        char_select_class.get_field(process, module, "characterSelected")?;
                    log!("Created binding for class CharacterSelectionScreen");
                    self.char_select_offset = Some(char_select_offset);
                    self.char_select_offset.unwrap()
                }
            };

            let selection_screen: Address64 =
                process.read(title_screen + selection_screen_offset).ok()?;

            if selection_screen.is_null() {
                return None;
            }

            let selected = process
                .read(selection_screen + u64::from(char_select_offset))
                .ok()?;

            Some(selected)
        }

        pub fn game_start(
            &mut self,
            process: &Process,
            module: &Module,
            image: &Image,
        ) -> GameStart {
            match self.selected(process, module, image) {
                Some(true) => GameStart::JustStarted,
                Some(false) => GameStart::NotStarted,
                None => GameStart::AlreadyRunning,
            }
        }
    }
}

mod combat {
    #[cfg(debugger)]
    use core::fmt;

    #[cfg(debugger)]
    use asr::{
        arrayvec::{ArrayString, ArrayVec},
        Address64,
    };
    use asr::{
        game_engine::unity::il2cpp::{Class, Image, Module},
        Process,
    };
    #[cfg(debugger)]
    use bytemuck::AnyBitPattern;
    use csharp_mem::Pointer;
    #[cfg(debugger)]
    use csharp_mem::{CSString, List, Map};

    use super::Singleton;

    pub struct Combat {
        manager: Singleton<CombatManagerBinding>,
        encounter: EncounterBinding,
        #[cfg(debugger)]
        loot: EncounterLootBinding,
        #[cfg(debugger)]
        actor: EnemyCombatActorBinding,
        #[cfg(debugger)]
        char_data: EnemyCharacterDataBinding,
        #[cfg(debugger)]
        by_damage: FloatByEDamageTypeBinding,
        #[cfg(debugger)]
        xp: XPDataBinding,
        #[cfg(debugger)]
        e_target: EnemyCombatTargetBinding,
    }

    #[cfg(debugger)]
    impl fmt::Debug for Combat {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Combat").finish_non_exhaustive()
        }
    }

    impl Combat {
        #[cfg(not(debugger))]
        pub async fn new(process: &Process, module: &Module, image: &Image) -> Self {
            let encounter = binds!(process, module, image, (Encounter,));
            let manager = CombatManagerBinding::new(process, module, image).await;
            Self { manager, encounter }
        }

        #[cfg(debugger)]
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

    impl CombatManagerBinding {
        singleton!(CombatManager);
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

    binding!(EncounterBinding => Encounter);

    #[cfg(debugger)]
    #[derive(Class, Debug)]
    struct EncounterLoot {
        #[rename = "goldToAward"]
        gold: u32,
    }

    #[cfg(debugger)]
    binding!(EncounterLootBinding => EncounterLoot);

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
    binding!(EnemyCombatActorBinding => EnemyCombatActor);

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

        #[rename = "nameLocalizationId"]
        name_localization_id: super::loc::LocalizationId,
    }

    #[cfg(debugger)]
    binding!(EnemyCharacterDataBinding => EnemyCharacterData);

    #[cfg(debugger)]
    #[derive(Class, Debug)]
    struct FloatByEDamageType {
        dictionary: Pointer<Map<EDamageType, f32>>,
    }

    #[cfg(debugger)]
    binding!(FloatByEDamageTypeBinding => FloatByEDamageType);

    #[cfg(debugger)]
    #[derive(Class, Debug)]
    struct XPData {
        #[rename = "goldReward"]
        gold: u32,
    }

    #[cfg(debugger)]
    binding!(XPDataBinding => XPData);

    #[cfg(debugger)]
    #[derive(Class, Debug)]
    struct EnemyCombatTarget {
        #[rename = "currentHP"]
        current_hp: u32,
    }

    #[cfg(debugger)]
    binding!(EnemyCombatTargetBinding => EnemyCombatTarget);

    #[cfg(debugger)]
    #[derive(Copy, Clone, Debug, AnyBitPattern)]
    #[repr(C)]
    struct EDamageType {
        value: u32,
    }

    #[cfg(debugger)]
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

    #[cfg(debugger)]
    impl core::ops::BitAnd for DamageType {
        type Output = bool;

        fn bitand(self, rhs: Self) -> Self::Output {
            (self as u32) & (rhs as u32) != 0
        }
    }

    #[cfg(debugger)]
    impl From<EDamageType> for DamageType {
        fn from(value: EDamageType) -> Self {
            unsafe { core::mem::transmute(value.value) }
        }
    }

    impl Combat {
        pub fn current_encounter(&self, process: &Process) -> Option<Encounter> {
            let combat = CombatManagerBinding::resolve(&self.manager, process)?;
            let encounter = combat.encounter.resolve_with((process, &self.encounter))?;
            Some(encounter)
        }

        #[cfg(debugger)]
        pub fn resolve(&self, process: &Process, loc: &super::loc::Loc) -> Option<BattleEncounter> {
            let process = super::Proc(process);

            let combat = CombatManagerBinding::resolve(&self.manager, process.0)?;
            let encounter = combat.encounter.resolve_with((process, &self.encounter))?;
            let loot = encounter.loot.resolve_with((process, &self.loot))?;

            let actors = encounter.enemy_actors.resolve(process);
            let mut enemies = actors
                .into_iter()
                .flat_map(|o| {
                    o.iter(process).filter_map(|o| {
                        let actor = o.resolve_with((process, &self.actor))?;
                        let char_data = actor.data.resolve_with((process, &self.char_data))?;

                        let stats = EnemyStats {
                            current_hp: 0,
                            max_hp: char_data.hp,
                            level: char_data.level,
                            speed: char_data.speed,
                            attack: char_data.base_physical_attack,
                            defense: char_data.base_physical_defense,
                            magic_attack: char_data.base_magic_attack,
                            magic_defense: char_data.base_magic_defense,
                        };

                        let mut mods = EnemyMods {
                            any: 1.0,
                            sword: 1.0,
                            sun: 1.0,
                            moon: 1.0,
                            eclipse: 1.0,
                            poison: 1.0,
                            arcane: 1.0,
                            stun: 1.0,
                            blunt: 1.0,
                        };

                        let damage_type_modifiers = char_data
                            .damage_type_modifiers
                            .resolve_with((process, &self.by_damage));
                        let damage_type_modifiers = damage_type_modifiers
                            .and_then(|o| o.dictionary.resolve(process))
                            .into_iter()
                            .flat_map(|o| o.iter(process).map(|(k, v)| (DamageType::from(k), v)));

                        let damage_type_override = char_data
                            .damage_type_override
                            .resolve_with((process, &self.by_damage));
                        let damage_type_override = damage_type_override
                            .and_then(|o| o.dictionary.resolve(process))
                            .into_iter()
                            .flat_map(|o| o.iter(process).map(|(k, v)| (DamageType::from(k), v)));

                        for (dmg, modifier) in damage_type_modifiers.chain(damage_type_override) {
                            if dmg & DamageType::Any {
                                mods.any = modifier;
                            }
                            if dmg & DamageType::Sword {
                                mods.sword = modifier;
                            }
                            if dmg & DamageType::Sun {
                                mods.sun = modifier;
                            }
                            if dmg & DamageType::Moon {
                                mods.moon = modifier;
                            }
                            if dmg & DamageType::Eclipse {
                                mods.eclipse = modifier;
                            }
                            if dmg & DamageType::Poison {
                                mods.poison = modifier;
                            }
                            if dmg & DamageType::Arcane {
                                mods.arcane = modifier;
                            }
                            if dmg & DamageType::Stun {
                                mods.stun = modifier;
                            }
                            if dmg & DamageType::Blunt {
                                mods.blunt = modifier;
                            }
                        }

                        let gold = char_data
                            .xp
                            .resolve_with((process, &self.xp))
                            .map_or(0, |o| o.gold);

                        let e_guid = char_data.guid.resolve(process)?;
                        let id = e_guid.to_string(process);

                        let name = char_data.name_localization_id;
                        let name = loc.localized(process, name);

                        Some(EnemyInfo {
                            hide_hp: actor.hide_hp,
                            gives_xp: actor.xp,
                            gold,
                            id,
                            name,
                            stats,
                            mods,
                        })
                    })
                })
                .take(6)
                .collect::<ArrayVec<_, 6>>();

            let targets = encounter.enemy_targets.resolve(process);
            for (target, enemy) in targets
                .into_iter()
                .flat_map(|o| o.iter(process))
                .zip(enemies.iter_mut())
            {
                if let Some(target) = target.resolve_with((process, &self.e_target)) {
                    enemy.stats.current_hp = target.current_hp;
                }
            }

            Some(BattleEncounter {
                done: encounter.done,
                boss: encounter.boss,
                full_heal_after: encounter.full_heal_after,
                underwater: encounter.underwater,
                all_enemies_killed: encounter.all_enemies_killed,
                running: encounter.running,
                xp_gain: encounter.xp_gain,
                has_achievement: !encounter.has_achievement.is_null(),
                loot,
                enemies,
            })
        }
    }

    #[cfg(debugger)]
    #[derive(Debug)]
    pub struct BattleEncounter {
        pub done: bool,
        pub boss: bool,
        full_heal_after: bool,
        underwater: bool,
        all_enemies_killed: bool,
        running: bool,
        xp_gain: u32,
        has_achievement: bool,
        loot: EncounterLoot,
        enemies: ArrayVec<EnemyInfo, 6>,
    }

    #[cfg(debugger)]
    impl BattleEncounter {
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
            .chain(self.enemies.iter().flat_map(move |o| {
                [
                    EnemyEncounter::Enemy(Enemy {
                        id: o.id.as_str(),
                        name: &o.name,
                        hide_hp: o.hide_hp,
                        award_xp: o.gives_xp,
                        gold_drop: o.gold,
                    }),
                    EnemyEncounter::EnemyStats(o.stats),
                    EnemyEncounter::EnemyMods(o.mods),
                ]
            }))
        }
    }

    #[cfg(debugger)]
    #[derive(Debug)]
    struct EnemyInfo {
        hide_hp: bool,
        gives_xp: bool,
        gold: u32,
        id: ArrayString<36>,
        name: String,
        stats: EnemyStats,
        mods: EnemyMods,
    }

    #[cfg(debugger)]
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

    #[cfg(debugger)]
    #[derive(Copy, Clone, Debug)]
    pub struct Enemy<'a> {
        pub id: &'a str,
        pub name: &'a str,
        pub award_xp: bool,
        pub gold_drop: u32,
        pub hide_hp: bool,
    }

    #[cfg(debugger)]
    #[derive(Copy, Clone, Debug)]
    pub struct EnemyStats {
        pub current_hp: u32,
        pub max_hp: u32,
        pub level: u32,
        pub speed: u32,
        pub attack: u32,
        pub defense: u32,
        pub magic_attack: u32,
        pub magic_defense: u32,
    }

    #[cfg(debugger)]
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

    #[cfg(debugger)]
    pub enum EnemyEncounter<'a> {
        General(General),
        Enemy(Enemy<'a>),
        EnemyStats(EnemyStats),
        EnemyMods(EnemyMods),
    }

    #[cfg(debugger)]
    impl fmt::Debug for EnemyEncounter<'_> {
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
                        id, name, hide_hp, ..
                    }) => {
                        write!(f, "Enemy: id={} name={} hide_hp={}", id, name, hide_hp)
                    }
                    EnemyEncounter::EnemyStats(EnemyStats {
                        current_hp,
                        max_hp,
                        level,
                        speed,
                        attack,
                        defense,
                        magic_attack,
                        magic_defense,
                        ..
                    }) => {
                        write!(
                            f,
                            "|--stats: hp={}/{} level={} speed={} A/MA={}/{} D/MD={}/{}",
                            current_hp,
                            max_hp,
                            level,
                            speed,
                            attack,
                            magic_attack,
                            defense,
                            magic_defense
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
                                "moon={} poison={} arcane={} eclipse={} stun={}"
                            ),
                            any, sword, blunt, sun, moon, poison, arcane, eclipse, stun
                        )
                    }
                }
            }
        }
    }
}

mod progress {
    #[cfg(debugger)]
    use std::rc::Rc;

    #[cfg(debugger)]
    use ahash::HashMap;
    #[cfg(debugger)]
    use asr::{arrayvec::ArrayString, time::Duration};
    use asr::{
        game_engine::unity::il2cpp::{Class, Image, Module},
        Process,
    };
    #[cfg(debugger)]
    use bytemuck::AnyBitPattern;
    #[cfg(debugger)]
    use csharp_mem::{CSString, Map, Pointer, Set};

    use super::Singleton;

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
        #[cfg(debugger)]
        #[cfg_attr(debugger, rename = "allActivityData")]
        all_activities: Pointer<Map<ActivityReference, Pointer<ActivityData>>>,
    }

    #[cfg(debugger)]
    #[derive(Copy, Clone, Debug, AnyBitPattern)]
    struct ActivityReference {
        guid: Pointer<CSString>,
    }

    #[cfg(debugger)]
    #[derive(Class, Debug)]
    struct ActivityData {
        #[rename = "activityNameLoc"]
        name: super::loc::LocalizationId,
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
        #[rename = "levelDefinitionPerLevel"]
        all_levels: Pointer<Map<LevelReference, Pointer<LevelDefinition>>>,
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
    #[derive(Class, Debug)]
    struct LevelDefinition {
        #[rename = "levelNameLocId"]
        name: super::loc::LocalizationId,
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
    #[cfg(debugger)]
    binding!(ActivityDataBinding => ActivityData);
    #[cfg(debugger)]
    binding!(LevelDefinitionBinding => LevelDefinition);

    pub struct Progression {
        level_manager: Singleton<LevelManagerBinding>,
        #[cfg(debugger)]
        progression_manager: Singleton<ProgressionManagerBinding>,
        #[cfg(debugger)]
        activity_manager: Singleton<ActivityManagerBinding>,
        #[cfg(debugger)]
        all_activities: HashMap<String, ActivityData>,
        #[cfg(debugger)]
        loaded_level_info: LoadedLevelInfoBinding,
        #[cfg(debugger)]
        all_levels: HashMap<String, LevelDefinition>,
    }

    impl Progression {
        pub async fn new(process: &Process, module: &Module, image: &Image) -> Self {
            let level_manager = LevelManagerBinding::new(process, module, image).await;

            #[cfg(debugger)]
            {
                let (loaded_level_info, activity_data, level_definition) = binds!(
                    process,
                    module,
                    image,
                    (LoadedLevelInfo, ActivityData, LevelDefinition)
                );

                let progression_manager =
                    ProgressionManagerBinding::new(process, module, image).await;
                let activity_manager = ActivityManagerBinding::new(process, module, image).await;

                let all_activities = ActivityManagerBinding::resolve(&activity_manager, process)
                    .into_iter()
                    .flat_map(|o| {
                        let process = super::Proc(process);
                        let activity_data = &activity_data;
                        o.all_activities
                            .resolve(process)
                            .into_iter()
                            .flat_map(move |o| {
                                o.iter(process).filter_map(move |(ar, ad)| {
                                    let ad = ad.resolve_with((process, activity_data))?;
                                    let ar = ar.guid.resolve(process)?.to_std_string(process);
                                    Some((ar, ad))
                                })
                            })
                    })
                    .collect();

                let all_levels = LevelManagerBinding::resolve(&level_manager, process)
                    .into_iter()
                    .flat_map(|o| {
                        let process = super::Proc(process);
                        let level_definition = &level_definition;
                        o.all_levels
                            .resolve(process)
                            .into_iter()
                            .flat_map(move |o| {
                                o.iter(process).filter_map(move |(lr, ld)| {
                                    let ld = ld.resolve_with((process, level_definition))?;
                                    let lr = lr.guid.resolve(process)?.to_std_string(process);
                                    Some((lr, ld))
                                })
                            })
                    })
                    .collect();

                Self {
                    level_manager,
                    progression_manager,
                    activity_manager,
                    all_activities,
                    loaded_level_info,
                    all_levels,
                }
            }

            #[cfg(not(debugger))]
            {
                Self { level_manager }
            }
        }

        #[cfg(debugger)]
        pub fn get_progress(
            &self,
            process: &Process,
            loc: &super::loc::Loc,
        ) -> Option<CurrentProgress> {
            let process = super::Proc(process);

            let level = LevelManagerBinding::resolve(&self.level_manager, process.0)?;

            let progression =
                ProgressionManagerBinding::resolve(&self.progression_manager, process.0)?;

            let number_of_defeated_perma_death_enemies = progression
                .defeated_perma_death_enemies
                .resolve(process)?
                .size();

            let activity = ActivityManagerBinding::resolve(&self.activity_manager, process.0)?;
            let main_activity = activity.main_activity.resolve(process)?.to_string(process);
            let activity_name = self
                .all_activities
                .get(main_activity.as_str())
                .map_or_else(|| "".into(), |o| loc.localized(process, o.name).into());

            let current_level = level
                .current_level
                .guid
                .resolve(process)?
                .to_string(process);
            let current_level_name = self
                .all_levels
                .get(current_level.as_str())
                .map_or_else(|| "".into(), |o| loc.localized(process, o.name).into());

            let previous_level = level
                .previous_level_info
                .resolve_with((process, &self.loaded_level_info))
                .and_then(|o| o.level.guid.resolve(process).map(|o| o.to_string(process)))
                .unwrap_or_default();
            let previous_level_name = self
                .all_levels
                .get(previous_level.as_str())
                .map_or_else(|| "".into(), |o| loc.localized(process, o.name).into());

            Some(CurrentProgress {
                is_loading: level.is_loading,
                timestamp: progression.timestamp,
                play_time: progression.play_time,
                main_activity,
                activity_name,
                sub_activity_index: activity.sub_activity_index,
                current_level,
                current_level_name,
                previous_level,
                previous_level_name,
                number_of_defeated_perma_death_enemies,
            })
        }

        #[cfg(not(debugger))]
        pub fn get_progress(&self, process: &Process) -> Option<CurrentProgress> {
            let process = super::Proc(process);
            let level = LevelManagerBinding::resolve(&self.level_manager, process.0)?;

            Some(CurrentProgress {
                is_loading: level.is_loading,
            })
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
        pub activity_name: Rc<str>,
        #[cfg(debugger)]
        pub sub_activity_index: u32,
        #[cfg(debugger)]
        pub current_level: ArrayString<36>,
        #[cfg(debugger)]
        pub current_level_name: Rc<str>,
        #[cfg(debugger)]
        pub previous_level: ArrayString<36>,
        #[cfg(debugger)]
        pub previous_level_name: Rc<str>,
        #[cfg(debugger)]
        pub number_of_defeated_perma_death_enemies: u32,
    }

    #[cfg(debugger)]
    impl CurrentProgress {
        pub fn play_time(&self) -> PlayTime {
            PlayTime {
                session: Duration::seconds_f64(self.timestamp),
                total: Duration::seconds_f64(self.play_time),
            }
        }

        pub fn activity(&self) -> Activity {
            Activity {
                id: self.main_activity,
                name: Rc::clone(&self.activity_name),
                sub_index: self.sub_activity_index,
                perma_dead: self.number_of_defeated_perma_death_enemies,
            }
        }

        pub fn current_level(&self) -> Level {
            Level {
                is_loading: self.is_loading,
                id: self.current_level,
                name: Rc::clone(&self.current_level_name),
            }
        }

        pub fn prev_level(&self) -> Level {
            Level {
                is_loading: self.is_loading,
                id: self.previous_level,
                name: Rc::clone(&self.previous_level_name),
            }
        }
    }

    #[cfg(debugger)]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct PlayTime {
        pub session: Duration,
        pub total: Duration,
    }

    #[cfg(debugger)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Activity {
        pub id: ArrayString<36>,
        pub name: Rc<str>,
        pub sub_index: u32,
        pub perma_dead: u32,
    }

    #[cfg(debugger)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Level {
        pub is_loading: bool,
        pub id: ArrayString<36>,
        pub name: Rc<str>,
    }
}

#[cfg(debugger)]
mod loc {
    use ahash::HashMap;
    use asr::{
        game_engine::unity::il2cpp::{Class, Image, Module},
        Process,
    };
    use bytemuck::AnyBitPattern;
    use csharp_mem::{CSString, List, Map, MemReader, Pointer};

    use super::Singleton;

    #[derive(Class, Debug)]
    pub struct LocalizationManager {
        #[rename = "locCategories"]
        pub loc_categories: Pointer<Map<Pointer<CSString>, Pointer<LocCategory>>>,
        #[rename = "locCategoryLanguages"]
        pub loc_category_languages: Pointer<Map<Pointer<CSString>, Pointer<LocCategoryLanguage>>>,
    }

    impl LocalizationManagerBinding {
        singleton!(LocalizationManager);
    }

    #[derive(Class, Debug)]
    pub struct LocCategory {
        #[rename = "categoryId"]
        pub category_id: Pointer<CSString>,
        #[rename = "locIndexByLocStringId"]
        pub loc_index_by_loc_string_id: Pointer<LocIndexByLocStringId>,
    }

    #[derive(Class, Debug)]
    pub struct LocIndexByLocStringId {
        pub dictionary: Pointer<Map<Pointer<CSString>, u32>>,
    }

    #[derive(Class, Debug)]
    pub struct LocCategoryLanguage {
        #[rename = "locCategoryId"]
        pub loc_category_id: Pointer<CSString>,
        pub language: ELanguage,
        pub strings: Pointer<List<Pointer<CSString>>>,
    }

    binding!(LocCategoryBinding => LocCategory);
    binding!(LocIndexByLocStringIdBinding => LocIndexByLocStringId);
    binding!(LocCategoryLanguageBinding => LocCategoryLanguage);

    #[derive(Copy, Clone, Debug, AnyBitPattern)]
    #[repr(C)]
    pub struct LocalizationId {
        pub category_name: Pointer<CSString>,
        pub loc_id: Pointer<CSString>,
    }

    #[derive(Copy, Clone, Debug, AnyBitPattern)]
    #[repr(C)]
    pub struct ELanguage {
        value: u32,
    }

    #[allow(unused)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(u32)]
    pub enum Language {
        EN = 0,
        JP = 1,
        RU = 2,
        KO = 3,
        QC = 4,
        FR = 5,
        DE = 6,
        ES = 7,
        BR = 8,
        CN = 9,
        HK = 10,
    }

    impl From<ELanguage> for Language {
        fn from(value: ELanguage) -> Self {
            unsafe { core::mem::transmute(value.value) }
        }
    }

    pub struct Localization {
        manager: Singleton<LocalizationManagerBinding>,
        category: LocCategoryBinding,
        index_by_id: LocIndexByLocStringIdBinding,
        category_language: LocCategoryLanguageBinding,
    }

    impl Localization {
        pub async fn new(process: &Process, module: &Module) -> Self {
            let image = module
                .wait_get_image(process, "Sabotage.Localization")
                .await;
            let manager = LocalizationManagerBinding::new(process, module, &image).await;

            let (category, index_by_id, category_language) = binds!(
                process,
                module,
                &image,
                (LocCategory, LocIndexByLocStringId, LocCategoryLanguage)
            );

            Self {
                manager,
                category,
                index_by_id,
                category_language,
            }
        }
    }

    #[derive(Debug)]
    pub struct Loc {
        pub categories: HashMap<String, Category>,
        pub strings: HashMap<String, CategoryLanguage>,
    }

    impl Localization {
        pub fn resolve(&self, process: &Process) -> Option<Loc> {
            let manager = LocalizationManagerBinding::resolve(&self.manager, process)?;
            let process = super::Proc(process);

            let categories = manager.loc_categories.resolve(process)?;
            let categories = categories
                .iter(process)
                .filter_map(|(id, cateogry)| {
                    let id = id.resolve(process)?.to_std_string(process);
                    let category = cateogry.resolve_with((process, &self.category))?;
                    let category = category.resolve(process, &self.index_by_id)?;
                    Some((id, category))
                })
                .collect();

            let strings = manager.loc_category_languages.resolve(process)?;
            let strings = strings
                .iter(process)
                .filter_map(|(id, lang)| {
                    let id = id.resolve(process)?.to_std_string(process);

                    let lang = lang.resolve_with((process, &self.category_language))?;
                    let lang = lang.resolve(process)?;
                    Some((id, lang))
                })
                .collect();

            Some(Loc {
                categories,
                strings,
            })
        }
    }

    #[derive(Debug)]
    pub struct Category {
        pub id: Box<str>,
        pub index: HashMap<Box<str>, usize>,
    }

    impl LocCategory {
        fn resolve(
            &self,
            process: super::Proc<'_>,
            index_by_id: &LocIndexByLocStringIdBinding,
        ) -> Option<Category> {
            let id = self
                .category_id
                .resolve(process)?
                .to_std_string(process)
                .into_boxed_str();

            let index = self
                .loc_index_by_loc_string_id
                .resolve_with((process, index_by_id))?;

            let index = index.dictionary.resolve(process)?;

            let index = index
                .iter(process)
                .into_iter()
                .flat_map(|(id, index)| {
                    let id = id.resolve(process)?.to_std_string(process).into_boxed_str();
                    let index = index as usize;
                    Some((id, index))
                })
                .collect();

            Some(Category { id, index })
        }
    }

    #[derive(Debug)]
    pub struct CategoryLanguage {
        pub id: Box<str>,
        pub language: Language,
        pub strings: List<Pointer<CSString>>,
    }

    impl LocCategoryLanguage {
        fn resolve(&self, process: super::Proc<'_>) -> Option<CategoryLanguage> {
            let id = self
                .loc_category_id
                .resolve(process)?
                .to_std_string(process)
                .into_boxed_str();

            let language = self.language.into();

            let strings = self.strings.resolve(process)?;

            Some(CategoryLanguage {
                id,
                language,
                strings,
            })
        }
    }

    impl Loc {
        pub async fn new(process: &Process, module: &Module, _image: &Image) -> Option<Self> {
            let loc = Localization::new(process, module).await;
            match loc.resolve(process) {
                Some(loc) => {
                    // loc.dump(process);
                    Some(loc)
                }
                None => None,
            }
        }

        pub fn localized<R: MemReader + Copy>(&self, process: R, id: LocalizationId) -> String {
            self.lookup(process, id).map_or_else(
                || {
                    id.loc_id
                        .resolve(process)
                        .map_or_else(String::new, |o| o.to_std_string(process))
                },
                |(n, _)| n,
            )
        }

        pub fn lookup<R: MemReader + Copy>(
            &self,
            process: R,
            loc_id: LocalizationId,
        ) -> Option<(String, Language)> {
            let cat_id = loc_id
                .category_name
                .resolve(process)?
                .to_std_string(process);

            let cat = self.categories.get(&cat_id)?;
            let strings = self.strings.get(&cat_id)?;

            let loc_id = loc_id
                .loc_id
                .resolve(process)?
                .to_std_string(process)
                .into_boxed_str();

            let index = *cat.index.get(&loc_id)?;

            let string = strings.strings.get(process, index)?;
            let string = string.resolve(process)?.to_std_string(process);

            Some((string, strings.language))
        }

        #[allow(unused)]
        fn dump(&self, process: &Process) {
            let categories = self
                .categories
                .iter()
                .map(|(k, v)| {
                    let v = v
                        .index
                        .iter()
                        .map(|(k, v)| (*v, k.clone()))
                        .collect::<HashMap<_, _>>();
                    (k.clone(), v)
                })
                .collect::<HashMap<_, _>>();

            let process = super::Proc(process);

            for (k, v) in self.strings.iter() {
                log!("--- {k} ---");
                log!("id, name");
                if let Some(cat) = categories.get(k) {
                    for (idx, name) in v.strings.iter(process).enumerate() {
                        let name = name
                            .resolve(process)
                            .map(|o| o.to_std_string(process))
                            .unwrap_or_default();
                        let id = cat.get(&idx).map(|o| &**o).unwrap_or_default();
                        log!("{id}, {name}");
                    }
                }
            }
        }
    }
}

#[cfg(debugger)]
mod inventory {
    use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
    use asr::{
        game_engine::unity::il2cpp::{Class, Image, Module},
        string::ArrayString,
        watcher::Watcher,
        Process,
    };
    use bytemuck::AnyBitPattern;

    use csharp_mem::{CSString, Map, Pointer};

    use super::{
        loc::{Loc, LocalizationId},
        Singleton,
    };

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
        #[rename = "nameLocalizationId"]
        name: LocalizationId,
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

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct NamedKeyItem {
        pub id: ArrayString<32>,
        pub name: String,
    }

    pub struct Inventory {
        inventory_item: InventoryItemBinding,
        key_item: KeyItemBinding,
        quantity: QuantityByInventoryItemReferenceBinding,
        manager: Singleton<InventoryManagerBinding>,
        all_key_items: HashMap<ArrayString<32>, NamedKeyItem>,
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
                all_key_items: HashMap::new(),
                owned_key_items: HashSet::new(),
            }
        }

        pub fn check_new<'a>(
            &'a mut self,
            process: &'a Process,
            module: &'a Module,
            loc: &'a Loc,
        ) -> impl Iterator<Item = &'a NamedKeyItem> + 'a {
            self.cache_available_items(process, module, loc);
            self.new_owned_key_items(process)
        }

        pub fn check_lost<'a>(
            &'a mut self,
            process: &'a Process,
            module: &'a Module,
            loc: &'a Loc,
        ) -> impl Iterator<Item = NamedKeyItem> + 'a {
            self.cache_available_items(process, module, loc);
            self.lost_owned_key_items(process)
        }

        fn cache_available_items(&mut self, process: &Process, module: &Module, loc: &Loc) -> bool {
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
                        let id = item.guid.resolve(process)?.to_string(process);
                        let name = loc.localized(process, item.name);
                        let item = NamedKeyItem { id, name };
                        self.all_key_items.insert(id, item);
                    }
                }

                Some(())
            })();

            !self.all_key_items.is_empty()
        }

        fn new_owned_key_items<'a>(
            &'a mut self,
            process: &'a Process,
        ) -> impl Iterator<Item = &'a NamedKeyItem> + 'a {
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

                    let first = self.number_of_owned_items.pair.is_none();
                    let now_owned = self.number_of_owned_items.update_infallible(amount);

                    (first || now_owned.changed()).then(move || {
                        owned.iter(process).filter_map(move |(item, _amount)| {
                            let item = item.guid.resolve(process)?;
                            let item = item.to_string(process);
                            key_items
                                .get(&item)
                                .and_then(|v| owned_items.insert(item).then_some(v))
                        })
                    })
                })
                .into_iter()
                .flatten()
        }

        fn lost_owned_key_items<'a>(
            &'a mut self,
            process: &'a Process,
        ) -> impl Iterator<Item = NamedKeyItem> + 'a {
            Some(&mut self.all_key_items)
                .and_then(|key_items| {
                    let process = super::Proc(process);
                    let manager = InventoryManagerBinding::resolve(&self.manager, process.0)?;
                    let owned = manager
                        .owned_items
                        .resolve_with((process, &self.quantity))?;
                    let owned = owned.dictionary.resolve(process)?;

                    let amount = owned.size();
                    let owned_items = &mut self.owned_key_items;

                    let first = self.number_of_owned_items.pair.is_none();
                    let now_owned = self.number_of_owned_items.update_infallible(amount);

                    (first || now_owned.changed()).then(move || {
                        let mut prev_owned = owned_items.clone();
                        for (item, _amount) in owned.iter(process) {
                            let item = item.guid.resolve(process)?;
                            let item = item.to_string(process);
                            prev_owned.remove(&item);
                        }
                        Some(prev_owned.into_iter().filter_map(|o| key_items.remove(&o)))
                    })
                })
                .into_iter()
                .flatten()
                .flatten()
        }
    }
}
