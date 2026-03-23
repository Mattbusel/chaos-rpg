//! Characters, classes, stats, and leveling.
//!
//! CHAOS RPG — now with 8 classes, each with unique passives and playstyles.
//! Stats can exceed any limit — there is no cap. The universe is your cap.

use crate::body::{Body, BodyPart};
use crate::chaos_pipeline::{chaos_roll_verbose, destiny_roll, roll_stat};
use crate::misery_system::MiseryState;
use crate::run_stats::RunStats;
use serde::{Deserialize, Serialize};

// ─── CLASSES ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterClass {
    // Original four
    Mage,
    Berserker,
    Ranger,
    Thief,
    // New four
    Necromancer,
    Alchemist,
    Paladin,
    VoidWalker,
    // Extended roster
    Warlord,
    Trickster,
    Runesmith,
    Chronomancer,
}

impl CharacterClass {
    pub fn name(&self) -> &'static str {
        match self {
            CharacterClass::Mage => "Mage",
            CharacterClass::Berserker => "Berserker",
            CharacterClass::Ranger => "Ranger",
            CharacterClass::Thief => "Thief",
            CharacterClass::Necromancer => "Necromancer",
            CharacterClass::Alchemist => "Alchemist",
            CharacterClass::Paladin => "Paladin",
            CharacterClass::VoidWalker  => "VoidWalker",
            CharacterClass::Warlord     => "Warlord",
            CharacterClass::Trickster   => "Trickster",
            CharacterClass::Runesmith   => "Runesmith",
            CharacterClass::Chronomancer => "Chronomancer",
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
            CharacterClass::Necromancer => {
                "Death is not an end — it's a variable. Drains life on kill. Max ENTROPY and MANA."
            }
            CharacterClass::Alchemist => {
                "The logistic map of chemical chaos. Items grant +50% effect. High CUNNING and MANA."
            }
            CharacterClass::Paladin => {
                "A divine constant in a chaotic universe. Regenerates HP each round. High VIT+FORCE."
            }
            CharacterClass::VoidWalker => {
                "Exists between the Mandelbrot boundary and everywhere else. Phase-dodges attacks."
            }
            CharacterClass::Warlord => {
                "Commands through raw authority. High FORCE+VIT. War Cry boosts every stat for 3 rounds."
            }
            CharacterClass::Trickster => {
                "Fights through misdirection. Illusion strikes CUNNING+LUCK. Redirects 20% of attacks."
            }
            CharacterClass::Runesmith => {
                "Inscribes chaos equations into weapons. ENTROPY+FORCE. Each inscription stacks +10% dmg."
            }
            CharacterClass::Chronomancer => {
                "Warps the action sequence. MANA+ENTROPY. 15% chance to act twice per combat round."
            }
        }
    }

    pub fn ascii_art(&self) -> &'static str {
        match self {
            CharacterClass::Mage => "   /\\\n  (o)\n   ||",
            CharacterClass::Berserker => "  ><\n[RAGE]\n  \\/",
            CharacterClass::Ranger => "  />\\  \n  ||| \n  vvv",
            CharacterClass::Thief => "  .~~.\n  {~}\n  /|\\",
            CharacterClass::Necromancer => "  ___\n (x_x)\n  |||",
            CharacterClass::Alchemist => "  ___\n [~~~]\n  |||",
            CharacterClass::Paladin => "  [+]\n  |||\n  |/|",
            CharacterClass::VoidWalker   => "  ...\n (~_~)\n  ~~~",
            CharacterClass::Warlord      => " [WAR]\n  | |\n  |_|",
            CharacterClass::Trickster    => "  {?}\n /||\\\n  ||",
            CharacterClass::Runesmith    => "  ᚱᚢᚾ\n [###]\n  |||",
            CharacterClass::Chronomancer => "  ⌚\n (∞)\n  |||",
        }
    }

    pub fn passive_name(&self) -> &'static str {
        match self {
            CharacterClass::Mage => "Arcane Overflow",
            CharacterClass::Berserker => "Blood Frenzy",
            CharacterClass::Ranger => "Prime Sight",
            CharacterClass::Thief => "Chaos Dodge",
            CharacterClass::Necromancer => "Death Drain",
            CharacterClass::Alchemist => "Transmutation",
            CharacterClass::Paladin => "Divine Regen",
            CharacterClass::VoidWalker   => "Phase Shift",
            CharacterClass::Warlord      => "War Cry",
            CharacterClass::Trickster    => "Misdirection",
            CharacterClass::Runesmith    => "Runic Etch",
            CharacterClass::Chronomancer => "Time Dilation",
        }
    }

    pub fn passive_desc(&self) -> &'static str {
        match self {
            CharacterClass::Mage => "Critical spells deal ENTROPY/10 bonus damage",
            CharacterClass::Berserker => "Below 30% HP: +40% damage, attack twice on crit",
            CharacterClass::Ranger => "PRECISION/20 bonus accuracy on every attack",
            CharacterClass::Thief => "CUNNING/200 + 10% chance to dodge incoming hits",
            CharacterClass::Necromancer => "On kill: absorb 8% of enemy max HP as your own",
            CharacterClass::Alchemist => "Items and potions grant 50% more effect",
            CharacterClass::Paladin => "Regenerate (3 + VIT/20) HP at start of each round",
            CharacterClass::VoidWalker   => "15% + LCK/400 chance to phase-dodge any attack",
            CharacterClass::Warlord      => "Every 5 kills: +3 FORCE and +3 VIT permanently this run",
            CharacterClass::Trickster    => "20% + CUN/300 chance to redirect incoming hit to enemy",
            CharacterClass::Runesmith    => "On kill: etch +10% weapon damage (stacks, max 10 times)",
            CharacterClass::Chronomancer => "15% + ENT/400 chance to take an extra action each round",
        }
    }

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
            CharacterClass::Necromancer => StatBlock {
                vitality: 30,
                force: 25,
                mana: 85,
                cunning: 55,
                precision: 40,
                entropy: 95,
                luck: 50,
            },
            CharacterClass::Alchemist => StatBlock {
                vitality: 50,
                force: 45,
                mana: 70,
                cunning: 85,
                precision: 75,
                entropy: 60,
                luck: 65,
            },
            CharacterClass::Paladin => StatBlock {
                vitality: 85,
                force: 80,
                mana: 55,
                cunning: 40,
                precision: 50,
                entropy: 35,
                luck: 55,
            },
            CharacterClass::VoidWalker => StatBlock {
                vitality: 35,
                force: 45,
                mana: 65,
                cunning: 80,
                precision: 85,
                entropy: 90,
                luck: 90,
            },
            CharacterClass::Warlord => StatBlock {
                vitality: 80,
                force: 85,
                mana: 40,
                cunning: 65,
                precision: 60,
                entropy: 30,
                luck: 45,
            },
            CharacterClass::Trickster => StatBlock {
                vitality: 45,
                force: 35,
                mana: 60,
                cunning: 85,
                precision: 80,
                entropy: 65,
                luck: 90,
            },
            CharacterClass::Runesmith => StatBlock {
                vitality: 55,
                force: 70,
                mana: 65,
                cunning: 50,
                precision: 60,
                entropy: 80,
                luck: 40,
            },
            CharacterClass::Chronomancer => StatBlock {
                vitality: 40,
                force: 30,
                mana: 90,
                cunning: 60,
                precision: 70,
                entropy: 85,
                luck: 65,
            },
        }
    }
}

