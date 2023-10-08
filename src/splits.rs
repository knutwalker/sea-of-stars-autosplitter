use core::num::NonZeroU32;

use crate::{
    data::Data,
    game::{Enemy, Event, Game, KeyItem, Level},
};
use crate::{Action, Split};
use asr::{arrayvec::ArrayVec, msg};

pub struct Progress {
    game: Game,
    actions: ArrayVec<Action, 4>,
    cutscenes: Delayed,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            game: Game::new(),
            actions: ArrayVec::new(),
            cutscenes: Delayed::default(),
        }
    }

    pub(crate) fn actions(&mut self) -> impl Iterator<Item = Action> + '_ {
        self.handle_events();
        self.actions.drain(..)
    }

    pub fn not_running(&mut self, data: &mut Data<'_>) {
        self.game.not_running(data);
    }

    pub fn running(&mut self, data: &mut Data<'_>) {
        self.game.running(data);
    }

    fn handle_events(&mut self) {
        let cutscenes = &mut self.cutscenes;
        for event in self.game.events().flat_map(Event::expand) {
            macro_rules! unhandled {
                () => {{
                    msg!("Unhandled event: {:?}", event);
                    continue;
                }};
            }

            let action = match event {
                Event::GameStarted => Action::Start,
                Event::LoadStart => Action::Pause,
                Event::LoadEnd => Action::Resume,
                Event::PauseStart => Action::GamePause,
                Event::PauseEnd => Action::GameResume,
                Event::LevelChange { from, to } => {
                    use Level::*;
                    match (from, to) {
                        (HomeWorld, ForbiddenCavern) => Action::Split(Split::Training),
                        (MountainTrail, ElderMistTrials) => Action::Split(Split::MountainTrails),
                        (HomeWorld, ArchivistRoom) => Action::Split(Split::Yeet),
                        (HomeWorld, Moorland) => Action::Split(Split::XtolsLanding),
                        (Moorland, HomeWorld) => Action::Split(Split::SolarRain),
                        (CoralCascade, HomeWorld) => Action::Split(Split::ChoralCascades),
                        (BriskOriginal, HomeWorld) => Action::Split(Split::Boat),
                        (Docks, HomeWorld) => Action::Split(Split::WraitIslandDocks),
                        (CursedWood, Lucent) => Action::Split(Split::CursedWoods),
                        (FloodedGraveyard, Lucent) => Action::Split(Split::EnchantedScarf),
                        (BriskDestroyed, Peninsula) => Action::Split(Split::BattleOfBrisk),
                        (BriskRebuilt, HomeWorld) => Action::Split(Split::Ship),
                        (HomeWorld, Mirth) => Action::Split(Split::BuildMirth),
                        (Mirth, ArchivistRoom) => Action::Split(Split::Mirth),
                        (HomeWorld, WaterTemple) => Action::Split(Split::ShoppingConches),
                        (ArchivistRoom, GlacialPeak) => Action::Split(Split::Antsudlo),
                        (GlacialPeak, ArchivistRoom) => Action::Split(Split::SignetOfclarity),
                        (Vespertine, HomeWorld) => Action::Split(Split::BackToMirth),
                        (MesaHike, HomeWorld) => Action::Split(Split::MesaHike),
                        (BambooCreek, HomeWorld) => Action::Split(Split::BambooCreek),
                        (SongShroomMarsh, HomeWorld) => Action::Split(Split::SongshroomMarsh),
                        (SkywardShrine, HomeWorld) => Action::Split(Split::SkywardShrine),
                        (SkyGiantsVillage, HomeWorld) => Action::Split(Split::Council),
                        (Skyland, StormCallerIsland) => Action::Split(Split::AirElemental),
                        (Mooncradle, SkyGiantsVillage) => Action::Split(Split::RIPGarl),
                        (SeraisWorld, Repine) => Action::Split(Split::DerelictFactory),
                        (Repine, SeraisWorld) => Action::Split(Split::Repine),
                        (CeruleanExpanse, LostOnesHamlet) => Action::Split(Split::CeruleanExpense),
                        (SeraisWorld, SacrosanctSpires) => Action::Split(Split::LeavingforSpires),
                        (EstristaesLookout, SeraisWorld) => Action::Split(Split::JustKickIt),
                        (HomeWorld, WizardLab) => {
                            cutscenes.set(2, Action::Split(Split::Brisk));
                            continue;
                        }
                        (_, WorldEeater) => {
                            cutscenes.set(1, Action::Split(Split::WorldEater));
                            continue;
                        }
                        _ => unhandled!(),
                    }
                }
                Event::CutsceneStart => match cutscenes.tick() {
                    Some(action) => action,
                    None => continue,
                },
                Event::CutsceneEnd => continue,
                Event::EncounterStart(enemy) => {
                    use Enemy::*;
                    match enemy {
                        Wyrd => Action::Split(Split::Tutorial),
                        Bossslug => Action::Split(Split::ForbiddenCavern),
                        ElderMist => Action::Split(Split::ElderMistTrials),
                        Salamander => Action::Split(Split::WindMineTunnels),
                        ChromaticApparition => Action::Split(Split::DemoWizardLab),
                        Duke => Action::Split(Split::FloodedGraveyard),
                        Romaya => Action::Split(Split::RopeDart),
                        BotanicalHorror => Action::Split(Split::Garden),
                        DwellerOfWoe => Action::Split(Split::DwellerOfWoeP1),
                        Stormcaller => Action::Split(Split::ThreeTowers),
                        DwellerOfTorment => Action::Split(Split::TormentPeak),
                        LeafMonster => Action::Split(Split::AutumnHills),
                        Toadcano => Action::Split(Split::Volcano),
                        Guardian => Action::Split(Split::SeaofStars),
                        Meduso => Action::Split(Split::LostOnesHamlet),
                        LeJugg => Action::Split(Split::FleshmancersLair),
                        PhaseReaper => Action::Split(Split::NolanSimulator),
                        Elysandarelle1 => Action::Split(Split::FFVIISimulator),
                        Malkomud | Malkomount | BonePile | FleshPile | BottomFlower | TopFlower
                        | BrugavesAlly | ErlynaAlly | One | Two | Three | Four | Erlina
                        | Brugaves | DwellerOfStrife1 | DwellerOfStrife2 | Tail | Hydralion
                        | Casugin | Abstarak | Rachater | Repeater | Catalyst | Tentacle
                        | DwellerOfDread | Elysandarelle2 => {
                            unhandled!()
                        }
                    }
                }
                Event::EncountersStart(ref es) => {
                    use Enemy::*;
                    match es.as_slice() {
                        [One, Three] => Action::Split(Split::JunglePath),
                        [Two, Four] => Action::Split(Split::GlacialPeak),
                        [One, Two, Three, Four] => Action::Split(Split::Watchmaker),
                        [Casugin, Abstarak, Rachater] => Action::Split(Split::HuntingFields),
                        [Repeater, Repeater] => Action::Split(Split::SkyBase),
                        [Tentacle, Tentacle] => Action::Split(Split::InfiniteAbyss),
                        _ => unhandled!(),
                    }
                }
                Event::EncounterEnd(enemy) => {
                    use Enemy::*;
                    match enemy {
                        Wyrd => Action::Split(Split::Wyrd),
                        Bossslug => Action::Split(Split::Bossslug),
                        ElderMist => Action::Split(Split::ElderMist),
                        Salamander => Action::Split(Split::Rockie),
                        Malkomud => Action::Split(Split::Malkomud),
                        ChromaticApparition => Action::Split(Split::Chromatic),
                        Duke => Action::Split(Split::Duke),
                        Romaya => Action::Split(Split::Romaya),
                        BotanicalHorror => Action::Split(Split::BigPlant),
                        DwellerOfWoe => Action::Split(Split::DwellerOfWoe),
                        Stormcaller => Action::Split(Split::Stormcaller),
                        DwellerOfTorment => Action::Split(Split::DwellerOfTorment),
                        LeafMonster => Action::Split(Split::LeafMonster),
                        DwellerOfStrife1 => Action::Split(Split::DwellerOfStrifeP1),
                        DwellerOfStrife2 => Action::Split(Split::DwellerOfStrifeP2),
                        Hydralion => Action::Split(Split::Hydralion),
                        Toadcano => Action::Split(Split::Toadcano),
                        Guardian => Action::Split(Split::Guardian),
                        Meduso => Action::Split(Split::Meduso),
                        Catalyst => Action::Split(Split::Catalyst),
                        DwellerOfDread => Action::Split(Split::DwellerOfDread),
                        LeJugg => Action::Split(Split::LeJugg),
                        PhaseReaper => Action::Split(Split::Reaper),
                        Elysandarelle1 => Action::Split(Split::ElysandarelleP1),
                        Elysandarelle2 => Action::Split(Split::ElysandarelleP2),
                        Malkomount | BonePile | FleshPile | BottomFlower | TopFlower
                        | BrugavesAlly | ErlynaAlly | One | Two | Three | Four | Erlina
                        | Brugaves | Tail | Casugin | Abstarak | Rachater | Repeater | Tentacle => {
                            unhandled!()
                        }
                    }
                }
                Event::EncountersEnd(ref es) => {
                    use Enemy::*;
                    match es.as_slice() {
                        [One, Three] => Action::Split(Split::OneThree),
                        [Two, Four] => Action::Split(Split::TwoFour),
                        [One, Two, Three, Four] => Action::Split(Split::OneTwoThreeFour),
                        [Erlina, Brugaves] => Action::Split(Split::ErlynaAndBrugaves),
                        [Casugin, Abstarak, Rachater] => Action::Split(Split::Triumvirate),
                        _ => unhandled!(),
                    }
                }
                Event::PickedUpKeyItem(KeyItem::Graplou) => Action::Split(Split::NecromancerssLair),
                Event::PickedUpKeyItem(KeyItem::Map) => {
                    cutscenes.set(1, Action::Split(Split::Map));
                    continue;
                }
                Event::LostKeyItem(KeyItem::MasterGhostSandwich) => Action::Split(Split::Cooking),
                Event::LostKeyItem(KeyItem::Seashell) => Action::Split(Split::SacredGrove),
                _ => unhandled!(),
            };
            self.actions.push(action);
        }
    }
}

impl Event {
    fn expand(self) -> ArrayVec<Self, 7> {
        match self {
            Event::EncountersStart(es) => es
                .clone()
                .into_iter()
                .map(Event::EncounterStart)
                .chain(Some(Event::EncountersStart(es)))
                .collect(),
            Event::EncountersEnd(es) => es
                .clone()
                .into_iter()
                .map(Event::EncounterEnd)
                .chain(Some(Event::EncountersEnd(es)))
                .collect(),
            e => [e].into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Delayed {
    delay: Option<NonZeroU32>,
    action: Option<Action>,
}

impl Delayed {
    fn set(&mut self, amount: u32, action: Action) {
        self.delay = NonZeroU32::new(amount);
        self.action = Some(action);
    }

    fn tick(&mut self) -> Option<Action> {
        let n = self.delay?;
        self.delay = NonZeroU32::new(n.get() - 1);
        self.delay
            .is_none()
            .then(|| self.action.take().expect("double tick"))
    }
}
