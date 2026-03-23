//! Path of Exile-style passive skill tree.
//!
//! 82 nodes arranged around 8 class starting positions.
//! Stat nodes: bonus amount is chaos-rolled at allocation time -- you don't know the value until you commit.
//! Engine nodes: modify how a specific chaos engine behaves for THIS character.
//! Keystone nodes: major build-defining choices at the far edge of the tree.
//! Synergy clusters: 5-6 weak nodes that together activate a powerful bonus.

use crate::character::CharacterClass;
use serde::{Deserialize, Serialize};

pub type NodeId = u8;

// ─── ENGINE MODIFIER ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineModifier {
    /// Clamp this engine's output to positive values only
    ForcePositive,
    /// Use more volatile parameters (higher ceiling, lower floor)
    Volatile,
    /// Cap iterations/steps (prevents pathological extremes)
    CapIterations(u32),
    /// Sample near the set boundary (maximum chaos)
    BoundaryMagnet,
    /// Double this engine's output contribution to the chain
    DoubleOutput,
    /// This engine is always included in YOUR chain
    AlwaysInclude,
    /// This engine is never included in YOUR chain
    AlwaysExclude,
}

impl EngineModifier {
    pub fn describe(&self) -> &'static str {
        match self {
            EngineModifier::ForcePositive => "Output always positive",
            EngineModifier::Volatile => "Higher ceiling, lower floor",
            EngineModifier::CapIterations(_) => "Iterations capped (prevents extremes)",
            EngineModifier::BoundaryMagnet => "Samples near set boundary (max chaos)",
            EngineModifier::DoubleOutput => "Output contribution doubled",
            EngineModifier::AlwaysInclude => "Always in your chain",
            EngineModifier::AlwaysExclude => "Never in your chain",
        }
    }
}

// ─── KEYSTONE ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Keystone {
    /// Never take > 50% max HP in one hit. Damage also capped at 50% enemy max HP.
    ChaosImmunity,
    /// All negative chaos rolls become positive. All positive become negative.
    EntropyInversion,
    /// Always exactly 4 engines. Same 4 every roll. No variance in chain length.
    MathematicalCertainty,
    /// HP is always 1. Damage chains use 15 engines instead of 4-10.
    GlassCannonInfinite,
    /// On kill: restore to full HP. On miss: deal 1 damage to yourself.
    BloodReckoning,
    /// Critical hits are reflected 50% of the time. Crits deal 3x if survived.
    ChaosMirror,
}

impl Keystone {
    pub fn name(self) -> &'static str {
        match self {
            Keystone::ChaosImmunity => "Chaos Immunity",
            Keystone::EntropyInversion => "Entropy Inversion",
            Keystone::MathematicalCertainty => "Mathematical Certainty",
            Keystone::GlassCannonInfinite => "Glass Cannon of the Infinite",
            Keystone::BloodReckoning => "Blood Reckoning",
            Keystone::ChaosMirror => "Chaos Mirror",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Keystone::ChaosImmunity =>
                "Never take > 50% max HP per hit. Damage also capped at 50% enemy max HP.",
            Keystone::EntropyInversion =>
                "Negative rolls become positive. Positive rolls become negative. You live in the mirror.",
            Keystone::MathematicalCertainty =>
                "All rolls use exactly 4 engines. Same 4, every time. Predictable chaos.",
            Keystone::GlassCannonInfinite =>
                "HP is always 1. Damage chains use 15 engines. The math is your only armor.",
            Keystone::BloodReckoning =>
                "Kill = restore to full HP. Miss = deal 1 damage to yourself.",
            Keystone::ChaosMirror =>
                "50% chance crits reflect back on you. Reflected crits deal 3x if you survive.",
        }
    }
}

// ─── SYNERGY CLUSTER ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SynergyCluster {
    FractalMastery,    // Mandelbrot + Fibonacci outputs doubled
    PrimeConspiracy,   // Prime Density gains bonus for primes in floor number
    LorenzAmplifier,   // Lorenz chain gains 2 extra iterations
    ZetaResonance,     // Riemann Zeta uses s values closer to 1 (volatile)
    CollatzShortcut,   // Collatz chains capped at 50 steps
    EntropicHarmony,   // All engine outputs averaged with 20% bonus
}