impl std::fmt::Display for CharacterClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ─── BACKGROUNDS ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Background {
    Scholar,   // +MANA +ENTROPY
    Wanderer,  // +LUCK +PRECISION
    Gladiator, // +FORCE +VITALITY
    Outcast,   // +CUNNING +ENTROPY
    Merchant,  // +LUCK +CUNNING
    Cultist,   // +MANA +ENTROPY (extreme, -VIT)
    Exile,     // +CUNNING +ENTROPY, -MANA
    Oracle,    // +LUCK +MANA, -FORCE
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
            Background::Exile => "Exile",
            Background::Oracle => "Oracle",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Background::Scholar => "+15 MANA, +10 ENTROPY",
            Background::Wanderer => "+15 LUCK, +10 PRECISION",
            Background::Gladiator => "+15 FORCE, +10 VITALITY",
            Background::Outcast => "+15 CUNNING, +10 ENTROPY",
            Background::Merchant => "+15 CUNNING, +10 LUCK",
            Background::Cultist => "+20 MANA, +20 ENTROPY, -10 VITALITY",
            Background::Exile => "+20 CUNNING, +10 ENTROPY, -10 MANA",
            Background::Oracle => "+20 LUCK, +10 MANA, -15 FORCE",
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
            Background::Exile => StatBlock {
                cunning: 20,
                entropy: 10,
                mana: -10,
                ..StatBlock::zero()
            },
            Background::Oracle => StatBlock {
                luck: 20,
                mana: 10,
                force: -15,
                ..StatBlock::zero()
            },
        }
    }
}

// ─── STATS ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatBlock {
    pub vitality: i64,
    pub force: i64,
    pub mana: i64,
    pub cunning: i64,
    pub precision: i64,
    pub entropy: i64,
    pub luck: i64,
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
        PowerTier::from_total(self.total())
    }
}

// PowerTier is now defined in power_tier.rs — re-export for backwards compatibility
pub use crate::power_tier::PowerTier;

// ─── STATUS EFFECTS ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusEffect {
    // ── Original 11 ───────────────────────────────────────────────────────────
    Burning(u32),
    Poisoned(u32),
    Stunned(u32),
    Cursed(u32),
    Blessed(u32),
    Shielded(i64),
    Enraged(u32),
    Frozen(u32),
    Regenerating(u32),
    Phasing(u32),
    Empowered(u32),
    // ── Deep Ailments ─────────────────────────────────────────────────────────
    /// A portion of chaos rolls use only 1 engine instead of 4-10. Extremely volatile.
    Fracture(u32),
    /// Output of roll N is added to input of roll N+1. Good streaks beget good; bad cascades bad.
    Resonance(u32),
    /// All rolls use the same engine seed for N turns. Locked into whatever the first roll was.
    PhaseLock(u32),
    /// Enemy chaos rolls use YOUR stat biases instead of theirs.
    DimensionalBleed(u32),
    /// Each engine in the chain runs twice, feeding back into itself. Doubles chaos depth.
    Recursive(u32),
    /// All chaos rolls return exactly 0.0 for N turns. Mathematical silence. Base stats only.
    Nullified(u32),
}

