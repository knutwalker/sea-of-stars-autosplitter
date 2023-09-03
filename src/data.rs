use asr::{
    game_engine::unity::il2cpp::{Game, Module, Version},
    Process,
};

use self::{
    combat::{Combat, Encounter},
    inventory::Inventory,
    progress::{CurrentProgression, Progression},
    title_screen::TitleScreen,
};

pub use self::{combat::CurrentEncounter, inventory::Change, title_screen::GameStart};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Enemy {
    Wyrd,
    Bossslug,
    ElderMist,
    Salamander,
    Malkomud,
    Malkomount,
    ChromaticApparition,
    Duke,
    BonePile,
    FleshPile,
    Romaya,
    BottomFlower,
    TopFlower,
    BotanicalHorror,
    BrugavesAlly,
    ErlynaAlly,
    DwellerOfWoe,
    Stormcaller,
    One,
    Two,
    Three,
    Four,
    DwellerOfTorment,
    LeafMonster,
    Erlina,
    Brugaves,
    DwellerOfStrife1,
    DwellerOfStrife2,
    Tail,
    Hydralion,
    Toadcano,
    Guardian,
    Meduso,
    Casugin,
    Abstarak,
    Rachater,
    Repeater,
    Catalyst,
    Tentacle,
    DwellerOfDread,
    LeJugg,
    PhaseReaper,
    Elysandarelle1,
    Elysandarelle2,
}

