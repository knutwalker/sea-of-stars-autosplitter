#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Enemy {
    Wyrd,
    Bossslug,
    ElderMist,
    Salamander,
    Malkomud,
    ChromaticApparition,
    Duke,
    Romaya,
    BotanicalHorror,
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
    Hydralion,
    Toadcano,
    Guardian,
    Meduso,
    Casugin,
    Abstarak,
    Rachater,
    Catalyst,
    DwellerOfDread,
    LeJugg,
    PhaseReaper,
    Elysandarelle1,
    Elysandarelle2,
}

impl Enemy {
    pub fn resolve(mut id: impl Iterator<Item = char>) -> Option<Self> {
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
            '1' => eq!("1894c41627be94d408bd64295ab6dd18" => Self::Erlina),
            '5' => next! {
                '4' => eq!("54abc79fbf9dd2f4a8bd19cab8245391" => Self::PhaseReaper),
                '7' => eq!("5750f181921e1f349b595e8e47760d33" => Self::Bossslug),
                'c' => eq!("5cdedb65d17f3b24c8b7ad5bcbe1bea6" => Self::Romaya),
            },
            '6' => eq!("64246a3a9059257409ea628466ced26e" => Self::BotanicalHorror),
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
                '4' => eq!("c4480713abcb0d04f8a21a702987e6e1" => Self::Rachater),
                '9' => eq!("c99b902697c6f734f9fc64b421c06728" => Self::LeafMonster),
                'c' => eq!("cc767e360aab54d4ca314a206e32ffee" => Self::Brugaves),
            },
            'd' => eq!("d0f2cf59f69f42842ac0703193f39c85" => Self::Salamander),
            'e' => eq!("e77c07b22ee83854e8c006101ef5731f" => Self::Three),
            'f' => eq!("f4032b2323bc31d4590cf5197db3c3f1" => Self::Abstarak),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    FleshmancersLair,
    WorldEeater,
}

impl Level {
    pub fn resolve(mut id: impl Iterator<Item = char>) -> Option<Self> {
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
            '2' => eq!("20fd3c19d96a9cb41b1db9841837e8e4" => Self::FleshmancersLair),
            '3' => eq!("3dab1b3e3a5221c40989f1c68cfcd352" => Self::WorldEeater),
        }
    }
}
