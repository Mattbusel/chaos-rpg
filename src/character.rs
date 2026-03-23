//! Characters, classes, stats, and leveling.
//!
//! CHAOS RPG has 4 character classes and 7 unbounded stats.
//! Stats can exceed any limit — there is no cap. The universe is your cap.

use crate::chaos_pipeline::{chaos_roll_verbose, destiny_roll, roll_stat};
use serde::{Deserialize, Serialize};

// ─── CLASSES ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterClass {
    Mage,
    Berserker,
    Ranger,
    Thief,
}

impl CharacterClass {
    pub fn name(&self) -> &'static str {
        match self {
            CharacterClass::Mage => "Mage",
            CharacterClass::Berserker => "Berserker",
            CharacterClass::Ranger => "Ranger",
            CharacterClass::Thief => "Thief",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CharacterClass::Mage => {
                "Bends chaos through pure mathematical will. High MANA and ENTROPY, low VITALITY."
            }
            CharacterClass::Berserker => {
                "Channels pain into exponential power. VITALITY and FORCE scale catastrophically."
            }
            CharacterClass::Ranger => {
                "Reads the prime number patterns in nature. Balanced stats with deadly PRECISION."
            }
            CharacterClass::Thief => {
                "Exploits the logistic map's chaotic phase transitions. Master of CUNNING and LUCK."
            }
        }
    }

    pub fn ascii_art(&self) -> &'static str {
        match self {
            CharacterClass::Mage => "  /\\ \n (∞) \n  ||",
            CharacterClass::Berserker => "  ><  \n [RAGE]\n  \\/ ",
            CharacterClass::Ranger => "  />\\\n  |||  \n  vvv",
            CharacterClass::Thief => "  .~~.\n  {~} \n  /|\\",
        }
    }

    /// Base stat multipliers for each class
    pub fn stat_weights(&self) -> StatBlock {
        match self {
            CharacterClass::Mage => StatBlock {
                vitality: 40,
                force: 30,
                mana: 90,
                cunning: 60,
                precision: 50,
                entropy: 80,
                luck: 55,
            },
            CharacterClass::Berserker => StatBlock {
                vitality: 90,
                force: 85,
                mana: 20,
                cunning: 25,
                precision: 40,
                entropy: 70,
                luck: 35,
            },
            CharacterClass::Ranger => StatBlock {
                vitality: 55,
                force: 55,
                mana: 45,
                cunning: 60,
                precision: 90,
                entropy: 50,
                luck: 60,
            },
            CharacterClass::Thief => StatBlock {
                vitality: 45,
                force: 40,
                mana: 55,
                cunning: 90,
                precision: 70,
                entropy: 65,
                luck: 85,
            },
        }
    }
}

impl std::fmt::Display for CharacterClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ─── BACKGROUNDS ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Background {
    Scholar,   // +MANA +ENTROPY
    Wanderer,  // +LUCK +PRECISION
    Gladiator, // +FORCE +VITALITY
    Outcast,   // +CUNNING +ENTROPY
    Merchant,  // +LUCK +CUNNING
    Cultist,   // +MANA +ENTROPY (extreme)
}

impl Background {
    pub fn name(&self) -> &'static str {
        match self {
            Background::Scholar => "Scholar",
            Background::Wanderer => "Wanderer",
            Background::Gladiator => "Gladiator",
            Background::Outcast => "Outcast",
            Background::Merchant => "Merchant",
            Background::Cultist => "Cultist",
        }
    }

    pub fn stat_bonus(&self) -> StatBlock {
        match self {
            Background::Scholar => StatBlock {
                mana: 15,
                entropy: 10,
                ..StatBlock::zero()
            },
            Background::Wanderer => StatBlock {
                luck: 15,
                precision: 10,
                ..StatBlock::zero()
            },
            Background::Gladiator => StatBlock {
                force: 15,
                vitality: 10,
                ..StatBlock::zero()
            },
            Background::Outcast => StatBlock {
                cunning: 15,
                entropy: 10,
                ..StatBlock::zero()
            },
            Background::Merchant => StatBlock {
                luck: 10,
                cunning: 15,
                ..StatBlock::zero()
            },
            Background::Cultist => StatBlock {
                mana: 20,
                entropy: 20,
                vitality: -10,
                ..StatBlock::zero()
            },
        }
    }
}

// ─── STATS ───────────────────────────────────────────────────────────────────