impl Enemy {
    fn resolve(mut id: impl Iterator<Item = char>) -> Option<Self> {
        let mut depth = 0;
        let mut digits = id.by_ref().inspect(|_| depth += 1);

        macro_rules! eq {
            ($needle:literal => $res:expr) => {
                (id.eq(($needle[depth..]).chars())).then_some($res)
            };
        }

        macro_rules! next {
            ($($arm:literal => $branch:expr),+ $(,)?) => {
                match digits.next()? {
                    $($arm => $branch),+,
                    _ => None,
                }
            };
        }

        next! {
            '0' => next! {
                '2' => eq!("028beee8dff72234a9ad3d8578f6e588" => Self::Duke),
                'c' => next! {
                    '2' => eq!("0c24f27ebab6b854ba75700be2df5b21" => Self::Elysandarelle1),
                    '8' => eq!("0c831eb6bc1c0c648828b405cb8c0667" => Self::Four),
                },
                'e' => eq!("0e5b91e5ad0b2784da76ba6314004370" => Self::Elysandarelle2),
            },
            '1' => next! {
                '8' => eq!("1894c41627be94d408bd64295ab6dd18" => Self::Erlina),
                '9' => eq!("19255ab0a339bd44a8873944c866afc9" => Self::Repeater),
                'd' => eq!("1dc70fb2d0f1b374cbecf052b953824b" => Self::ErlynaAlly),
            },
            '3' => next! {
                '0' => eq!("30bd6b9747d75724496a60116d875f96" => Self::BonePile),
                'b' => eq!("3bbc6ad42918c444c9947d156e7674aa" => Self::BottomFlower),
            },
            '5' => next! {
                '0' => eq!("5044e84c74fc97343ad3c8bcd3c08fdf" => Self::Tail),
                '4' => eq!("54abc79fbf9dd2f4a8bd19cab8245391" => Self::PhaseReaper),
                '7' => eq!("5750f181921e1f349b595e8e47760d33" => Self::Bossslug),
                'c' => eq!("5cdedb65d17f3b24c8b7ad5bcbe1bea6" => Self::Romaya),
            },
            '6' => next! {
                '2' => eq!("621eeda6cacd76740b9b24518c3d211b" => Self::TopFlower),
                '4' => eq!("64246a3a9059257409ea628466ced26e" => Self::BotanicalHorror),
            },
            '7' => next! {
                '3' => eq!("73c4c0922e5ae274eb759f86702353a8" => Self::Two),
                '6' => eq!("76c4290aa2a896b4cb405e5a2d29b3a0" => Self::One),
                '8' => eq!("78457137461e7d345b2287aab380e2e0" => Self::Guardian),
                'a' => eq!("7a8be6ca5e9b7bd49ac7d2da414442cc" => Self::LeJugg),
                'e' => eq!("7e2e026eb3354c74685427b26cf9acb8" => Self::Hydralion),
            },
            '8' => next! {
                '1' => next! {
                    '0' => eq!("810980f005079324fb9fb643243eccee" => Self::Malkomud),
                    '6' => eq!("816de006c125b9b4eaa7139bac5c6b77" => Self::Toadcano),
                },
                'b' => eq!("8beb20a7311444a47b1764ae7ace6658" => Self::Wyrd),
            },
            '9' => next! {
                '4' => eq!("94680e3651254c54ca6030f9461b3ed7" => Self::DwellerOfTorment),
                '6' => eq!("962aa552d33fc124782b230fce9185ce" => Self::ElderMist),
            },
            'a' => next! {
                '1' => eq!("a1c7a4d91b5c8c54b96c3a159ad3a1b5" => Self::DwellerOfDread),
                '3' => eq!("a3b51cc4bda782c41a9ada029c202824" => Self::ChromaticApparition),
                '5' => eq!("a5d39cc10d1848d478b59c892f636e3b" => Self::DwellerOfStrife1),
            },
            'b' => next! {
                '2' => eq!("b2e5237a9dd152643abaf1fb3e3d7206" => Self::Catalyst),
                '4' => eq!("b4e6c3b0168970144a55f4d41fe344c4" => Self::Stormcaller),
                'b' => eq!("bb02eb1602e1ec142b85cd6b505ef5b6" => Self::Meduso),
                'c' => eq!("bcde1eb0ea076f846a0ee20287d88204" => Self::DwellerOfWoe),
                'd' => eq!("bdff582229a41f3438d4c4faac714255" => Self::Casugin),
            },
            'c' => next! {
                '1' => eq!("c109e23c16e478b4e992161662fa81c0" => Self::Tentacle),
                '4' => eq!("c4480713abcb0d04f8a21a702987e6e1" => Self::Rachater),
                '9' => eq!("c99b902697c6f734f9fc64b421c06728" => Self::LeafMonster),
                'c' => eq!("cc767e360aab54d4ca314a206e32ffee" => Self::Brugaves),
            },
            'd' => eq!("d0f2cf59f69f42842ac0703193f39c85" => Self::Salamander),
            'e' => next! {
                '7' => eq!("e77c07b22ee83854e8c006101ef5731f" => Self::Three),
                'b' => eq!("ebf760c7aea1c1d46b18e9db92c5af76" => Self::FleshPile),
                'c' => eq!("ec0b935c78a26044f89a236921671642" => Self::BrugavesAlly),
            },
            'f' => next! {
                '4' => eq!("f4032b2323bc31d4590cf5197db3c3f1" => Self::Abstarak),
                'c' => eq!("fc51f181f5f913f4e99195da947b1425" => Self::Malkomount),
            }
        }
    }

