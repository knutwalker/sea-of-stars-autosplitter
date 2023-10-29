use crate::data::{Change, CurrentEncounter, Data, GameStart};
pub use crate::data::{Enemy, KeyItem, Level};
use asr::{arrayvec::ArrayVec, watcher::Watcher};

pub struct Game {
    loading: Watcher<bool>,
    cutscene: Watcher<bool>,
    level: Watcher<Level>,
    encounter: Option<ArrayVec<Enemy, 6>>,
    events: ArrayVec<Event, 7>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    GameStarted,
    LoadStart,
    LoadEnd,
    #[allow(dead_code)]
    PauseStart,
    #[allow(dead_code)]
    PauseEnd,
    LevelChange {
        from: Level,
        to: Level,
    },
    CutsceneStart,
    CutsceneEnd,
    EncounterStart(Enemy),
    EncounterEnd(Enemy),
    EncountersStart(ArrayVec<Enemy, 6>),
    EncountersEnd(ArrayVec<Enemy, 6>),
    PickedUpKeyItem(KeyItem),
    LostKeyItem(KeyItem),
}

impl Game {
    pub fn new() -> Self {
        Self {
            loading: Watcher::new(),
            cutscene: Watcher::new(),
            level: Watcher::new(),
            encounter: None,
            events: ArrayVec::new(),
        }
    }

    pub fn events(&mut self) -> impl Iterator<Item = Event> + '_ {
        self.events.drain(..)
    }

    pub fn not_running(&mut self, data: &mut Data<'_>) {
        self.start(data);
    }

    pub fn running(&mut self, data: &mut Data<'_>) {
        self.key_item_changes(data);
        self.level_changes(data);
        self.encounter_changes(data);
    }

    fn start(&mut self, data: &mut Data<'_>) -> Option<()> {
        let level = data.current_progression()?.level?;
        if level == Level::TitleScreen {
            let start = data.game_start()?;
            if start == GameStart::JustStarted {
                self.events.push(Event::GameStarted);
            }
        }

        Some(())
    }

    fn key_item_changes(&mut self, data: &mut Data<'_>) {
        for item in data.key_item_changes() {
            let event = match item {
                Change::PickedUp(item) => Event::PickedUpKeyItem(item),
                Change::Lost(item) => Event::LostKeyItem(item),
            };
            self.events.push(event);
        }
    }

    fn level_changes(&mut self, data: &mut Data<'_>) -> Option<()> {
        let progression = data.current_progression()?;

        let loading = self.loading.update_infallible(progression.is_loading);
        if loading.changed_to(&true) {
            self.events.push(Event::LoadStart);
        } else if loading.changed_to(&false) {
            self.events.push(Event::LoadEnd);
        }

        let cutscene = self.cutscene.update_infallible(progression.is_in_cutscene);
        if cutscene.changed_to(&true) {
            self.events.push(Event::CutsceneStart);
        } else if cutscene.changed_to(&false) {
            self.events.push(Event::CutsceneEnd);
        }

        let level = self
            .level
            .update(progression.level)
            .filter(|o| o.changed())?;

        self.events.push(Event::LevelChange {
            from: level.old,
            to: level.current,
        });

        Some(())
    }

    fn encounter_changes(&mut self, data: &mut Data<'_>) {
        match (&mut self.encounter, data.encounter_done()) {
            // we are in an encounter, and now it's done
            (start @ Some(_), Some(true)) => {
                let Some(start) = start.take() else {
                    unreachable!();
                };

                let event = if start.len() == 1 {
                    Event::EncounterEnd(start[0])
                } else {
                    Event::EncountersEnd(start)
                };
                self.events.push(event);
            }
            // we aren't in an encounter, and now we are
            (None, Some(false)) => {
                let CurrentEncounter::InEncounter(mut enemies) = data.current_enemies() else {
                    log!("encounter without enemies");
                    unreachable!();
                };
                enemies.sort_unstable();
                self.encounter = Some(enemies.clone());

                let event = if enemies.len() == 1 {
                    Event::EncounterStart(enemies[0])
                } else {
                    Event::EncountersStart(enemies)
                };
                self.events.push(event);
            }
            // we thought we are in an encounter, but we're not
            (enc @ Some(_), None) => {
                *enc = None;
            }
            // we are in an encounter and it's still going
            (Some(_), Some(false)) => {}
            // we aren't in an encounter and there isn't one or it just finished
            (None, None | Some(true)) => {}
        }
    }
}