/// The 7 unbounded stats of CHAOS RPG.
/// Values can theoretically grow without limit (wrapping at ±100,000 in display).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatBlock {
    pub vitality: i64,  // HP multiplier, resistance
    pub force: i64,     // Physical damage, carry weight
    pub mana: i64,      // Spell power, magic resist
    pub cunning: i64,   // Critical chance, trap detection
    pub precision: i64, // Accuracy, ranged damage
    pub entropy: i64,   // Chaos bonus to all rolls
    pub luck: i64,      // General fortune modifier
}

impl StatBlock {
    pub fn zero() -> Self {
        StatBlock {
            vitality: 0,
            force: 0,
            mana: 0,
            cunning: 0,
            precision: 0,
            entropy: 0,
            luck: 0,
        }
    }

    pub fn add(&self, other: &StatBlock) -> StatBlock {
        StatBlock {
            vitality: self.vitality + other.vitality,
            force: self.force + other.force,
            mana: self.mana + other.mana,
            cunning: self.cunning + other.cunning,
            precision: self.precision + other.precision,
            entropy: self.entropy + other.entropy,
            luck: self.luck + other.luck,
        }
    }

    pub fn total(&self) -> i64 {
        self.vitality
            + self.force
            + self.mana
            + self.cunning
            + self.precision
            + self.entropy
            + self.luck
    }