    #[allow(unused)]
    fn id(self) -> &'static str {
        match self {
            Self::Wyrd => "8beb20a7311444a47b1764ae7ace6658",
            Self::Bossslug => "5750f181921e1f349b595e8e47760d33",
            Self::ElderMist => "962aa552d33fc124782b230fce9185ce",
            Self::Salamander => "d0f2cf59f69f42842ac0703193f39c85",
            Self::Malkomud => "810980f005079324fb9fb643243eccee",
            Self::Malkomount => "fc51f181f5f913f4e99195da947b1425",
            Self::ChromaticApparition => "a3b51cc4bda782c41a9ada029c202824",
            Self::Duke => "028beee8dff72234a9ad3d8578f6e588",
            Self::BonePile => "30bd6b9747d75724496a60116d875f96",
            Self::FleshPile => "ebf760c7aea1c1d46b18e9db92c5af76",
            Self::Romaya => "5cdedb65d17f3b24c8b7ad5bcbe1bea6",
            Self::BottomFlower => "3bbc6ad42918c444c9947d156e7674aa",
            Self::TopFlower => "621eeda6cacd76740b9b24518c3d211b",
            Self::BotanicalHorror => "64246a3a9059257409ea628466ced26e",
            Self::BrugavesAlly => "ec0b935c78a26044f89a236921671642",
            Self::ErlynaAlly => "1dc70fb2d0f1b374cbecf052b953824b",
            Self::DwellerOfWoe => "bcde1eb0ea076f846a0ee20287d88204",
            Self::Stormcaller => "b4e6c3b0168970144a55f4d41fe344c4",
            Self::One => "76c4290aa2a896b4cb405e5a2d29b3a0",
            Self::Two => "73c4c0922e5ae274eb759f86702353a8",
            Self::Three => "e77c07b22ee83854e8c006101ef5731f",
            Self::Four => "0c831eb6bc1c0c648828b405cb8c0667",
            Self::DwellerOfTorment => "94680e3651254c54ca6030f9461b3ed7",
            Self::LeafMonster => "c99b902697c6f734f9fc64b421c06728",
            Self::Erlina => "1894c41627be94d408bd64295ab6dd18",
            Self::Brugaves => "cc767e360aab54d4ca314a206e32ffee",
            Self::DwellerOfStrife1 => "a5d39cc10d1848d478b59c892f636e3b",
            Self::DwellerOfStrife2 => "a5d39cc10d1848d478b59c892f636e3b",
            Self::Tail => "5044e84c74fc97343ad3c8bcd3c08fdf",
            Self::Hydralion => "7e2e026eb3354c74685427b26cf9acb8",
            Self::Toadcano => "816de006c125b9b4eaa7139bac5c6b77",
            Self::Guardian => "78457137461e7d345b2287aab380e2e0",
            Self::Meduso => "bb02eb1602e1ec142b85cd6b505ef5b6",
            Self::Casugin => "bdff582229a41f3438d4c4faac714255",
            Self::Abstarak => "f4032b2323bc31d4590cf5197db3c3f1",
            Self::Rachater => "c4480713abcb0d04f8a21a702987e6e1",
            Self::Repeater => "19255ab0a339bd44a8873944c866afc9",
            Self::Catalyst => "b2e5237a9dd152643abaf1fb3e3d7206",
            Self::Tentacle => "c109e23c16e478b4e992161662fa81c0",
            Self::DwellerOfDread => "a1c7a4d91b5c8c54b96c3a159ad3a1b5",
            Self::LeJugg => "7a8be6ca5e9b7bd49ac7d2da414442cc",
            Self::PhaseReaper => "54abc79fbf9dd2f4a8bd19cab8245391",
            Self::Elysandarelle1 => "0c24f27ebab6b854ba75700be2df5b21",
            Self::Elysandarelle2 => "0e5b91e5ad0b2784da76ba6314004370",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Level {
    Skyland,
    ArchivistRoom,
    BambooCreek,
    CeruleanExpanse,
    CoralCascade,
    CursedWood,
    Docks,
    ElderMistTrials,
    EstristaesLookout,
    FloodedGraveyard,
    ForbiddenCavern,
    GlacialPeak,
    HomeWorld,
    LostOnesHamlet,
    Lucent,
    MesaHike,
    Mirth,
    Mooncradle,
    Moorland,
    MountainTrail,
    Peninsula,
    BriskRebuilt,
    BriskDestroyed,
    BriskOriginal,
    Repine,
    SacrosanctSpires,
    SeraisWorld,
    SkyGiantsVillage,
    SkywardShrine,
    SongShroomMarsh,
    StormCallerIsland,
    TitleScreen,
    Vespertine,
    WaterTemple,
    WizardLab,
    WorldEeater,
}

impl Level {
    fn resolve(mut id: impl Iterator<Item = char>) -> Option<Self> {
        let mut depth = 0;
        let mut digits = id.by_ref().inspect(|_| depth += 1);

        macro_rules! eq {
            ($needle:literal => $res:expr) => {
                (id.eq(($needle[depth..]).chars())).then_some($res)
            };
        }

        macro_rules! next {
            ($($arm:literal => $branch:expr),+ $(,)?) => {
                match digits.next()? {
                    $($arm => $branch),+,
                    _ => None,
                }
            };
        }

        next! {
            '0' => eq!("02b4d6511eeaf81428fc06320bb08cb8" => Self::WizardLab),
            '1' => eq!("11810c4630980eb43abf7fecebfd5a6b" => Self::ElderMistTrials),
            '2' => eq!("266a901e65780e94fba5cd7c25b58957" => Self::Skyland),
            '3' => next! {
                '0' => eq!("304315e8f18ddf149a746e9ecb9db201" => Self::Mooncradle),
                '1' => eq!("3148529996942724aac85141f9d5a42d" => Self::LostOnesHamlet),
                '6' => eq!("36d8c0b7f6372704b88a40a23c0f44f9" => Self::BriskRebuilt),
                '9' => eq!("3914dcaa548d2f3488777a5b5339a5c8" => Self::EstristaesLookout),
                'a' => eq!("3aea0c635edd6d144b8c0deac0bc62d3" => Self::Repine),
                'c' => eq!("3cd46afe466424b41a5fa858f91aab0d" => Self::MesaHike),
                'd' => next! {
                    '1' => eq!("3d1c3e6c6c2511743ac0278f551d299c" => Self::StormCallerIsland),
                    'a' => eq!("3dab1b3e3a5221c40989f1c68cfcd352" => Self::WorldEeater),
                },
            },
            '4' => next! {
                '4' => eq!("44a416e48c8d7d345b5e4507eb27e4de" => Self::TitleScreen),
                '7' => eq!("4776b2f6ccdb0fe4195c6c0d89206875" => Self::HomeWorld),
                'd' => eq!("4d9b70c53db5b8c49bb5c60c6ef858bd" => Self::ForbiddenCavern),
            },
            '6' => next! {
                '0' => eq!("6089fb6bc29dbfe4a8ef1be0245a27ee" => Self::WaterTemple),
                '2' => eq!("62d9b9e11ce314a4da0c04eb812e696d" => Self::Lucent),
                '6' => eq!("66299b28257ea224ca45113c4ff6f45d" => Self::BambooCreek),
            },
            '7' => next! {
                '2' => eq!("72e9f2699f7c8394b93afa1d273ce67a" => Self::MountainTrail),
                '3' => eq!("737979d8a1b9e6c4a82d7eb776953244" => Self::Docks),
                '4' => eq!("745f076f7188dfa4d93d4ffed10232ca" => Self::BriskDestroyed),
                '5' => eq!("75a16b768d23caf4987bfe1515b04c57" => Self::CursedWood),
                '6' => eq!("763e6cf37dffb6b46a2d842bf01c24fe" => Self::BriskOriginal),
                '7' => eq!("77a7111e97c4dab449722b724cdc8d3f" => Self::Moorland),
                'f' => eq!("7f36e70224f47d344a794e3648fe630b" => Self::SongShroomMarsh),
            },
            '8' => eq!("87f3c0b8e8e6cb34daf39ec5cfdeae70" => Self::SeraisWorld),
            '9' => eq!("9ed0e7229b30c6c458f6b8bf1d210e68" => Self::SkyGiantsVillage),
            'a' => eq!("adc3d53fe3e2f114086b8c0b4db69ded" => Self::SacrosanctSpires),
            'b' => next! {
                '3' => eq!("b3d251f726c4a9444b1051ea8509d8e2" => Self::Peninsula),
                'f' => eq!("bfe9060167f8f0b42ac1c56a554f16a5" => Self::ArchivistRoom),
            },
            'c' => eq!("cdda6d8e9433a2e43b5f78d1732db12e" => Self::GlacialPeak),
            'd' => eq!("dab5e0be1025fa7449bd4b5141b58dad" => Self::CoralCascade),
            'f' => next! {
                '1' => next! {
                    'a' => eq!("f1a3d633f8079654398f8266fc9feffb" => Self::Vespertine),
                    'f' => eq!("f1f754c32cb8d5c489e1124505587759" => Self::CeruleanExpanse),
                },
                '2' => eq!("f25152a99bdd7af4c8e454c8e2089d72" => Self::SkywardShrine),
                '6' => eq!("f66543e45ee80264085b007f8f59d56a" => Self::FloodedGraveyard),
                'e' => eq!("fe2cfebc0cf6bc540892964ac8db2274" => Self::Mirth),
            },
        }
    }

    #[allow(unused)]
    fn id(self) -> &'static str {
        match self {
            Self::Skyland => "266a901e65780e94fba5cd7c25b58957",
            Self::ArchivistRoom => "bfe9060167f8f0b42ac1c56a554f16a5",
            Self::BambooCreek => "66299b28257ea224ca45113c4ff6f45d",
            Self::CeruleanExpanse => "f1f754c32cb8d5c489e1124505587759",
            Self::CoralCascade => "dab5e0be1025fa7449bd4b5141b58dad",
            Self::CursedWood => "75a16b768d23caf4987bfe1515b04c57",
            Self::Docks => "737979d8a1b9e6c4a82d7eb776953244",
            Self::ElderMistTrials => "11810c4630980eb43abf7fecebfd5a6b",
            Self::EstristaesLookout => "3914dcaa548d2f3488777a5b5339a5c8",
            Self::FloodedGraveyard => "f66543e45ee80264085b007f8f59d56a",
            Self::ForbiddenCavern => "4d9b70c53db5b8c49bb5c60c6ef858bd",
            Self::GlacialPeak => "cdda6d8e9433a2e43b5f78d1732db12e",
            Self::HomeWorld => "4776b2f6ccdb0fe4195c6c0d89206875",
            Self::LostOnesHamlet => "3148529996942724aac85141f9d5a42d",
            Self::Lucent => "62d9b9e11ce314a4da0c04eb812e696d",
            Self::MesaHike => "3cd46afe466424b41a5fa858f91aab0d",
            Self::Mirth => "fe2cfebc0cf6bc540892964ac8db2274",
            Self::Mooncradle => "304315e8f18ddf149a746e9ecb9db201",
            Self::Moorland => "77a7111e97c4dab449722b724cdc8d3f",
            Self::MountainTrail => "72e9f2699f7c8394b93afa1d273ce67a",
            Self::Peninsula => "b3d251f726c4a9444b1051ea8509d8e2",
            Self::BriskRebuilt => "36d8c0b7f6372704b88a40a23c0f44f9",
            Self::BriskDestroyed => "745f076f7188dfa4d93d4ffed10232ca",
            Self::BriskOriginal => "763e6cf37dffb6b46a2d842bf01c24fe",
            Self::Repine => "3aea0c635edd6d144b8c0deac0bc62d3",
            Self::SacrosanctSpires => "adc3d53fe3e2f114086b8c0b4db69ded",
            Self::SeraisWorld => "87f3c0b8e8e6cb34daf39ec5cfdeae70",
            Self::SkyGiantsVillage => "9ed0e7229b30c6c458f6b8bf1d210e68",
            Self::SkywardShrine => "f25152a99bdd7af4c8e454c8e2089d72",
            Self::SongShroomMarsh => "7f36e70224f47d344a794e3648fe630b",
            Self::StormCallerIsland => "3d1c3e6c6c2511743ac0278f551d299c",
            Self::TitleScreen => "44a416e48c8d7d345b5e4507eb27e4de",
            Self::Vespertine => "f1a3d633f8079654398f8266fc9feffb",
            Self::WaterTemple => "6089fb6bc29dbfe4a8ef1be0245a27ee",
            Self::WizardLab => "02b4d6511eeaf81428fc06320bb08cb8",
            Self::WorldEeater => "3dab1b3e3a5221c40989f1c68cfcd352",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum KeyItem {
    Graplou,
    MasterGhostSandwich,
    Map,
    Seashell,
}

impl KeyItem {
    fn resolve(mut id: impl Iterator<Item = char>) -> Option<Self> {
        let mut depth = 0;
        let mut digits = id.by_ref().inspect(|_| depth += 1);

        macro_rules! eq {
            ($needle:literal => $res:expr) => {
                (id.eq(($needle[depth..]).chars())).then_some($res)
            };
        }

        macro_rules! next {
            ($($arm:literal => $branch:expr),+ $(,)?) => {
                match digits.next()? {
                    $($arm => $branch),+,
                    _ => None,
                }
            };
        }

        next! {
            '2' => eq!("2295d1bfeec0f8844b477f95c919c74f" => Self::Seashell),
            'c' => eq!("c9447122a421a2640b315d36b2562ad2" => Self::Graplou),
            'e' => eq!("e94e5414de65af34a810b8f89c117b6b" => Self::MasterGhostSandwich),
            'a' => eq!("aefb6b3d640e4804d85814203c8baa2c " => Self::Map),
        }
    }

    #[allow(unused)]
    fn id(self) -> &'static str {
        match self {
            Self::Graplou => "c9447122a421a2640b315d36b2562ad2",
            Self::MasterGhostSandwich => "e94e5414de65af34a810b8f89c17b6b",
            Self::Map => "aefb6b3d640e4804d85814203c8baa2c",
            Self::Seashell => "2295d1bfeec0f8844b477f95c919c74f",
        }
    }
}

pub struct Data<'a> {
    game: Game<'a>,
    title_screen: TitleScreen,
    combat: Combat,
    progression: Progression,
    inventory: Inventory,
}

impl Data<'_> {
    pub fn game_start(&mut self) -> Option<GameStart> {
        self.title_screen.game_start(&self.game)
    }

    pub fn current_progression(&mut self) -> Option<CurrentProgression> {
        self.progression.current_progression(&self.game)
    }

    pub fn encounter(&mut self) -> Option<Encounter> {
        self.combat.current_encounter(&self.game)
    }

    pub fn current_enemies(&mut self) -> CurrentEncounter {
        self.combat.current_enemy_encounter(&self.game)
    }

    pub fn key_item_changes(&mut self) -> impl Iterator<Item = Change<KeyItem>> + '_ {
        self.inventory.check_key_items(&self.game)
    }
}