impl StatusEffect {
    pub fn name(&self) -> &'static str {
        match self {
            StatusEffect::Burning(_) => "BURNING",
            StatusEffect::Poisoned(_) => "POISONED",
            StatusEffect::Stunned(_) => "STUNNED",
            StatusEffect::Cursed(_) => "CURSED",
            StatusEffect::Blessed(_) => "BLESSED",
            StatusEffect::Shielded(_) => "SHIELDED",
            StatusEffect::Enraged(_) => "ENRAGED",
            StatusEffect::Frozen(_) => "FROZEN",
            StatusEffect::Regenerating(_) => "REGEN",
            StatusEffect::Phasing(_) => "PHASING",
            StatusEffect::Empowered(_) => "EMPOWERED",
            StatusEffect::Fracture(_) => "FRACTURE",
            StatusEffect::Resonance(_) => "RESONANCE",
            StatusEffect::PhaseLock(_) => "PHASE LOCK",
            StatusEffect::DimensionalBleed(_) => "DIM.BLEED",
            StatusEffect::Recursive(_) => "RECURSIVE",
            StatusEffect::Nullified(_) => "NULLIFIED",
        }
    }

    pub fn badge(&self) -> &'static str {
        match self {
            StatusEffect::Burning(_) => "[FIRE]",
            StatusEffect::Poisoned(_) => "[PSN]",
            StatusEffect::Stunned(_) => "[STN]",
            StatusEffect::Cursed(_) => "[CRS]",
            StatusEffect::Blessed(_) => "[BLS]",
            StatusEffect::Shielded(_) => "[SHD]",
            StatusEffect::Enraged(_) => "[RAG]",
            StatusEffect::Frozen(_) => "[FRZ]",
            StatusEffect::Regenerating(_) => "[REG]",
            StatusEffect::Phasing(_) => "[PHS]",
            StatusEffect::Empowered(_) => "[EMP]",
            StatusEffect::Fracture(_) => "[FRC]",
            StatusEffect::Resonance(_) => "[RES]",
            StatusEffect::PhaseLock(_) => "[PLK]",
            StatusEffect::DimensionalBleed(_) => "[DLB]",
            StatusEffect::Recursive(_) => "[RCV]",
            StatusEffect::Nullified(_) => "[NUL]",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            StatusEffect::Burning(_) => "\x1b[91m",
            StatusEffect::Poisoned(_) => "\x1b[32m",
            StatusEffect::Stunned(_) => "\x1b[36m",
            StatusEffect::Cursed(_) => "\x1b[35m",
            StatusEffect::Blessed(_) => "\x1b[33m",
            StatusEffect::Shielded(_) => "\x1b[34m",
            StatusEffect::Enraged(_) => "\x1b[31m",
            StatusEffect::Frozen(_) => "\x1b[94m",
            StatusEffect::Regenerating(_) => "\x1b[92m",
            StatusEffect::Phasing(_) => "\x1b[95m",
            StatusEffect::Empowered(_) => "\x1b[93m",
            StatusEffect::Fracture(_) => "\x1b[96m",
            StatusEffect::Resonance(_) => "\x1b[93m",
            StatusEffect::PhaseLock(_) => "\x1b[97m",
            StatusEffect::DimensionalBleed(_) => "\x1b[35m",
            StatusEffect::Recursive(_) => "\x1b[91m",
            StatusEffect::Nullified(_) => "\x1b[90m",
        }
    }

    pub fn describe(&self) -> &'static str {
        match self {
            StatusEffect::Fracture(_) => "Some rolls use only 1 engine (extreme volatility)",
            StatusEffect::Resonance(_) => "Roll output feeds into next roll input (streak effects)",
            StatusEffect::PhaseLock(_) => {
                "All rolls use same seed (good = invincible, bad = stuck)"
            }
            StatusEffect::DimensionalBleed(_) => "Enemy rolls use YOUR stat biases",
            StatusEffect::Recursive(_) => "Each engine runs twice -- chaos depth doubled",
            StatusEffect::Nullified(_) => "All rolls return 0.0 -- base stats only, no crits",
            _ => "",
        }
    }

    pub fn tick_damage(&self) -> i64 {
        match self {
            StatusEffect::Burning(_) => 8,
            StatusEffect::Poisoned(_) => 3,
            _ => 0,
        }
    }

    pub fn tick_heal(&self, vit: i64) -> i64 {
        match self {
            StatusEffect::Regenerating(_) => (3 + vit / 20).max(1),
            _ => 0,
        }
    }

    pub fn tick(&mut self) -> bool {
        match self {
            StatusEffect::Burning(n)
            | StatusEffect::Poisoned(n)
            | StatusEffect::Stunned(n)
            | StatusEffect::Cursed(n)
            | StatusEffect::Blessed(n)
            | StatusEffect::Enraged(n)
            | StatusEffect::Frozen(n)
            | StatusEffect::Regenerating(n)
            | StatusEffect::Phasing(n)
            | StatusEffect::Empowered(n)
            | StatusEffect::Fracture(n)
            | StatusEffect::Resonance(n)
            | StatusEffect::PhaseLock(n)
            | StatusEffect::DimensionalBleed(n)
            | StatusEffect::Recursive(n)
            | StatusEffect::Nullified(n) => {
                if *n == 0 {
                    return true;
                }
                *n -= 1;
                *n == 0
            }
            StatusEffect::Shielded(hp) => *hp <= 0,
        }
    }
}

// ─── DIFFICULTY ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Normal,
    Brutal,
    Chaos,
}

impl Difficulty {
    pub fn name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Brutal => "Brutal",
            Difficulty::Chaos => "CHAOS",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Enemies deal 70% damage. Extra gold. Score x1.",
            Difficulty::Normal => "Standard experience. The math is balanced. Score x2.",
            Difficulty::Brutal => "Enemies deal 140% damage. Less gold. Score x4.",
            Difficulty::Chaos => "200% damage. All random effects maximized. Score x10.",
        }
    }

    pub fn enemy_damage_mult(&self) -> i64 {
        match self {
            Difficulty::Easy => 70,
            Difficulty::Normal => 100,
            Difficulty::Brutal => 140,
            Difficulty::Chaos => 200,
        }
    }

    pub fn gold_mult(&self) -> i64 {
        match self {
            Difficulty::Easy => 130,
            Difficulty::Normal => 100,
            Difficulty::Brutal => 75,
            Difficulty::Chaos => 50,
        }
    }

    pub fn xp_mult(&self) -> i64 {
        match self {
            Difficulty::Easy => 80,
            Difficulty::Normal => 100,
            Difficulty::Brutal => 120,
            Difficulty::Chaos => 200,
        }
    }

    pub fn score_mult(&self) -> u64 {
        match self {
            Difficulty::Easy => 1,
            Difficulty::Normal => 2,
            Difficulty::Brutal => 4,
            Difficulty::Chaos => 10,
        }
    }
}

// ─── COLOR THEME ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorTheme {
    Classic,
    Neon,
    Blood,
    Void,
    Monochrome,
}