impl SynergyCluster {
    pub fn name(self) -> &'static str {
        match self {
            SynergyCluster::FractalMastery => "Fractal Mastery",
            SynergyCluster::PrimeConspiracy => "Prime Conspiracy",
            SynergyCluster::LorenzAmplifier => "Lorenz Amplifier",
            SynergyCluster::ZetaResonance => "Zeta Resonance",
            SynergyCluster::CollatzShortcut => "Collatz Shortcut",
            SynergyCluster::EntropicHarmony => "Entropic Harmony",
        }
    }

    pub fn bonus_description(self) -> &'static str {
        match self {
            SynergyCluster::FractalMastery =>
                "All 5 allocated: Mandelbrot + Fibonacci engine outputs doubled",
            SynergyCluster::PrimeConspiracy =>
                "All 5 allocated: Prime Density gets +0.3 bonus on prime-numbered floors",
            SynergyCluster::LorenzAmplifier =>
                "All 5 allocated: Lorenz attractor runs 2 extra simulation steps",
            SynergyCluster::ZetaResonance =>
                "All 5 allocated: Riemann Zeta uses s=1.1 (maximum volatility near pole)",
            SynergyCluster::CollatzShortcut =>
                "All 5 allocated: Collatz chains capped at 50 steps (prevents long orbits)",
            SynergyCluster::EntropicHarmony =>
                "All 5 allocated: Final chain output gets +20% bonus",
        }
    }
}

// ─── NODE EFFECT ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum NodeEffect {
    /// Flat stat bonus; base_amount is chaos-rolled at allocation time
    StatBonus { stat: &'static str, base_amount: i64 },
    /// Modifies a specific chaos engine for this character
    EngineNode { engine: &'static str, modifier: EngineModifier },
    /// Build-defining permanent change
    Keystone(Keystone),
    /// Part of a synergy cluster; bonus activates when all cluster nodes allocated
    Synergy { cluster: SynergyCluster, index: u8 },
}

impl NodeEffect {
    pub fn short_label(&self) -> &'static str {
        match self {
            NodeEffect::StatBonus { stat, .. } => match *stat {
                "Vitality" => "+VIT",
                "Force" => "+FOR",
                "Mana" => "+MAN",
                "Cunning" => "+CUN",
                "Precision" => "+PRE",
                "Entropy" => "+ENT",
                "Luck" => "+LCK",
                _ => "+STA",
            },
            NodeEffect::EngineNode { engine, .. } => match *engine {
                "Lorenz Attractor" => "LRZ",
                "Riemann Zeta" => "RZT",
                "Collatz Chain" => "CLZ",
                "Mandelbrot" => "MND",
                "Fibonacci" => "FIB",
                "Logistic Map" => "LOG",
                "Euler Totient" => "EUL",
                "Prime Density" => "PRM",
                "Fourier" => "FRR",
                "Modular Hash" => "MOD",
                _ => "ENG",
            },
            NodeEffect::Keystone(k) => match k {
                Keystone::ChaosImmunity => "[CI]",
                Keystone::EntropyInversion => "[EI]",
                Keystone::MathematicalCertainty => "[MC]",
                Keystone::GlassCannonInfinite => "[GC]",
                Keystone::BloodReckoning => "[BR]",
                Keystone::ChaosMirror => "[CM]",
            },
            NodeEffect::Synergy { cluster, .. } => match cluster {
                SynergyCluster::FractalMastery => "FRC",
                SynergyCluster::PrimeConspiracy => "PRM",
                SynergyCluster::LorenzAmplifier => "AMP",
                SynergyCluster::ZetaResonance => "ZTA",
                SynergyCluster::CollatzShortcut => "CLT",
                SynergyCluster::EntropicHarmony => "HRM",
            },
        }
    }
}

// ─── SKILL NODE ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SkillNode {
    pub id: NodeId,
    /// Position on the ASCII map (x=col, y=row in a 60x22 grid)
    pub x: u8,
    pub y: u8,
    pub effect: NodeEffect,
    pub neighbors: &'static [NodeId],
    /// This is the class starting node for this class (None = universal)
    pub class_start: Option<CharacterClass>,
    pub is_notable: bool,
    pub is_keystone: bool,
}

// ─── THE TREE ─────────────────────────────────────────────────────────────────