impl<'a> Data<'a> {
    pub async fn new(process: &'a Process) -> Data<'a> {
        let module = Module::wait_attach(process, Version::V2020).await;
        let image = module.wait_get_default_image(process).await;
        log!("Attached to the game");
        let game = Game::new(process, module, image);

        Self {
            game,
            title_screen: TitleScreen::new(),
            combat: Combat::new(),
            progression: Progression::new(),
            inventory: Inventory::new(),
        }
    }
}

mod title_screen {
    use asr::{
        game_engine::unity::il2cpp::{Class2, Game},
        Pointer,
    };

    #[derive(Debug, PartialEq, Eq)]
    pub enum GameStart {
        NotStarted,
        JustStarted,
    }

    pub struct TitleScreen {
        manager: TitleSequenceManagerBinding,
        char_select: CharacterSelectionScreenBinding,
    }

    #[derive(Class2, Debug)]
    struct TitleSequenceManager {
        #[singleton]
        #[rename = "instance"]
        _instance: Pointer<Self>,
        #[rename = "characterSelectionScreen"]
        char_selection_screen: Pointer<CharacterSelectionScreen>,
    }

    #[derive(Class2, Debug)]
    struct CharacterSelectionScreen {
        #[rename = "characterSelected"]
        char_selected: bool,
    }