    pub fn power_level(&self) -> PowerTier {
        let total = self.total();
        match total {
            i64::MIN..=99 => PowerTier::Mortal,
            100..=299 => PowerTier::Awakened,
            300..=599 => PowerTier::Champion,
            600..=999 => PowerTier::Legendary,
            1000..=2999 => PowerTier::Transcendent,
            _ => PowerTier::Godlike,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerTier {
    Mortal,
    Awakened,
    Champion,
    Legendary,
    Transcendent,
    Godlike,
}

impl PowerTier {
    pub fn name(&self) -> &'static str {
        match self {
            PowerTier::Mortal => "Mortal",
            PowerTier::Awakened => "Awakened",
            PowerTier::Champion => "Champion",
            PowerTier::Legendary => "Legendary",
            PowerTier::Transcendent => "Transcendent",
            PowerTier::Godlike => "GODLIKE",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            PowerTier::Mortal => "\x1b[37m",       // white
            PowerTier::Awakened => "\x1b[32m",     // green
            PowerTier::Champion => "\x1b[36m",     // cyan
            PowerTier::Legendary => "\x1b[33m",    // yellow
            PowerTier::Transcendent => "\x1b[35m", // magenta
            PowerTier::Godlike => "\x1b[91m",      // bright red
        }
    }
}

// ─── CHARACTER ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub class: CharacterClass,
    pub background: Background,
    pub stats: StatBlock,
    pub max_hp: i64,
    pub current_hp: i64,
    pub level: u32,
    pub xp: u64,
    pub floor: u32,
    pub gold: i64,
    pub kills: u32,
    pub seed: u64,
}

impl Character {
    /// Roll a new character with destiny (all 10 engines)
    pub fn roll_new(
        name: String,
        class: CharacterClass,
        background: Background,
        seed: u64,
    ) -> Self {
        let weights = class.stat_weights();
        let bg_bonus = background.stat_bonus();

        // Each stat gets a chaos roll influenced by its class weight
        let roll_with_weight = |weight: i64, stat_seed: u64| -> i64 {
            let base = roll_stat(weight / 2, weight + weight / 3, stat_seed);
            base + destiny_roll(stat_seed as f64 * 1e-12, stat_seed).to_range(0, weight / 4)
        };

        let stats = StatBlock {
            vitality: roll_with_weight(weights.vitality, seed.wrapping_add(1)),
            force: roll_with_weight(weights.force, seed.wrapping_add(2)),
            mana: roll_with_weight(weights.mana, seed.wrapping_add(3)),
            cunning: roll_with_weight(weights.cunning, seed.wrapping_add(4)),
            precision: roll_with_weight(weights.precision, seed.wrapping_add(5)),
            entropy: roll_with_weight(weights.entropy, seed.wrapping_add(6)),
            luck: roll_with_weight(weights.luck, seed.wrapping_add(7)),
        };

        let stats = stats.add(&bg_bonus);
        let max_hp = 50 + stats.vitality * 3 + stats.force;

        Character {
            name,
            class,
            background,
            max_hp,
            current_hp: max_hp,
            stats,
            level: 1,
            xp: 0,
            floor: 1,
            gold: roll_stat(5, 30, seed.wrapping_add(999)),
            kills: 0,
            seed,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    pub fn hp_percent(&self) -> f64 {
        (self.current_hp as f64 / self.max_hp as f64).clamp(0.0, 1.0)
    }

    pub fn power_tier(&self) -> PowerTier {
        self.stats.power_level()
    }

    pub fn take_damage(&mut self, amount: i64) {
        self.current_hp = (self.current_hp - amount).max(0);
    }

    pub fn heal(&mut self, amount: i64) {
        self.current_hp = (self.current_hp + amount).min(self.max_hp);
    }

    pub fn gain_xp(&mut self, xp: u64) {
        self.xp += xp;
        let xp_needed = (self.level as u64 * 100) * (self.level as u64 + 1) / 2;
        if self.xp >= xp_needed {
            self.level_up();
        }
    }

    fn level_up(&mut self) {
        self.level += 1;
        let seed = self.seed.wrapping_add(self.level as u64 * 31337);
        let roll = chaos_roll_verbose(self.level as f64 * 0.1, seed);

        // Stats grow based on class weights, amplified by chaos roll
        let weights = self.class.stat_weights();
        let chaos_mult = (roll.final_value + 1.5).max(0.5);

        self.stats.vitality += (weights.vitality / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.force += (weights.force / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.mana += (weights.mana / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.cunning += (weights.cunning / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.precision += (weights.precision / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.entropy += (weights.entropy / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.luck += (weights.luck / 20 + 1) * chaos_mult as i64 + 1;

        // HP scales with vitality
        let old_max = self.max_hp;
        self.max_hp = 50 + self.stats.vitality * 3 + self.stats.force;
        self.current_hp += self.max_hp - old_max;
        self.current_hp = self.current_hp.min(self.max_hp);
    }

    pub fn score(&self) -> u64 {
        let stat_total = self.stats.total() as u64;
        let floor_bonus = self.floor as u64 * 100;
        let level_bonus = self.level as u64 * 50;
        let kill_bonus = self.kills as u64 * 10;
        stat_total + floor_bonus + level_bonus + kill_bonus + self.gold as u64
    }

    pub fn hp_bar(&self, width: usize) -> String {
        let filled = ((self.hp_percent() * width as f64) as usize).min(width);
        let bar = "█".repeat(filled) + &"░".repeat(width - filled);
        format!("[{}] {}/{}", bar, self.current_hp, self.max_hp)
    }
}

// ─── STAT DISPLAY ────────────────────────────────────────────────────────────

pub fn stat_color(value: i64) -> &'static str {
    match value {
        i64::MIN..=29 => "\x1b[31m", // red
        30..=59 => "\x1b[33m",       // yellow
        60..=89 => "\x1b[32m",       // green
        90..=149 => "\x1b[36m",      // cyan
        _ => "\x1b[35m",             // magenta (godlike)
    }
}

pub fn display_stat(name: &str, value: i64) -> String {
    let color = stat_color(value);
    let reset = "\x1b[0m";
    let bar_len = (value.clamp(0, 100) as usize / 5).min(20);
    let bar = "▓".repeat(bar_len) + &"░".repeat(20 - bar_len);
    format!("  {:12} {}{:>6}{} [{}]", name, color, value, reset, bar)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_creation_produces_valid_stats() {
        let c = Character::roll_new(
            "TestHero".to_string(),
            CharacterClass::Mage,
            Background::Scholar,
            42,
        );
        assert!(c.max_hp > 0);
        assert!(c.current_hp > 0);
        assert_eq!(c.level, 1);
        assert!(
            c.stats.mana > c.stats.force,
            "Mage should have mana > force"
        );
    }

    #[test]
    fn berserker_has_high_vitality() {
        let c = Character::roll_new(
            "Rage".to_string(),
            CharacterClass::Berserker,
            Background::Gladiator,
            12345,
        );
        assert!(c.stats.vitality > 40);
    }

    #[test]
    fn character_takes_damage_correctly() {
        let mut c = Character::roll_new(
            "X".to_string(),
            CharacterClass::Thief,
            Background::Outcast,
            1,
        );
        let initial_hp = c.current_hp;
        c.take_damage(10);
        assert_eq!(c.current_hp, initial_hp - 10);
    }

    #[test]
    fn hp_cannot_go_below_zero() {
        let mut c = Character::roll_new(
            "X".to_string(),
            CharacterClass::Ranger,
            Background::Wanderer,
            2,
        );
        c.take_damage(1_000_000);
        assert_eq!(c.current_hp, 0);
        assert!(!c.is_alive());
    }
}
