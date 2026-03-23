//! Character creation system.
//!
//! Provides races, backgrounds, stat generation methods, and a full
//! `CharacterSheet` builder for the CHAOS RPG character creation flow.

use std::collections::HashMap;

// ─── STAT METHOD ─────────────────────────────────────────────────────────────

/// How ability scores are generated during character creation.
#[derive(Debug, Clone, PartialEq)]
pub enum StatMethod {
    /// Player assigns points from a budget (8–15 range per stat).
    PointBuy { points: u32 },
    /// Classic array: [15, 14, 13, 12, 10, 8].
    StandardArray,
    /// Roll 4d6, drop the lowest die, repeat six times.
    RollFourDropOne { seed: u64 },
    /// Caller supplies stats directly (no validation).
    Manual,
}

// ─── BACKGROUND ──────────────────────────────────────────────────────────────

/// Character backgrounds, each granting skill proficiencies and starting gear.
#[derive(Debug, Clone, PartialEq)]
pub enum Background {
    Acolyte,
    Criminal,
    FolkHero,
    Noble,
    Sage,
    Soldier,
    Outlander,
    Entertainer,
}

impl Background {
    /// Skill proficiencies granted by this background.
    pub fn skill_bonuses(&self) -> Vec<String> {
        match self {
            Background::Acolyte => vec!["Insight".into(), "Religion".into()],
            Background::Criminal => vec!["Deception".into(), "Stealth".into()],
            Background::FolkHero => vec!["Animal Handling".into(), "Survival".into()],
            Background::Noble => vec!["History".into(), "Persuasion".into()],
            Background::Sage => vec!["Arcana".into(), "History".into()],
            Background::Soldier => vec!["Athletics".into(), "Intimidation".into()],
            Background::Outlander => vec!["Athletics".into(), "Survival".into()],
            Background::Entertainer => vec!["Acrobatics".into(), "Performance".into()],
        }
    }

    /// Starting equipment provided by this background.
    pub fn starting_equipment(&self) -> Vec<String> {
        match self {
            Background::Acolyte => vec![
                "Holy Symbol".into(),
                "Prayer Book".into(),
                "5 Sticks of Incense".into(),
                "Vestments".into(),
                "Common Clothes".into(),
                "Pouch (15 gp)".into(),
            ],
            Background::Criminal => vec![
                "Crowbar".into(),
                "Dark Common Clothes with Hood".into(),
                "Pouch (15 gp)".into(),
            ],
            Background::FolkHero => vec![
                "Artisan's Tools".into(),
                "Shovel".into(),
                "Iron Pot".into(),
                "Common Clothes".into(),
                "Pouch (10 gp)".into(),
            ],
            Background::Noble => vec![
                "Fine Clothes".into(),
                "Signet Ring".into(),
                "Scroll of Pedigree".into(),
                "Purse (25 gp)".into(),
            ],
            Background::Sage => vec![
                "Bottle of Black Ink".into(),
                "Quill".into(),
                "Small Knife".into(),
                "Letter from Mentor".into(),
                "Common Clothes".into(),
                "Pouch (10 gp)".into(),
            ],
            Background::Soldier => vec![
                "Insignia of Rank".into(),
                "Trophy from Fallen Enemy".into(),
                "Deck of Cards".into(),
                "Common Clothes".into(),
                "Pouch (10 gp)".into(),
            ],
            Background::Outlander => vec![
                "Staff".into(),
                "Hunting Trap".into(),
                "Trophy from Animal".into(),
                "Traveler's Clothes".into(),
                "Pouch (10 gp)".into(),
            ],
            Background::Entertainer => vec![
                "Musical Instrument".into(),
                "Favor of an Admirer".into(),
                "Costume".into(),
                "Pouch (15 gp)".into(),
            ],
        }
    }

    /// Display name for the background.
    pub fn name(&self) -> &'static str {
        match self {
            Background::Acolyte => "Acolyte",
            Background::Criminal => "Criminal",
            Background::FolkHero => "Folk Hero",
            Background::Noble => "Noble",
            Background::Sage => "Sage",
            Background::Soldier => "Soldier",
            Background::Outlander => "Outlander",
            Background::Entertainer => "Entertainer",
        }
    }
}

