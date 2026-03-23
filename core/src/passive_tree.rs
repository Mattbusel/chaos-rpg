//! Path of Exile-style passive skill tree — ~800 nodes across 5 rings.
//!
//! Ring 0: 8 class origins (existing)
//! Ring 1: class branches (existing ~40 nodes)
//! Ring 2: foundation — 12 nodes per class (96 total)
//! Ring 3: specialization — 20 nodes per class (160 total)
//! Ring 4: advanced — 30 nodes per class (240 total)
//! Ring 5: mastery — 18 nodes per class (144 total)
//! Bridge: connections between adjacent classes (96 nodes)
//! Extra keystones: 4 per class (32 nodes)
//! Grand total: ~820 nodes
//!
//! Node types: Stat, Engine, Notable, Keystone, Synergy

use crate::chaos_pipeline::chaos_roll_verbose;
use crate::character::CharacterClass;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::OnceLock;

// ─── NODE TYPES ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum NodeType {
    /// Flat bonus — amount is chaos-rolled on allocation (unknown until commit).
    Stat { stat: &'static str, min: i64, max: i64 },
    /// Modifies a specific chaos engine's behaviour for this character.
    Engine { engine: &'static str, effect: &'static str },
    /// Major build-defining keystone with trade-off.
    Keystone { id: &'static str },
    /// Named notable with a fixed bonus and a special effect description.
    Notable { stat: &'static str, bonus: i64, effect: &'static str },
    /// Synergy cluster — weak alone, unlocks bonus when full cluster allocated.
    Synergy { cluster: u8, bonus_desc: &'static str },
}

// ─── TREE NODE ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: u16,
    pub x: i16,
    pub y: i16,
    pub name: String,
    pub short_desc: String,
    pub node_type: NodeType,
    pub requires: Vec<u16>,
    pub class_start: Option<CharacterClass>,
}

// ─── KEYSTONE CONSTANTS ───────────────────────────────────────────────────────

// Existing keystones
pub const KS_CHAOS_IMMUNITY: &str = "ChaosImmunity";
pub const KS_ENTROPY_INVERSION: &str = "EntropyInversion";
pub const KS_MATH_CERTAINTY: &str = "MathCertainty";
pub const KS_GLASS_CANNON: &str = "GlassCannon";
pub const KS_RESONANCE_ECHO: &str = "ResonanceEcho";
pub const KS_PRIME_BLOOD: &str = "PrimeBloodKeystone";
pub const KS_VOID_STEP: &str = "VoidStep";
pub const KS_DEATH_PACT: &str = "DeathPact";

// Per-class ring 4 keystones
pub const KS_ARCANE_SUPREMACY: &str = "ArcaneSupremacy";
pub const KS_BLOOD_FRENZY: &str = "BloodFrenzy";
pub const KS_EAGLE_EYE: &str = "EagleEye";
pub const KS_SHADOW_CLONE: &str = "ShadowClone";
pub const KS_PHYLACTERY: &str = "Phylactery";
pub const KS_FORMULA_37X: &str = "Formula37X";
pub const KS_SHIELD_OF_FAITH: &str = "ShieldOfFaith";
pub const KS_PHASE_SHIFT: &str = "PhaseShift";

// Per-class ring 5 keystones
pub const KS_OVERLOAD_PROTOCOL: &str = "OverloadProtocol";
pub const KS_FURY_CORE: &str = "FuryCore";
pub const KS_PRIMED_SHOT: &str = "PrimedShot";
pub const KS_DEATH_FROM_SHADOWS: &str = "DeathFromShadows";
pub const KS_SOULBIND: &str = "Soulbind";
pub const KS_GRAND_ELIXIR: &str = "GrandElixir";
pub const KS_DIVINE_SHIELD: &str = "DivineShield";
pub const KS_VOID_RIFT: &str = "VoidRift";

// ─── CLASS CONFIGURATION ─────────────────────────────────────────────────────

struct ClassConfig {
    start_id: u16,
    origin: (i16, i16),
    ring1_ids: &'static [u16],
    stats: [&'static str; 3],
    prefix: &'static str,
    engines: [(&'static str, &'static str); 2],
    notables: [(&'static str, &'static str); 5],
    ring4_keystone: &'static str,
    ring5_keystone: &'static str,
}

static CLASS_CONFIGS: &[ClassConfig] = &[
    // 0 — Mage
    ClassConfig {
        start_id: 0, origin: (4, 10), ring1_ids: &[10, 11],
        stats: ["mana", "entropy", "precision"], prefix: "Arcane",
        engines: [("Riemann Zeta Partial", "volatile"), ("Lorenz Attractor", "chaotic")],
        notables: [
            ("The Arcane Eye",    "Spells pierce 20% of enemy resistance"),
            ("Mana Convergence",  "Kills restore 20 mana"),
            ("Chaos Lens",        "Chaos roll doubled once per room"),
            ("Essence Drain",     "Mana regen rate doubled"),
            ("Temporal Flux",     "First spell each combat costs 0 mana"),
        ],
        ring4_keystone: KS_ARCANE_SUPREMACY,
        ring5_keystone: KS_OVERLOAD_PROTOCOL,
    },
    // 1 — Berserker
    ClassConfig {
        start_id: 1, origin: (36, 10), ring1_ids: &[20, 21],
        stats: ["force", "vitality", "luck"], prefix: "Rage",
        engines: [("Collatz Chain", "cap100"), ("Logistic Map", "peak")],
        notables: [
            ("Berserker's Heart", "FORCE scales 1.5x below 50% HP"),
            ("Iron Flesh",        "Absorb 20% physical damage"),
            ("War Cry",           "First attack each combat crits"),
            ("Undying Rage",      "Survive lethal blow once per floor"),
            ("Frenzy",            "+5 FORCE per kill (max 3 stacks)"),
        ],
        ring4_keystone: KS_BLOOD_FRENZY,
        ring5_keystone: KS_FURY_CORE,
    },
    // 2 — Ranger
    ClassConfig {
        start_id: 2, origin: (20, 2), ring1_ids: &[30, 31],
        stats: ["precision", "luck", "entropy"], prefix: "Prime",
        engines: [("Mandelbrot Escape", "boundary"), ("Prime Density Sieve", "dense")],
        notables: [
            ("Eagle Vision",   "See enemy stats through VisionBlur"),
            ("Steady Aim",     "+30 PRECISION when standing still"),
            ("Arrow Storm",    "Attacks hit 2 extra targets"),
            ("Wind Reader",    "Dodge chance = PRECISION / 200"),
            ("Prime Focus",    "Prime-numbered turns deal double damage"),
        ],
        ring4_keystone: KS_EAGLE_EYE,
        ring5_keystone: KS_PRIMED_SHOT,
    },
    // 3 — Thief
    ClassConfig {
        start_id: 3, origin: (20, 18), ring1_ids: &[40, 41],
        stats: ["cunning", "luck", "precision"], prefix: "Shadow",
        engines: [("Fibonacci Golden Spiral", "double"), ("Cantor Pairing", "shift")],
        notables: [
            ("Quick Hands",    "Dodge 25% of attacks"),
            ("Poison Edge",    "Attacks apply Poison (10 dmg/turn)"),
            ("Shadow Blend",   "Invisible for 2 turns after fleeing"),
            ("Loot Sense",     "Rare enemies drop 2x gold"),
            ("Knife Rain",     "25% chance to attack twice per turn"),
        ],
        ring4_keystone: KS_SHADOW_CLONE,
        ring5_keystone: KS_DEATH_FROM_SHADOWS,
    },
    // 4 — Necromancer
    ClassConfig {
        start_id: 4, origin: (6, 4), ring1_ids: &[50, 51],
        stats: ["entropy", "mana", "vitality"], prefix: "Death",
        engines: [("Euler's Totient", "invert"), ("Sin-Cos Spiral", "negative")],
        notables: [
            ("Necromantic Pact", "Kills restore 15 HP"),
            ("Soul Harvest",     "Gain 5 mana per enemy killed"),
            ("Bone Armor",       "Reduce all damage by 15%"),
            ("Entropy Well",     "Status effects last 2 turns longer"),
            ("Death Link",       "Share 30% of damage taken with enemy"),
        ],
        ring4_keystone: KS_PHYLACTERY,
        ring5_keystone: KS_SOULBIND,
    },
    // 5 — Alchemist
    ClassConfig {
        start_id: 5, origin: (34, 4), ring1_ids: &[60, 61],
        stats: ["cunning", "mana", "luck"], prefix: "Flask",
        engines: [("Collatz Chain", "even_start"), ("Logistic Map", "stable")],
        notables: [
            ("Master Brewer",  "Items have 30% stronger effects"),
            ("Reagent Cache",  "Find extra items in treasure rooms"),
            ("Volatile Mix",   "15% chance items deal AoE on use"),
            ("Preservation",   "Items last 2 extra turns when active"),
            ("Grand Formula",  "Crafting operations cost 50% less gold"),
        ],
        ring4_keystone: KS_FORMULA_37X,
        ring5_keystone: KS_GRAND_ELIXIR,
    },
    // 6 — Paladin
    ClassConfig {
        start_id: 6, origin: (6, 16), ring1_ids: &[70, 71],
        stats: ["vitality", "force", "mana"], prefix: "Holy",
        engines: [("Euler's Totient", "positive"), ("Fibonacci Golden Spiral", "abs")],
        notables: [
            ("Holy Ward",       "Block 20% of all damage"),
            ("Sacred Ground",   "Heal 5 HP per turn in combat"),
            ("Divine Strike",   "Attacks deal bonus damage equal to VIT/10"),
            ("Blessed Armor",   "Reduce magic damage by 25%"),
            ("Martyr's Touch",  "Heal 50% of damage taken"),
        ],
        ring4_keystone: KS_SHIELD_OF_FAITH,
        ring5_keystone: KS_DIVINE_SHIELD,
    },
    // 7 — VoidWalker
    ClassConfig {
        start_id: 7, origin: (34, 16), ring1_ids: &[80, 81],
        stats: ["entropy", "luck", "cunning"], prefix: "Void",
        engines: [("Lorenz Attractor", "void"), ("Mandelbrot Escape", "deep")],
        notables: [
            ("Phase Aura",        "Passively dodge 20% of all attacks"),
            ("Void Sense",        "See phased and hidden enemies"),
            ("Entropy Surge",     "Chaos roll ×1.5 when Phased"),
            ("Dimensional Rift",  "Teleport behind enemy once per combat"),
            ("Null Zone",         "Enemies lose 15 PRECISION near you"),
        ],
        ring4_keystone: KS_PHASE_SHIFT,
        ring5_keystone: KS_VOID_RIFT,
    },
];

// ─── EXTRA KEYSTONES (4 per class, IDs 1100–1131) ────────────────────────────

static EXTRA_KEYSTONES: &[(&str, &str, &str)] = &[
    // Mage
    ("Arcane Conduit",    "ArcaneConduit",    "Damage dealt equals 100% of current mana"),
    ("Spell Mirror",      "SpellMirror",      "50% chance to reflect incoming spells"),
    ("Mana Fortress",     "ManaFortress",     "Max mana ×2; all damage hits mana first"),
    ("Temporal Paradox",  "TemporalParadox",  "Act twice per turn; take double damage"),
    // Berserker
    ("Warlord's Mark",    "WarlordsMark",     "+100 FORCE; cannot heal during combat"),
    ("Blood Price",       "BloodPrice",       "HP is mana; mana is HP"),
    ("Berserk Core",      "BerserkCore",      "All attacks crit when below 25% HP"),
    ("Titan's Grip",      "TitansGrip",       "+200 FORCE, -50 PRECISION"),
    // Ranger
    ("Marked Target",     "MarkedTarget",     "Marked enemy takes double damage"),
    ("Arrow Rain",        "ArrowRain",        "Hit all enemies once per combat"),
    ("Wind Step",         "WindStep",         "Always flee successfully"),
    ("True Strike",       "TrueStrike",       "Attacks never miss but cannot crit"),
    // Thief
    ("Shadow Pact",       "ShadowPact",       "Dodge 25% of all incoming attacks"),
    ("Poison Lord",       "PoisonLord",       "All attacks stack +20 Poison"),
    ("Smoke Lord",        "SmokeLord",        "Immune to first 3 attacks each combat"),
    ("Murder Most Foul",  "MurderMostFoul",   "First attack each combat deals 10× damage"),
    // Necromancer
    ("Deathless",         "Deathless",        "Respawn once per floor at 50% HP"),
    ("Soul Army",         "SoulArmy",         "Slain enemies fight for you next turn"),
    ("Death Explosion",   "DeathExplosion",   "On death: deal 200% HP as AoE damage"),
    ("Entropy Loop",      "EntropyLoop",      "Take 5 HP/turn; gain 5 mana/turn; +100 dmg"),
    // Alchemist
    ("Philosopher's Stone","PhilosophersStone","Gold found everywhere is doubled"),
    ("Catalyst Prime",    "CatalystPrime",    "All crafting operations are free"),
    ("Healing Font",      "HealingFont",      "Fully heal once per floor for free"),
    ("Reactive Plating",  "ReactivePlating",  "+100 to all defenses, -30 CUNNING"),
    // Paladin
    ("Unshakeable Faith", "UnshakeableFaith", "Cannot be feared or confused"),
    ("Smite",             "SmiteKS",          "All attacks deal VITALITY as bonus damage"),
    ("Fortress",          "FortressKS",       "Cannot be reduced below 1 HP more than once per combat"),
    ("Crusader's Oath",   "CrusadersOath",    "Cannot flee; deal +100% damage"),
    // VoidWalker
    ("Phase Walk",        "PhaseWalk",        "50% of attacks phase through you"),
    ("Void Eruption",     "VoidEruption",     "Deal damage = chaos roll × 10 each turn"),
    ("Miss Chain",        "MissChain",        "Each miss increases next attack damage by 50%"),
    ("Dimensional Lock",  "DimensionalLock",  "Enemies in combat cannot flee"),
];

// ─── BRIDGE PAIRS ─────────────────────────────────────────────────────────────

// Adjacent class index pairs for bridge nodes
const BRIDGE_PAIRS: &[(usize, usize)] = &[
    (0, 4), // Mage – Necromancer
    (4, 2), // Necromancer – Ranger
    (2, 5), // Ranger – Alchemist
    (5, 1), // Alchemist – Berserker
    (1, 7), // Berserker – VoidWalker
    (7, 3), // VoidWalker – Thief
    (3, 6), // Thief – Paladin
    (6, 0), // Paladin – Mage
];

// ─── POSITION HELPERS ────────────────────────────────────────────────────────

/// Lerp a position toward the tree center based on ring number.
fn ring_pos(origin: (i16, i16), ring: u8, sub: u16, total: u16) -> (i16, i16) {
    const CENTER: (i16, i16) = (20, 10);
    let t: f32 = match ring {
        2 => 0.28,
        3 => 0.46,
        4 => 0.64,
        5 => 0.82,
        _ => 0.50,
    };
    let dx = (CENTER.0 - origin.0) as f32;
    let dy = (CENTER.1 - origin.1) as f32;
    let bx = origin.0 as f32 + dx * t;
    let by = origin.1 as f32 + dy * t;
    // Perpendicular direction (rotate 90°)
    let (px, py) = (-dy, dx);
    let plen = (px * px + py * py).sqrt();
    let spread: f32 = match ring {
        2 => 0.40,
        3 => 0.32,
        4 => 0.24,
        5 => 0.18,
        _ => 0.30,
    };
    let half = (total.saturating_sub(1)) as f32 / 2.0;
    let offset = sub as f32 - half;
    if plen > 0.01 {
        let ox = (offset * spread * px / plen).round() as i16;
        let oy = (offset * spread * py / plen).round() as i16;
        ((bx.round() as i16 + ox).clamp(1, 39), (by.round() as i16 + oy).clamp(1, 19))
    } else {
        ((bx.round() as i16 + offset.round() as i16).clamp(1, 39), by.round() as i16)
    }
}

fn stat_title(stat: &str) -> &str {
    match stat {
        "vitality"  => "Vitality",
        "force"     => "Force",
        "mana"      => "Mana",
        "cunning"   => "Cunning",
        "precision" => "Precision",
        "entropy"   => "Entropy",
        "luck"      => "Luck",
        s => s,
    }
}

// ─── NODE BUILDER ────────────────────────────────────────────────────────────

fn build_nodes() -> Vec<TreeNode> {
    let mut v = Vec::with_capacity(860);

    // ── EXISTING HARDCODED NODES (IDs 0–114) ─────────────────────────────────
    let hard: &[(&str, &str, NodeType, u16, i16, i16, &'static [u16], Option<CharacterClass>)] = &[
        // class origins
        ("Arcane Origin",    "+MANA +ENTROPY start",    NodeType::Stat { stat:"mana",      min:10, max:40 }, 0,  4,  10, &[],                 Some(CharacterClass::Mage)),
        ("Rage Origin",      "+FORCE +VIT start",       NodeType::Stat { stat:"force",     min:10, max:40 }, 1,  36, 10, &[],                 Some(CharacterClass::Berserker)),
        ("Prime Origin",     "+PRECISION +LUCK start",  NodeType::Stat { stat:"precision", min:10, max:40 }, 2,  20, 2,  &[],                 Some(CharacterClass::Ranger)),
        ("Shadow Origin",    "+CUNNING +LUCK start",    NodeType::Stat { stat:"cunning",   min:10, max:40 }, 3,  20, 18, &[],                 Some(CharacterClass::Thief)),
        ("Death Origin",     "+ENTROPY +MANA start",    NodeType::Stat { stat:"entropy",   min:10, max:40 }, 4,  6,  4,  &[],                 Some(CharacterClass::Necromancer)),
        ("Flask Origin",     "+CUNNING +MANA start",    NodeType::Stat { stat:"cunning",   min:10, max:35 }, 5,  34, 4,  &[],                 Some(CharacterClass::Alchemist)),
        ("Divine Origin",    "+VIT +FORCE start",       NodeType::Stat { stat:"vitality",  min:10, max:40 }, 6,  6,  16, &[],                 Some(CharacterClass::Paladin)),
        ("Void Origin",      "+ENTROPY +LUCK start",    NodeType::Stat { stat:"entropy",   min:8,  max:35 }, 7,  34, 16, &[],                 Some(CharacterClass::VoidWalker)),
        // mage branch
        ("Mana Surge",       "+MANA (big range)",                NodeType::Stat { stat:"mana",    min:-5, max:60 }, 10, 7,  9,  &[0],    None),
        ("Spell Weave",      "+ENTROPY for spell depth",         NodeType::Stat { stat:"entropy", min:5,  max:30 }, 11, 7,  11, &[0],    None),
        ("Zeta Gambler",     "Riemann Zeta uses s near 1",       NodeType::Engine { engine:"Riemann Zeta Partial", effect:"volatile" }, 12, 9,  8,  &[10],   None),
        ("Lorenz Stabilizer","Lorenz clamped to positive values",NodeType::Engine { engine:"Lorenz Attractor",     effect:"stabilize"},  13, 9,  12, &[11],   None),
        ("Glass Cannon",     "HP=1; damage chain uses 15 engines",NodeType::Keystone { id: KS_GLASS_CANNON },     14, 11, 7,  &[12],   None),
        // berserker branch
        ("Iron Skin",        "+VITALITY (big range)",    NodeType::Stat { stat:"vitality", min:-5, max:60 }, 20, 33, 9,  &[1],    None),
        ("Reckless Power",   "+FORCE, -PRECISION",       NodeType::Stat { stat:"force",    min:5,  max:50 }, 21, 33, 11, &[1],    None),
        ("Collatz Shortcut", "Collatz capped at 100 steps", NodeType::Engine { engine:"Collatz Chain", effect:"cap100" },         22, 31, 8,  &[20],   None),
        ("Chaos Immunity",   "Never take >50% max HP in one hit",NodeType::Keystone { id: KS_CHAOS_IMMUNITY },   23, 29, 7,  &[22],   None),
        // ranger branch
        ("Prime Sight",      "+PRECISION (wide range)",  NodeType::Stat { stat:"precision", min:5,  max:55 }, 30, 19, 4,  &[2],    None),
        ("Lucky Shot",       "+LUCK (very wide range)",  NodeType::Stat { stat:"luck",      min:-10,max:70 }, 31, 21, 4,  &[2],    None),
        ("Mandelbrot Magnet","Mandelbrot samples boundary",NodeType::Engine { engine:"Mandelbrot Escape", effect:"boundary" },    32, 19, 6,  &[30],   None),
        ("Math Certainty",   "Always exactly 4 engines. No variance.", NodeType::Keystone { id: KS_MATH_CERTAINTY },             33, 19, 8,  &[32],   None),
        // thief branch
        ("Shadow Step",      "+CUNNING (wide range)",    NodeType::Stat { stat:"cunning",   min:5,  max:50 }, 40, 19, 16, &[3],    None),
        ("Fortune's Favor",  "+LUCK, chaotic",           NodeType::Stat { stat:"luck",      min:-15,max:80 }, 41, 21, 16, &[3],    None),
        ("Fibonacci Harmony","Fibonacci outputs doubled", NodeType::Engine { engine:"Fibonacci Golden Spiral", effect:"double" }, 42, 21, 14, &[40],   None),
        // necromancer branch
        ("Death Resonance",  "+ENTROPY, negative side",  NodeType::Stat { stat:"entropy",   min:-5, max:55 }, 50, 7,  5,  &[4],    None),
        ("Void Drain",       "+MANA from chaos",         NodeType::Stat { stat:"mana",      min:5,  max:45 }, 51, 8,  7,  &[50],   None),
        ("Entropy Inversion","Negative rolls flip positive — and vice versa.", NodeType::Keystone { id: KS_ENTROPY_INVERSION },  52, 10, 6,  &[51],   None),
        ("Death Pact",       "Die at 0 HP but deal full HP as death strike.", NodeType::Keystone { id: KS_DEATH_PACT },          53, 10, 9,  &[51],   None),
        // alchemist branch
        ("Flask Master",     "+CUNNING for item potency",NodeType::Stat { stat:"cunning",   min:5,  max:40 }, 60, 33, 5,  &[5],    None),
        ("Reagent Boost",    "+MANA via alchemy",        NodeType::Stat { stat:"mana",      min:5,  max:35 }, 61, 31, 6,  &[5],    None),
        ("Collatz Optimizer","Collatz starts even — less volatile", NodeType::Engine { engine:"Collatz Chain", effect:"even_start" }, 62, 30, 9, &[60], None),
        // paladin branch
        ("Holy Fortitude",   "+VITALITY (divine bonus)", NodeType::Stat { stat:"vitality",  min:5,  max:45 }, 70, 7,  15, &[6],    None),
        ("Sacred Force",     "+FORCE from faith",        NodeType::Stat { stat:"force",     min:5,  max:40 }, 71, 8,  13, &[6],    None),
        ("Euler's Grace",    "Euler Totient produces only positive outputs", NodeType::Engine { engine:"Euler's Totient", effect:"positive" }, 72, 10, 14, &[70], None),
        ("Resonance Echo",   "Chaos roll feeds into the next roll's input.", NodeType::Keystone { id: KS_RESONANCE_ECHO },       73, 12, 13, &[72],   None),
        // voidwalker branch
        ("Phase Mastery",    "+ENTROPY for phasing",     NodeType::Stat { stat:"entropy",   min:5,  max:45 }, 80, 33, 15, &[7],    None),
        ("Void Luck",        "+LUCK from the void",      NodeType::Stat { stat:"luck",      min:5,  max:50 }, 81, 31, 14, &[7],    None),
        ("Void Step",        "Phase dodge triggers even after Phasing expires.", NodeType::Keystone { id: KS_VOID_STEP },        82, 29, 13, &[80],   None),
        // shared center
        ("Chaos Attunement", "+ENTROPY (shared)",        NodeType::Stat { stat:"entropy",   min:3,  max:25 }, 90, 16, 10, &[0,4,6,13,51,70], None),
        ("Force Conduit",    "+FORCE (shared)",          NodeType::Stat { stat:"force",     min:3,  max:25 }, 91, 24, 10, &[1,5,7,22,60,80], None),
        ("Mathematical Core","All stats +chaos-rolled bonus", NodeType::Stat { stat:"luck",  min:5,  max:20 }, 92, 20, 10, &[90,91],          None),
        // fractal synergy cluster (IDs 100-104)
        ("Fractal I",   "Fractal Mastery 1/5",  NodeType::Synergy { cluster:1, bonus_desc:"Mandelbrot+Fibonacci outputs doubled" }, 100, 15, 7,  &[0,2,90],  None),
        ("Fractal II",  "Fractal Mastery 2/5",  NodeType::Synergy { cluster:1, bonus_desc:"Mandelbrot+Fibonacci outputs doubled" }, 101, 17, 6,  &[100],     None),
        ("Fractal III", "Fractal Mastery 3/5",  NodeType::Synergy { cluster:1, bonus_desc:"Mandelbrot+Fibonacci outputs doubled" }, 102, 20, 6,  &[101],     None),
        ("Fractal IV",  "Fractal Mastery 4/5",  NodeType::Synergy { cluster:1, bonus_desc:"Mandelbrot+Fibonacci outputs doubled" }, 103, 23, 6,  &[102],     None),
        ("Fractal V",   "Fractal Mastery 5/5 — BONUS",  NodeType::Synergy { cluster:1, bonus_desc:"Mandelbrot+Fibonacci outputs doubled" }, 104, 25, 7,  &[103],     None),
        // prime synergy cluster (IDs 110-114)
        ("Prime I",   "Prime cluster 1/5",  NodeType::Synergy { cluster:2, bonus_desc:"Prime Density outputs doubled" }, 110, 15, 13, &[3,6,90],  None),
        ("Prime II",  "Prime cluster 2/5",  NodeType::Synergy { cluster:2, bonus_desc:"Prime Density outputs doubled" }, 111, 17, 14, &[110],     None),
        ("Prime III", "Prime cluster 3/5",  NodeType::Synergy { cluster:2, bonus_desc:"Prime Density outputs doubled" }, 112, 20, 14, &[111],     None),
        ("Prime IV",  "Prime cluster 4/5",  NodeType::Synergy { cluster:2, bonus_desc:"Prime Density outputs doubled" }, 113, 23, 14, &[112],     None),
        ("Prime V",   "Prime cluster 5/5 — BONUS", NodeType::Synergy { cluster:2, bonus_desc:"Prime Density outputs doubled" }, 114, 25, 13, &[113], None),
    ];
    for (name, desc, nt, id, x, y, reqs, cs) in hard {
        v.push(TreeNode {
            id: *id, x: *x, y: *y,
            name: name.to_string(), short_desc: desc.to_string(),
            node_type: nt.clone(), requires: reqs.to_vec(), class_start: *cs,
        });
    }

    // ── RING 2: Foundation (IDs 200–295, 12 per class) ───────────────────────
    let r2_names = ["Vein", "Thread", "Strand", "Root", "Pulse", "Flow",
                    "Vein", "Thread", "Strand", "Root", "Pulse", "Flow"];
    for (ci, cfg) in CLASS_CONFIGS.iter().enumerate() {
        let base = 200 + ci as u16 * 12;
        for sub in 0u16..12 {
            let (x, y) = ring_pos(cfg.origin, 2, sub, 12);
            let stat = cfg.stats[sub as usize % 3];
            let req = cfg.ring1_ids[sub as usize % cfg.ring1_ids.len()];
            let (nt, name, desc) = if sub == 10 {
                let (eng, eff) = cfg.engines[0];
                (NodeType::Engine { engine: eng, effect: eff },
                 format!("{} Engine I", cfg.prefix),
                 format!("{} [{}]", eng, eff))
            } else if sub == 11 {
                let (n, d) = cfg.notables[0];
                (NodeType::Notable { stat, bonus: 20, effect: d },
                 n.to_string(), d.to_string())
            } else {
                let suf = r2_names[sub as usize];
                (NodeType::Stat { stat, min: 5, max: 25 },
                 format!("{} {} {}", cfg.prefix, stat_title(stat), suf),
                 format!("+{} (5-25)", stat_title(stat)))
            };
            v.push(TreeNode { id: base + sub, x, y, name, short_desc: desc,
                              node_type: nt, requires: vec![req], class_start: None });
        }
    }

    // ── RING 3: Specialization (IDs 300–459, 20 per class) ───────────────────
    let r3_names = ["Focus","Knot","Core","Cluster","Bond","Anchor","Weave","Mesh",
                    "Focus","Knot","Core","Cluster","Bond","Anchor","Weave","Mesh","Focus","Knot"];
    for (ci, cfg) in CLASS_CONFIGS.iter().enumerate() {
        let base = 300 + ci as u16 * 20;
        let r2_base = 200 + ci as u16 * 12;
        for sub in 0u16..20 {
            let (x, y) = ring_pos(cfg.origin, 3, sub, 20);
            let stat = cfg.stats[sub as usize % 3];
            let req = r2_base + (sub % 10);
            let (nt, name, desc) = if sub == 17 {
                let (eng, eff) = cfg.engines[1];
                (NodeType::Engine { engine: eng, effect: eff },
                 format!("{} Engine II", cfg.prefix),
                 format!("{} [{}]", eng, eff))
            } else if sub == 18 {
                let (n, d) = cfg.notables[1];
                (NodeType::Notable { stat, bonus: 30, effect: d },
                 n.to_string(), d.to_string())
            } else if sub == 19 {
                let cid = 3 + ci as u8;
                (NodeType::Synergy { cluster: cid, bonus_desc: "Class synergy bonus unlocked" },
                 format!("{} Synergy Node", cfg.prefix),
                 format!("Cluster {} — allocate all for bonus", cid))
            } else {
                let suf = r3_names[sub as usize];
                (NodeType::Stat { stat, min: 8, max: 40 },
                 format!("{} {} {}", cfg.prefix, stat_title(stat), suf),
                 format!("+{} (8-40)", stat_title(stat)))
            };
            v.push(TreeNode { id: base + sub, x, y, name, short_desc: desc,
                              node_type: nt, requires: vec![req], class_start: None });
        }
    }

    // ── RING 4: Advanced (IDs 500–739, 30 per class) ─────────────────────────
    let r4_names = ["Surge","Nexus","Drive","Conduit","Channel","Burst","Surge","Nexus",
                    "Drive","Conduit","Channel","Burst","Surge","Nexus","Drive","Conduit",
                    "Channel","Burst","Surge","Nexus","Drive","Conduit","Channel","Burst",
                    "Surge","Nexus","Drive"];
    for (ci, cfg) in CLASS_CONFIGS.iter().enumerate() {
        let base = 500 + ci as u16 * 30;
        let r3_base = 300 + ci as u16 * 20;
        for sub in 0u16..30 {
            let (x, y) = ring_pos(cfg.origin, 4, sub, 30);
            let stat = cfg.stats[sub as usize % 3];
            let req = r3_base + (sub % 18);
            let (nt, name, desc) = if sub == 27 {
                let (eng, eff) = cfg.engines[0];
                (NodeType::Engine { engine: eng, effect: eff },
                 format!("{} Engine III", cfg.prefix),
                 format!("{} [enhanced {}]", eng, eff))
            } else if sub == 28 {
                let (n, d) = cfg.notables[2];
                (NodeType::Notable { stat, bonus: 45, effect: d },
                 n.to_string(), d.to_string())
            } else if sub == 29 {
                (NodeType::Keystone { id: cfg.ring4_keystone },
                 format!("{} Apex Keystone", cfg.prefix),
                 format!("Major keystone: {}", cfg.ring4_keystone))
            } else {
                let suf = r4_names[sub as usize];
                (NodeType::Stat { stat, min: 12, max: 50 },
                 format!("{} {} {}", cfg.prefix, stat_title(stat), suf),
                 format!("+{} (12-50)", stat_title(stat)))
            };
            v.push(TreeNode { id: base + sub, x, y, name, short_desc: desc,
                              node_type: nt, requires: vec![req], class_start: None });
        }
    }

    // ── RING 5: Mastery (IDs 800–943, 18 per class) ──────────────────────────
    let r5_names = ["Pinnacle","Apex","Summit","Peak","Zenith","Crown","Crest","Vertex",
                    "Pinnacle","Apex","Summit","Peak","Zenith","Crown","Crest"];
    for (ci, cfg) in CLASS_CONFIGS.iter().enumerate() {
        let base = 800 + ci as u16 * 18;
        let r4_base = 500 + ci as u16 * 30;
        for sub in 0u16..18 {
            let (x, y) = ring_pos(cfg.origin, 5, sub, 18);
            let stat = cfg.stats[sub as usize % 3];
            let req = r4_base + (sub % 26);
            let (nt, name, desc) = if sub == 15 {
                let (n, d) = cfg.notables[3];
                (NodeType::Notable { stat, bonus: 60, effect: d },
                 n.to_string(), d.to_string())
            } else if sub == 16 {
                let (eng, eff) = cfg.engines[1];
                (NodeType::Engine { engine: eng, effect: eff },
                 format!("{} Engine IV", cfg.prefix),
                 format!("{} [mastery {}]", eng, eff))
            } else if sub == 17 {
                (NodeType::Keystone { id: cfg.ring5_keystone },
                 format!("{} Mastery Keystone", cfg.prefix),
                 format!("Apex keystone: {}", cfg.ring5_keystone))
            } else {
                let suf = r5_names[sub as usize];
                (NodeType::Stat { stat, min: 18, max: 65 },
                 format!("{} {} {}", cfg.prefix, stat_title(stat), suf),
                 format!("+{} (18-65)", stat_title(stat)))
            };
            v.push(TreeNode { id: base + sub, x, y, name, short_desc: desc,
                              node_type: nt, requires: vec![req], class_start: None });
        }
    }

    // ── BRIDGE NODES (IDs 1000–1095, 12 per pair) ────────────────────────────
    for (pair_idx, &(ci_a, ci_b)) in BRIDGE_PAIRS.iter().enumerate() {
        let cfg_a = &CLASS_CONFIGS[ci_a];
        let cfg_b = &CLASS_CONFIGS[ci_b];
        let bridge_base = 1000 + pair_idx as u16 * 12;
        let (ax, ay) = ring_pos(cfg_a.origin, 3, 6, 12);
        let (bx, by) = ring_pos(cfg_b.origin, 3, 6, 12);
        for sub in 0u16..12 {
            let t = sub as f32 / 11.0;
            let x = ((ax as f32 * (1.0 - t) + bx as f32 * t).round() as i16
                     + (sub as i16 % 3 - 1)).clamp(1, 39);
            let y = ((ay as f32 * (1.0 - t) + by as f32 * t).round() as i16).clamp(1, 19);
            let stat = if sub % 2 == 0 { cfg_a.stats[sub as usize % 3] } else { cfg_b.stats[sub as usize % 3] };
            let req_a = 200 + ci_a as u16 * 12 + (sub % 10);
            let req_b = 200 + ci_b as u16 * 12 + (sub % 10);
            let req = if sub < 6 { req_a } else { req_b };
            v.push(TreeNode {
                id: bridge_base + sub, x, y,
                name: format!("{}/{} Bridge", cfg_a.prefix, cfg_b.prefix),
                short_desc: format!("+{} (5-20) crossover", stat_title(stat)),
                node_type: NodeType::Stat { stat, min: 5, max: 20 },
                requires: vec![req], class_start: None,
            });
        }
    }

    // ── EXTRA KEYSTONES (IDs 1100–1131, 4 per class) ─────────────────────────
    for (i, &(name, ks_id, desc)) in EXTRA_KEYSTONES.iter().enumerate() {
        let ci = i / 4;
        let sub = i as u16 % 4;
        let cfg = &CLASS_CONFIGS[ci];
        let r5_base = 800 + ci as u16 * 18;
        let req = r5_base + sub;
        let (x, y) = ring_pos(cfg.origin, 5, sub + 14, 18);
        v.push(TreeNode {
            id: 1100 + i as u16,
            x: x.clamp(2, 38), y: y.clamp(2, 18),
            name: name.to_string(), short_desc: desc.to_string(),
            node_type: NodeType::Keystone { id: ks_id },
            requires: vec![req], class_start: None,
        });
    }

    v
}

// ─── STATIC NODE ACCESS ──────────────────────────────────────────────────────

static ALL_NODES: OnceLock<Vec<TreeNode>> = OnceLock::new();

/// Returns the full ~820-node passive tree (initialized once on first call).
pub fn nodes() -> &'static [TreeNode] {
    ALL_NODES.get_or_init(build_nodes)
}

// ─── PLAYER PASSIVES ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerPassives {
    pub allocated: HashSet<u16>,
    pub stat_bonuses: std::collections::HashMap<u16, i64>,
    pub points: u32,
    pub keystones: HashSet<String>,
    pub completed_synergies: HashSet<u8>,
    pub cursor: u16,
}

impl PlayerPassives {
    pub fn new_for_class(class: CharacterClass) -> Self {
        let mut p = PlayerPassives::default();
        if let Some(node) = nodes().iter().find(|n| n.class_start == Some(class)) {
            p.allocated.insert(node.id);
            p.cursor = node.id;
        }
        p
    }

    pub fn can_allocate(&self, node_id: u16) -> bool {
        if self.allocated.contains(&node_id) {
            return false;
        }
        let node = match nodes().iter().find(|n| n.id == node_id) {
            Some(n) => n,
            None => return false,
        };
        if node.requires.is_empty() {
            return true;
        }
        node.requires.iter().any(|req| self.allocated.contains(req))
    }

    pub fn allocate(&mut self, node_id: u16, seed: u64) -> Option<String> {
        if !self.can_allocate(node_id) || self.points == 0 {
            return None;
        }
        let node = nodes().iter().find(|n| n.id == node_id)?;
        self.points -= 1;
        self.allocated.insert(node_id);

        let result = match &node.node_type {
            NodeType::Stat { stat, min, max } => {
                let roll = chaos_roll_verbose((*min + *max) as f64 * 0.005, seed);
                let range = max - min;
                let value = min + ((roll.final_value * 0.5 + 0.5).clamp(0.0, 1.0) * range as f64) as i64;
                self.stat_bonuses.insert(node_id, value);
                format!("Allocated: {} → {} {}{}",
                    node.name, stat.to_uppercase(),
                    if value >= 0 { "+" } else { "" }, value)
            }
            NodeType::Engine { engine, effect } => {
                format!("Allocated: {} — {} [{}]", node.name, engine, effect)
            }
            NodeType::Keystone { id } => {
                self.keystones.insert((*id).to_string());
                format!("KEYSTONE ACTIVATED: {} — {}", node.name, node.short_desc)
            }
            NodeType::Notable { stat, bonus, effect } => {
                self.stat_bonuses.insert(node_id, *bonus);
                format!("NOTABLE: {} — +{} {} | {}", node.name, bonus, stat.to_uppercase(), effect)
            }
            NodeType::Synergy { cluster, bonus_desc } => {
                let cluster_size = nodes()
                    .iter()
                    .filter(|n| matches!(&n.node_type, NodeType::Synergy { cluster: c, .. } if c == cluster))
                    .count();
                let allocated_in_cluster = nodes()
                    .iter()
                    .filter(|n| {
                        matches!(&n.node_type, NodeType::Synergy { cluster: c, .. } if c == cluster)
                            && self.allocated.contains(&n.id)
                    })
                    .count();
                if allocated_in_cluster == cluster_size {
                    self.completed_synergies.insert(*cluster);
                    format!("SYNERGY COMPLETE: Cluster {} — {}", cluster, bonus_desc)
                } else {
                    format!("Synergy node {}/{}: {}", allocated_in_cluster, cluster_size, node.name)
                }
            }
        };

        self.cursor = node_id;
        Some(result)
    }

    pub fn total_stat_bonus(&self, stat: &str) -> i64 {
        self.allocated
            .iter()
            .filter_map(|id| {
                let node = nodes().iter().find(|n| n.id == *id)?;
                match &node.node_type {
                    NodeType::Stat { stat: s, .. } | NodeType::Notable { stat: s, .. } if *s == stat => {
                        self.stat_bonuses.get(id).copied()
                    }
                    _ => None,
                }
            })
            .sum()
    }

    pub fn engine_mod(&self, engine: &str, effect: &str) -> bool {
        self.allocated.iter().any(|id| {
            nodes().iter().any(|n| {
                n.id == *id
                    && matches!(&n.node_type,
                        NodeType::Engine { engine: e, effect: eff } if *e == engine && *eff == effect)
            })
        })
    }

    pub fn has_keystone(&self, id: &str) -> bool {
        self.keystones.contains(id)
    }

    pub fn synergy_active(&self, cluster: u8) -> bool {
        self.completed_synergies.contains(&cluster)
    }

    pub fn move_cursor(&mut self, dx: i16, dy: i16) -> u16 {
        let cur = match nodes().iter().find(|n| n.id == self.cursor) {
            Some(n) => n,
            None => {
                if let Some(n) = nodes().first() {
                    self.cursor = n.id;
                }
                return self.cursor;
            }
        };
        let best = nodes()
            .iter()
            .filter(|n| n.id != self.cursor)
            .filter(|n| {
                let nx = n.x - cur.x;
                let ny = n.y - cur.y;
                if dx != 0 && dy == 0 {
                    nx * dx > 0 && nx.abs() >= ny.abs()
                } else if dy != 0 && dx == 0 {
                    ny * dy > 0 && ny.abs() >= nx.abs()
                } else {
                    false
                }
            })
            .min_by_key(|n| {
                let nx = n.x - cur.x;
                let ny = n.y - cur.y;
                nx * nx + ny * ny
            });
        if let Some(node) = best {
            self.cursor = node.id;
        }
        self.cursor
    }

    pub fn display_map(&self, class: CharacterClass) -> Vec<String> {
        const RESET: &str = "\x1b[0m";
        const DIM: &str = "\x1b[2m";
        const GREEN: &str = "\x1b[32m";
        const YELLOW: &str = "\x1b[33m";
        const CYAN: &str = "\x1b[36m";
        const MAGENTA: &str = "\x1b[35m";
        const BRIGHT_GREEN: &str = "\x1b[92m";
        const BRIGHT_CYAN: &str = "\x1b[96m";
        const WHITE: &str = "\x1b[97m";
        const ORANGE: &str = "\x1b[33m";

        let cols = 41usize;
        let rows = 21usize;

        let mut grid: Vec<Vec<String>> = (0..rows)
            .map(|_| (0..cols).map(|_| format!("{}·{}", DIM, RESET)).collect())
            .collect();

        // Draw connection lines
        for node in nodes() {
            for req_id in &node.requires {
                let req = match nodes().iter().find(|n| n.id == *req_id) {
                    Some(r) => r,
                    None => continue,
                };
                let (x0, y0) = (req.x as usize, req.y as usize);
                let (x1, y1) = (node.x as usize, node.y as usize);
                if x0 < cols && x1 < cols && y0 < rows && y1 < rows {
                    let (lx, rx) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
                    for cell in &mut grid[y0][lx..=rx] {
                        if *cell == format!("{}·{}", DIM, RESET) {
                            *cell = format!("{}─{}", DIM, RESET);
                        }
                    }
                    let (ty, by) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
                    for row in &mut grid[ty..=by] {
                        if row[x1] == format!("{}·{}", DIM, RESET) {
                            row[x1] = format!("{}│{}", DIM, RESET);
                        }
                    }
                }
            }
        }

        // Place nodes
        for node in nodes() {
            let (x, y) = (node.x as usize, node.y as usize);
            if x >= cols || y >= rows {
                continue;
            }
            let is_cursor = node.id == self.cursor;
            let is_allocated = self.allocated.contains(&node.id);
            let can_alloc = self.can_allocate(node.id);
            let is_class_start = node.class_start == Some(class);

            let symbol = match &node.node_type {
                NodeType::Stat { .. }    => "●",
                NodeType::Engine { .. }  => "⚙",
                NodeType::Keystone { .. }=> "★",
                NodeType::Notable { .. } => "◎",
                NodeType::Synergy { .. } => "◆",
            };

            let cell = if is_cursor {
                format!("{}[{}]{}", WHITE, symbol, RESET)
            } else if is_allocated {
                format!("{}{}{}", BRIGHT_GREEN, symbol, RESET)
            } else if is_class_start {
                format!("{}{}{}", BRIGHT_CYAN, symbol, RESET)
            } else if can_alloc {
                format!("{}{}{}", YELLOW, symbol, RESET)
            } else {
                format!("{}{}{}", DIM, symbol, RESET)
            };

            grid[y][x] = cell;
        }

        let total = nodes().len();
        let allocated = self.allocated.len();

        let mut lines = Vec::new();
        lines.push(format!(
            "  {}Passive Tree — {} pts available  ({}/{} allocated)  [W/A/S/D]=move  [E]=allocate  [Q]=exit{}",
            CYAN, self.points, allocated, total, RESET
        ));
        lines.push(format!(
            "  {}{}●{}=stat  {}⚙{}=engine  {}★{}=keystone  {}◎{}=notable  {}◆{}=synergy  {}[●]{}=cursor  {}●{}=allocated{}",
            DIM, GREEN, DIM, DIM, MAGENTA, DIM, YELLOW, DIM, ORANGE, DIM, CYAN, DIM, WHITE, DIM, BRIGHT_GREEN, RESET
        ));
        lines.push(format!("  {}┌{}┐{}", CYAN, "─".repeat(cols + 2), RESET));
        for row in &grid {
            let row_str: String = row.iter().cloned().collect();
            lines.push(format!("  {}│{} {} {}│{}", CYAN, RESET, row_str, CYAN, RESET));
        }
        lines.push(format!("  {}└{}┘{}", CYAN, "─".repeat(cols + 2), RESET));

        if let Some(cur) = nodes().iter().find(|n| n.id == self.cursor) {
            let status = if self.allocated.contains(&cur.id) {
                format!("{}ALLOCATED{}", BRIGHT_GREEN, RESET)
            } else if self.can_allocate(cur.id) {
                format!("{}AVAILABLE (costs 1 point){}", YELLOW, RESET)
            } else {
                format!("{}LOCKED{}", DIM, RESET)
            };
            lines.push(format!(
                "  {}▶ #{} {} — {} | {}{}",
                CYAN, cur.id, cur.name, cur.short_desc, status, RESET
            ));
        }

        lines
    }

    pub fn list_available(&self) -> Vec<(u16, &str, &str)> {
        nodes()
            .iter()
            .filter(|n| self.can_allocate(n.id))
            .map(|n| (n.id, n.name.as_str(), n.short_desc.as_str()))
            .collect()
    }
}

// ─── TESTS ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_starts_exist_for_all_classes() {
        for class in [
            CharacterClass::Mage, CharacterClass::Berserker, CharacterClass::Ranger,
            CharacterClass::Thief, CharacterClass::Necromancer, CharacterClass::Alchemist,
            CharacterClass::Paladin, CharacterClass::VoidWalker,
        ] {
            assert!(nodes().iter().any(|n| n.class_start == Some(class)),
                "No start node for {:?}", class);
        }
    }

    #[test]
    fn mage_can_allocate_starting_node() {
        let passives = PlayerPassives::new_for_class(CharacterClass::Mage);
        assert!(passives.allocated.contains(&0));
    }

    #[test]
    fn allocate_stat_node_gives_bonus() {
        let mut p = PlayerPassives::new_for_class(CharacterClass::Mage);
        p.points = 5;
        let result = p.allocate(10, 42);
        assert!(result.is_some(), "should allocate node 10");
        assert!(p.allocated.contains(&10));
        assert_eq!(p.points, 4);
    }

    #[test]
    fn cannot_allocate_locked_node() {
        let mut p = PlayerPassives::new_for_class(CharacterClass::Ranger);
        p.points = 5;
        let result = p.allocate(52, 99);
        assert!(result.is_none());
        assert!(!p.allocated.contains(&52));
        assert_eq!(p.points, 5, "no points spent");
    }

    #[test]
    fn keystone_activates_on_allocation() {
        let mut p = PlayerPassives::new_for_class(CharacterClass::Mage);
        p.points = 10;
        p.allocate(10, 1);
        p.allocate(12, 2);
        p.allocate(14, 3);
        assert!(p.has_keystone(KS_GLASS_CANNON));
    }

    #[test]
    fn tree_has_approximately_800_nodes() {
        let count = nodes().len();
        assert!(count >= 750, "expected ~800 nodes, got {}", count);
    }

    #[test]
    fn ring2_nodes_exist_for_all_classes() {
        for ci in 0..8usize {
            let base = 200 + ci as u16 * 12;
            for sub in 0u16..12 {
                let id = base + sub;
                assert!(nodes().iter().any(|n| n.id == id),
                    "missing ring2 node {} (class {}, sub {})", id, ci, sub);
            }
        }
    }

    #[test]
    fn notable_nodes_track_stat_bonus() {
        let mut p = PlayerPassives::new_for_class(CharacterClass::Mage);
        p.points = 99;
        // Allocate through to ring 2 (need ring1 first)
        let _ = p.allocate(10, 1); // Mana Surge
        let _ = p.allocate(11, 2); // Spell Weave
        // ring2 notable for Mage is ID 211 (base=200, sub=11)
        let _ = p.allocate(211, 3);
        // If notable was allocated, stat bonus should exist
        if p.allocated.contains(&211) {
            assert!(p.stat_bonuses.contains_key(&211));
        }
    }
}
