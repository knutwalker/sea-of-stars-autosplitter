use std::{
    error::Error,
    io::{BufRead, Write},
};

#[derive(Copy, Clone, Default, PartialEq, Eq)]
struct Encounter {
    boss: bool,
    achievement: bool,
    defined: bool,
    line: Option<usize>,
}

impl std::fmt::Debug for Encounter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Encounter")
            .field("boss", &self.boss)
            .field("achievement", &self.achievement)
            .finish()
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct EncLine {
    boss: bool,
    achievement: bool,
    done: bool,
}

impl EncLine {
    fn update(self, (key, value): (&str, &str)) -> Self {
        match key {
            "boss" => Self {
                boss: value == "true",
                ..self
            },
            "achievement" => Self {
                achievement: value == "true",
                ..self
            },
            "done" => Self {
                done: value == "true",
                ..self
            },
            _ => self,
        }
    }
}

#[derive(Clone, Default, Eq)]
struct Enemy {
    id: String,
    name: String,
    hp: i32,
    level: i32,
    at: i32,
    mat: i32,
    de: i32,
    mde: i32,
    dmg: String,
    encounter: Encounter,
    line: Option<usize>,
}

impl PartialEq for Enemy {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Ord for Enemy {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.encounter
            .boss
            .cmp(&other.encounter.boss)
            .reverse()
            .then(self.name.cmp(&other.name))
            .then(self.id.cmp(&other.id))
            .then(self.hp.cmp(&other.hp))
            .then(self.encounter.achievement.cmp(&other.encounter.achievement))
            .then(self.level.cmp(&other.level))
            .then(self.at.cmp(&other.at))
            .then(self.mat.cmp(&other.mat))
            .then(self.de.cmp(&other.de))
            .then(self.mde.cmp(&other.mde))
            .then(self.dmg.cmp(&other.dmg))
    }
}

impl PartialOrd for Enemy {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Debug for Enemy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("")
            // .field("boss", &self.encounter.boss)
            .field(
                "name",
                &if self.name.is_empty() {
                    "TODO"
                } else {
                    self.name.as_str()
                },
            )
            .field("id", &self.id)
            .field("hp", &self.hp)
            .field("achievement", &self.encounter.achievement)
            .field("level", &self.level)
            .field("at", &self.at)
            .field("mat", &self.mat)
            .field("de", &self.de)
            .field("mde", &self.mde)
            .field("dmg", &self.dmg)
            .finish()
    }
}

impl Enemy {
    fn update(mut self, (key, value): (&str, &str)) -> Self {
        match key {
            "id" => Self {
                id: value.to_owned(),
                ..self
            },
            "name" => Self {
                name: value.to_owned(),
                ..self
            },
            "hp" => Self {
                hp: value
                    .split_once('/')
                    .and_then(|(_, v)| v.parse().ok())
                    .unwrap_or(-1),
                ..self
            },
            "level" => Self {
                level: value.parse().unwrap_or(-1),
                ..self
            },
            "A/MA" => {
                let (at, mat) = value
                    .trim_end_matches(',')
                    .split_once('/')
                    .and_then(|(a, m)| Some((a.parse().ok()?, m.parse().ok()?)))
                    .unwrap_or((-1, -1));
                Self { at, mat, ..self }
            }
            "D/MD" => {
                let (de, mde) = value
                    .split_once('/')
                    .and_then(|(d, m)| Some((d.parse().ok()?, m.parse().ok()?)))
                    .unwrap_or((-1, -1));
                Self { de, mde, ..self }
            }
            "any" | "sword" | "blunt" | "sun" | "moon" | "poison" | "arcane" | "eclipse"
            | "stun" => {
                if !self.dmg.is_empty() {
                    self.dmg.push(' ');
                }
                self.dmg.push_str(key);
                self.dmg.push('=');
                self.dmg.push_str(value.trim_end_matches(','));
                self
            }
            "hide_hp" | "speed" => self,
            _ => {
                self.name.push(' ');
                self.name.push_str(value);
                self
            }
        }
    }
}

fn split(line: &str) -> impl Iterator<Item = (&str, &str)> {
    line.split_whitespace()
        .map(|o| o.split_once('=').unwrap_or(("", o)))
}

fn dump<E: Error + 'static>(
    enemy: &mut Enemy,
    out: impl FnOnce(Enemy) -> Result<(), E>,
) -> Result<(), Box<dyn Error>> {
    if enemy.encounter.defined && !enemy.id.is_empty() {
        let enemy = std::mem::take(enemy);
        out(enemy)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut enemies = Vec::new();
    let mut stdout = std::io::stdout().lock();

    let mut out = |e: Enemy| -> std::io::Result<()> {
        // writeln!(stdout, "{e:?}")?;
        enemies.push(e);
        Ok(())
    };

    let stdin = std::io::stdin().lock();
    let mut encounter = Encounter::default();
    let mut enemy = Enemy::default();
    for (line_number, line) in stdin.lines().enumerate() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line == "--" {
            encounter = Encounter::default();
            continue;
        }

        if line.starts_with("Encounter:") {
            let enc = split(line).fold(EncLine::default(), EncLine::update);
            if enc.done == false {
                encounter = Encounter {
                    boss: enc.boss,
                    achievement: enc.achievement,
                    defined: true,
                    line: Some(line_number + 1),
                };
            } else {
                encounter = Encounter::default();
            }
        } else {
            if line.starts_with("Enemy: ") {
                dump(&mut enemy, &mut out)?;
                enemy.encounter = encounter;
                enemy.line = Some(line_number + 1);
            }
            match line.split_once(' ') {
                Some((
                    "Enemy:" | "|--stats:" | "|--mods:" | "--mods:" | "---mods:" | "=--mods:"
                    | "├──stats:" | "└──mods:",
                    line,
                )) => {
                    enemy = split(line).fold(enemy, Enemy::update);
                }
                _ => {
                    eprintln!("Ignoring line {}: {line}", line_number + 1);
                }
            };
        }
    }

    dump(&mut enemy, out)?;

    enemies.sort();
    enemies.dedup();
    let mid = enemies.partition_point(|o| o.encounter.boss);
    let (bosses, enemies) = enemies.split_at(mid);

    writeln!(stdout, "Bosses")?;
    for e in bosses {
        writeln!(stdout, "{e:?}")?;
    }
    writeln!(stdout, "Regular enemies")?;
    for e in enemies {
        writeln!(stdout, "{e:?}")?;
    }

    Ok(())
}
