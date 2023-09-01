use asr::{
    game_engine::unity::il2cpp::{Class, Module, Version},
    Address, Address64, Process,
};

pub struct Data<'a> {
    process: &'a Process,
    char_select: CharacterSelectionScreenBinding,
    level: Singleton<LevelManagerBinding>,
    combat: Singleton<CombatManagerBinding>,
    encounter: EncounterBinding,
    title_screen: TitleScreen,
}

pub enum GameState {
    NotStarted,
    JustStarted,
    AlreadyRunning,
}

impl Data<'_> {
    pub fn game_start(&self) -> GameState {
        if let Some(char_select) = self.title_screen.get(self.process).and_then(|o| {
            self.char_select
                .read(self.process, o.selection_screen.into())
                .ok()
        }) {
            if char_select.selected {
                GameState::JustStarted
            } else {
                GameState::NotStarted
            }
        } else {
            GameState::AlreadyRunning
        }
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
}

#[derive(Class)]
struct CharacterSelectionScreen {
    #[rename = "characterSelected"]
    selected: bool,
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
        let level = bind!(singleton LevelManager);
        let combat = bind!(singleton CombatManager);
        let encounter = bind!(Encounter);

        let title_screen = bind!(TitleSequenceManager);
        let title_screen = TitleScreen {
            module,
            bind: title_screen,
        };

        Self {
            process,
            char_select,
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

struct TitleScreen {
    module: Module,
    bind: TitleSequenceManagerBinding,
}

impl TitleScreen {
    fn get(&self, process: &Process) -> Option<TitleSequenceManager> {
        let parent = self.bind.class().get_parent(process, &self.module)?;
        let static_table = parent.get_static_table(process, &self.module)?;
        let instance_offset = parent.get_field(process, &self.module, "instance")?;
        let location = static_table + instance_offset;

        let addr = process.read::<Address64>(location).ok()?;
        if !addr.is_null() {
            self.bind.read(process, addr.into()).ok()
        } else {
            None
        }
    }
}