impl std::fmt::Display for Background {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ─── RACE TRAITS ─────────────────────────────────────────────────────────────

/// Racial traits applied to a character sheet.
#[derive(Debug, Clone)]
pub struct RaceTraits {
    pub name: String,
    pub stat_bonuses: HashMap<String, i32>,
    pub traits: Vec<String>,
    pub speed_ft: u32,
    pub size: String,
    pub languages: Vec<String>,
}

// ─── CHARACTER SHEET ─────────────────────────────────────────────────────────

/// A completed character sheet produced by `CharacterCreator::create`.
#[derive(Debug, Clone)]
pub struct CharacterSheet {
    pub name: String,
    pub race: String,
    pub class: String,
    pub level: u32,
    pub stats: HashMap<String, u8>,
    pub background: Background,
    pub skills: Vec<String>,
    pub personality: String,
    pub backstory: String,
}

// ─── CHARACTER CREATOR ───────────────────────────────────────────────────────

/// Orchestrates character creation: races, stat generation, and sheet assembly.
pub struct CharacterCreator;

impl CharacterCreator {
    // ── Races ────────────────────────────────────────────────────────────────

    /// Returns all playable races with their traits.
    pub fn available_races() -> Vec<RaceTraits> {
        vec![
            RaceTraits {
                name: "Human".into(),
                stat_bonuses: [
                    ("STR", 1), ("DEX", 1), ("CON", 1),
                    ("INT", 1), ("WIS", 1), ("CHA", 1),
                ].iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                traits: vec!["Extra Language".into(), "Extra Skill".into()],
                speed_ft: 30,
                size: "Medium".into(),
                languages: vec!["Common".into(), "One extra".into()],
            },
            RaceTraits {
                name: "Elf".into(),
                stat_bonuses: [("DEX", 2)].iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                traits: vec!["Darkvision".into(), "Keen Senses".into(), "Fey Ancestry".into(), "Trance".into()],
                speed_ft: 30,
                size: "Medium".into(),
                languages: vec!["Common".into(), "Elvish".into()],
            },
            RaceTraits {
                name: "Dwarf".into(),
                stat_bonuses: [("CON", 2)].iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                traits: vec!["Darkvision".into(), "Dwarven Resilience".into(), "Stonecunning".into()],
                speed_ft: 25,
                size: "Medium".into(),
                languages: vec!["Common".into(), "Dwarvish".into()],
            },
            RaceTraits {
                name: "Halfling".into(),
                stat_bonuses: [("DEX", 2)].iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                traits: vec!["Lucky".into(), "Brave".into(), "Halfling Nimbleness".into()],
                speed_ft: 25,
                size: "Small".into(),
                languages: vec!["Common".into(), "Halfling".into()],
            },
            RaceTraits {
                name: "Orc".into(),
                stat_bonuses: [("STR", 2), ("CON", 1)].iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                traits: vec!["Darkvision".into(), "Aggressive".into(), "Menacing".into()],
                speed_ft: 30,
                size: "Medium".into(),
                languages: vec!["Common".into(), "Orc".into()],
            },
            RaceTraits {
                name: "Tiefling".into(),
                stat_bonuses: [("INT", 1), ("CHA", 2)].iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                traits: vec!["Darkvision".into(), "Hellish Resistance".into(), "Infernal Legacy".into()],
                speed_ft: 30,
                size: "Medium".into(),
                languages: vec!["Common".into(), "Infernal".into()],
            },
            RaceTraits {
                name: "Dragonborn".into(),
                stat_bonuses: [("STR", 2), ("CHA", 1)].iter().map(|(k, v)| (k.to_string(), *v)).collect(),
                traits: vec!["Draconic Ancestry".into(), "Breath Weapon".into(), "Damage Resistance".into()],
                speed_ft: 30,
                size: "Medium".into(),
                languages: vec!["Common".into(), "Draconic".into()],
            },
        ]
    }

    // ── Stat Generation ──────────────────────────────────────────────────────