    impl TitleScreen {
        pub fn new() -> Self {
            Self {
                manager: TitleSequenceManager::bind(),
                char_select: CharacterSelectionScreen::bind(),
            }
        }

        pub fn game_start(&mut self, game: &Game<'_>) -> Option<GameStart> {
            let manager = self.manager.read(game)?;
            let char_select = self
                .char_select
                .read_pointer(game, manager.char_selection_screen)?;
            Some(if char_select.char_selected {
                GameStart::JustStarted
            } else {
                GameStart::NotStarted
            })
        }
    }
}

mod combat {
    use asr::{
        arrayvec::ArrayVec,
        game_engine::unity::il2cpp::{Class2, Game},
        Pointer,
    };
    use csharp_mem::{CSString, List};

    pub struct Combat {
        manager: CombatManagerBinding,
        encounter: EncounterBinding,
        enemy: EnemyCombat,
    }

    impl Combat {
        pub fn new() -> Self {
            Self {
                manager: CombatManager::bind(),
                encounter: Encounter::bind(),
                enemy: EnemyCombat::new(),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum CurrentEncounter {
        NotInEncounter,
        InEncounter(ArrayVec<super::Enemy, 6>),
    }

    struct EnemyCombat {
        actor: EnemyCombatActorBinding,
        char_data: EnemyCharacterDataBinding,
    }

    impl EnemyCombat {
        fn new() -> Self {
            Self {
                actor: EnemyCombatActor::bind(),
                char_data: EnemyCharacterData::bind(),
            }
        }
    }

    #[derive(Class2)]
    struct CombatManager {
        #[singleton]
        #[rename = "instance"]
        _instance: Pointer<Self>,
        #[rename = "currentEncounter"]
        encounter: Pointer<Encounter>,
    }

    #[derive(Class2, Debug)]
    pub struct Encounter {
        #[rename = "encounterDone"]
        pub done: bool,
        #[rename = "bossEncounter"]
        pub boss: bool,
        #[rename = "enemyActors"]
        enemy_actors: Pointer<List<Pointer<EnemyCombatActor>>>,
    }

    #[derive(Class2, Debug)]
    struct EnemyCombatActor {
        #[rename = "enemyData"]
        data: Pointer<EnemyCharacterData>,
    }

    #[derive(Class2, Debug)]
    struct EnemyCharacterData {
        guid: Pointer<CSString>,
    }

    impl Combat {
        pub fn current_enemy_encounter(&mut self, game: &Game<'_>) -> CurrentEncounter {
            fn current_enemies(
                this: &mut Combat,
                game: &Game<'_>,
            ) -> Option<ArrayVec<super::Enemy, 6>> {
                let combat = this.manager.read(game)?;
                let encounter = this.encounter.read_pointer(game, combat.encounter)?;
                let actors = encounter.enemy_actors.resolve(game);

                let enemies = match actors {
                    Some(actors) => actors
                        .iter(game)
                        .filter_map(|o| this.enemy.enemy(game, o))
                        .map(|e| match e {
                            super::Enemy::DwellerOfStrife1 if !encounter.boss => {
                                super::Enemy::DwellerOfStrife2
                            }
                            e => e,
                        })
                        .take(6)
                        .collect(),
                    None => ArrayVec::new(),
                };

                Some(enemies)
            }

            match current_enemies(self, game) {
                Some(enemies) => CurrentEncounter::InEncounter(enemies),
                None => CurrentEncounter::NotInEncounter,
            }
        }

        pub fn current_encounter(&mut self, game: &Game<'_>) -> Option<Encounter> {
            let manager = self.manager.read(game)?;
            let encounter = self.encounter.read_pointer(game, manager.encounter)?;
            Some(encounter)
        }
    }

    impl EnemyCombat {
        fn enemy(
            &mut self,
            game: &Game<'_>,
            eca: Pointer<EnemyCombatActor>,
        ) -> Option<super::Enemy> {
            let actor = self.actor.read_pointer(game, eca)?;
            let char_data = self.char_data.read_pointer(game, actor.data)?;
            let e_guid = char_data.guid.resolve(game)?;
            let enemy = super::Enemy::resolve(e_guid.chars(game));

            enemy
        }
    }
}

mod progress {
    use asr::{
        game_engine::unity::il2cpp::{Class2, Game},
        MemReader, Pointer,
    };
    use bytemuck::AnyBitPattern;
    use csharp_mem::CSString;

    #[derive(Class2, Debug)]
    struct LevelManager {
        #[singleton]
        #[rename = "instance"]
        _instance: Pointer<Self>,

        #[rename = "loadingLevel"]
        is_loading: bool,

        #[rename = "currentLevel"]
        current_level: LevelReference,
    }

    #[derive(Copy, Clone, Debug, AnyBitPattern)]
    struct LevelReference {
        guid: Pointer<CSString>,
    }

    #[derive(Class2, Debug)]
    struct CutsceneManager {
        #[singleton]
        #[rename = "instance"]
        _instance: Pointer<Self>,
    }

    pub struct Progression {
        level_manager: LevelManagerBinding,
        cutscene_manager: CutsceneManagerBinding,
    }

    impl Progression {
        pub fn new() -> Self {
            Self {
                level_manager: LevelManager::bind(),
                cutscene_manager: CutsceneManager::bind(),
            }
        }

        pub fn current_progression(&mut self, game: &Game<'_>) -> Option<CurrentProgression> {
            let level = self.level_manager.read(game)?;
            let is_loading = level.is_loading;
            let is_in_cutscene = self.is_in_cutscene(game);
            let level = level
                .current_level
                .guid
                .resolve(game)
                .and_then(|o| super::Level::resolve(o.chars(game)));
            Some(CurrentProgression {
                is_loading,
                is_in_cutscene,
                level,
            })
        }

        pub fn is_in_cutscene(&mut self, game: &Game<'_>) -> bool {
            fn inner(this: &mut Progression, game: &Game<'_>) -> Option<bool> {
                let cutscene_manager = this.cutscene_manager.read(game)?;
                let is_in_cutscene = game.read(cutscene_manager._instance.address() + 0x30)?;
                Some(is_in_cutscene)
            }

            inner(self, game).unwrap_or(false)
        }
    }

    #[derive(Clone, PartialEq)]
    pub struct CurrentProgression {
        pub is_loading: bool,
        pub is_in_cutscene: bool,
        pub level: Option<super::Level>,
    }
}

mod inventory {
    use std::collections::hash_map::Entry;

    use ahash::{HashMap, HashMapExt};
    use asr::{
        game_engine::unity::il2cpp::{Class2, Game},
        watcher::Watcher,
        Pointer,
    };
    use bytemuck::AnyBitPattern;

    use csharp_mem::{CSString, Map};

    #[derive(Class2, Debug)]
    struct InventoryManager {
        #[singleton]
        #[rename = "instance"]
        _instance: Pointer<Self>,

        #[rename = "ownedInventoryItems"]
        owned_items: Pointer<QuantityByInventoryItemReference>,
    }

    #[derive(Debug, Copy, Clone, AnyBitPattern)]
    struct InventoryItemReference {
        guid: Pointer<CSString>,
    }

    #[derive(Class2, Debug)]
    struct QuantityByInventoryItemReference {
        dictionary: Pointer<Map<InventoryItemReference, u32>>,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Change<T> {
        PickedUp(T),
        Lost(T),
    }

    pub struct Inventory {
        quantity: QuantityByInventoryItemReferenceBinding,
        manager: InventoryManagerBinding,
        number_of_owned_items: Watcher<u32>,
        owned_key_items: HashMap<super::KeyItem, (u32, u32)>,
        generation: u32,
    }

    impl Inventory {
        pub fn new() -> Self {
            Self {
                quantity: QuantityByInventoryItemReference::bind(),
                manager: InventoryManager::bind(),
                number_of_owned_items: Watcher::new(),
                owned_key_items: HashMap::new(),
                generation: 0,
            }
        }

        pub fn check_key_items<'a>(
            &'a mut self,
            game: &'a Game<'_>,
        ) -> impl Iterator<Item = Change<super::KeyItem>> + 'a {
            fn inner<'a>(
                this: &'a mut Inventory,
                game: &'a Game<'_>,
            ) -> Option<impl Iterator<Item = Change<super::KeyItem>> + 'a> {
                let manager = this.manager.read(game)?;
                let owned = this.quantity.read_pointer(game, manager.owned_items)?;
                let owned = owned.dictionary.resolve(game)?;

                let first = this.number_of_owned_items.pair.is_none();
                let now_owned = this.number_of_owned_items.update_infallible(owned.size());

                if !first && !now_owned.changed() {
                    return None;
                }

                this.generation += 1;
                let generation = this.generation;

                for item in owned.iter(game).filter_map(move |(item, _amount)| {
                    let item = item.guid.resolve(game)?;
                    super::KeyItem::resolve(item.chars(game))
                }) {
                    match this.owned_key_items.entry(item) {
                        Entry::Occupied(existing) => {
                            existing.into_mut().1 = generation;
                        }
                        Entry::Vacant(missing) => {
                            missing.insert((generation, generation));
                        }
                    };
                }

                let items =
                    this.owned_key_items
                        .iter()
                        .filter_map(move |(item, &(insert, current))| {
                            if current == generation {
                                if insert == current {
                                    Some(Change::PickedUp(*item))
                                } else {
                                    None
                                }
                            } else {
                                Some(Change::Lost(*item))
                            }
                        });

                Some(items)
            }

            inner(self, game).into_iter().flatten()
        }
    }
}
