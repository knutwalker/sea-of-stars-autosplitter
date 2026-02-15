use core::{fmt::Write, num::NonZeroU32};

use crate::{
    Action, Settings,
    data::{Data, EncounterData, SpeedrunRelic},
    mapping::{Enemy, Level},
    utils::{EnumSet, EnumSetMember},
};
use asr::{Process, arrayvec::ArrayVec, string::ArrayString, timer, watcher::Watcher};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Split {
    Wyrd,
    Bossslug,
    ElderMist,
    Rockie,
    Malkomud,
    Chromatic,
    Duke,
    Romaya,
    BigPlant,
    DwellerOfWoe,
    Stormcaller,
    OneThree,
    TwoFour,
    DwellerOfTorment,
    LeafMonster,
    ErlynaAndBrugaves,
    OneTwoThreeFour,
    DwellerOfStrifeP1,
    DwellerOfStrifeP2,
    Hydralion,
    Toadcano,
    Guardian,
    Meduso,
    Triumvirate,
    Catalyst,
    DwellerOfDread,
    LeJugg,
    Reaper,
    ElysandarelleP1,
    ElysandarelleP2,
    WorldEater,
}

impl Split {
    pub fn all() -> impl Iterator<Item = Split> {
        (u8::MIN..u8::MAX).map_while(|o| Split::try_from_primitive(o).ok())
    }

    pub fn is_enabled(self, settings: &Settings) -> bool {
        if settings.split && self.compat() {
            return true;
        }
        return self.filter(settings);
    }

    fn compat(&self) -> bool {
        matches!(
            self,
            Split::Bossslug
                | Split::ElderMist
                | Split::Malkomud
                | Split::Chromatic
                | Split::Romaya
                | Split::BigPlant
                | Split::DwellerOfWoe
                | Split::Stormcaller
                | Split::DwellerOfTorment
                | Split::LeafMonster
                | Split::ErlynaAndBrugaves
                | Split::OneTwoThreeFour
                | Split::DwellerOfStrifeP1
                | Split::Hydralion
                | Split::Toadcano
                | Split::Guardian
                | Split::Triumvirate
                | Split::Catalyst
                | Split::DwellerOfDread
                | Split::LeJugg
                | Split::Reaper
                | Split::ElysandarelleP2
                | Split::WorldEater
        )
    }

    pub fn filter(&self, settings: &Settings) -> bool {
        match self {
            Split::Wyrd => settings.wyrd,
            Split::Bossslug => settings.bossslug,
            Split::ElderMist => settings.elder_mist,
            Split::Rockie => settings.rockie,
            Split::Malkomud => settings.malkomud,
            Split::Chromatic => settings.chromatic,
            Split::Duke => settings.duke,
            Split::Romaya => settings.romaya,
            Split::BigPlant => settings.big_plant,
            Split::DwellerOfWoe => settings.dweller_of_woe,
            Split::Stormcaller => settings.stormcaller,
            Split::OneThree => settings._13,
            Split::TwoFour => settings._24,
            Split::DwellerOfTorment => settings.dweller_of_torment,
            Split::LeafMonster => settings.leaf_monster,
            Split::ErlynaAndBrugaves => settings.erlyna_and_brugaves,
            Split::OneTwoThreeFour => settings._1234,
            Split::DwellerOfStrifeP1 => settings.dweller_of_strife_p1,
            Split::DwellerOfStrifeP2 => settings.dweller_of_strife_p2,
            Split::Hydralion => settings.hydralion,
            Split::Toadcano => settings.toadcano,
            Split::Guardian => settings.guardian,
            Split::Meduso => settings.meduso,
            Split::Triumvirate => settings.triumvirate,
            Split::Catalyst => settings.catalyst,
            Split::DwellerOfDread => settings.dweller_of_dread,
            Split::LeJugg => settings.le_jugg,
            Split::Reaper => settings.reaper,
            Split::ElysandarelleP1 => settings.elysandarelle_p1,
            Split::ElysandarelleP2 => settings.elysandarelle_p2,
            Split::WorldEater => settings.world_eater,
        }
    }
}