    /// Point-buy: validates that assignments stay in [8, 15] and consume ≤ budget.
    ///
    /// Point cost per stat value:
    ///   8→0, 9→1, 10→2, 11→3, 12→4, 13→5, 14→7, 15→9
    pub fn point_buy_stats(
        points: u32,
        assignments: &HashMap<String, u8>,
    ) -> Result<HashMap<String, u8>, String> {
        fn cost(v: u8) -> Result<u32, String> {
            match v {
                8 => Ok(0),
                9 => Ok(1),
                10 => Ok(2),
                11 => Ok(3),
                12 => Ok(4),
                13 => Ok(5),
                14 => Ok(7),
                15 => Ok(9),
                _ => Err(format!("stat value {} out of range [8, 15]", v)),
            }
        }

        let mut total_cost = 0u32;
        for (stat, &val) in assignments {
            let c = cost(val).map_err(|e| format!("{}: {}", stat, e))?;
            total_cost += c;
        }

        if total_cost > points {
            return Err(format!("assignments cost {} points but budget is {}", total_cost, points));
        }

        Ok(assignments.clone())
    }

    /// Returns the standard array: [15, 14, 13, 12, 10, 8].
    pub fn standard_array() -> Vec<u8> {
        vec![15, 14, 13, 12, 10, 8]
    }

    /// Simulates rolling 4d6 drop lowest for 6 stats using a simple LCG seeded by `seed`.
    pub fn roll_stats(seed: u64) -> Vec<u8> {
        // LCG parameters (Knuth / Numerical Recipes)
        const A: u64 = 6_364_136_223_846_793_005;
        const C: u64 = 1_442_695_040_888_963_407;

        let mut state = seed;
        let mut next = || -> u8 {
            state = state.wrapping_mul(A).wrapping_add(C);
            ((state >> 33) % 6 + 1) as u8
        };

        let mut stats = Vec::with_capacity(6);
        for _ in 0..6 {
            let mut rolls = [next(), next(), next(), next()];
            rolls.sort_unstable();
            // drop the lowest (index 0 after sort)
            let sum: u8 = rolls[1..].iter().sum();
            stats.push(sum);
        }
        stats
    }

    // ── Racial Bonuses ───────────────────────────────────────────────────────

    /// Applies racial stat bonuses to an existing stat map (clamped to 20).
    pub fn apply_race_bonuses(stats: &mut HashMap<String, u8>, race: &RaceTraits) {
        for (stat, bonus) in &race.stat_bonuses {
            let entry = stats.entry(stat.clone()).or_insert(8);
            *entry = (*entry as i32 + bonus).clamp(1, 20) as u8;
        }
    }

    // ── Modifiers ────────────────────────────────────────────────────────────

    /// Calculates ability modifiers: floor((score − 10) / 2).
    pub fn calculate_modifiers(stats: &HashMap<String, u8>) -> HashMap<String, i8> {
        stats.iter().map(|(k, &v)| {
            let modifier = ((v as i16 - 10) / 2).clamp(-5, 10) as i8;
            (k.clone(), modifier)
        }).collect()
    }

    // ── Backstory Generator ──────────────────────────────────────────────────

    /// Generates a template-based backstory string.
    pub fn generate_backstory(
        race: &str,
        class: &str,
        background: &Background,
        seed: u64,
    ) -> String {
        let locations = ["the Northern Wastes", "a bustling port city", "a quiet mountain village",
                         "the Sunken Marshes", "an ancient elven grove", "the Crimson Desert"];
        let motivations = ["seeking revenge for a fallen comrade", "searching for a legendary artifact",
                           "fleeing a dark past", "pursuing forbidden knowledge", "protecting the innocent",
                           "amassing wealth and power"];
        let events = ["a great calamity struck", "an unexpected mentor appeared", "a prophecy was revealed",
                      "a loved one was taken", "a powerful enemy emerged", "a hidden talent awakened"];

        let idx = (seed as usize) % locations.len();
        let location = locations[idx % locations.len()];
        let motivation = motivations[(seed as usize / 7) % motivations.len()];
        let event = events[(seed as usize / 13) % events.len()];

        format!(
            "Born in {location}, {race} {class} of the {background} background. \
             Once {event} that changed everything. Now they wander {motivation}.",
        )
    }

    // ── Full Creation ────────────────────────────────────────────────────────

