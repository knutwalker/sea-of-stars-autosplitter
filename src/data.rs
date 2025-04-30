use asr::{
    game_engine::unity::il2cpp::{Class, Module, Version},
    Address, Address64, Process,
};

pub struct Data<'a> {
    process: &'a Process,
    level: Singleton<LevelManagerBinding>,
    combat: Singleton<CombatManagerBinding>,
    encounter: EncounterBinding,
    title_screen: TitleScreen<'a>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GameStart {
    TitleScreen,
    CharSelected,
    DifficultyScreen,
    RelicScreen,
    JustStarted,
    Unknown,
}

impl Data<'_> {
    pub fn game_start(&self, current: GameStart) -> GameStart {
        self.try_game_start(current).unwrap_or(GameStart::Unknown)
    }

    fn try_game_start(&self, current: GameStart) -> Option<GameStart> {
        let title_screen = self.title_screen.get()?;
        let char_select = self.title_screen.char_select(&title_screen)?;
        let difficulty_select = self.title_screen.difficulty_select(&title_screen)?;
        let relic_select = self.title_screen.relic_select(&title_screen)?;

        if !char_select.selected {
            return Some(GameStart::TitleScreen);
        }

        if matches!(current, GameStart::CharSelected) && difficulty_select.active {
            return Some(GameStart::DifficultyScreen);
        }

        if matches!(current, GameStart::DifficultyScreen) && relic_select.active {
            return Some(GameStart::RelicScreen);
        }

        if matches!(current, GameStart::RelicScreen)
            && !relic_select.active
            && !difficulty_select.active
        {
            return Some(GameStart::JustStarted);
        }

        Some(if matches!(current, GameStart::TitleScreen) {
            GameStart::CharSelected
        } else {
            current
        })
    }

    pub fn is_loading(&self) -> Option<bool> {
        Some(self.level.read(self.process)?.is_loading)
    }

    pub fn encounter(&self) -> Option<(Address64, Encounter)> {
        let combat = self.combat.read(self.process)?;
        let address = combat.encounter;
        let encounter = self.resolve_encounter(address)?;
        Some((address, encounter))
    }

    pub fn resolve_encounter(&self, address: Address64) -> Option<Encounter> {
        self.encounter.read(self.process, address.into()).ok()
    }
}

#[derive(Class)]
struct LevelManager {
    #[rename = "loadingLevel"]
    is_loading: bool,
}

#[derive(Class)]
struct CombatManager {
    #[rename = "currentEncounter"]
    encounter: Address64,
}

#[derive(Class)]
struct TitleSequenceManager {
    #[rename = "characterSelectionScreen"]
    selection_screen: Address64,
    #[rename = "relicSelectionScreen"]
    relic_screen: Address64,
    #[rename = "difficultySelectionScreen"]
    difficulty_screen: Address64,
}

#[derive(Class)]
struct CharacterSelectionScreen {
    #[rename = "characterSelected"]
    selected: bool,
}

#[derive(Class)]
struct RelicSelectionScreen {
    active: bool,
}

#[derive(Class)]
struct DifficultySelectionScreen {
    active: bool,
}

#[derive(Class, Debug)]
pub struct Encounter {
    #[rename = "encounterDone"]
    pub done: bool,
    #[rename = "bossEncounter"]
    pub boss: bool,
}

impl<'a> Data<'a> {
    pub async fn new(process: &'a Process) -> Data<'a> {
        let module = Module::wait_attach(process, Version::V2020).await;
        let image = module.wait_get_default_image(process).await;
        log!("Attached to the game");

        macro_rules! bind {
            ($cls:ty) => {{
                let binding = <$cls>::bind(process, &module, &image).await;
                log!(concat!("Created binding for class ", stringify!($cls)));
                binding
            }};
            (singleton $cls:ty) => {{
                let binding = <$cls>::bind(process, &module, &image).await;
                let address = binding
                    .class()
                    .wait_get_parent(process, &module)
                    .await
                    .wait_get_static_instance(process, &module, "instance")
                    .await;

                log!(
                    concat!("found ", stringify!($cls), " instance at {}"),
                    address
                );

                Singleton { binding, address }
            }};
        }

        let char_select = bind!(CharacterSelectionScreen);
        let relic_select = bind!(RelicSelectionScreen);
        let difficulty_select = bind!(DifficultySelectionScreen);
        let level = bind!(singleton LevelManager);
        let combat = bind!(singleton CombatManager);
        let encounter = bind!(Encounter);

        let title_screen = bind!(TitleSequenceManager);
        let title_screen = TitleScreen {
            process,
            module,
            bind: title_screen,
            char_select,
            relic_select,
            difficulty_select,
        };

        Self {
            process,
            level,
            combat,
            encounter,
            title_screen,
        }
    }
}

struct Singleton<T> {
    binding: T,
    address: Address,
}

macro_rules! impl_binding {
    ($($cls:ty),+ $(,)?) => {
        $(::paste::paste! {
            impl Singleton<[<$cls Binding>]> {
                fn read(&self, process: &Process) -> Option<$cls> {
                    self.binding.read(process, self.address).ok()
                }
            }
        })+
    };
}

impl_binding!(LevelManager, CombatManager,);

struct TitleScreen<'a> {
    process: &'a Process,
    module: Module,
    bind: TitleSequenceManagerBinding,
    char_select: CharacterSelectionScreenBinding,
    relic_select: RelicSelectionScreenBinding,
    difficulty_select: DifficultySelectionScreenBinding,
}

impl TitleScreen<'_> {
    fn get(&self) -> Option<TitleSequenceManager> {
        let parent = self.bind.class().get_parent(self.process, &self.module)?;
        let static_table = parent.get_static_table(self.process, &self.module)?;
        let instance_offset = parent.get_field_offset(self.process, &self.module, "instance")?;
        let location = static_table + instance_offset;

        let addr = self.process.read::<Address64>(location).ok()?;
        if !addr.is_null() {
            self.bind.read(self.process, addr.into()).ok()
        } else {
            None
        }
    }

    fn char_select(&self, title_screen: &TitleSequenceManager) -> Option<CharacterSelectionScreen> {
        self.char_select
            .read(self.process, title_screen.selection_screen.into())
            .ok()
    }

    fn relic_select(&self, title_screen: &TitleSequenceManager) -> Option<RelicSelectionScreen> {
        self.relic_select
            .read(self.process, title_screen.relic_screen.into())
            .ok()
    }

    fn difficulty_select(
        &self,
        title_screen: &TitleSequenceManager,
    ) -> Option<DifficultySelectionScreen> {
        self.difficulty_select
            .read(self.process, title_screen.difficulty_screen.into())
            .ok()
    }
}
