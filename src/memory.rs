use asr::{
    Process,
    game_engine::unity::il2cpp::{Class, Image, Module, UnityPointer},
};

use crate::utils::{CSString, List, Pointer, ptrpath1};

pub struct TitleScreen {
    pub char_selected: UnityPointer<3>,
    pub difficulty_active: UnityPointer<3>,
    pub relic_active: UnityPointer<3>,
}

impl TitleScreen {
    pub fn new() -> Self {
        return Self {
            char_selected: ptrpath1(
                "TitleSequenceManager",
                ["instance", "characterSelectionScreen", "characterSelected"],
            ),
            difficulty_active: ptrpath1(
                "TitleSequenceManager",
                ["instance", "difficultySelectionScreen", "active"],
            ),
            relic_active: ptrpath1(
                "TitleSequenceManager",
                ["instance", "relicSelectionScreen", "active"],
            ),
        };
    }
}

pub struct Relic {
    pub active: UnityPointer<2>,
    pub time: UnityPointer<3>,
}

impl Relic {
    pub fn new() -> Self {
        return Self {
            active: ptrpath1("SpeedrunManager", ["instance", "isSpeedRunning"]),
            time: ptrpath1(
                "SpeedrunManager",
                ["instance", "speedrunTimer", "timerInSecond"],
            ),
        };
    }
}

pub struct Loading {
    pub is_loading: UnityPointer<2>,
}

impl Loading {
    pub fn new() -> Self {
        return Self {
            is_loading: ptrpath1("LevelManager", ["instance", "loadingLevel"]),
        };
    }
}

#[derive(Copy, Clone, Debug, Class)]
pub struct Encounter {
    #[rename = "bossEncounter"]
    pub boss: bool,
    #[rename = "enemyActors"]
    pub enemy_actors: Pointer<List<Pointer<EnemyCombatActor>>>,
}

#[derive(Copy, Clone, Debug, Class)]
pub struct EnemyCombatActor {
    #[rename = "enemyData"]
    pub data: Pointer<EnemyCharacterData>,
}

#[derive(Copy, Clone, Debug, Class)]
pub struct EnemyCharacterData {
    pub guid: Pointer<CSString>,
}

pub struct Combat {
    pub encounter: UnityPointer<2>,
    pub done: UnityPointer<3>,
    pub enc: EncounterBinding,
    pub actor: EnemyCombatActorBinding,
    pub char: EnemyCharacterDataBinding,
}

impl Combat {
    pub async fn new(process: &Process, module: &Module, image: &Image) -> Self {
        return Self {
            encounter: ptrpath1("CombatManager", ["instance", "currentEncounter"]),
            done: ptrpath1(
                "CombatManager",
                ["instance", "currentEncounter", "encounterDone"],
            ),
            enc: Encounter::bind(process, module, image).await,
            actor: EnemyCombatActor::bind(process, module, image).await,
            char: EnemyCharacterData::bind(process, module, image).await,
        };
    }
}

pub struct Progress {
    pub current_level: UnityPointer<3>,
    pub is_in_cutscene: UnityPointer<2>,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            current_level: ptrpath1("LevelManager", ["instance", "currentLevelInfo", "level"]),
            is_in_cutscene: ptrpath1("CutsceneManager", ["instance", "cutsceneCount"]),
        }
    }
}

pub struct Inventory {
    pub owned_items: UnityPointer<3>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            owned_items: ptrpath1(
                "InventoryManager",
                ["instance", "ownedInventoryItems", "dictionary"],
            ),
        }
    }
}