impl EnumSetMember for Split {
    fn ordinal(&self) -> Option<u8> {
        Some(u8::from(*self))
    }
}

pub struct Running {
    seen: EnumSet<Split>,
    relic_time: Watcher<f64>,
    paused: bool,
    loading: Watcher<bool>,
    cutscene: Watcher<bool>,
    level: Watcher<Level>,
    full_level: Watcher<ArrayString<32>>,
    encounter: Option<EncounterData>,
}

impl Running {
    pub fn new(time: f64) -> Self {
        let mut running = Self {
            seen: EnumSet::empty(),
            relic_time: Watcher::new(),
            paused: false,
            loading: Watcher::new(),
            cutscene: Watcher::new(),
            level: Watcher::new(),
            full_level: Watcher::new(),
            encounter: None,
        };
        let _ = running.relic_time.update_infallible(time);
        running
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Event {
    LoadStart,
    LoadEnd,
    LevelChange { from: Level, to: Level },
    CutsceneStart,
    EncounterEnd(Enemy, bool),
    EncountersEnd(ArrayVec<Enemy, 6>, bool),
}

pub struct EventHandler<'s> {
    pub settings: &'s mut Settings,
    actions: ArrayVec<Action, 8>,
    cutscenes: Delayed,
}

impl<'s> EventHandler<'s> {
    pub fn new(settings: &'s mut Settings) -> Self {
        Self {
            settings,
            actions: ArrayVec::new(),
            cutscenes: Delayed::default(),
        }
    }

    fn accept(&mut self, ev: Event, time: f64) {
        match ev {
            Event::LoadStart | Event::LoadEnd | Event::CutsceneStart => {}
            _ => log!("({:.02}) Event: {:?}", time, ev),
        }

        match ev {
            Event::LoadStart => self.act(Action::Pause, time),
            Event::LoadEnd => self.act(Action::Resume, time),
            Event::LevelChange { from, to } => {
                match (from, to) {
                    (Level::FleshmancersLair, Level::WorldEeater) => {
                        self.cutscenes.set(1, Split::WorldEater);
                        return;
                    }
                    _ => return,
                };
            }
            Event::CutsceneStart => {
                let split = match self.cutscenes.tick() {
                    Some(split) => split,
                    None => return,
                };
                self.act(Action::Split(split), time);
            }
            Event::EncounterEnd(enemy, boss) => {
                if boss {
                    self.act(Action::SplitBoss, time);
                }
                use Enemy::*;
                let split = match enemy {
                    Wyrd => Split::Wyrd,
                    Bossslug => Split::Bossslug,
                    ElderMist => Split::ElderMist,
                    Salamander => Split::Rockie,
                    Malkomud => Split::Malkomud,
                    ChromaticApparition => Split::Chromatic,
                    Duke => Split::Duke,
                    Romaya => Split::Romaya,
                    BotanicalHorror => Split::BigPlant,
                    DwellerOfWoe => Split::DwellerOfWoe,
                    Stormcaller => Split::Stormcaller,
                    DwellerOfTorment => Split::DwellerOfTorment,
                    LeafMonster => Split::LeafMonster,
                    DwellerOfStrife1 => Split::DwellerOfStrifeP1,
                    DwellerOfStrife2 => Split::DwellerOfStrifeP2,
                    Hydralion => Split::Hydralion,
                    Toadcano => Split::Toadcano,
                    Guardian => Split::Guardian,
                    Meduso => Split::Meduso,
                    Catalyst => Split::Catalyst,
                    DwellerOfDread => Split::DwellerOfDread,
                    LeJugg => Split::LeJugg,
                    PhaseReaper => Split::Reaper,
                    Elysandarelle1 => Split::ElysandarelleP1,
                    Elysandarelle2 => Split::ElysandarelleP2,
                    One | Two | Three | Four | Erlina | Brugaves | Casugin | Abstarak
                    | Rachater => return,
                };
                self.act(Action::Split(split), time);
            }
            Event::EncountersEnd(enemies, boss) => {
                if boss {
                    self.act(Action::SplitBoss, time);
                }
                for enemy in enemies.iter() {
                    self.accept(Event::EncounterEnd(*enemy, false), time);
                }
                use Enemy::*;
                let split = match enemies.as_slice() {
                    [One, Three] => Split::OneThree,
                    [Two, Four] => Split::TwoFour,
                    [One, Two, Three, Four] => Split::OneTwoThreeFour,
                    [Erlina, Brugaves] => Split::ErlynaAndBrugaves,
                    [Casugin, Abstarak, Rachater] => Split::Triumvirate,
                    _ => return,
                };
                self.act(Action::Split(split), time);
            }
        }
    }

