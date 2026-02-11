use asr::{
    Address, Process,
    game_engine::unity::il2cpp::{Module, Version},
};

use crate::{
    memory::{Combat, Encounter, Loading, Relic, TitleScreen},
    utils::{Assembly, UnityPointerExt},
};

pub struct Data {
    asm: Assembly,
    title: TitleScreen,
    relic: Relic,
    combat: Combat,
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

    pub fn encounter(&self, process: &Process) -> Option<(Address, Encounter)> {
        let address = self.combat.encounter.addr(process, &self.asm)?;
        let encounter = self.resolve_encounter(process, address)?;
        Some((address, encounter))
    }

    pub fn resolve_encounter(&self, process: &Process, address: Address) -> Option<Encounter> {
        return self.combat.enc.read(process, address).ok();
    }
}