impl ColorTheme {
    pub fn name(&self) -> &'static str {
        match self {
            ColorTheme::Classic => "Classic",
            ColorTheme::Neon => "Neon",
            ColorTheme::Blood => "Blood",
            ColorTheme::Void => "Void",
            ColorTheme::Monochrome => "Monochrome",
        }
    }

    pub fn primary(&self) -> &'static str {
        match self {
            ColorTheme::Classic => "\x1b[36m",
            ColorTheme::Neon => "\x1b[96m",
            ColorTheme::Blood => "\x1b[91m",
            ColorTheme::Void => "\x1b[95m",
            ColorTheme::Monochrome => "\x1b[97m",
        }
    }

    pub fn danger(&self) -> &'static str {
        match self {
            ColorTheme::Classic => "\x1b[31m",
            ColorTheme::Neon => "\x1b[91m",
            ColorTheme::Blood => "\x1b[31m",
            ColorTheme::Void => "\x1b[35m",
            ColorTheme::Monochrome => "\x1b[37m",
        }
    }

    pub fn success(&self) -> &'static str {
        match self {
            ColorTheme::Classic => "\x1b[32m",
            ColorTheme::Neon => "\x1b[92m",
            ColorTheme::Blood => "\x1b[33m",
            ColorTheme::Void => "\x1b[94m",
            ColorTheme::Monochrome => "\x1b[97m",
        }
    }

    pub fn warning(&self) -> &'static str {
        match self {
            ColorTheme::Classic => "\x1b[33m",
            ColorTheme::Neon => "\x1b[93m",
            ColorTheme::Blood => "\x1b[91m",
            ColorTheme::Void => "\x1b[93m",
            ColorTheme::Monochrome => "\x1b[37m",
        }
    }

    pub fn magic(&self) -> &'static str {
        match self {
            ColorTheme::Classic => "\x1b[35m",
            ColorTheme::Neon => "\x1b[95m",
            ColorTheme::Blood => "\x1b[35m",
            ColorTheme::Void => "\x1b[95m",
            ColorTheme::Monochrome => "\x1b[97m",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            ColorTheme::Classic => "\x1b[31m",
            ColorTheme::Neon => "\x1b[96m",
            ColorTheme::Blood => "\x1b[91m",
            ColorTheme::Void => "\x1b[35m",
            ColorTheme::Monochrome => "\x1b[97m",
        }
    }
}

// ─── BOONS ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Boon {
    BloodPact,       // +50 max HP, take 2 HP per room
    ChaosBlessing,   // luck +10, chaos rolls more favourable
    GoldVein,        // start with 200 gold
    ScholarGift,     // 3 extra starting spells
    WarriorBlessing, // +20 Force, +15 Vitality
    LuckyBirth,      // +30 Luck
    EntropicSoul,    // 2x Entropy+Mana, 0.5x Vitality
    CrystalSkin,     // start with 80 HP shield
    MathSavant,      // all spell damage ×1.75
    VoidTouched,     // all stats ×1.5
    PrimeBlood,      // each kill: +1 to highest stat
    ShadowStart,     // 50% HP, 3x XP
}

impl Boon {
    pub fn name(self) -> &'static str {
        match self {
            Boon::BloodPact => "Blood Pact",
            Boon::ChaosBlessing => "Chaos Blessing",
            Boon::GoldVein => "Gold Vein",
            Boon::ScholarGift => "Scholar's Gift",
            Boon::WarriorBlessing => "Warrior's Blessing",
            Boon::LuckyBirth => "Lucky Birth",
            Boon::EntropicSoul => "Entropic Soul",
            Boon::CrystalSkin => "Crystal Skin",
            Boon::MathSavant => "Math Savant",
            Boon::VoidTouched => "Void Touched",
            Boon::PrimeBlood => "Prime Blood",
            Boon::ShadowStart => "Shadow Start",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Boon::BloodPact => "+50 max HP. Take 2 HP damage entering each room.",
            Boon::ChaosBlessing => "+10 Luck. Chaos rolls biased in your favor.",
            Boon::GoldVein => "Start with 200 gold.",
            Boon::ScholarGift => "Start with 3 extra chaos-generated spells.",
            Boon::WarriorBlessing => "+20 Force, +15 Vitality.",
            Boon::LuckyBirth => "+30 Luck.",
            Boon::EntropicSoul => "2× Entropy+Mana, half Vitality.",
            Boon::CrystalSkin => "Start with an 80 HP shield.",
            Boon::MathSavant => "All spell damage ×1.75.",
            Boon::VoidTouched => "All stats ×1.5.",
            Boon::PrimeBlood => "Each kill: +1 to your highest stat.",
            Boon::ShadowStart => "Start at 50% HP. All XP ×3.",
        }
    }

    pub fn color_code(self) -> &'static str {
        match self {
            Boon::BloodPact | Boon::ShadowStart => "\x1b[31m",
            Boon::ChaosBlessing | Boon::EntropicSoul | Boon::VoidTouched => "\x1b[35m",
            Boon::GoldVein | Boon::LuckyBirth => "\x1b[33m",
            Boon::ScholarGift | Boon::MathSavant => "\x1b[36m",
            Boon::WarriorBlessing | Boon::PrimeBlood => "\x1b[32m",
            Boon::CrystalSkin => "\x1b[34m",
        }
    }

    pub fn random_three(seed: u64) -> [Boon; 3] {
        use Boon::*;
        const ALL: [Boon; 12] = [
            BloodPact,
            ChaosBlessing,
            GoldVein,
            ScholarGift,
            WarriorBlessing,
            LuckyBirth,
            EntropicSoul,
            CrystalSkin,
            MathSavant,
            VoidTouched,
            PrimeBlood,
            ShadowStart,
        ];
        let a = (seed % 12) as usize;
        let b = ((seed.wrapping_mul(31337)) % 12) as usize;
        let b = if b == a { (b + 1) % 12 } else { b };
        let c = ((seed.wrapping_mul(99991)) % 12) as usize;
        let c = if c == a || c == b { (c + 2) % 12 } else { c };
        [ALL[a], ALL[b], ALL[c]]
    }
}

