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
#[cfg(debugger)]
pub use self::{
    combat::{Enemy as EnemyData, EnemyEncounter, EnemyMods, EnemyStats, General},
    inventory::NamedKeyItem,
    loc::{Loc, Localization},
    progress::{Activity, Level as LevelData, PlayTime},
};

pub use self::{
    combat::CurrentEncounter, inventory::Change, progress::CurrentProgress, title_screen::GameStart,
};

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
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
    #[cfg(debugger)]
    loc: Localization<'a>,
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

    #[cfg(debugger)]
    pub fn progress(&mut self) -> Option<CurrentProgress> {
        let loc = Self::loc(&self.game, &mut self.loc)?;
        self.progression.get_progress(&self.game, loc)
    }

    #[cfg(debugger)]
    pub fn deep_resolve_encounter(&mut self) -> Option<combat::BattleEncounter> {
        let loc = Self::loc(&self.game, &mut self.loc)?;
        self.combat.resolve(&self.game, loc)
    }

    #[cfg(debugger)]
    pub fn check_for_changed_key_items(
        &mut self,
    ) -> impl Iterator<Item = Change<&'_ NamedKeyItem>> + '_ {
        fn inner<'a>(
            this: &'a mut Data<'_>,
        ) -> Option<impl Iterator<Item = Change<&'a NamedKeyItem>> + 'a> {
            let loc = Data::loc(&this.game, &mut this.loc)?;
            Some(this.inventory.check_new(&this.game, loc))
        }
        inner(self).into_iter().flatten()
    }

    #[cfg(debugger)]
    fn loc<'a, 'p>(game: &Game<'p>, loc: &'a mut Localization<'p>) -> Option<&'a Loc> {
        loc.resolve(game.process(), game.module())
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
            #[cfg(debugger)]
            loc: Localization::new(),
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
    #[cfg(debugger)]
    use core::fmt;

    #[cfg(debugger)]
    use asr::{arrayvec::ArrayString, Address64};
    use asr::{
        arrayvec::ArrayVec,
        game_engine::unity::il2cpp::{Class2, Game},
        Pointer,
    };
    #[cfg(debugger)]
    use bytemuck::AnyBitPattern;
    #[cfg(debugger)]
    use csharp_mem::Map;
    use csharp_mem::{CSString, List};

    pub struct Combat {
        manager: CombatManagerBinding,
        encounter: EncounterBinding,
        enemy: EnemyCombat,

        #[cfg(debugger)]
        loot: EncounterLootBinding,
        #[cfg(debugger)]
        e_target: EnemyCombatTargetBinding,
    }

    #[cfg(debugger)]
    impl fmt::Debug for Combat {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Combat").finish_non_exhaustive()
        }
    }

    impl Combat {
        #[cfg(not(debugger))]
        pub fn new() -> Self {
            Self {
                manager: CombatManager::bind(),
                encounter: Encounter::bind(),
                enemy: EnemyCombat::new(),
            }
        }

        #[cfg(debugger)]
        pub fn new() -> Self {
            Self {
                manager: CombatManager::bind(),
                encounter: Encounter::bind(),
                enemy: EnemyCombat::new(),
                loot: EncounterLoot::bind(),
                e_target: EnemyCombatTarget::bind(),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum CurrentEncounter {
        NotInEncounter,
        InEncounter(ArrayVec<super::Enemy, 6>),
    }

    #[cfg(not(debugger))]
    struct EnemyCombat {
        actor: EnemyCombatActorBinding,
        char_data: EnemyCharacterDataBinding,
    }

    #[cfg(not(debugger))]
    impl EnemyCombat {
        fn new() -> Self {
            Self {
                actor: EnemyCombatActor::bind(),
                char_data: EnemyCharacterData::bind(),
            }
        }
    }

    #[cfg(debugger)]
    struct EnemyCombat {
        actor: EnemyCombatActorBinding,
        char_data: EnemyCharacterDataBinding,
        by_damage: FloatByEDamageTypeBinding,
        xp: XPDataBinding,
    }

    #[cfg(debugger)]
    impl EnemyCombat {
        fn new() -> Self {
            Self {
                actor: EnemyCombatActor::bind(),
                char_data: EnemyCharacterData::bind(),
                by_damage: FloatByEDamageType::bind(),
                xp: XPData::bind(),
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

    #[cfg(not(debugger))]
    #[derive(Class2, Debug)]
    pub struct Encounter {
        #[rename = "encounterDone"]
        pub done: bool,
        #[rename = "bossEncounter"]
        pub boss: bool,
        #[rename = "enemyActors"]
        enemy_actors: Pointer<List<Pointer<EnemyCombatActor>>>,
    }

    #[cfg(debugger)]
    #[derive(Class2, Debug)]
    pub struct Encounter {
        #[rename = "encounterDone"]
        pub done: bool,
        #[rename = "bossEncounter"]
        pub boss: bool,

        #[rename = "fullHealPartyAfterEncounter"]
        full_heal_after: bool,
        #[rename = "isUnderwater"]
        underwater: bool,
        #[rename = "allEnemiesKilled"]
        all_enemies_killed: bool,
        #[rename = "isRunning"]
        running: bool,

        #[rename = "xpGained"]
        xp_gain: u32,

        #[rename = "encounterDoneAchievement"]
        has_achievement: Address64,

        #[rename = "encounterLoot"]
        loot: Pointer<EncounterLoot>,

        #[rename = "enemyActors"]
        enemy_actors: Pointer<List<Pointer<EnemyCombatActor>>>,

        #[rename = "enemyTargets"]
        enemy_targets: Pointer<List<Pointer<EnemyCombatTarget>>>,
    }

    #[cfg(debugger)]
    #[derive(Class2, Debug)]
    struct EncounterLoot {
        #[rename = "goldToAward"]
        gold: u32,
    }

    #[cfg(not(debugger))]
    #[derive(Class2, Debug)]
    struct EnemyCombatActor {
        #[rename = "enemyData"]
        data: Pointer<EnemyCharacterData>,
    }

    #[cfg(debugger)]
    #[derive(Class2, Debug)]
    struct EnemyCombatActor {
        #[rename = "hideHP"]
        hide_hp: bool,
        #[rename = "awardXP"]
        xp: bool,
        #[rename = "enemyData"]
        data: Pointer<EnemyCharacterData>,
    }

    #[cfg(not(debugger))]
    #[derive(Class2, Debug)]
    struct EnemyCharacterData {
        guid: Pointer<CSString>,
    }

    #[cfg(debugger)]
    #[derive(Class2, Debug)]
    struct EnemyCharacterData {
        guid: Pointer<CSString>,

        hp: u32,
        speed: u32,
        #[rename = "basePhysicalDefense"]
        base_physical_defense: u32,
        #[rename = "basePhysicalAttack"]
        base_physical_attack: u32,
        #[rename = "baseMagicAttack"]
        base_magic_attack: u32,
        #[rename = "baseMagicDefense"]
        base_magic_defense: u32,
        #[rename = "damageTypeModifiers"]
        damage_type_modifiers: Pointer<FloatByEDamageType>,
        #[rename = "damageTypeModifiersOverride"]
        damage_type_override: Pointer<FloatByEDamageType>,

        #[rename = "enemyLevel"]
        level: u32,
        #[rename = "xpData"]
        xp: Pointer<XPData>,

        #[rename = "nameLocalizationId"]
        name_localization_id: super::loc::LocalizationId,
    }

    #[cfg(debugger)]
    #[derive(Class2, Debug)]
    struct FloatByEDamageType {
        dictionary: Pointer<Map<EDamageType, f32>>,
    }

    #[cfg(debugger)]
    #[derive(Class2, Debug)]
    struct XPData {
        #[rename = "goldReward"]
        gold: u32,
    }

    #[cfg(debugger)]
    #[derive(Class2, Debug)]
    struct EnemyCombatTarget {
        #[rename = "currentHP"]
        current_hp: u32,
    }

    #[cfg(debugger)]
    #[derive(Copy, Clone, Debug, AnyBitPattern)]
    #[repr(C)]
    struct EDamageType {
        value: u32,
    }

    #[cfg(debugger)]
    #[allow(unused)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(u32)]
    enum DamageType {
        None = 0x0000,
        Any = 0x0001,
        Sword = 0x0002,
        Sun = 0x0004,
        Moon = 0x0008,
        Eclipse = 0x0010,
        Poison = 0x0020,
        Arcane = 0x0040,
        Stun = 0x0080,
        Magical = 0x00fc, // Stun | Arcane | Poison | Eclipse | Sun | Moon
        Blunt = 0x0100,
    }

    #[cfg(debugger)]
    impl core::ops::BitAnd for DamageType {
        type Output = bool;

        fn bitand(self, rhs: Self) -> Self::Output {
            (self as u32) & (rhs as u32) != 0
        }
    }

    #[cfg(debugger)]
    impl From<EDamageType> for DamageType {
        fn from(value: EDamageType) -> Self {
            unsafe { core::mem::transmute(value.value) }
        }
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

        #[cfg(debugger)]
        pub fn resolve(
            &mut self,
            game: &Game<'_>,
            loc: &super::loc::Loc,
        ) -> Option<BattleEncounter> {
            let combat = self.manager.read(game)?;

            let encounter = self.encounter.read_pointer(game, combat.encounter)?;
            let loot = self.loot.read_pointer(game, encounter.loot)?;

            let actors = encounter.enemy_actors.resolve(game);

            let mut enemies = match actors {
                Some(actors) => actors
                    .iter(game)
                    .filter_map(|o| self.enemy.resolve(loc, game, o))
                    .take(6)
                    .collect(),
                None => ArrayVec::new(),
            };

            let targets = encounter.enemy_targets.resolve(game);
            for (target, enemy) in targets
                .into_iter()
                .flat_map(|o| o.iter(game))
                .zip(enemies.iter_mut())
            {
                if let Some(target) = self.e_target.read_pointer(game, target) {
                    enemy.stats.current_hp = target.current_hp;
                }
            }

            Some(BattleEncounter {
                done: encounter.done,
                boss: encounter.boss,
                full_heal_after: encounter.full_heal_after,
                underwater: encounter.underwater,
                all_enemies_killed: encounter.all_enemies_killed,
                running: encounter.running,
                xp_gain: encounter.xp_gain,
                has_achievement: !encounter.has_achievement.is_null(),
                loot,
                enemies,
            })
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

        #[cfg(debugger)]
        fn resolve(
            &mut self,
            loc: &super::loc::Loc,
            game: &Game<'_>,
            eca: Pointer<EnemyCombatActor>,
        ) -> Option<EnemyInfo> {
            let actor = self.actor.read_pointer(game, eca)?;
            let char_data = self.char_data.read_pointer(game, actor.data)?;

            let stats = EnemyStats {
                current_hp: 0,
                max_hp: char_data.hp,
                level: char_data.level,
                speed: char_data.speed,
                attack: char_data.base_physical_attack,
                defense: char_data.base_physical_defense,
                magic_attack: char_data.base_magic_attack,
                magic_defense: char_data.base_magic_defense,
            };

            let mut mods = EnemyMods {
                any: 1.0,
                sword: 1.0,
                sun: 1.0,
                moon: 1.0,
                eclipse: 1.0,
                poison: 1.0,
                arcane: 1.0,
                stun: 1.0,
                blunt: 1.0,
            };

            let damage_type_modifiers = self
                .by_damage
                .read_pointer(game, char_data.damage_type_modifiers);

            let damage_type_modifiers = damage_type_modifiers
                .and_then(|o| o.dictionary.resolve(game))
                .into_iter()
                .flat_map(|o| o.iter(game).map(|(k, v)| (DamageType::from(k), v)));

            let damage_type_override = self
                .by_damage
                .read_pointer(game, char_data.damage_type_override);
            let damage_type_override = damage_type_override
                .and_then(|o| o.dictionary.resolve(game))
                .into_iter()
                .flat_map(|o| o.iter(game).map(|(k, v)| (DamageType::from(k), v)));

            for (dmg, modifier) in damage_type_modifiers.chain(damage_type_override) {
                if dmg & DamageType::Any {
                    mods.any = modifier;
                }
                if dmg & DamageType::Sword {
                    mods.sword = modifier;
                }
                if dmg & DamageType::Sun {
                    mods.sun = modifier;
                }
                if dmg & DamageType::Moon {
                    mods.moon = modifier;
                }
                if dmg & DamageType::Eclipse {
                    mods.eclipse = modifier;
                }
                if dmg & DamageType::Poison {
                    mods.poison = modifier;
                }
                if dmg & DamageType::Arcane {
                    mods.arcane = modifier;
                }
                if dmg & DamageType::Stun {
                    mods.stun = modifier;
                }
                if dmg & DamageType::Blunt {
                    mods.blunt = modifier;
                }
            }

            let gold = self
                .xp
                .read_pointer(game, char_data.xp)
                .map_or(0, |o| o.gold);

            let e_guid = char_data.guid.resolve(game)?;

            let enemy = super::Enemy::resolve(e_guid.chars(game));

            let id = e_guid.to_string(game);

            let name = char_data.name_localization_id;
            let name = loc.localized(game, name);

            Some(EnemyInfo {
                hide_hp: actor.hide_hp,
                gives_xp: actor.xp,
                gold,
                id,
                name,
                enemy,
                stats,
                mods,
            })
        }
    }

    #[cfg(debugger)]
    #[derive(Debug)]
    pub struct BattleEncounter {
        pub done: bool,
        pub boss: bool,
        full_heal_after: bool,
        underwater: bool,
        all_enemies_killed: bool,
        running: bool,
        xp_gain: u32,
        has_achievement: bool,
        loot: EncounterLoot,
        enemies: ArrayVec<EnemyInfo, 6>,
    }

    #[cfg(debugger)]
    impl BattleEncounter {
        pub fn enemies(&self) -> impl Iterator<Item = EnemyEncounter> + '_ {
            Some(EnemyEncounter::General(General {
                boss: self.boss,
                has_achievement: self.has_achievement,
                done: self.done,
                is_running: self.running,
                all_enemies_killed: self.all_enemies_killed,
                full_heal_after: self.full_heal_after,
                underwater: self.underwater,
                xp_gained: self.xp_gain,
                gold_gained: self.loot.gold,
            }))
            .into_iter()
            .chain(self.enemies.iter().flat_map(move |o| {
                [
                    EnemyEncounter::Enemy(Enemy {
                        id: o.id.as_str(),
                        name: &o.name,
                        enemy: o.enemy,
                        hide_hp: o.hide_hp,
                        award_xp: o.gives_xp,
                        gold_drop: o.gold,
                    }),
                    EnemyEncounter::EnemyStats(o.stats),
                    EnemyEncounter::EnemyMods(o.mods),
                ]
            }))
        }
    }

    #[cfg(debugger)]
    #[derive(Debug)]
    struct EnemyInfo {
        hide_hp: bool,
        gives_xp: bool,
        gold: u32,
        id: ArrayString<36>,
        name: String,
        enemy: Option<super::Enemy>,
        stats: EnemyStats,
        mods: EnemyMods,
    }

    #[cfg(debugger)]
    #[derive(Copy, Clone, Debug)]
    pub struct General {
        pub boss: bool,
        pub has_achievement: bool,
        pub done: bool,
        pub is_running: bool,
        pub all_enemies_killed: bool,

        pub full_heal_after: bool,
        pub underwater: bool,

        pub xp_gained: u32,
        pub gold_gained: u32,
    }

    #[cfg(debugger)]
    #[derive(Copy, Clone, Debug)]
    pub struct Enemy<'a> {
        pub id: &'a str,
        pub name: &'a str,
        pub enemy: Option<super::Enemy>,
        pub award_xp: bool,
        pub gold_drop: u32,
        pub hide_hp: bool,
    }

    #[cfg(debugger)]
    #[derive(Copy, Clone, Debug)]
    pub struct EnemyStats {
        pub current_hp: u32,
        pub max_hp: u32,
        pub level: u32,
        pub speed: u32,
        pub attack: u32,
        pub defense: u32,
        pub magic_attack: u32,
        pub magic_defense: u32,
    }

    #[cfg(debugger)]
    #[derive(Copy, Clone, Debug)]
    pub struct EnemyMods {
        pub any: f32,
        pub sword: f32,
        pub blunt: f32,
        pub sun: f32,
        pub moon: f32,
        pub poison: f32,
        pub arcane: f32,
        pub eclipse: f32,
        pub stun: f32,
    }

    #[cfg(debugger)]
    pub enum EnemyEncounter<'a> {
        General(General),
        Enemy(Enemy<'a>),
        EnemyStats(EnemyStats),
        EnemyMods(EnemyMods),
    }

    #[cfg(debugger)]
    impl fmt::Debug for EnemyEncounter<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if f.alternate() {
                match self {
                    EnemyEncounter::General(e) => e.fmt(f),
                    EnemyEncounter::Enemy(e) => e.fmt(f),
                    EnemyEncounter::EnemyStats(e) => e.fmt(f),
                    EnemyEncounter::EnemyMods(e) => e.fmt(f),
                }
            } else {
                match self {
                    EnemyEncounter::General(General {
                        boss,
                        has_achievement,
                        done,
                        is_running,
                        all_enemies_killed,
                        ..
                    }) => {
                        write!(
                            f,
                            "Encounter: boss={} achievement={} done={} running={} killed={}",
                            boss, has_achievement, done, is_running, all_enemies_killed
                        )
                    }
                    EnemyEncounter::Enemy(Enemy {
                        id,
                        name,
                        enemy,
                        hide_hp,
                        ..
                    }) => {
                        write!(
                            f,
                            "Enemy: id={} name={} kind={:?} hide_hp={}",
                            id, name, enemy, hide_hp
                        )
                    }
                    EnemyEncounter::EnemyStats(EnemyStats {
                        current_hp,
                        max_hp,
                        level,
                        speed,
                        attack,
                        defense,
                        magic_attack,
                        magic_defense,
                        ..
                    }) => {
                        write!(
                            f,
                            "|--stats: hp={}/{} level={} speed={} A/MA={}/{} D/MD={}/{}",
                            current_hp,
                            max_hp,
                            level,
                            speed,
                            attack,
                            magic_attack,
                            defense,
                            magic_defense
                        )
                    }
                    EnemyEncounter::EnemyMods(EnemyMods {
                        any,
                        sword,
                        blunt,
                        sun,
                        moon,
                        poison,
                        arcane,
                        eclipse,
                        stun,
                    }) => {
                        write!(
                            f,
                            concat!(
                                "|--mods: any={} sword={} blunt={} sun={} ",
                                "moon={} poison={} arcane={} eclipse={} stun={}"
                            ),
                            any, sword, blunt, sun, moon, poison, arcane, eclipse, stun
                        )
                    }
                }
            }
        }
    }
}

mod progress {
    #[cfg(debugger)]
    use std::rc::Rc;

    #[cfg(debugger)]
    use ahash::HashMap;
    #[cfg(debugger)]
    use asr::{arrayvec::ArrayString, time::Duration};
    use asr::{
        game_engine::unity::il2cpp::{Class2, Game},
        MemReader, Pointer,
    };
    use bytemuck::AnyBitPattern;
    use csharp_mem::CSString;
    #[cfg(debugger)]
    use csharp_mem::Map;

    #[cfg(debugger)]
    #[derive(Class2, Debug)]
    struct ProgressionManager {
        #[singleton]
        #[rename = "instance"]
        _instance: Pointer<Self>,

        timestamp: f64,

        #[rename = "playTime"]
        play_time: f64,
    }

    #[cfg(debugger)]
    #[derive(Class2, Debug)]
    struct ActivityManager {
        #[singleton]
        #[rename = "instance"]
        _instance: Pointer<Self>,

        #[rename = "mainActivity"]
        main_activity: Pointer<CSString>,

        #[rename = "subActivityIndex"]
        sub_activity_index: u32,

        #[cfg(debugger)]
        #[cfg_attr(debugger, rename = "allActivityData")]
        all_activities: Pointer<Map<ActivityReference, Pointer<ActivityData>>>,
    }

    #[cfg(debugger)]
    #[derive(Copy, Clone, Debug, AnyBitPattern)]
    struct ActivityReference {
        guid: Pointer<CSString>,
    }

    #[cfg(debugger)]
    #[derive(Class2, Debug)]
    struct ActivityData {
        #[rename = "activityNameLoc"]
        name: super::loc::LocalizationId,
    }

    #[cfg(not(debugger))]
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

    #[cfg(debugger)]
    #[derive(Class2, Debug)]
    struct LevelManager {
        #[singleton]
        #[rename = "instance"]
        _instance: Pointer<Self>,

        #[rename = "loadingLevel"]
        is_loading: bool,
        #[rename = "currentLevel"]
        current_level: LevelReference,
        #[rename = "previousLevelInfo"]
        previous_level_info: Pointer<LoadedLevelInfo>,
        #[rename = "levelDefinitionPerLevel"]
        all_levels: Pointer<Map<LevelReference, Pointer<LevelDefinition>>>,
    }

    #[cfg(debugger)]
    #[derive(Class2, Debug)]
    struct LoadedLevelInfo {
        level: LevelReference,
    }

    #[derive(Copy, Clone, Debug, AnyBitPattern)]
    struct LevelReference {
        guid: Pointer<CSString>,
    }

    #[cfg(debugger)]
    #[derive(Class2, Debug)]
    struct LevelDefinition {
        #[rename = "levelNameLocId"]
        name: super::loc::LocalizationId,
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
        #[cfg(debugger)]
        progression_manager: ProgressionManagerBinding,
        #[cfg(debugger)]
        activity_manager: ActivityManagerBinding,
        #[cfg(debugger)]
        activity_data: ActivityDataBinding,
        #[cfg(debugger)]
        all_activities: HashMap<String, ActivityData>,
        #[cfg(debugger)]
        loaded_level_info: LoadedLevelInfoBinding,
        #[cfg(debugger)]
        level_definition: LevelDefinitionBinding,
        #[cfg(debugger)]
        all_levels: HashMap<String, LevelDefinition>,
    }

    impl Progression {
        #[cfg(not(debugger))]
        pub fn new() -> Self {
            Self {
                level_manager: LevelManager::bind(),
                cutscene_manager: CutsceneManager::bind(),
            }
        }

        #[cfg(debugger)]
        pub fn new() -> Self {
            use ahash::HashMapExt;

            Self {
                level_manager: LevelManager::bind(),
                cutscene_manager: CutsceneManager::bind(),
                progression_manager: ProgressionManager::bind(),
                activity_manager: ActivityManager::bind(),
                activity_data: ActivityData::bind(),
                all_activities: HashMap::new(),
                loaded_level_info: LoadedLevelInfo::bind(),
                level_definition: LevelDefinition::bind(),
                all_levels: HashMap::new(),
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

        #[cfg(debugger)]
        pub fn get_progress(
            &mut self,
            game: &Game<'_>,
            loc: &super::loc::Loc,
        ) -> Option<CurrentProgress> {
            let activity = self.activity_manager.read(game)?;

            if self.all_activities.is_empty() {
                if let Some(activities) = activity.all_activities.resolve(game) {
                    self.all_activities = activities
                        .iter(game)
                        .filter_map(|(ar, ad)| {
                            let ad = self.activity_data.read_pointer(game, ad)?;
                            let ar = ar.guid.resolve(game)?.to_std_string(game);
                            Some((ar, ad))
                        })
                        .collect();
                }
            }

            let level = self.level_manager.read(game)?;

            if self.all_levels.is_empty() {
                if let Some(levels) = level.all_levels.resolve(game) {
                    self.all_levels = levels
                        .iter(game)
                        .filter_map(|(lr, ld)| {
                            let ld = self.level_definition.read_pointer(game, ld)?;
                            let lr = lr.guid.resolve(game)?.to_std_string(game);
                            Some((lr, ld))
                        })
                        .collect();
                }
            }

            let progression = self.progression_manager.read(game)?;

            let main_activity = activity.main_activity.resolve(game)?.to_string(game);
            let activity_name = self
                .all_activities
                .get(main_activity.as_str())
                .map_or_else(|| "".into(), |o| loc.localized(game, o.name).into());

            let current_level = level.current_level.guid.resolve(game)?;
            let current_as_level = super::Level::resolve(current_level.chars(game));

            let current_level = current_level.to_string(game);
            let current_level_name = self
                .all_levels
                .get(current_level.as_str())
                .map_or_else(|| "".into(), |o| loc.localized(game, o.name).into());

            let previous_level = self
                .loaded_level_info
                .read_pointer(game, level.previous_level_info)
                .and_then(|o| o.level.guid.resolve(game).map(|o| o.to_string(game)))
                .unwrap_or_default();
            let previous_level_name = self
                .all_levels
                .get(previous_level.as_str())
                .map_or_else(|| "".into(), |o| loc.localized(game, o.name).into());

            let cutscene_manager = self.cutscene_manager.read(game)?;
            let is_in_cutscene = game.read(cutscene_manager._instance.address() + 0x30)?;

            Some(CurrentProgress {
                is_loading: level.is_loading,
                level: current_as_level,
                timestamp: progression.timestamp,
                play_time: progression.play_time,
                main_activity,
                activity_name,
                sub_activity_index: activity.sub_activity_index,
                current_level,
                current_level_name,
                previous_level,
                previous_level_name,
                is_in_cutscene,
            })
        }
    }

    #[derive(Clone, PartialEq)]
    pub struct CurrentProgression {
        pub is_loading: bool,
        pub is_in_cutscene: bool,
        pub level: Option<super::Level>,
    }

    #[derive(Clone, PartialEq)]
    pub struct CurrentProgress {
        pub is_loading: bool,
        pub level: Option<super::Level>,
        #[cfg(debugger)]
        pub timestamp: f64,
        #[cfg(debugger)]
        pub play_time: f64,
        #[cfg(debugger)]
        pub main_activity: ArrayString<36>,
        #[cfg(debugger)]
        pub activity_name: Rc<str>,
        #[cfg(debugger)]
        pub sub_activity_index: u32,
        #[cfg(debugger)]
        pub current_level: ArrayString<36>,
        #[cfg(debugger)]
        pub current_level_name: Rc<str>,
        #[cfg(debugger)]
        pub previous_level: ArrayString<36>,
        #[cfg(debugger)]
        pub previous_level_name: Rc<str>,
        #[cfg(debugger)]
        pub is_in_cutscene: bool,
    }

    #[cfg(debugger)]
    impl CurrentProgress {
        pub fn play_time(&self) -> PlayTime {
            PlayTime {
                session: Duration::seconds_f64(self.timestamp),
                total: Duration::seconds_f64(self.play_time),
            }
        }

        pub fn activity(&self) -> Activity {
            Activity {
                id: self.main_activity,
                name: Rc::clone(&self.activity_name),
                sub_index: self.sub_activity_index,
            }
        }

        pub fn current_level(&self) -> Level {
            Level {
                is_loading: self.is_loading,
                id: self.current_level,
                name: Rc::clone(&self.current_level_name),
            }
        }

        pub fn prev_level(&self) -> Level {
            Level {
                is_loading: self.is_loading,
                id: self.previous_level,
                name: Rc::clone(&self.previous_level_name),
            }
        }
    }

    #[cfg(debugger)]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct PlayTime {
        pub session: Duration,
        pub total: Duration,
    }

    #[cfg(debugger)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Activity {
        pub id: ArrayString<36>,
        pub name: Rc<str>,
        pub sub_index: u32,
    }

    #[cfg(debugger)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Level {
        pub is_loading: bool,
        pub id: ArrayString<36>,
        pub name: Rc<str>,
    }
}

#[cfg(debugger)]
mod loc {
    use ahash::HashMap;
    use asr::{
        game_engine::unity::il2cpp::{Class2, Game, Module},
        MemReader, Pointer, Process,
    };
    use bytemuck::AnyBitPattern;
    use csharp_mem::{CSString, List, Map};

    #[derive(Class2, Debug)]
    pub struct LocalizationManager {
        #[singleton]
        #[rename = "instance"]
        _instance: Pointer<Self>,

        #[rename = "locCategories"]
        pub loc_categories: Pointer<Map<Pointer<CSString>, Pointer<LocCategory>>>,
        #[rename = "locCategoryLanguages"]
        pub loc_category_languages: Pointer<Map<Pointer<CSString>, Pointer<LocCategoryLanguage>>>,
    }

    #[derive(Class2, Debug)]
    pub struct LocCategory {
        #[rename = "categoryId"]
        pub category_id: Pointer<CSString>,
        #[rename = "locIndexByLocStringId"]
        pub loc_index_by_loc_string_id: Pointer<LocIndexByLocStringId>,
    }

    #[derive(Class2, Debug)]
    pub struct LocIndexByLocStringId {
        pub dictionary: Pointer<Map<Pointer<CSString>, u32>>,
    }

    #[derive(Class2, Debug)]
    pub struct LocCategoryLanguage {
        #[rename = "locCategoryId"]
        pub loc_category_id: Pointer<CSString>,
        pub language: ELanguage,
        pub strings: Pointer<List<Pointer<CSString>>>,
    }

    #[derive(Copy, Clone, Debug, AnyBitPattern)]
    #[repr(C)]
    pub struct LocalizationId {
        pub category_name: Pointer<CSString>,
        pub loc_id: Pointer<CSString>,
    }

    #[derive(Copy, Clone, Debug, AnyBitPattern)]
    #[repr(C)]
    pub struct ELanguage {
        value: u32,
    }

    #[allow(unused)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(u32)]
    pub enum Language {
        EN = 0,
        JP = 1,
        RU = 2,
        KO = 3,
        QC = 4,
        FR = 5,
        DE = 6,
        ES = 7,
        BR = 8,
        CN = 9,
        HK = 10,
    }

    impl From<ELanguage> for Language {
        fn from(value: ELanguage) -> Self {
            unsafe { core::mem::transmute(value.value) }
        }
    }

    pub struct Localization<'a> {
        manager: LocalizationManagerBinding,
        category: LocCategoryBinding,
        index_by_id: LocIndexByLocStringIdBinding,
        category_language: LocCategoryLanguageBinding,
        game: Option<Game<'a>>,
        loc: Option<Loc>,
    }

    impl<'a> Localization<'a> {
        pub fn new() -> Self {
            Self {
                manager: LocalizationManager::bind(),
                category: LocCategory::bind(),
                index_by_id: LocIndexByLocStringId::bind(),
                category_language: LocCategoryLanguage::bind(),
                game: None,
                loc: None,
            }
        }
    }

    #[derive(Debug)]
    pub struct Loc {
        pub categories: HashMap<String, Category>,
        pub strings: HashMap<String, CategoryLanguage>,
    }

    impl<'a> Localization<'a> {
        pub fn resolve(&mut self, process: &'a Process, module: &Module) -> Option<&Loc> {
            #[allow(clippy::match_as_ref)]
            match self.loc {
                Some(ref loc) => Some(loc),
                None => {
                    let loc = self.resolve_loc(process, module)?;
                    self.loc = Some(loc);
                    self.loc.as_ref()
                }
            }
        }

        fn resolve_loc(&mut self, process: &'a Process, module: &Module) -> Option<Loc> {
            let game = match self.game {
                Some(ref game) => game,
                None => {
                    let image = module.get_image(process, "Sabotage.Localization")?;
                    let game = Game::new(process, module.clone(), image);
                    self.game = Some(game);
                    self.game.as_ref().unwrap()
                }
            };

            let manager = self.manager.read(game)?;

            let categories = manager.loc_categories.resolve(game)?;
            let categories = categories
                .iter(game)
                .filter_map(|(id, category)| {
                    let id = id.resolve(game)?.to_std_string(game);
                    let category = self.category.read_pointer(game, category)?;
                    let category = category.resolve(game, &mut self.index_by_id)?;
                    Some((id, category))
                })
                .collect::<HashMap<_, _>>();

            if categories.is_empty() {
                return None;
            }

            let strings = manager.loc_category_languages.resolve(game)?;
            let strings = strings
                .iter(game)
                .filter_map(|(id, lang)| {
                    let id = id.resolve(game)?.to_std_string(game);

                    let lang = self.category_language.read_pointer(game, lang)?;
                    let lang = lang.resolve(game)?;
                    Some((id, lang))
                })
                .collect::<HashMap<_, _>>();

            if strings.is_empty() {
                return None;
            }

            Some(Loc {
                categories,
                strings,
            })
        }
    }

    #[derive(Debug)]
    pub struct Category {
        pub id: Box<str>,
        pub index: HashMap<Box<str>, usize>,
    }

    impl LocCategory {
        fn resolve(
            &self,
            game: &Game<'_>,
            index_by_id: &mut LocIndexByLocStringIdBinding,
        ) -> Option<Category> {
            let id = self
                .category_id
                .resolve(game)?
                .to_std_string(game)
                .into_boxed_str();

            let index = index_by_id.read_pointer(game, self.loc_index_by_loc_string_id)?;
            let index = index.dictionary.resolve(game)?;

            let index = index
                .iter(game)
                .flat_map(|(id, index)| {
                    let id = id.resolve(game)?.to_std_string(game).into_boxed_str();
                    let index = index as usize;
                    Some((id, index))
                })
                .collect();

            Some(Category { id, index })
        }
    }

    #[derive(Debug)]
    pub struct CategoryLanguage {
        pub id: Box<str>,
        pub language: Language,
        pub strings: List<Pointer<CSString>>,
    }

    impl LocCategoryLanguage {
        fn resolve(&self, game: &Game<'_>) -> Option<CategoryLanguage> {
            let id = self
                .loc_category_id
                .resolve(game)?
                .to_std_string(game)
                .into_boxed_str();

            let language = self.language.into();

            let strings = self.strings.resolve(game)?;

            Some(CategoryLanguage {
                id,
                language,
                strings,
            })
        }
    }

    impl Loc {
        pub fn localized<R: MemReader>(&self, process: &R, id: LocalizationId) -> String {
            self.lookup(process, id).map_or_else(
                || {
                    id.loc_id
                        .resolve(process)
                        .map_or_else(String::new, |o| o.to_std_string(process))
                },
                |(n, _)| n,
            )
        }

        pub fn lookup<R: MemReader>(
            &self,
            process: &R,
            loc_id: LocalizationId,
        ) -> Option<(String, Language)> {
            let cat_id = loc_id
                .category_name
                .resolve(process)?
                .to_std_string(process);

            let cat = self.categories.get(&cat_id)?;
            let strings = self.strings.get(&cat_id)?;

            let loc_id = loc_id
                .loc_id
                .resolve(process)?
                .to_std_string(process)
                .into_boxed_str();

            let index = *cat.index.get(&loc_id)?;

            let string = strings.strings.get(process, index)?;
            let string = string.resolve(process)?.to_std_string(process);

            Some((string, strings.language))
        }
    }
}

#[cfg(not(debugger))]
mod inventory {
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
        owned_key_items: [(u32, u32); 4],
        generation: u32,
    }

    impl Inventory {
        pub fn new() -> Self {
            Self {
                quantity: QuantityByInventoryItemReference::bind(),
                manager: InventoryManager::bind(),
                number_of_owned_items: Watcher::new(),
                owned_key_items: [(u32::MAX, u32::MAX); 4], // capacity is number of KeyItem variants
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
                    match this.owned_key_items[item as usize] {
                        (u32::MAX, u32::MAX) => {
                            this.owned_key_items[item as usize] = (generation, generation);
                        }
                        (_, ref mut current) => {
                            *current = generation;
                        }
                    }
                }

                let items = this.owned_key_items.iter().enumerate().filter_map(
                    move |(item, &(insert, current))| {
                        let item = match item {
                            0 => super::KeyItem::Graplou,
                            1 => super::KeyItem::MasterGhostSandwich,
                            2 => super::KeyItem::Map,
                            3 => super::KeyItem::Seashell,
                            _ => return None,
                        };
                        if current == generation {
                            if insert == current {
                                Some(Change::PickedUp(item))
                            } else {
                                None
                            }
                        } else {
                            Some(Change::Lost(item))
                        }
                    },
                );

                Some(items)
            }

            inner(self, game).into_iter().flatten()
        }
    }
}