    fn act(&mut self, action: Action, time: f64) {
        if self.filter(&action, time) {
            let _ = self.actions.try_push(action);
        }
    }

    fn filter(&mut self, action: &Action, time: f64) -> bool {
        match action {
            Action::SplitBoss => {
                if self.settings.split == false {
                    log!(
                        "({:.02}) Skipping encounter_boss: Disabled in settings",
                        time
                    );
                    return false;
                }
            }
            Action::Split(split) => {
                if split.is_enabled(self.settings) == false {
                    log!("({:.02}) Skipping {:?}: Disabled in settings", time, split);
                    return false;
                }
            }
            Action::Pause => {
                if self.settings.remove_loads == false {
                    return false;
                }
            }
            Action::Resume => {
                if self.settings.remove_loads == false {
                    return false;
                }
            }
            Action::StartCharacter | Action::StartRelic => {}
        };
        return true;
    }
}

impl Running {
    pub fn act(&mut self, handler: &mut EventHandler<'_>, action: Action, time: f64) {
        if handler.filter(&action, time) {
            self.act_internal(action, time);
        }
    }

    fn act_internal(&mut self, action: Action, time: f64) {
        match action {
            Action::SplitBoss => {
                log!("({:.02}) Splitting: encounter_boss", time);
                if cfg!(not(dummy)) {
                    timer::split();
                }
            }
            Action::Split(split) => {
                if self.seen.insert(&split) {
                    log!("({:.02}) Splitting: {:?}", time, split);
                    if cfg!(not(dummy)) {
                        timer::split();
                    }
                } else {
                    log!("({:.02}) Skipping {:?}: Duplicate split", time, split);
                }
            }
            Action::Pause => {
                self.paused = true;
                if cfg!(not(dummy)) {
                    timer::pause_game_time();
                }
            }
            Action::Resume => {
                self.paused = false;
                if cfg!(not(dummy)) {
                    timer::resume_game_time();
                }
            }
            Action::StartCharacter | Action::StartRelic => {}
        }
    }

    pub fn tick(&mut self, handler: &mut EventHandler<'_>, process: &Process, data: &Data) {
        let time = self.load_removal(handler, process, data).unwrap_or(0.0);
        self.check_encounter(handler, process, data, time);
        self.check_level(handler, process, data, time);

        for action in handler.actions.drain(..) {
            self.act_internal(action, time);
        }
    }

    fn load_removal(
        &mut self,
        handler: &mut EventHandler<'_>,
        process: &Process,
        data: &Data,
    ) -> Option<f64> {
        if handler.settings.remove_loads == false {
            return None;
        }
        if handler.settings.custom_load_remover == false
            && let Some(SpeedrunRelic::Active(time)) = data.speedrun_time(process)
        {
            let relic_time = self.relic_time.update_infallible(time);
            let time = relic_time.current;
            match (relic_time.increased(), self.paused) {
                // relic and lrt are both running
                (true, false) => {}
                // relic and lrt are both paused
                (false, true) => {}
                // relic paused, lrt is running, need to pause lrt
                (false, false) => handler.accept(Event::LoadStart, time),
                // relic is running, lrt is paused, need to unpause lrt
                (true, true) => handler.accept(Event::LoadEnd, time),
            };
            return Some(time);
        } else if handler.settings.custom_load_remover == true {
            match self.loading.update(data.is_loading(process)) {
                Some(l) if l.changed_to(&false) => handler.accept(Event::LoadEnd, 0.0),
                Some(l) if l.changed_to(&true) => handler.accept(Event::LoadStart, 0.0),
                _ => {}
            };
        }
        return None;
    }

