use asr::{
    Process,
    arrayvec::ArrayVec,
    game_engine::unity::il2cpp::{Module, Version},
};
use bytemuck::AnyBitPattern;

use crate::{
    mapping::{AnyEnemy, Enemy, Level},
    memory::{
        Combat, Encounter, EnemyCombatActor, Inventory, Loading, Progress, Relic, TitleScreen,
    },
    utils::{Assembly, CSString, Map, Pointer, UnityPointerExt},
};

pub struct Data {
    asm: Assembly,
    title: TitleScreen,
    relic: Relic,
    combat: Combat,
    progress: Progress,
    inventory: Inventory,
    loading: Loading,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StartScreen {
    Undecided,
    TitleScreen,
    CharSelected,
    DifficultyScreen,
    RelicScreen,
}

#[derive(Copy, Clone, Debug)]
pub enum SpeedrunRelic {
    Inactive,
    Active(f64),
}

pub type Enemies = ArrayVec<AnyEnemy, 6>;

pub struct EncounterData {
    pub enemies: Enemies,
    pub boss: bool,
}

pub struct Progression {
    pub in_cutscene: bool,
    pub level: Option<Level>,
}

#[derive(Copy, Clone, Debug, AnyBitPattern)]
pub struct Reference {
    pub guid: Pointer<CSString>,
}

impl Data {
    pub async fn wait_new(process: &Process) -> Data {
        let module = Module::wait_attach(process, Version::V2020).await;
        let image = module.wait_get_default_image(process).await;
        log!("Attached to the game");
        let combat = Combat::new(process, &module, &image).await;
        let asm = Assembly { module, image };

        Self {
            asm,
            title: TitleScreen::new(),
            relic: Relic::new(),
            combat,
            progress: Progress::new(),
            inventory: Inventory::new(),
            loading: Loading::new(),
        }
    }

    pub fn start_screen(&self, process: &Process, current: StartScreen) -> Option<StartScreen> {
        let next = self
            .find_start_screen(process, current)
            .unwrap_or(StartScreen::Undecided);
        if current != next {
            return Some(next);
        }
        return None;
    }

    fn find_start_screen(&self, process: &Process, current: StartScreen) -> Option<StartScreen> {
        let char_selected = self.title.char_selected.bool(process, &self.asm)?;
        if char_selected == false {
            return Some(StartScreen::TitleScreen);
        }

        let relic_select = self.title.relic_active.bool(process, &self.asm)?;
        if relic_select && current < StartScreen::RelicScreen {
            return Some(StartScreen::RelicScreen);
        }

        let difficulty_select = self.title.difficulty_active.bool(process, &self.asm)?;
        if difficulty_select && current < StartScreen::DifficultyScreen {
            return Some(StartScreen::DifficultyScreen);
        }

        if current == StartScreen::RelicScreen
            && relic_select == false
            && difficulty_select == false
        {
            return Some(StartScreen::DifficultyScreen);
        }

        (current == StartScreen::TitleScreen)
            .then_some(StartScreen::CharSelected)
            .or(Some(current))
    }

    pub fn is_loading(&self, process: &Process) -> Option<bool> {
        self.loading.is_loading.read(process, &self.asm)
    }

    pub fn speedrun_time(&self, process: &Process) -> Option<SpeedrunRelic> {
        let active = self.relic.active.bool(process, &self.asm)?;
        if active == false {
            return Some(SpeedrunRelic::Inactive);
        }

        let time = self.relic.time.read(process, &self.asm)?;
        return Some(SpeedrunRelic::Active(time));
    }

    pub fn encounter_done(&self, process: &Process) -> Option<bool> {
        return self.combat.done.read(process, &self.asm);
    }

    pub fn encounter_data(&self, process: &Process) -> Option<EncounterData> {
        let read_enemy = |ptr: Pointer<EnemyCombatActor>| -> Option<AnyEnemy> {
            let actor = self.combat.actor.read(process, ptr.addr()).ok()?;
            let data = self.combat.char.read(process, actor.data.addr()).ok()?;
            let guid = data.guid.chars(process)?;
            Enemy::resolve(guid).map(AnyEnemy::Known).or_else(|| {
                #[cfg(debugger)]
                return data.guid.to_string(process).map(AnyEnemy::Unknown);
                #[cfg(not(debugger))]
                return Some(AnyEnemy::Unknown(()));
            })
        };
        let ptr = self
            .combat
            .encounter
            .read::<Pointer<Encounter>>(process, &self.asm)?;
        let enc = self.combat.enc.read(process, ptr.addr()).ok()?;
        let enemies = enc.enemy_actors.iter(process)?;
        let enemies = enemies
            .filter_map(read_enemy)
            .map(|o| match o {
                AnyEnemy::Known(Enemy::DwellerOfStrife1) if enc.boss == false => {
                    AnyEnemy::Known(Enemy::DwellerOfStrife2)
                }
                otherwise => otherwise,
            })
            .collect::<Enemies>();
        if enemies.is_empty() {
            return None;
        }
        Some(EncounterData {
            enemies,
            boss: enc.boss,
        })
    }

    pub fn progression(&self, process: &Process) -> Progression {
        let level = self
            .progress
            .current_level
            .read::<Reference>(process, &self.asm)
            .and_then(|o| o.guid.chars(process))
            .and_then(Level::resolve);

        let in_cutscene = self
            .progress
            .is_in_cutscene
            .bool(process, &self.asm)
            .unwrap_or(false);

        Progression { in_cutscene, level }
    }

    pub fn owned_items(&self, process: &Process) -> Option<Pointer<Map<Reference, u32>>> {
        return self
            .inventory
            .owned_items
            .ptr::<Map<Reference, u32>>(process, &self.asm);
    }
}