// ─── CHARACTER ────────────────────────────────────────────────────────────────

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
    pub inventory: Vec<crate::items::Item>,
    pub known_spells: Vec<crate::spells::Spell>,
    pub status_effects: Vec<StatusEffect>,
    pub bonus_points_spent: u32,
    pub total_damage_dealt: i64,
    pub total_damage_taken: i64,
    pub spells_cast: u32,
    pub items_used: u32,
    pub rooms_cleared: u32,
    pub difficulty: Difficulty,
    // Boon system
    pub boon: Option<Boon>,
    pub spell_damage_mult: f64,
    pub xp_mult: f64,
    // Body system
    pub body: Body,
    // Skill points for passive tree
    pub skill_points: u32,
    pub allocated_nodes: Vec<u32>,
    // Faction reputation
    pub faction_rep: crate::factions::FactionRep,
    // Chaos Corruption — every kill adds a stack; every 50 mutates engines permanently
    pub corruption: u32,
    // The Hunger — rooms without a kill (floor 50+: every 5 → lose 5% max HP)
    pub rooms_without_kill: u32,
    // Misery / Spite / Defiance system (negative-tier runs)
    #[serde(default)]
    pub misery: MiseryState,
    // Per-run statistics tracker
    #[serde(default)]
    pub run_stats: RunStats,
}

impl Character {
    pub fn roll_new(
        name: String,
        class: CharacterClass,
        background: Background,
        seed: u64,
        difficulty: Difficulty,
    ) -> Self {
        let weights = class.stat_weights();
        let bg_bonus = background.stat_bonus();

        let roll_with_weight = |weight: i64, stat_seed: u64| -> i64 {
            let destiny = destiny_roll(stat_seed as f64 * 1e-12, stat_seed);
            let chaos_mult = 1.0 + destiny.final_value * 3.0;
            let base = (weight as f64 * chaos_mult) as i64;
            base + roll_stat(
                -(weight / 5 + 1),
                weight / 5 + 1,
                stat_seed.wrapping_add(77),
            )
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
        let max_hp = (50 + stats.vitality * 3 + stats.force).max(1);
        let body = Body::generate(max_hp, seed.wrapping_add(55555));

        let known_spells = match class {
            CharacterClass::Mage => vec![
                crate::spells::Spell::generate(seed.wrapping_add(10001)),
                crate::spells::Spell::generate(seed.wrapping_add(10002)),
                crate::spells::Spell::generate(seed.wrapping_add(10003)),
            ],
            CharacterClass::Necromancer => vec![
                crate::spells::Spell::generate(seed.wrapping_add(20001)),
                crate::spells::Spell::generate(seed.wrapping_add(20002)),
            ],
            CharacterClass::Alchemist => {
                vec![crate::spells::Spell::generate(seed.wrapping_add(30001))]
            }
            _ => vec![crate::spells::Spell::generate(seed.wrapping_add(10001))],
        };

        let inventory = if class == CharacterClass::Alchemist {
            vec![crate::items::Item::generate(seed.wrapping_add(99991))]
        } else {
            Vec::new()
        };

        let mut status_effects: Vec<StatusEffect> = Vec::new();
        if class == CharacterClass::VoidWalker {
            status_effects.push(StatusEffect::Phasing(3));
        }

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
            inventory,
            known_spells,
            status_effects,
            bonus_points_spent: 0,
            total_damage_dealt: 0,
            total_damage_taken: 0,
            spells_cast: 0,
            items_used: 0,
            rooms_cleared: 0,
            difficulty,
            boon: None,
            spell_damage_mult: 1.0,
            xp_mult: 1.0,
            body,
            skill_points: 0,
            allocated_nodes: Vec::new(),
            faction_rep: crate::factions::FactionRep::default(),
            corruption: 0,
            rooms_without_kill: 0,
            misery: MiseryState::new(),
            run_stats: RunStats::new(),
        }
    }

    /// Apply a starting boon to this character.
    pub fn apply_boon(&mut self, boon: Boon) {
        self.boon = Some(boon);
        match boon {
            Boon::BloodPact => {
                self.max_hp += 50;
                self.current_hp = self.max_hp;
            }
            Boon::ChaosBlessing => {
                self.stats.luck += 10;
            }
            Boon::GoldVein => {
                self.gold += 200;
            }
            Boon::ScholarGift => {
                for i in 0..3u64 {
                    self.known_spells.push(crate::spells::Spell::generate(
                        self.seed.wrapping_add(88888 + i * 31337),
                    ));
                }
            }
            Boon::WarriorBlessing => {
                self.stats.force += 20;
                self.stats.vitality += 15;
                self.max_hp = (50 + self.stats.vitality * 3 + self.stats.force).max(1);
                self.current_hp = self.max_hp;
            }
            Boon::LuckyBirth => {
                self.stats.luck += 30;
            }
            Boon::EntropicSoul => {
                self.stats.entropy *= 2;
                self.stats.mana *= 2;
                self.stats.vitality /= 2;
                self.max_hp = (50 + self.stats.vitality * 3 + self.stats.force).max(1);
                self.current_hp = self.max_hp;
            }
            Boon::CrystalSkin => {
                self.add_status(StatusEffect::Shielded(80));
            }
            Boon::MathSavant => {
                self.spell_damage_mult = 1.75;
            }
            Boon::VoidTouched => {
                let mult = |v: i64| (v as f64 * 1.5) as i64;
                self.stats.vitality = mult(self.stats.vitality);
                self.stats.force = mult(self.stats.force);
                self.stats.mana = mult(self.stats.mana);
                self.stats.cunning = mult(self.stats.cunning);
                self.stats.precision = mult(self.stats.precision);
                self.stats.entropy = mult(self.stats.entropy);
                self.stats.luck = mult(self.stats.luck);
                self.max_hp = (50 + self.stats.vitality * 3 + self.stats.force).max(1);
                self.current_hp = self.max_hp;
            }
            Boon::PrimeBlood => {} // Applied on kill in combat
            Boon::ShadowStart => {
                self.current_hp = (self.max_hp / 2).max(1);
                self.xp_mult = 3.0;
            }
        }
    }