/// The full passive skill tree -- 82 nodes.
pub static TREE: &[SkillNode] = &[
    // ── CLASS STARTS (0-7) ────────────────────────────────────────────────
    SkillNode {
        id: 0, x: 10, y: 11,
        effect: NodeEffect::StatBonus { stat: "Mana", base_amount: 20 },
        neighbors: &[8, 9, 10],
        class_start: Some(CharacterClass::Mage),
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 1, x: 50, y: 11,
        effect: NodeEffect::StatBonus { stat: "Force", base_amount: 20 },
        neighbors: &[11, 12, 13],
        class_start: Some(CharacterClass::Berserker),
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 2, x: 30, y: 2,
        effect: NodeEffect::StatBonus { stat: "Precision", base_amount: 20 },
        neighbors: &[14, 15, 16],
        class_start: Some(CharacterClass::Ranger),
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 3, x: 30, y: 20,
        effect: NodeEffect::StatBonus { stat: "Cunning", base_amount: 20 },
        neighbors: &[17, 18, 19],
        class_start: Some(CharacterClass::Thief),
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 4, x: 14, y: 18,
        effect: NodeEffect::StatBonus { stat: "Entropy", base_amount: 20 },
        neighbors: &[20, 21, 22],
        class_start: Some(CharacterClass::Necromancer),
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 5, x: 46, y: 18,
        effect: NodeEffect::StatBonus { stat: "Cunning", base_amount: 15 },
        neighbors: &[23, 24, 25],
        class_start: Some(CharacterClass::Alchemist),
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 6, x: 46, y: 4,
        effect: NodeEffect::StatBonus { stat: "Vitality", base_amount: 20 },
        neighbors: &[26, 27, 28],
        class_start: Some(CharacterClass::Paladin),
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 7, x: 14, y: 4,
        effect: NodeEffect::StatBonus { stat: "Luck", base_amount: 20 },
        neighbors: &[29, 30, 31],
        class_start: Some(CharacterClass::VoidWalker),
        is_notable: true, is_keystone: false,
    },

    // ── MAGE BRANCH (8-10 + 32-35) ────────────────────────────────────────
    SkillNode {
        id: 8, x: 12, y: 9,
        effect: NodeEffect::StatBonus { stat: "Mana", base_amount: 10 },
        neighbors: &[0, 32], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 9, x: 8, y: 11,
        effect: NodeEffect::StatBonus { stat: "Entropy", base_amount: 8 },
        neighbors: &[0, 33], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 10, x: 12, y: 13,
        effect: NodeEffect::StatBonus { stat: "Luck", base_amount: 6 },
        neighbors: &[0, 34], class_start: None,
        is_notable: false, is_keystone: false,
    },
    // ── BERSERKER BRANCH (11-13 + 36-39) ──────────────────────────────────
    SkillNode {
        id: 11, x: 48, y: 9,
        effect: NodeEffect::StatBonus { stat: "Force", base_amount: 10 },
        neighbors: &[1, 36], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 12, x: 52, y: 11,
        effect: NodeEffect::StatBonus { stat: "Vitality", base_amount: 8 },
        neighbors: &[1, 37], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 13, x: 48, y: 13,
        effect: NodeEffect::StatBonus { stat: "Entropy", base_amount: 6 },
        neighbors: &[1, 38], class_start: None,
        is_notable: false, is_keystone: false,
    },
    // ── RANGER BRANCH (14-16 + 40-43) ─────────────────────────────────────
    SkillNode {
        id: 14, x: 27, y: 3,
        effect: NodeEffect::StatBonus { stat: "Precision", base_amount: 10 },
        neighbors: &[2, 40], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 15, x: 30, y: 1,
        effect: NodeEffect::StatBonus { stat: "Luck", base_amount: 8 },
        neighbors: &[2, 41], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 16, x: 33, y: 3,
        effect: NodeEffect::StatBonus { stat: "Cunning", base_amount: 6 },
        neighbors: &[2, 42], class_start: None,
        is_notable: false, is_keystone: false,
    },
    // ── THIEF BRANCH (17-19 + 44-47) ──────────────────────────────────────
    SkillNode {
        id: 17, x: 27, y: 19,
        effect: NodeEffect::StatBonus { stat: "Cunning", base_amount: 10 },
        neighbors: &[3, 44], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 18, x: 30, y: 21,
        effect: NodeEffect::StatBonus { stat: "Luck", base_amount: 8 },
        neighbors: &[3, 45], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 19, x: 33, y: 19,
        effect: NodeEffect::StatBonus { stat: "Precision", base_amount: 6 },
        neighbors: &[3, 46], class_start: None,
        is_notable: false, is_keystone: false,
    },
    // ── NECROMANCER BRANCH (20-22 + 48-50) ────────────────────────────────
    SkillNode {
        id: 20, x: 12, y: 17,
        effect: NodeEffect::StatBonus { stat: "Entropy", base_amount: 10 },
        neighbors: &[4, 48], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 21, x: 16, y: 19,
        effect: NodeEffect::StatBonus { stat: "Mana", base_amount: 8 },
        neighbors: &[4, 49], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 22, x: 10, y: 19,
        effect: NodeEffect::StatBonus { stat: "Luck", base_amount: 6 },
        neighbors: &[4, 50], class_start: None,
        is_notable: false, is_keystone: false,
    },
    // ── ALCHEMIST BRANCH (23-25 + 51-53) ──────────────────────────────────
    SkillNode {
        id: 23, x: 48, y: 17,
        effect: NodeEffect::StatBonus { stat: "Cunning", base_amount: 10 },
        neighbors: &[5, 51], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 24, x: 44, y: 19,
        effect: NodeEffect::StatBonus { stat: "Mana", base_amount: 8 },
        neighbors: &[5, 52], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 25, x: 50, y: 19,
        effect: NodeEffect::StatBonus { stat: "Luck", base_amount: 6 },
        neighbors: &[5, 53], class_start: None,
        is_notable: false, is_keystone: false,
    },
    // ── PALADIN BRANCH (26-28 + 54-56) ────────────────────────────────────
    SkillNode {
        id: 26, x: 48, y: 5,
        effect: NodeEffect::StatBonus { stat: "Vitality", base_amount: 10 },
        neighbors: &[6, 54], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 27, x: 44, y: 3,
        effect: NodeEffect::StatBonus { stat: "Force", base_amount: 8 },
        neighbors: &[6, 55], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 28, x: 50, y: 3,
        effect: NodeEffect::StatBonus { stat: "Mana", base_amount: 6 },
        neighbors: &[6, 56], class_start: None,
        is_notable: false, is_keystone: false,
    },
    // ── VOIDWALKER BRANCH (29-31 + 57-59) ─────────────────────────────────
    SkillNode {
        id: 29, x: 12, y: 5,
        effect: NodeEffect::StatBonus { stat: "Luck", base_amount: 10 },
        neighbors: &[7, 57], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 30, x: 16, y: 3,
        effect: NodeEffect::StatBonus { stat: "Entropy", base_amount: 8 },
        neighbors: &[7, 58], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 31, x: 10, y: 3,
        effect: NodeEffect::StatBonus { stat: "Precision", base_amount: 6 },
        neighbors: &[7, 59], class_start: None,
        is_notable: false, is_keystone: false,
    },

    // ── ENGINE NODES (32-47) ───────────────────────────────────────────────
    SkillNode {
        id: 32, x: 15, y: 7,
        effect: NodeEffect::EngineNode {
            engine: "Riemann Zeta",
            modifier: EngineModifier::Volatile,
        },
        neighbors: &[8, 60], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 33, x: 6, y: 11,
        effect: NodeEffect::EngineNode {
            engine: "Lorenz Attractor",
            modifier: EngineModifier::DoubleOutput,
        },
        neighbors: &[9, 61], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 34, x: 15, y: 15,
        effect: NodeEffect::EngineNode {
            engine: "Mandelbrot",
            modifier: EngineModifier::BoundaryMagnet,
        },
        neighbors: &[10, 62], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 36, x: 45, y: 7,
        effect: NodeEffect::EngineNode {
            engine: "Collatz Chain",
            modifier: EngineModifier::CapIterations(100),
        },
        neighbors: &[11, 63], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 37, x: 54, y: 11,
        effect: NodeEffect::EngineNode {
            engine: "Logistic Map",
            modifier: EngineModifier::ForcePositive,
        },
        neighbors: &[12, 64], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 38, x: 45, y: 15,
        effect: NodeEffect::EngineNode {
            engine: "Euler Totient",
            modifier: EngineModifier::AlwaysInclude,
        },
        neighbors: &[13, 65], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 40, x: 24, y: 4,
        effect: NodeEffect::EngineNode {
            engine: "Prime Density",
            modifier: EngineModifier::DoubleOutput,
        },
        neighbors: &[14, 66], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 41, x: 30, y: 0,
        effect: NodeEffect::EngineNode {
            engine: "Fibonacci",
            modifier: EngineModifier::AlwaysInclude,
        },
        neighbors: &[15, 67], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 42, x: 36, y: 4,
        effect: NodeEffect::EngineNode {
            engine: "Fourier",
            modifier: EngineModifier::Volatile,
        },
        neighbors: &[16, 68], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 44, x: 24, y: 18,
        effect: NodeEffect::EngineNode {
            engine: "Modular Hash",
            modifier: EngineModifier::DoubleOutput,
        },
        neighbors: &[17, 69], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 45, x: 30, y: 22,
        effect: NodeEffect::EngineNode {
            engine: "Logistic Map",
            modifier: EngineModifier::BoundaryMagnet,
        },
        neighbors: &[18, 70], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 46, x: 36, y: 18,
        effect: NodeEffect::EngineNode {
            engine: "Prime Density",
            modifier: EngineModifier::ForcePositive,
        },
        neighbors: &[19, 71], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 48, x: 10, y: 16,
        effect: NodeEffect::EngineNode {
            engine: "Mandelbrot",
            modifier: EngineModifier::AlwaysInclude,
        },
        neighbors: &[20, 72], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 49, x: 16, y: 21,
        effect: NodeEffect::EngineNode {
            engine: "Collatz Chain",
            modifier: EngineModifier::DoubleOutput,
        },
        neighbors: &[21, 73], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 50, x: 8, y: 21,
        effect: NodeEffect::EngineNode {
            engine: "Riemann Zeta",
            modifier: EngineModifier::ForcePositive,
        },
        neighbors: &[22, 74], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 51, x: 50, y: 16,
        effect: NodeEffect::EngineNode {
            engine: "Fourier",
            modifier: EngineModifier::DoubleOutput,
        },
        neighbors: &[23, 75], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 52, x: 44, y: 21,
        effect: NodeEffect::EngineNode {
            engine: "Lorenz Attractor",
            modifier: EngineModifier::BoundaryMagnet,
        },
        neighbors: &[24, 76], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 53, x: 52, y: 21,
        effect: NodeEffect::EngineNode {
            engine: "Euler Totient",
            modifier: EngineModifier::Volatile,
        },
        neighbors: &[25, 77], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 54, x: 50, y: 6,
        effect: NodeEffect::EngineNode {
            engine: "Fibonacci",
            modifier: EngineModifier::ForcePositive,
        },
        neighbors: &[26, 78], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 55, x: 44, y: 1,
        effect: NodeEffect::EngineNode {
            engine: "Modular Hash",
            modifier: EngineModifier::AlwaysInclude,
        },
        neighbors: &[27, 79], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 56, x: 52, y: 1,
        effect: NodeEffect::EngineNode {
            engine: "Collatz Chain",
            modifier: EngineModifier::AlwaysExclude,
        },
        neighbors: &[28, 80], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 57, x: 10, y: 6,
        effect: NodeEffect::EngineNode {
            engine: "Lorenz Attractor",
            modifier: EngineModifier::ForcePositive,
        },
        neighbors: &[29, 81], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 58, x: 16, y: 1,
        effect: NodeEffect::EngineNode {
            engine: "Mandelbrot",
            modifier: EngineModifier::AlwaysExclude,
        },
        neighbors: &[30, 60], class_start: None,
        is_notable: true, is_keystone: false,
    },
    SkillNode {
        id: 59, x: 8, y: 1,
        effect: NodeEffect::EngineNode {
            engine: "Riemann Zeta",
            modifier: EngineModifier::DoubleOutput,
        },
        neighbors: &[31, 61], class_start: None,
        is_notable: true, is_keystone: false,
    },

    // ── SYNERGY CLUSTERS (60-74) ───────────────────────────────────────────
    SkillNode {
        id: 60, x: 20, y: 8,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::FractalMastery, index: 0 },
        neighbors: &[32, 58, 61], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 61, x: 20, y: 10,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::FractalMastery, index: 1 },
        neighbors: &[33, 59, 60, 62], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 62, x: 20, y: 14,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::FractalMastery, index: 2 },
        neighbors: &[34, 61, 63], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 63, x: 40, y: 8,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::LorenzAmplifier, index: 0 },
        neighbors: &[36, 62, 64], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 64, x: 40, y: 10,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::LorenzAmplifier, index: 1 },
        neighbors: &[37, 63, 65, 66], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 65, x: 40, y: 14,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::LorenzAmplifier, index: 2 },
        neighbors: &[38, 64, 67], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 66, x: 26, y: 7,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::PrimeConspiracy, index: 0 },
        neighbors: &[40, 64, 67, 68], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 67, x: 30, y: 6,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::PrimeConspiracy, index: 1 },
        neighbors: &[41, 65, 66, 68], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 68, x: 34, y: 7,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::PrimeConspiracy, index: 2 },
        neighbors: &[42, 66, 67, 69], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 69, x: 26, y: 15,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::ZetaResonance, index: 0 },
        neighbors: &[44, 68, 70, 71], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 70, x: 30, y: 16,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::ZetaResonance, index: 1 },
        neighbors: &[45, 69, 71], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 71, x: 34, y: 15,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::ZetaResonance, index: 2 },
        neighbors: &[46, 69, 70, 72], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 72, x: 18, y: 16,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::CollatzShortcut, index: 0 },
        neighbors: &[48, 71, 73], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 73, x: 20, y: 18,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::CollatzShortcut, index: 1 },
        neighbors: &[49, 72, 74], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 74, x: 16, y: 18,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::CollatzShortcut, index: 2 },
        neighbors: &[50, 73], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 75, x: 42, y: 16,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::EntropicHarmony, index: 0 },
        neighbors: &[51, 76, 77], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 76, x: 40, y: 18,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::EntropicHarmony, index: 1 },
        neighbors: &[52, 75, 77], class_start: None,
        is_notable: false, is_keystone: false,
    },
    SkillNode {
        id: 77, x: 44, y: 18,
        effect: NodeEffect::Synergy { cluster: SynergyCluster::EntropicHarmony, index: 2 },
        neighbors: &[53, 75, 76], class_start: None,
        is_notable: false, is_keystone: false,
    },

    // ── KEYSTONES (78-81) at the extreme edges ─────────────────────────────
    SkillNode {
        id: 78, x: 55, y: 6,
        effect: NodeEffect::Keystone(Keystone::ChaosImmunity),
        neighbors: &[54], class_start: None,
        is_notable: false, is_keystone: true,
    },
    SkillNode {
        id: 79, x: 57, y: 1,
        effect: NodeEffect::Keystone(Keystone::GlassCannonInfinite),
        neighbors: &[55], class_start: None,
        is_notable: false, is_keystone: true,
    },
    SkillNode {
        id: 80, x: 57, y: 3,
        effect: NodeEffect::Keystone(Keystone::MathematicalCertainty),
        neighbors: &[56], class_start: None,
        is_notable: false, is_keystone: true,
    },
    SkillNode {
        id: 81, x: 3, y: 6,
        effect: NodeEffect::Keystone(Keystone::EntropyInversion),
        neighbors: &[57], class_start: None,
        is_notable: false, is_keystone: true,
    },
];