#[cfg(debugger)]
mod inventory {
    use std::collections::hash_map::Entry;

    use ahash::{HashMap, HashMapExt};
    use asr::{
        game_engine::unity::il2cpp::{Class2, Game},
        string::ArrayString,
        watcher::Watcher,
        Pointer,
    };
    use bytemuck::AnyBitPattern;

    use csharp_mem::{CSString, Map};

    use super::loc::{Loc, LocalizationId};

    #[derive(Class2, Debug)]
    struct InventoryManager {
        #[singleton]
        #[rename = "instance"]
        _instance: Pointer<Self>,

        #[rename = "allInventoryItemData"]
        all_items: Pointer<Map<InventoryItemReference, Pointer<InventoryItem>>>,
        #[rename = "ownedInventoryItems"]
        owned_items: Pointer<QuantityByInventoryItemReference>,
    }

    #[derive(Class2, Debug)]
    struct InventoryItem {
        guid: Pointer<CSString>,
        #[rename = "nameLocalizationId"]
        name: LocalizationId,
    }

    #[derive(Class2, Debug)]
    struct KeyItem {}

    #[derive(Debug, Copy, Clone, AnyBitPattern)]
    struct InventoryItemReference {
        guid: Pointer<CSString>,
    }

    #[derive(Class2, Debug)]
    struct QuantityByInventoryItemReference {
        dictionary: Pointer<Map<InventoryItemReference, u32>>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct NamedKeyItem {
        pub id: ArrayString<32>,
        pub name: String,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Change<T> {
        PickedUp(T),
        Lost(T),
    }

    impl<T> Change<T> {
        pub fn transform<U>(self, f: impl FnOnce(T) -> Option<U>) -> Option<Change<U>> {
            match self {
                Change::PickedUp(t) => f(t).map(Change::PickedUp),
                Change::Lost(t) => f(t).map(Change::Lost),
            }
        }
    }

    pub struct Inventory {
        inventory_item: InventoryItemBinding,
        key_item: KeyItemBinding,
        quantity: QuantityByInventoryItemReferenceBinding,
        manager: InventoryManagerBinding,
        all_key_items: HashMap<ArrayString<32>, NamedKeyItem>,
        number_of_owned_items: Watcher<u32>,
        owned_key_item_ids: HashMap<ArrayString<32>, (u32, u32)>,
        owned_key_items: HashMap<super::KeyItem, (u32, u32)>,
        generation: u32,
    }

    impl Inventory {
        pub fn new() -> Self {
            Self {
                inventory_item: InventoryItem::bind(),
                key_item: KeyItem::bind(),
                quantity: QuantityByInventoryItemReference::bind(),
                manager: InventoryManager::bind(),
                all_key_items: HashMap::new(),
                number_of_owned_items: Watcher::new(),
                owned_key_item_ids: HashMap::new(),
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

        pub fn check_new<'a>(
            &'a mut self,
            game: &'a Game<'_>,
            loc: &'a Loc,
        ) -> impl Iterator<Item = Change<&'a NamedKeyItem>> + 'a {
            self.cache_available_items(game, loc);
            self.changed_owned_key_items(game)
        }

        fn cache_available_items(&mut self, game: &Game<'_>, loc: &Loc) -> bool {
            if !self.all_key_items.is_empty() {
                return false;
            }

            (|| {
                let manager = self.manager.read(game)?;
                let all_items = manager.all_items.resolve(game)?;

                for (_, v) in all_items.iter(game) {
                    let is_key_item = self
                        .key_item
                        .class(game)?
                        .is_instance(game.process(), game.module(), v.address())
                        .ok()?;

                    if is_key_item {
                        let item = self.inventory_item.read_pointer(game, v)?;
                        let id = item.guid.resolve(game)?.to_string(game);
                        let name = loc.localized(game, item.name);
                        let item = NamedKeyItem { id, name };
                        self.all_key_items.insert(id, item);
                    }
                }

                Some(())
            })();

            !self.all_key_items.is_empty()
        }

        fn changed_owned_key_items<'a>(
            &'a mut self,
            game: &'a Game<'_>,
        ) -> impl Iterator<Item = Change<&'a NamedKeyItem>> + 'a {
            Some(&self.all_key_items)
                .filter(|o| !o.is_empty())
                .and_then(|key_items| {
                    let manager = self.manager.read(game)?;
                    let owned = self.quantity.read_pointer(game, manager.owned_items)?;
                    let owned = owned.dictionary.resolve(game)?;

                    let first = self.number_of_owned_items.pair.is_none();
                    let now_owned = self.number_of_owned_items.update_infallible(owned.size());

                    if !first && !now_owned.changed() {
                        return None;
                    }

                    self.generation += 1;
                    let generation = self.generation;

                    for item in owned.iter(game).filter_map(move |(item, _amount)| {
                        let item = item.guid.resolve(game)?;
                        let item = item.to_string(game);
                        key_items.contains_key(&item).then_some(item)
                    }) {
                        match self.owned_key_item_ids.entry(item) {
                            Entry::Occupied(existing) => {
                                existing.into_mut().1 = generation;
                            }
                            Entry::Vacant(missing) => {
                                missing.insert((generation, generation));
                            }
                        };
                    }

                    let items = self
                        .owned_key_item_ids
                        .iter()
                        .filter_map(move |(item, &(insert, current))| {
                            if current == generation {
                                if insert == current {
                                    Some(Change::PickedUp(item))
                                } else {
                                    None
                                }
                            } else {
                                Some(Change::Lost(item))
                            }
                        })
                        .filter_map(|o| o.transform(|o| key_items.get(o)));
                    Some(items)
                })
                .into_iter()
                .flatten()
        }
    }
}