    fn check_encounter(
        &mut self,
        handler: &mut EventHandler<'_>,
        process: &Process,
        data: &Data,
        time: f64,
    ) {
        match (&mut self.encounter, data.encounter_done(process)) {
            // we were in an encounter, and now it's done
            (Some(start), Some(true)) => {
                match start.enemies.as_slice() {
                    [] => {}
                    [single] => handler.accept(Event::EncounterEnd(*single, start.boss), time),
                    _ => {
                        handler.accept(Event::EncountersEnd(start.enemies.take(), start.boss), time)
                    }
                }
                self.encounter = None;
            }
            // we weren't in an encounter, and now we are
            (None, Some(false)) => {
                let Some(mut enemies) = data.encounter_data(process, time) else {
                    return;
                };
                enemies.enemies.sort_unstable();
                if cfg!(vars) {
                    let mut buf = ArrayString::<128>::new();
                    for enemy in enemies.enemies.iter() {
                        if buf.is_empty() == false {
                            let _ = buf.try_push(',');
                        };
                        let _ = write!(&mut buf, "{:?}", enemy);
                    }
                    if buf.is_empty() == false {
                        log!("({:.02}) Encounter: {}", time, buf.as_str());
                    }
                }
                self.encounter = Some(enemies);
            }
            // we thought we were in an encounter, but we're not
            (Some(_), None) => self.encounter = None,
            // we are in an encounter and it's still going
            (Some(_), Some(false)) => {}
            // we aren't in an encounter and there isn't one or it just finished
            (None, None | Some(true)) => {}
        }
    }

    fn check_level(
        &mut self,
        handler: &mut EventHandler<'_>,
        process: &Process,
        data: &Data,
        time: f64,
    ) {
        let progression = data.progression(process);

        let cutscene = self.cutscene.update_infallible(progression.in_cutscene);
        if cutscene.changed_to(&true) {
            handler.accept(Event::CutsceneStart, time);
        }

        let Some(level) = progression.level else {
            return;
        };
        if cfg!(vars) {
            if level.0.is_empty() == false {
                let full_level = self.full_level.update_infallible(level.0);
                if full_level.changed() {
                    timer::set_variable("level", full_level.current.as_str());
                    log!(
                        "({:.02}) Level: {:?} -> {:?}",
                        time,
                        full_level.old,
                        full_level.current
                    );
                }
            }
        }

        let Some(level) = level.1 else {
            return;
        };
        let level = self.level.update_infallible(level);
        if level.changed() == false {
            return;
        }

        if cfg!(vars) {
            let mut buf = ArrayString::<32>::new();
            let _ = write!(&mut buf, "{:?}", level.current);
            timer::set_variable("level_name", buf.as_str());
        }

        log!(
            "({:.02}) Level changed from {:?} to {:?}",
            time,
            level.old,
            level.current
        );
        handler.accept(
            Event::LevelChange {
                from: level.old,
                to: level.current,
            },
            time,
        )
    }
}

#[derive(Debug, Default)]
struct Delayed {
    delay: Option<NonZeroU32>,
    split: Option<Split>,
}

impl Delayed {
    fn set(&mut self, amount: u32, split: Split) {
        self.delay = NonZeroU32::new(amount);
        self.split = Some(split);
    }

    fn tick(&mut self) -> Option<Split> {
        let n = self.delay?;
        self.delay = NonZeroU32::new(n.get() - 1);
        self.delay
            .is_none()
            .then(|| self.split.take().expect("double tick"))
    }
}