// ─── TREE LOGIC ───────────────────────────────────────────────────────────────

/// Returns the starting node ID for a given class.
pub fn class_start_node(class: CharacterClass) -> NodeId {
    TREE.iter()
        .find(|n| n.class_start == Some(class))
        .map(|n| n.id)
        .unwrap_or(0)
}

/// Returns all node IDs reachable from allocated nodes (i.e. available to allocate next).
pub fn reachable_nodes(allocated: &[NodeId], class: CharacterClass) -> Vec<NodeId> {
    let start = class_start_node(class);
    let mut reachable = std::collections::HashSet::new();

    // Always reachable: class start
    if !allocated.contains(&start) {
        reachable.insert(start);
    }

    // Neighbors of allocated nodes
    for &id in allocated {
        if let Some(node) = TREE.iter().find(|n| n.id == id) {
            for &neighbor in node.neighbors {
                if !allocated.contains(&neighbor) {
                    reachable.insert(neighbor);
                }
            }
        }
    }
    let mut v: Vec<NodeId> = reachable.into_iter().collect();
    v.sort();
    v
}

/// Get a node by ID.
pub fn get_node(id: NodeId) -> Option<&'static SkillNode> {
    TREE.iter().find(|n| n.id == id)
}

/// How many nodes of a synergy cluster are allocated?
pub fn cluster_count(allocated: &[NodeId], cluster: SynergyCluster) -> usize {
    TREE.iter()
        .filter(|n| {
            matches!(&n.effect, NodeEffect::Synergy { cluster: c, .. } if *c == cluster)
                && allocated.contains(&n.id)
        })
        .count()
}