    /// Assembles a complete character sheet.
    pub fn create(
        name: &str,
        race: RaceTraits,
        class: &str,
        background: Background,
        stat_method: StatMethod,
    ) -> CharacterSheet {
        let base_stat_names = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];

        let mut stats: HashMap<String, u8> = match &stat_method {
            StatMethod::StandardArray => {
                let arr = Self::standard_array();
                base_stat_names.iter().enumerate()
                    .map(|(i, &s)| (s.to_string(), arr[i]))
                    .collect()
            }
            StatMethod::RollFourDropOne { seed } => {
                let rolls = Self::roll_stats(*seed);
                base_stat_names.iter().enumerate()
                    .map(|(i, &s)| (s.to_string(), rolls[i]))
                    .collect()
            }
            StatMethod::PointBuy { points: _ } => {
                // Default 10 in all stats; caller can post-process with point_buy_stats
                base_stat_names.iter().map(|&s| (s.to_string(), 10u8)).collect()
            }
            StatMethod::Manual => {
                base_stat_names.iter().map(|&s| (s.to_string(), 10u8)).collect()
            }
        };

        Self::apply_race_bonuses(&mut stats, &race);

        let seed = name.bytes().fold(42u64, |acc, b| acc.wrapping_mul(131).wrapping_add(b as u64));
        let backstory = Self::generate_backstory(&race.name, class, &background, seed);

        let skills = background.skill_bonuses();

        let personalities = [
            "Thoughtful and deliberate in all things.",
            "Quick to laugh, slow to anger.",
            "Honor above all else.",
            "Curiosity drives every decision.",
            "Loyal to those who earn it.",
        ];
        let personality = personalities[seed as usize % personalities.len()].to_string();

        let race_name = race.name.clone();

        CharacterSheet {
            name: name.to_string(),
            race: race_name,
            class: class.to_string(),
            level: 1,
            stats,
            background,
            skills,
            personality,
            backstory,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_array_has_six_values() {
        let arr = CharacterCreator::standard_array();
        assert_eq!(arr.len(), 6);
        assert_eq!(arr[0], 15);
        assert_eq!(arr[5], 8);
    }

    #[test]
    fn point_buy_within_budget() {
        let mut assignments = HashMap::new();
        assignments.insert("STR".into(), 15u8); // costs 9
        assignments.insert("DEX".into(), 14u8); // costs 7
        // total = 16 ≤ 27
        let result = CharacterCreator::point_buy_stats(27, &assignments);
        assert!(result.is_ok());
    }

    #[test]
    fn point_buy_exceeds_budget() {
        let mut assignments = HashMap::new();
        assignments.insert("STR".into(), 15u8); // 9
        assignments.insert("DEX".into(), 15u8); // 9
        assignments.insert("CON".into(), 15u8); // 9 → total 27
        assignments.insert("INT".into(), 15u8); // 9 → total 36 > 27
        let result = CharacterCreator::point_buy_stats(27, &assignments);
        assert!(result.is_err());
    }

    #[test]
    fn modifiers_calculated_correctly() {
        let mut stats = HashMap::new();
        stats.insert("STR".into(), 10u8); // modifier 0
        stats.insert("DEX".into(), 16u8); // modifier +3
        stats.insert("CON".into(), 8u8);  // modifier -1
        let mods = CharacterCreator::calculate_modifiers(&stats);
        assert_eq!(mods["STR"], 0);
        assert_eq!(mods["DEX"], 3);
        assert_eq!(mods["CON"], -1);
    }

    #[test]
    fn roll_stats_produces_six_values() {
        let rolls = CharacterCreator::roll_stats(12345);
        assert_eq!(rolls.len(), 6);
        for &r in &rolls {
            assert!(r >= 3 && r <= 18, "roll {} out of range", r);
        }
    }

    #[test]
    fn create_builds_valid_sheet() {
        let races = CharacterCreator::available_races();
        let human = races.into_iter().find(|r| r.name == "Human").unwrap();
        let sheet = CharacterCreator::create(
            "Aldric",
            human,
            "Fighter",
            Background::Soldier,
            StatMethod::StandardArray,
        );
        assert_eq!(sheet.name, "Aldric");
        assert_eq!(sheet.level, 1);
        assert!(!sheet.backstory.is_empty());
    }
}
