use asr::{
    Process,
    game_engine::unity::il2cpp::{Class, Image, Module, UnityPointer},
};

use crate::utils::ptrpath1;

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
    #[rename = "encounterDone"]
    pub done: bool,
    #[rename = "bossEncounter"]
    pub boss: bool,
}

pub struct Combat {
    pub encounter: UnityPointer<2>,
    pub enc: EncounterBinding,
}

impl Combat {
    pub async fn new(process: &Process, module: &Module, image: &Image) -> Self {
        return Self {
            encounter: ptrpath1("CombatManager", ["instance", "currentEncounter"]),
            enc: Encounter::bind(process, module, image).await,
        };
    }
}
