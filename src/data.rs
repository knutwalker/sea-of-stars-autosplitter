use asr::{
    Process,
    arrayvec::ArrayVec,
    game_engine::unity::il2cpp::{Module, Version},
    string::ArrayString,
    timer,
};
use bytemuck::AnyBitPattern;

use crate::{
    mapping::{Enemy, Level},
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

pub type Enemies = ArrayVec<Enemy, 6>;

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
        let read_enemy = |(idx, ptr): (usize, Pointer<EnemyCombatActor>)| -> Option<Enemy> {
            let actor = self.combat.actor.read(process, ptr.addr()).ok()?;
            let data = self.combat.char.read(process, actor.data.addr()).ok()?;

            let guid = data.guid.to_string::<_, 32>(process).unwrap_or_default();
            let key = match idx {
                0 => "enemy_0",
                1 => "enemy_1",
                2 => "enemy_2",
                3 => "enemy_3",
                4 => "enemy_4",
                5 => "enemy_5",
                6 => "enemy_6",
                _ => "enemy_x",
            };
            timer::set_variable(key, guid.as_str());
            Enemy::resolve(data.guid.chars(process)?)
        };
        let ptr = self
            .combat
            .encounter
            .read::<Pointer<Encounter>>(process, &self.asm)?;
        let enc = self.combat.enc.read(process, ptr.addr()).ok()?;
        let enemies = enc.enemy_actors.iter(process)?;
        let enemies = enemies
            .enumerate()
            .filter_map(read_enemy)
            .map(|o| match o {
                Enemy::DwellerOfStrife1 if enc.boss == false => Enemy::DwellerOfStrife2,
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
            .and_then(|o| {
                let level_guid = o.guid.to_string::<_, 32>(process).unwrap_or_default();
                timer::set_variable("level", level_guid.as_str());
                o.guid.chars(process).and_then(Level::resolve)
            });

        if let Some(ref level) = level {
            use core::fmt::Write;
            let mut buf = ArrayString::<32>::new();
            let _ = write!(&mut buf, "{:?}", level);
            timer::set_variable("level_name", buf.as_str());
        }

        let in_cutscene = self
            .progress
            .is_in_cutscene
            .u32(process, &self.asm)
            .is_some_and(|o| o != 0);
        timer::set_variable("in_cutscene", if in_cutscene { "true" } else { "false" });

        Progression { in_cutscene, level }
    }

    pub fn owned_items(&self, process: &Process) -> Option<Pointer<Map<Reference, u32>>> {
        return self.inventory.owned_items.read(process, &self.asm);
    }
}