    /// PrimeBlood boon: +1 to highest stat on kill.
    pub fn prime_blood_tick(&mut self) {
        let max_val = [
            self.stats.vitality,
            self.stats.force,
            self.stats.mana,
            self.stats.cunning,
            self.stats.precision,
            self.stats.entropy,
            self.stats.luck,
        ]
        .iter()
        .copied()
        .max()
        .unwrap_or(0);
        if max_val == self.stats.vitality {
            self.stats.vitality += 1;
            self.max_hp += 3;
        } else if max_val == self.stats.force {
            self.stats.force += 1;
        } else if max_val == self.stats.mana {
            self.stats.mana += 1;
        } else if max_val == self.stats.cunning {
            self.stats.cunning += 1;
        } else if max_val == self.stats.precision {
            self.stats.precision += 1;
        } else if max_val == self.stats.entropy {
            self.stats.entropy += 1;
        } else {
            self.stats.luck += 1;
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

    /// VoidWalker passive: chance to phase-dodge an incoming attack entirely
    pub fn phase_dodge_roll(&self, attack_seed: u64) -> bool {
        if self.class != CharacterClass::VoidWalker {
            return false;
        }
        let dodge_pct = 15 + (self.stats.luck / 400).max(0) as u64;
        (attack_seed.wrapping_mul(1_000_003) % 100) < dodge_pct
    }

    pub fn take_damage(&mut self, amount: i64) {
        self.take_damage_to_part(amount, self.seed.wrapping_add(amount as u64));
    }

    /// Deal damage routed to a specific body part (determined by caller) or
    /// chaos-rolled to a random part if `part` is None.
    /// Returns (hit part, actual damage, injury).
    pub fn take_damage_to_part(
        &mut self,
        amount: i64,
        seed: u64,
    ) -> (BodyPart, i64, Option<crate::body::InjurySeverity>) {
        // 1. Absorb through shields first
        let mut remaining = amount;
        for effect in &mut self.status_effects {
            if let StatusEffect::Shielded(shield_hp) = effect {
                if *shield_hp >= remaining {
                    *shield_hp -= remaining;
                    remaining = 0;
                    break;
                } else {
                    remaining -= *shield_hp;
                    *shield_hp = 0;
                }
            }
        }
        self.status_effects
            .retain(|e| !matches!(e, StatusEffect::Shielded(0)));

        // 2. Route to body part
        let hit_part = Body::target_part(seed);
        let (body_dmg, injury) = self.body.damage_part(hit_part, remaining);

        // 3. Deduct from overall HP
        self.current_hp = (self.current_hp - remaining).max(0);
        self.total_damage_taken += remaining;

        // 4. Head death = instant kill
        if self.body.head_destroyed() {
            self.current_hp = 0;
        }

        (hit_part, body_dmg, injury)
    }

    pub fn add_status(&mut self, effect: StatusEffect) {
        self.status_effects.retain(|e| e.name() != effect.name());
        self.status_effects.push(effect);
    }

    pub fn has_status(&self, name: &str) -> bool {
        self.status_effects.iter().any(|e| e.name() == name)
    }

    pub fn tick_status_effects(&mut self) -> (i64, Vec<String>) {
        let mut net_dmg = 0i64;
        let mut msgs = Vec::new();

        let effects_copy = self.status_effects.clone();
        for effect in &effects_copy {
            let tick_dmg = effect.tick_damage();
            if tick_dmg > 0 {
                self.current_hp = (self.current_hp - tick_dmg).max(0);
                self.total_damage_taken += tick_dmg;
                net_dmg += tick_dmg;
                msgs.push(format!(
                    "{} deals {} damage! ({})",
                    effect.name(),
                    tick_dmg,
                    self.name
                ));
            }
            let tick_heal = effect.tick_heal(self.stats.vitality);
            if tick_heal > 0 {
                self.current_hp = (self.current_hp + tick_heal).min(self.max_hp);
                net_dmg -= tick_heal;
                msgs.push(format!("{} restores {} HP.", effect.name(), tick_heal));
            }
        }

        let mut expired = Vec::new();
        for effect in &mut self.status_effects {
            if effect.tick() {
                expired.push(effect.name());
            }
        }
        for name in &expired {
            msgs.push(format!("{} wore off.", name));
        }
        self.status_effects.retain(|e| !expired.contains(&e.name()));

        // Cursed body parts drain HP and stats each turn
        let cursed_drain = self.body.curse_drain_per_turn();
        if cursed_drain > 0 {
            self.current_hp = (self.current_hp - cursed_drain).max(0);
            self.total_damage_taken += cursed_drain;
            net_dmg += cursed_drain;
            msgs.push(format!("Cursed body parts drain {} HP!", cursed_drain));
        }

        (net_dmg, msgs)
    }

    pub fn add_item(&mut self, item: crate::items::Item) {
        self.inventory.push(item);
    }

    pub fn add_spell(&mut self, spell: crate::spells::Spell) {
        self.known_spells.push(spell);
    }

    /// Spend all available passive skill points automatically, prioritising
    /// nodes that match the class's primary stats.
    /// Returns one log line per allocated node.
    pub fn auto_allocate_passives(&mut self, seed: u64) -> Vec<String> {
        use crate::passive_tree::{nodes, NodeType, PlayerPassives};

        if self.skill_points == 0 {
            return vec!["No skill points to spend.".to_string()];
        }

        let mut passives = PlayerPassives {
            allocated:           self.allocated_nodes.iter().map(|&id| id as u16).collect(),
            stat_bonuses:        std::collections::HashMap::new(),
            points:              self.skill_points,
            keystones:           std::collections::HashSet::new(),
            completed_synergies: std::collections::HashSet::new(),
            cursor:              self.allocated_nodes.first().map(|&id| id as u16).unwrap_or(0),
        };

        let messages = passives.auto_allocate_all(self.class, seed);

        // Apply all stat gains to the live character
        for (&nid, &value) in &passives.stat_bonuses {
            if let Some(node) = nodes().iter().find(|n| n.id == nid) {
                match &node.node_type {
                    NodeType::Stat { stat, .. } | NodeType::Notable { stat, .. } => {
                        match *stat {
                            "vitality"  => { self.stats.vitality  += value; self.max_hp = (50 + self.stats.vitality * 3 + self.stats.force).max(1); }
                            "force"     => { self.stats.force     += value; self.max_hp = (50 + self.stats.vitality * 3 + self.stats.force).max(1); }
                            "mana"      => self.stats.mana      += value,
                            "cunning"   => self.stats.cunning   += value,
                            "precision" => self.stats.precision += value,
                            "entropy"   => self.stats.entropy   += value,
                            "luck"      => self.stats.luck      += value,
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        self.allocated_nodes = passives.allocated.into_iter().map(|id| id as u32).collect();
        self.skill_points    = passives.points;
        messages
    }

    pub fn use_item(&mut self, idx: usize) -> Option<crate::items::Item> {
        if idx < self.inventory.len() {
            self.items_used += 1;
            Some(self.inventory.remove(idx))
        } else {
            None
        }
    }

    pub fn heal(&mut self, amount: i64) {
        self.current_hp = (self.current_hp + amount).min(self.max_hp);
        // Distribute a fraction of healing to injured body parts (5% of each part's max HP)
        if amount > 0 {
            self.body.heal_all_pct(0.05);
        }
    }

    /// Heal that respects anti-heal scaling (floor 50+: -2% per floor, 0% at 100+).
    /// Use for potions, regen, spell heals, and passive regeneration.
    /// Shrines, Death Drain, and on-kill effects should use heal() directly (bypass).
    pub fn heal_scaled(&mut self, amount: i64) {
        let eff = self.heal_effectiveness_pct();
        let scaled = (amount * eff / 100).max(0);
        if scaled > 0 {
            self.heal(scaled);
        }
    }

    /// Returns healing effectiveness as a percentage (0–100).
    pub fn heal_effectiveness_pct(&self) -> i64 {
        if self.floor < 50 {
            return 100;
        }
        (100i64 - (self.floor as i64 - 50) * 2).max(0)
    }

    // ── Corruption system ─────────────────────────────────────────────────────

    /// Current corruption stage (0–8). One stage per 50 kills.
    /// Stage 0 = clean; stage 8 = maximum mutation (400+ kills).
    pub fn corruption_stage(&self) -> u32 {
        (self.kills / 50).min(8)
    }

    /// Backfire chance on normal attacks at 400+ kills (0–20%).
    pub fn corruption_backfire_pct(&self) -> u64 {
        if self.kills < 400 {
            return 0;
        }
        ((self.kills - 400) / 50 * 3 + 5).min(20) as u64
    }

    /// Corruption label for display ("Clean", "Tainted", "Corrupted", "Unraveling", "DESTABILIZED").
    pub fn corruption_label(&self) -> &'static str {
        match self.corruption_stage() {
            0 => "Clean",
            1 => "Tainted",
            2 => "Corrupted",
            3 => "Corrupted",
            4 => "Unraveling",
            5 => "Unraveling",
            6 => "DESTABILIZED",
            7 => "DESTABILIZED",
            _ => "CHAOS INCARNATE",
        }
    }

    // ── Stat helpers ──────────────────────────────────────────────────────────

    /// Returns (name, value) of the player's highest stat.
    pub fn highest_stat(&self) -> (&'static str, i64) {
        let stats: &[(&str, i64)] = &[
            ("Vitality",  self.stats.vitality),
            ("Force",     self.stats.force),
            ("Mana",      self.stats.mana),
            ("Cunning",   self.stats.cunning),
            ("Precision", self.stats.precision),
            ("Entropy",   self.stats.entropy),
            ("Luck",      self.stats.luck),
        ];
        stats.iter().max_by_key(|(_, v)| *v).copied().unwrap_or(("Vitality", 1))
    }

    /// Necromancer passive: drain HP on enemy kill
    pub fn necro_drain(&mut self, enemy_max_hp: i64) {
        if self.class == CharacterClass::Necromancer {
            let drain = (enemy_max_hp as f64 * 0.08) as i64;
            self.heal(drain.max(1));
        }
    }

    // ── Body-aware stat accessors ─────────────────────────────────────────────

    /// Effective precision including body part penalties (eye damage).
    pub fn effective_precision(&self) -> i64 {
        self.stats.precision + self.body.penalties().precision
    }

    /// Effective force including body part penalties (arm damage).
    pub fn effective_force(&self) -> i64 {
        self.stats.force + self.body.penalties().force
    }

    /// Flee luck modifier including leg/foot injury penalty.
    pub fn flee_luck_modifier(&self) -> i64 {
        self.stats.luck - (self.body.flee_penalty_pct() * 100.0) as i64
    }

    /// True entropy including body bonus from both eyes lost.
    pub fn effective_entropy(&self) -> i64 {
        self.stats.entropy + self.body.penalties().entropy
    }

    /// Total defense bonus from equipped body armor.
    pub fn body_armor_defense(&self) -> i64 {
        self.body.parts.values().map(|s| s.armor_defense).sum()
    }

    /// Alchemist passive: amplified item heal
    pub fn item_heal_bonus(&self, base: i64) -> i64 {
        if self.class == CharacterClass::Alchemist {
            base + base / 2
        } else {
            base
        }
    }

    pub fn gain_xp(&mut self, xp: u64) {
        let base = xp * self.difficulty.xp_mult() as u64 / 100;
        let scaled = (base as f64 * self.xp_mult) as u64;
        self.xp += scaled;
        let xp_needed = (self.level as u64 * 100) * (self.level as u64 + 1) / 2;
        if self.xp >= xp_needed {
            self.level_up_and_learn_spell();
        }
    }

    fn level_up(&mut self) {
        self.level += 1;
        let seed = self.seed.wrapping_add(self.level as u64 * 31337);
        let roll = chaos_roll_verbose(self.level as f64 * 0.1, seed);

        let weights = self.class.stat_weights();
        let chaos_mult = (roll.final_value + 1.5).max(0.5);

        self.stats.vitality += (weights.vitality / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.force += (weights.force / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.mana += (weights.mana / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.cunning += (weights.cunning / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.precision += (weights.precision / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.entropy += (weights.entropy / 20 + 1) * chaos_mult as i64 + 1;
        self.stats.luck += (weights.luck / 20 + 1) * chaos_mult as i64 + 1;

        let old_max = self.max_hp;
        self.max_hp = (50 + self.stats.vitality * 3 + self.stats.force).max(1);
        self.current_hp += self.max_hp - old_max;
        self.current_hp = self.current_hp.min(self.max_hp);

        // Grant chaos-rolled skill points (0–5, usually 1–2)
        let sp_seed = seed.wrapping_add(self.level as u64 * 77777);
        let sp_roll = chaos_roll_verbose(self.level as f64 * 0.05, sp_seed);
        let sp = (sp_roll.to_range(0, 5) as u32).min(5);
        self.skill_points += sp;
    }

    pub fn level_up_and_learn_spell(&mut self) {
        self.level_up();
        let spell_seed = self
            .seed
            .wrapping_add(self.level as u64 * 99991)
            .wrapping_mul(2654435761);
        self.known_spells
            .push(crate::spells::Spell::generate(spell_seed));
    }

    pub fn score(&self) -> u64 {
        let stat_total = self.stats.total().max(0) as u64;
        let floor_bonus = self.floor as u64 * 200;
        let level_bonus = self.level as u64 * 100;
        let kill_bonus = self.kills as u64 * 25;
        let room_bonus = self.rooms_cleared as u64 * 15;
        let spell_bonus = self.spells_cast as u64 * 5;
        let base = stat_total
            + floor_bonus
            + level_bonus
            + kill_bonus
            + room_bonus
            + spell_bonus
            + self.gold.max(0) as u64;
        base * self.difficulty.score_mult()
    }

    pub fn run_summary(&self) -> Vec<String> {
        vec![
            format!("  Floor reached:    {}", self.floor),
            format!("  Enemies slain:    {}", self.kills),
            format!("  Rooms cleared:    {}", self.rooms_cleared),
            format!("  Damage dealt:     {}", self.total_damage_dealt),
            format!("  Damage taken:     {}", self.total_damage_taken),
            format!("  Spells cast:      {}", self.spells_cast),
            format!("  Items used:       {}", self.items_used),
            format!("  Gold collected:   {}", self.gold),
            format!("  Final level:      {}", self.level),
            format!("  Difficulty:       {}", self.difficulty.name()),
            format!(
                "  Power tier:       {}{}\x1b[0m",
                self.power_tier().ansi_color(),
                self.power_tier().name()
            ),
        ]
    }

    pub fn hp_bar(&self, width: usize) -> String {
        let filled = ((self.hp_percent() * width as f64) as usize).min(width);
        let hp_color = if self.hp_percent() > 0.6 {
            "\x1b[32m"
        } else if self.hp_percent() > 0.3 {
            "\x1b[33m"
        } else {
            "\x1b[31m"
        };
        let reset = "\x1b[0m";
        let bar = format!(
            "{}{}{}{}",
            hp_color,
            "█".repeat(filled),
            "░".repeat(width - filled),
            reset
        );
        format!("[{}] {}/{}", bar, self.current_hp, self.max_hp)
    }

    /// Return the primary power display (label, value string) for the character sheet.
    /// Negative-tier chars with high misery show MISERY as primary metric.
    pub fn power_display(&self) -> (&'static str, String) {
        let tier = self.power_tier();
        self.misery.display_primary(self.stats.total(), tier.name())
    }

    /// Underdog XP multiplier (>1.0 only for negative stat totals).
    pub fn underdog_multiplier(&self) -> f64 {
        MiseryState::underdog_multiplier(self.stats.total())
    }

    pub fn status_badge_line(&self) -> String {
        if self.status_effects.is_empty() {
            return String::new();
        }
        self.status_effects
            .iter()
            .map(|e| format!("{}{}\x1b[0m", e.color(), e.badge()))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

// ─── STAT DISPLAY ─────────────────────────────────────────────────────────────

pub fn stat_color(value: i64) -> &'static str {
    match value {
        i64::MIN..=-1 => "\x1b[35m",
        0..=29 => "\x1b[31m",
        30..=59 => "\x1b[33m",
        60..=89 => "\x1b[32m",
        90..=149 => "\x1b[36m",
        _ => "\x1b[95m",
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

    fn make_char(class: CharacterClass, bg: Background, seed: u64) -> Character {
        Character::roll_new("Test".to_string(), class, bg, seed, Difficulty::Normal)
    }

    #[test]
    fn all_classes_create_valid_characters() {
        let classes = [
            CharacterClass::Mage,
            CharacterClass::Berserker,
            CharacterClass::Ranger,
            CharacterClass::Thief,
            CharacterClass::Necromancer,
            CharacterClass::Alchemist,
            CharacterClass::Paladin,
            CharacterClass::VoidWalker,
        ];
        for class in classes {
            let c = make_char(class, Background::Scholar, 42);
            assert!(c.max_hp >= 1);
            let _ = c.power_tier();
        }
    }

    #[test]
    fn alchemist_starts_with_item() {
        let c = make_char(CharacterClass::Alchemist, Background::Wanderer, 100);
        assert!(!c.inventory.is_empty());
    }

    #[test]
    fn voidwalker_starts_with_phasing() {
        let c = make_char(CharacterClass::VoidWalker, Background::Exile, 77);
        assert!(c.has_status("PHASING"));
    }

    #[test]
    fn necro_drain_heals_on_kill() {
        let mut c = make_char(CharacterClass::Necromancer, Background::Cultist, 1);
        c.take_damage(c.max_hp / 2);
        let hp_before = c.current_hp;
        c.necro_drain(100);
        assert!(c.current_hp >= hp_before);
    }

    #[test]
    fn character_takes_damage_correctly() {
        let mut c = make_char(CharacterClass::Thief, Background::Outcast, 1);
        let initial_hp = c.current_hp;
        c.take_damage(10);
        assert_eq!(c.current_hp, (initial_hp - 10).max(0));
    }

    #[test]
    fn difficulty_score_mult_increases() {
        assert!(Difficulty::Chaos.score_mult() > Difficulty::Easy.score_mult());
    }
}