/// Is a synergy cluster complete (all nodes allocated)?
pub fn cluster_active(allocated: &[NodeId], cluster: SynergyCluster) -> bool {
    let total = TREE
        .iter()
        .filter(|n| matches!(&n.effect, NodeEffect::Synergy { cluster: c, .. } if *c == cluster))
        .count();
    cluster_count(allocated, cluster) >= total
}

/// Which keystone (if any) is allocated?
pub fn active_keystone(allocated: &[NodeId]) -> Option<Keystone> {
    for &id in allocated {
        if let Some(node) = get_node(id) {
            if let NodeEffect::Keystone(k) = node.effect {
                return Some(k);
            }
        }
    }
    None
}

/// Chaos-roll the actual stat bonus at the moment of allocating a stat node.
pub fn roll_stat_bonus(base_amount: i64, seed: u64) -> i64 {
    use crate::chaos_pipeline::chaos_roll_verbose;
    let roll = chaos_roll_verbose(base_amount as f64 * 0.01, seed);
    // The roll can give anywhere from -2x to +4x the base amount
    let mult = 1.0 + roll.final_value * 2.5;
    (base_amount as f64 * mult) as i64
}

// ─── ASCII MAP RENDERER ───────────────────────────────────────────────────────

/// Render the skill tree as a 60x24 ASCII grid.
/// Returns a Vec of lines ready to print.
pub fn render_map(allocated: &[NodeId], cursor: NodeId, class: CharacterClass) -> Vec<String> {
    let reset = "\x1b[0m";
    let reachable = reachable_nodes(allocated, class);

    // Build a 60x24 grid of characters
    const W: usize = 60;
    const H: usize = 24;
    let mut grid: Vec<Vec<char>> = vec![vec![' '; W]; H];
    let mut colors: Vec<Vec<&'static str>> = vec![vec!["\x1b[90m"; W]; H];

    // Draw edges first (so nodes overdraw them)
    for node in TREE {
        let (nx, ny) = (node.x as usize, node.y as usize);
        if nx >= W || ny >= H { continue; }
        for &nid in node.neighbors {
            if let Some(nb) = get_node(nid) {
                let (mx, my) = (nb.x as usize, nb.y as usize);
                if mx >= W || my >= H { continue; }
                // Draw a simple line between the two nodes
                let dx = mx as i32 - nx as i32;
                let dy = my as i32 - ny as i32;
                let steps = dx.abs().max(dy.abs());
                if steps == 0 { continue; }
                for i in 1..steps {
                    let px = (nx as i32 + dx * i / steps) as usize;
                    let py = (ny as i32 + dy * i / steps) as usize;
                    if px < W && py < H && grid[py][px] == ' ' {
                        grid[py][px] = '·';
                    }
                }
            }
        }
    }

    // Draw nodes
    for node in TREE {
        let (x, y) = (node.x as usize, node.y as usize);
        if x >= W || y >= H { continue; }
        let is_alloc = allocated.contains(&node.id);
        let is_cursor = node.id == cursor;
        let is_reach = reachable.contains(&node.id);

        let label = node.effect.short_label();
        let ch = if is_cursor { '>' }
                 else if is_alloc && node.is_keystone { 'K' }
                 else if is_alloc && node.is_notable { 'N' }
                 else if is_alloc { '#' }
                 else if node.is_keystone { 'k' }
                 else if node.is_notable { 'n' }
                 else { 'o' };

        let col = if is_cursor { "\x1b[97m" }
                  else if is_alloc { "\x1b[92m" }
                  else if is_reach { "\x1b[93m" }
                  else { "\x1b[90m" };

        grid[y][x] = ch;
        colors[y][x] = col;
        // Optionally write the short label to the right if space
        if x + 1 < W && x + label.len() + 1 < W {
            for (i, c) in label.chars().enumerate() {
                if x + 1 + i < W && grid[y][x + 1 + i] == ' ' {
                    grid[y][x + 1 + i] = c;
                    colors[y][x + 1 + i] = col;
                }
            }
        }
    }

    // Render to strings
    let mut lines = Vec::new();
    for row in 0..H {
        let mut line = String::new();
        let mut last_col = "";
        for col in 0..W {
            let c = grid[row][col];
            let color = colors[row][col];
            if color != last_col {
                line.push_str(color);
                last_col = color;
            }
            line.push(c);
        }
        line.push_str(reset);
        lines.push(line);
    }
    lines
}
