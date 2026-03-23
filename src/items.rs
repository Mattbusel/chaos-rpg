//! Item generation — every item is a chaos-rolled catastrophe.

use crate::chaos_pipeline::{chaos_roll_verbose, roll_stat};
use serde::{Deserialize, Serialize};

// ─── Tables ──────────────────────────────────────────────────────────────────

// Weapons occupy indices 0..WEAPON_COUNT-1; everything else is non-weapon.
const WEAPON_COUNT: usize = 15;
const BASE_TYPES: &[&str] = &[
    // ── Weapons (0-14) ──────────────────────────────────────────────────────
    "Dagger",          // 0
    "Sword",           // 1
    "Axe",             // 2
    "Mace",            // 3
    "Wand",            // 4
    "Scepter",         // 5
    "Claw",            // 6
    "Rapier",          // 7
    "Greatsword",      // 8
    "Greataxe",        // 9
    "Staff",           // 10
    "Longbow",         // 11
    "Crossbow",        // 12
    "Halberd",         // 13
    "War Scythe",      // 14
    // ── Head Armor (15-18) ──────────────────────────────────────────────────
    "Helm",            // 15
    "Hood",            // 16
    "Crown",           // 17
    "Mask",            // 18
    // ── Torso Armor (19-22) ─────────────────────────────────────────────────
    "Plate",           // 19
    "Chainmail",       // 20
    "Robes",           // 21
    "Leather Vest",    // 22
    // ── Hand Armor (23-24) ──────────────────────────────────────────────────
    "Gauntlets",       // 23
    "Gloves",          // 24
    // ── Foot Armor (25-26) ──────────────────────────────────────────────────
    "Boots",           // 25
    "Greaves",         // 26
    // ── Accessories (27-34) ─────────────────────────────────────────────────
    "Ring",            // 27
    "Amulet",          // 28
    "Belt",            // 29
    "Cape",            // 30
    "Quiver",          // 31
    "Tome",            // 32
    "Orb",             // 33
    "Shield",          // 34
    // ── Special / Cursed (35-44) ─────────────────────────────────────────────
    "Focus Crystal",              // 35
    "Chaos Shard",                // 36
    "Placeholder That Became Real", // 37
    "Error",                      // 38
    "Mysterious Object",          // 39
    "Bottled Equation",           // 40
    "Portable Hole",              // 41
    "The Concept of an Item",     // 42
    "Prime Shard",                // 43
    "Fractal Lens",               // 44
];

/// Implicit modifier text shown under base type in item box.
const BASE_TYPE_IMPLICIT: &[&str] = &[
    // Weapons
    "+Crit Chance",       // Dagger
    "+Flat Damage",       // Sword
    "+Cleave Chance",     // Axe
    "+Stun Chance",       // Mace
    "+Mana on Kill",      // Wand
    "+Spell Damage %",    // Scepter
    "+Attack Speed",      // Claw
    "+Pierce Chance",     // Rapier
    "+AoE on Kill",       // Greatsword
    "+Brutality %",       // Greataxe
    "+Spell Damage %",    // Staff
    "+First Hit Bonus",   // Longbow
    "+Reload Crit",       // Crossbow
    "+Sweep Damage",      // Halberd
    "+Bleed on Hit",      // War Scythe
    // Head
    "+Head HP",           // Helm
    "+Evasion %",         // Hood
    "+All Stats",         // Crown
    "+Accuracy %",        // Mask
    // Torso
    "+Flat Defense",      // Plate
    "+Balanced Defense",  // Chainmail
    "+Mana %",            // Robes
    "+Dodge Chance",      // Leather Vest
    // Hands
    "+Force %",           // Gauntlets
    "+Precision %",       // Gloves
    // Feet
    "+Flee Chance %",     // Boots
    "+Leg HP",            // Greaves
    // Accessories
    "+Random Stat",       // Ring
    "+All Stats",         // Amulet
    "+Max HP %",          // Belt
    "+Evasion %",         // Cape
    "+Range Damage",      // Quiver
    "+Spell Slots",       // Tome
    "+Engine Count",      // Orb
    "+Block Chance %",    // Shield
    // Special
    "+Focus Bonus",       // Focus Crystal
    "+Chaos Amplitude",   // Chaos Shard
    "+All Stats",         // Placeholder
    "+Error Bonus",       // Error
    "+Mystery",           // Mysterious Object
    "+Equation Power",    // Bottled Equation
    "+Void Access",       // Portable Hole
    "+Conceptual Damage", // The Concept of an Item
    "+Prime Bonus",       // Prime Shard
    "+Clarity",           // Fractal Lens
];

/// 50 materials across 7 tiers. Higher tier = wider stat ranges and more mods.
const MATERIALS: &[&str] = &[
    // ── Tier 1 (0-7) — common, modest rolls ─────────────────────────────────
    "wood",
    "iron",
    "leather",
    "cloth",
    "bone",
    "stone",
    "copper",
    "tin",
    // ── Tier 2 (8-15) — uncommon ─────────────────────────────────────────────
    "steel",
    "silver",
    "darkwood",
    "silk",
    "obsidian",
    "crystal",
    "bronze",
    "jade",
    // ── Tier 3 (16-23) — rare ────────────────────────────────────────────────
    "mithril",
    "titanium",
    "dragonscale",
    "spidersilk",
    "volcanic glass",
    "moonstone",
    "electrum",
    "starmetal",
    // ── Tier 4 (24-31) — epic ────────────────────────────────────────────────
    "adamantine",
    "orichalcum",
    "ethereal silk",
    "phoenix feather",
    "frozen time",
    "liquid shadow",
    "prismatic crystal",
    "singing steel",
    // ── Tier 5 (32-39) — legendary ───────────────────────────────────────────
    "dark matter",
    "antimatter",
    "condensed screaming",
    "crystallized luck",
    "solidified math",
    "weaponized optimism",
    "bottled lightning",
    "decompiled soul",
    // ── Tier 6 (40-47) — mythical/divine ────────────────────────────────────
    "eigenstate alloy",
    "superposition glass",
    "non-euclidean bone",
    "prime-factored obsidian",
    "Turing-complete leather",
    "compressed infinity",
    "deterministic void",
    "recursive adamantine",
    // ── Tier 7 (48-49) — beyond/artifact ─────────────────────────────────────
    "the concept of sharpness",
    "Gödel's incompleteness metal",
];

/// Tier of each material (parallel to MATERIALS). Tier 1 = weakest, 7 = strongest.
const MATERIAL_TIERS: &[u32] = &[
    1,1,1,1,1,1,1,1, // tier 1
    2,2,2,2,2,2,2,2, // tier 2
    3,3,3,3,3,3,3,3, // tier 3
    4,4,4,4,4,4,4,4, // tier 4
    5,5,5,5,5,5,5,5, // tier 5
    6,6,6,6,6,6,6,6, // tier 6
    7,7,              // tier 7
];

const ADJECTIVES: &[&str] = &[
    "of the Forgotten",
    "of Absolute Tuesday",
    "of Infinite Regret",
    "of Suspicious Origin",
    "the Unbreakable (breaks immediately)",
    "of Certain Doom",
    "of Mild Inconvenience",
    "of the Last Algorithm",
    "that Shouldn't Exist",
    "of Someone Else",
    "of Accidental Greatness",
    "of Yesterday's Problems",
    "of Mathematical Inevitability",
    "of Schrödinger",
    "of the Prime Manifold",
    "of Undecidable Truth",
    "that Observes You Back",
    "of Non-Euclidean Design",
    "of the Lorenz Attractor",
    "of Bifurcation Point",
    "of the Omega Constant",
    "beyond the Mandelbrot Set",
];

const SPECIAL_EFFECTS: &[&str] = &[
    // ── ON-HIT (0-19) ────────────────────────────────────────────────────────
    "10% chance to inflict BURNING on hit (3 rounds)",
    "15% chance to STUN target for 1 round on hit",
    "8% chance to FREEZE target for 2 rounds on hit",
    "12% chance to apply POISON on hit (4 rounds)",
    "heals 5% of damage dealt as HP",
    "steals 10 of enemy's highest stat for the combat",
    "15% chance to strike twice (second hit at 50% damage)",
    "armor penetration: ignores 20% of enemy defense",
    "applies a stack of Decay on hit (+3% enemy damage per stack)",
    "vampiric: heals you for damage dealt, damages you for enemy heal",
    "reduces enemy Entropy by 15 per hit",
    "10% chance to execute enemies below 20% HP",
    "heals enemies on hit",
    "deals damage to you equal to damage dealt to enemy",
    "20% chance to phase through incoming attacks",
    "combo attacks deal double the normal combo bonus",
    "laughs at the concept of defense stats",
    "the prime factorization of its damage is always prime",
    "all critical hits deal 3× damage instead of 2×",
    "doubles all chaos rolls while equipped",
    // ── ON-KILL (20-34) ──────────────────────────────────────────────────────
    "explodes the corpse for 25 area damage on kill",
    "20% chance to drop an extra item on kill",
    "permanently raises max HP by 3 on kill",
    "permanently raises a random stat by 1 on kill",
    "gains Momentum on kill: +15% damage until hit",
    "restores 20% mana on kill",
    "reduces all spell cooldowns by 1 on kill",
    "15% chance enemy drops a gem on kill",
    "gold drops doubled for this kill",
    "next engine chain uses +1 engine after a kill",
    "heals a random body part injury on kill",
    "corruption decreases by 1 stack on kill",
    "XP tripled if the kill was a boss",
    "each kill permanently raises max HP by 5",
    "spell costs become negative: mana regenerates on cast",
    // ── DEFENSIVE (35-49) ────────────────────────────────────────────────────
    "block 15 flat damage from all attacks",
    "+15% dodge chance while below 50% HP",
    "when hit, 20% chance to retaliate for 30 damage",
    "regenerate 4 HP per round",
    "shield: absorbs 80 damage then recharges each combat",
    "survive one lethal hit per floor at 1 HP",
    "reduces incoming crit damage by 30%",
    "15% of incoming damage converted to mana",
    "status effect durations reduced by 1 round",
    "when stunned, automatically defend that turn",
    "enemies that hit you take 10 reflected damage",
    "body part damage redirected from head to torso",
    "immune to status effects (while conscious)",
    "each kill permanently raises max HP by 5 (defensive version)",
    "+25% defense when HP below 30%",
    // ── SPELL (50-62) ────────────────────────────────────────────────────────
    "+25% damage for fire-school spells",
    "+25% damage for ice-school spells",
    "+25% damage for lightning-school spells",
    "+25% damage for arcane-school spells",
    "+25% damage for shadow-school spells",
    "spells cost 15% less mana",
    "10% chance to cast any spell twice at no extra mana cost",
    "spell backfire damage reduced by 40%",
    "healing spells also deal damage equal to healing done",
    "damage spells also heal 8% of damage dealt",
    "when a spell crits, its cooldown resets",
    "spells add 1 combo stack",
    "grants a vision of your next enemy's weakness (blurry)",
    // ── ENGINE (63-74) ────────────────────────────────────────────────────────
    "forces Lorenz Attractor into every chain",
    "forces Mandelbrot Escape into every chain",
    "forces Fibonacci Spiral into every chain",
    "removes Lorenz Attractor from all chains",
    "all engine outputs shifted by +0.2",
    "engine chain is reversed (last engine goes first)",
    "engines returning near-zero are rerolled once",
    "the final engine in the chain runs twice",
    "engine chains have +1 minimum length",
    "doubles one chaos die per combat",
    "if all engines positive, double the final result",
    "rerolls one chaos die per combat (unverified)",
    // ── WEIRD / FLAVOR (75-114) ───────────────────────────────────────────────
    "teleports wielder randomly each turn",
    "attracts bees",
    "makes you invisible but also blind",
    "screams",
    "is slightly damp",
    "occasionally becomes a different item",
    "has opinions",
    "phases in and out of reality",
    "smells incredible",
    "makes all NPCs call you 'mother'",
    "reverses gravity within 10 feet",
    "speaks in riddles about the future (riddles are wrong)",
    "gives you +9999 INT but only while you're not thinking about it",
    "deals bonus damage to concepts",
    "is haunted by its previous owner (they're fine with it)",
    "None (but that itself is suspicious)",
    "exists in superposition: both useful and useless until observed",
    "grows stronger: +1% all stats per floor",
    "shrinks each floor: -1% all stats per floor",
    "detects traps in adjacent rooms",
    "detects and names boss before entering boss room",
    "your damage numbers display in hexadecimal",
    "all combat text CAPITALIZED while equipped",
    "the item's name changes every floor",
    "gets jealous of other items: occasionally unequips one",
    "is afraid of the dark: stats halved in dark rooms",
    "attracts enemies: 10% more encounters",
    "repels enemies: 10% fewer encounters",
    "contains a smaller item inside it",
    "is technically a weapon even if it is a hat",
    "was briefly president of something",
    "exists only on Tuesdays (other days: nothing)",
    "has a warranty card (expired)",
    "came with a manual but the manual is also the item",
    "makes you walk backwards but faster",
    "your gold value displayed as imaginary numbers",
    "cannot be equipped but provides full benefits anyway",
    "is haunted by the concept of itself",
    "briefly existed outside of time",
    "has won several awards (for what is unclear)",
    "once belonged to someone who knew what they were doing",
    "made by a craftsman who quit afterward",
    "emits a hum that sounds like Beethoven's 5th played by a bumblebee",
    // ── MYTHICAL / BUILD-DEFINING (115-130) ────────────────────────────────────
    "INFINITY MIRROR: all stat modifiers doubled (including negatives)",
    "CHAOS ANCHOR: all chains use a fixed seed (same result every roll)",
    "THE RECURSION: damage triggers second hit at 50%, third at 25%, etc. Total = ×2",
    "PHASE BLADE: all damage ignores enemy defense entirely",
    "EIGENWEAPON: damage equals enemy's highest stat value",
    "PERPETUAL MOTION: generates 1 mana per round (guaranteed, not chaos-rolled)",
    "OVERKILL BANKING: excess kill damage stored, applied to next hit at 50%",
    "THE ATTRACTOR: item drop chance doubled, drops at +1 rarity tier",
    "ASYMPTOTIC APPROACH: damage bonus scales toward infinity as HP approaches 0",
    "L'HÔPITAL'S SHIELD: survive one lethal hit per floor at 1 HP (resets per floor)",
    "BANACH-TARSKI: split into two half-stat copies each combat; recombines after",
    "ZENO'S SHIELD: each consecutive hit this combat takes half damage of previous",
    "HALTING PROBLEM: after round 20, all your rolls automatically succeed",
    "ERGODIC THEOREM: every 100 rolls, your average outcome becomes a flat bonus",
    "PROOF BY CONTRADICTION: 25% chance CATASTROPHE rolls become CRITICAL instead",
    "CONSTRUCTIVE INTERFERENCE: 3+ same-sign engines in chain multiplies output ×1.5",
    // ── LEGACY / HUMOR (131+) ────────────────────────────────────────────────
    "was used as currency in a civilization that no longer exists",
    "remembers every enemy it has killed (most were fine with it)",
    "contains trace amounts of a different dimension",
    "was described as 'adequate' in a review from 1847",
    "still has the price tag from before money was invented",
    "occasionally apologizes",
    "forgets what it's doing mid-combat",
    "claims to be a completely different item",
    "was lost for 400 years and is very confused",
    "considers itself to be the main character",
    "has strong opinions about the Riemann hypothesis",
    "is currently in a legal dispute with another item",
    "was briefly banned in 3 jurisdictions for unexplained reasons",
    "radiates mild existential dread",
    "is technically older than the universe",
    "was voted 'Most Likely to Succeed' by the other items",
    "secretly controls the weather in a small region",
    "has a sidekick (the sidekick is also this item)",
    "achieved enlightenment but came back because the loot was better",
    "is unironically the best thing that ever happened to someone (not you though)",
];

const STAT_NAMES: &[&str] = &[
    "Vitality",
    "Force",
    "Mana",
    "Cunning",
    "Precision",
    "Entropy",
    "Luck",
];

// ─── Rarity ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythical,
    Divine,
    Beyond,
    Artifact, // unique: one-of-a-kind chaos-generated masterpiece
}

impl Rarity {
    pub fn from_magnitude(mag: i64) -> Self {
        let m = mag.abs();
        match m {
            0..=10 => Rarity::Common,
            11..=50 => Rarity::Uncommon,
            51..=200 => Rarity::Rare,
            201..=1000 => Rarity::Epic,
            1001..=5000 => Rarity::Legendary,
            5001..=20000 => Rarity::Mythical,
            20001..=99999 => Rarity::Divine,
            100000..=999999 => Rarity::Beyond,
            _ => Rarity::Artifact,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Uncommon => "Uncommon",
            Rarity::Rare => "Rare",
            Rarity::Epic => "Epic",
            Rarity::Legendary => "Legendary",
            Rarity::Mythical => "Mythical",
            Rarity::Divine => "Divine",
            Rarity::Beyond => "???",
            Rarity::Artifact => "◈ ARTIFACT ◈",
        }
    }

    pub fn color_code(self) -> &'static str {
        match self {
            Rarity::Common => "\x1b[90m",    // dark grey
            Rarity::Uncommon => "\x1b[37m",  // white
            Rarity::Rare => "\x1b[32m",      // green
            Rarity::Epic => "\x1b[34m",      // blue
            Rarity::Legendary => "\x1b[35m", // magenta
            Rarity::Mythical => "\x1b[33m",  // yellow
            Rarity::Divine => "\x1b[31m",    // red
            Rarity::Beyond => "\x1b[96m",    // bright cyan
            Rarity::Artifact => "\x1b[97m",  // bright white (blinding)
        }
    }
}

// ─── Stat Modifier ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatModifier {
    pub stat: String,
    pub value: i64,
}

impl StatModifier {
    /// Generate a single chaos-rolled stat modifier from a seed.
    pub fn generate_random(seed: u64) -> Self {
        use crate::chaos_pipeline::{chaos_roll_verbose, roll_stat};
        let roll = chaos_roll_verbose(0.0, seed);
        let stat_idx = (seed % STAT_NAMES.len() as u64) as usize;
        let value = (roll.final_value * 2000.0) as i64 + roll_stat(-500, 500, seed.wrapping_add(1));
        StatModifier {
            stat: STAT_NAMES[stat_idx].to_string(),
            value,
        }
    }
}

// ─── Gem ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GemType {
    Skill,   // active ability socketed here
    Support, // modifies linked skill gems
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gem {
    pub name: String,
    pub gem_type: GemType,
    pub description: String,
    /// Engine modifier tag (e.g. "AddedChaos", "Fork", "ControlledDestruction")
    pub tag: String,
}

impl Gem {
    pub fn generate(seed: u64) -> Self {
        const SUPPORT_GEMS: &[(&str, &str, &str)] = &[
            (
                "Added Chaos Damage",
                "Adds an extra engine to the damage chain",
                "AddedChaos",
            ),
            (
                "Fork",
                "Spell hits two targets; each uses a separate chaos roll",
                "Fork",
            ),
            (
                "Controlled Destruction",
                "Removes Lorenz Attractor from chain (more predictable)",
                "ControlledDestruction",
            ),
            (
                "Increased Critical",
                "Crit threshold lowered from 90 to 70",
                "IncreasedCrit",
            ),
            (
                "Cast on Death",
                "Linked skill auto-casts when you die",
                "CastOnDeath",
            ),
            (
                "Chaos Amplification",
                "All engine outputs in chain are squared",
                "ChaosAmplify",
            ),
            (
                "Spell Echo",
                "Spell fires twice but second cast uses half engines",
                "SpellEcho",
            ),
            (
                "Minefield",
                "Skill deploys a mine; detonates on enemy turn",
                "Minefield",
            ),
            (
                "Concentrated Effect",
                "Area reduced to single target; damage x1.5",
                "ConcentratedEffect",
            ),
            (
                "Void Shot",
                "Skill pierces target and hits the floor for AoE",
                "VoidShot",
            ),
        ];
        const SKILL_GEMS: &[(&str, &str)] = &[
            ("Void Bolt", "Fires a bolt of compressed void energy"),
            (
                "Prime Explosion",
                "Detonates prime-factored energy at target",
            ),
            ("Entropy Cascade", "Chains entropy through adjacent enemies"),
            (
                "Lorenz Storm",
                "Summons a butterfly-effect storm; scales chaotically",
            ),
            (
                "Collatz Spiral",
                "3n+1 damage chain; unpredictable altitude",
            ),
            (
                "Fibonacci Arc",
                "Arc of golden-ratio energy; scales with streak",
            ),
            (
                "Mandelbrot Pulse",
                "Pulse from the set boundary; damages phase-shifted enemies",
            ),
            ("Zeta Wave", "Riemann zeta resonance wave"),
        ];

        let is_support = !seed.is_multiple_of(3); // 66% support gems
        if is_support {
            let idx = (seed.wrapping_mul(54321) % SUPPORT_GEMS.len() as u64) as usize;
            let (name, desc, tag) = SUPPORT_GEMS[idx];
            Gem {
                name: name.to_string(),
                gem_type: GemType::Support,
                description: desc.to_string(),
                tag: tag.to_string(),
            }
        } else {
            let idx = (seed.wrapping_mul(12345) % SKILL_GEMS.len() as u64) as usize;
            let (name, desc) = SKILL_GEMS[idx];
            Gem {
                name: name.to_string(),
                gem_type: GemType::Skill,
                description: desc.to_string(),
                tag: "Skill".to_string(),
            }
        }
    }
}

// ─── Item ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub base_type: String,
    /// Implicit modifier text (from base type, fixed at generation)
    pub implicit: String,
    pub damage_or_defense: i64,
    pub stat_modifiers: Vec<StatModifier>,
    pub special_effect: String,
    pub rarity: Rarity,
    pub is_weapon: bool,
    pub value: i64,
    /// Floor number when this item was generated (affects roll ranges)
    pub item_level: u32,
    /// Material tier (1-7) for display and scaling
    pub material_tier: u32,
    /// Number of gem sockets (0-6, chaos-rolled at generation)
    pub socket_count: u8,
    /// Number of linked socket pairs
    pub socket_links: u8,
    /// Socketed gems (up to socket_count)
    pub socketed_gems: Vec<Gem>,
    /// Engine locks: "+EngineName" = always include, "-EngineName" = always exclude
    pub engine_locks: Vec<String>,
    /// Corruption implicit (if corrupted)
    pub corruption: Option<String>,
}

impl Item {
    /// Generate an item at floor level 1 (backward-compatible wrapper).
    pub fn generate(seed: u64) -> Self {
        Self::generate_leveled(seed, 1)
    }

    /// Generate an item scaled to `item_level` (= floor number).
    /// Higher item_level + higher material tier = wider stat ranges and more mods.
    pub fn generate_leveled(seed: u64, item_level: u32) -> Self {
        let base_idx = (seed % BASE_TYPES.len() as u64) as usize;
        let mat_idx = ((seed.wrapping_mul(1234567)) % MATERIALS.len() as u64) as usize;
        let adj_idx = ((seed.wrapping_mul(9876543)) % ADJECTIVES.len() as u64) as usize;

        let base_type = BASE_TYPES[base_idx].to_string();
        let implicit = BASE_TYPE_IMPLICIT[base_idx].to_string();
        let material = MATERIALS[mat_idx];
        let material_tier = MATERIAL_TIERS[mat_idx];
        let adjective = ADJECTIVES[adj_idx];
        let name = format!("{} {} {}", material, base_type, adjective);

        let is_weapon = base_idx < WEAPON_COUNT;

        // Scale multiplier: combines item_level and material tier.
        // tier 1 + ilevel 1 → ~1× rolls; tier 7 + ilevel 80 → ~6× rolls
        let tier_mult = (material_tier as f64 * 0.5 + 0.5) // 1.0 to 4.0
            * (1.0 + item_level as f64 * 0.02).min(3.0);  // capped at 3× from level

        // Damage/defense — chaos roll scaled by tier
        let dmg_seed = seed.wrapping_add(111);
        let dmg_roll = chaos_roll_verbose(dmg_seed as f64 * 1e-9, dmg_seed);
        let base_range = (500.0 * tier_mult) as i64;
        let damage_or_defense = (dmg_roll.final_value * base_range as f64) as i64
            + roll_stat(-base_range / 2, base_range / 2, seed.wrapping_add(222));

        // Mod count: tier 1 = 1-2, tier 4 = 2-4, tier 7 = 4-6; also scales with ilevel
        let base_mods = (material_tier / 2).max(1) as usize;
        let level_bonus = ((item_level / 20) as usize).min(2);
        let max_mods = (base_mods + level_bonus).min(6);
        let n_mods = 1 + (seed.wrapping_mul(17) % max_mods as u64) as usize;

        let mut stat_modifiers = Vec::new();
        let mut total_magnitude = damage_or_defense.abs();

        for i in 0..n_mods {
            let mod_seed = seed.wrapping_add(333 + i as u64 * 77);
            let stat_idx = (mod_seed % STAT_NAMES.len() as u64) as usize;
            let roll = chaos_roll_verbose(mod_seed as f64 * 1e-9, mod_seed);
            let mod_range = (2000.0 * tier_mult) as i64;
            let val = (roll.final_value * mod_range as f64) as i64
                + roll_stat(-mod_range / 4, mod_range / 4, mod_seed.wrapping_add(1));
            total_magnitude += val.abs();
            stat_modifiers.push(StatModifier {
                stat: STAT_NAMES[stat_idx].to_string(),
                value: val,
            });
        }

        let effect_idx = ((seed.wrapping_mul(55555)) % SPECIAL_EFFECTS.len() as u64) as usize;
        let special_effect = SPECIAL_EFFECTS[effect_idx].to_string();

        let rarity = Rarity::from_magnitude(total_magnitude);
        let value = roll_stat(1, (10000.0 * tier_mult) as i64, seed.wrapping_add(9999));

        // Sockets: tier 1 = 0-1, tier 7 = 2-4; weapons get more
        let socket_roll = chaos_roll_verbose(seed as f64 * 1e-8 + 0.5, seed.wrapping_add(8888));
        let max_sockets: u8 = if is_weapon {
            (2 + material_tier / 2).min(6) as u8
        } else {
            (1 + material_tier / 3).min(4) as u8
        };
        let socket_count =
            (((socket_roll.final_value + 1.0) * 0.5 * max_sockets as f64) as u8).min(max_sockets);
        let link_roll = chaos_roll_verbose(seed as f64 * 1e-7, seed.wrapping_add(7777));
        let socket_links = if socket_count <= 1 {
            0
        } else {
            (((link_roll.final_value + 1.0) * 0.5 * (socket_count - 1) as f64) as u8)
                .min(socket_count.saturating_sub(1))
        };

        Item {
            name,
            base_type,
            implicit,
            damage_or_defense,
            stat_modifiers,
            special_effect,
            rarity,
            is_weapon,
            value,
            item_level,
            material_tier,
            socket_count,
            socket_links,
            socketed_gems: Vec::new(),
            engine_locks: Vec::new(),
            corruption: None,
        }
    }

    /// Generate a starting item biased toward the given class stat
    pub fn generate_for_class(seed: u64, _class_bias: u64) -> Self {
        Self::generate_leveled(seed, 1)
    }

    pub fn total_magnitude(&self) -> i64 {
        let mut total = self.damage_or_defense.abs();
        for m in &self.stat_modifiers {
            total += m.value.abs();
        }
        total
    }

    /// Display as a bordered ASCII box (returns lines)
    pub fn display_box(&self) -> Vec<String> {
        let reset = "\x1b[0m";
        let rarity_color = self.rarity.color_code();
        let width = 46usize;
        let inner = width - 2;

        let mut lines = Vec::new();
        lines.push(format!("{}╔{}╗{}", rarity_color, "═".repeat(width), reset));
        lines.push(format!(
            "{}║  {:width$}║{}",
            rarity_color,
            format!("★ {} ★", self.rarity.name()),
            reset,
            width = inner - 2
        ));
        lines.push(format!(
            "{}║  {:width$}║{}",
            rarity_color,
            self.name.chars().take(inner - 2).collect::<String>(),
            reset,
            width = inner - 2
        ));
        // Item level + tier indicator
        lines.push(format!(
            "{}║  ilvl:{:<width$}║{}",
            rarity_color,
            format!("{} T{}", self.item_level, self.material_tier),
            reset,
            width = inner - 7
        ));

        // Implicit modifier
        let impl_display = self.implicit.chars().take(inner - 6).collect::<String>();
        lines.push(format!(
            "{}║  {}[{}]{:<w$}║{}",
            rarity_color,
            "\x1b[2m",
            impl_display,
            "",
            reset,
            w = (inner - 4 - impl_display.len() - 2).max(1)
        ));

        lines.push(format!(
            "{}║  {}{}║{}",
            rarity_color,
            "─".repeat(inner - 2),
            " ",
            reset
        ));

        let stat_label = if self.is_weapon { "Damage" } else { "Defense" };
        let sign = if self.damage_or_defense >= 0 { "+" } else { "" };
        lines.push(format!(
            "{}║  {}: {}{:<width$}║{}",
            rarity_color,
            stat_label,
            sign,
            self.damage_or_defense,
            reset,
            width = inner - stat_label.len() - 4 - sign.len()
        ));

        for m in &self.stat_modifiers {
            let sign = if m.value >= 0 { "+" } else { "" };
            lines.push(format!(
                "{}║  {}: {}{}{}║{}",
                rarity_color,
                m.stat,
                sign,
                m.value,
                " ".repeat(
                    (inner - m.stat.len() - format!("{}{}", sign, m.value).len() - 4).max(1)
                ),
                reset
            ));
        }

        let fx = &self.special_effect;
        let fx_display = if fx.len() > inner - 4 {
            format!("{}...", &fx[..inner - 7])
        } else {
            fx.clone()
        };
        lines.push(format!(
            "{}║  FX: {:<width$}║{}",
            rarity_color,
            fx_display,
            reset,
            width = inner - 6
        ));

        // Sockets display
        if self.socket_count > 0 {
            let mut sock_str = String::new();
            for i in 0..self.socket_count {
                if i < self.socketed_gems.len() as u8 {
                    let gem = &self.socketed_gems[i as usize];
                    let gem_char = match gem.gem_type {
                        GemType::Skill => 'S',
                        GemType::Support => 's',
                    };
                    sock_str.push(gem_char);
                } else {
                    sock_str.push('O');
                }
                if i < self.socket_links && i + 1 < self.socket_count {
                    sock_str.push('-');
                } else if i + 1 < self.socket_count {
                    sock_str.push(' ');
                }
            }
            lines.push(format!(
                "{}║  Sockets: {:<width$}║{}",
                rarity_color,
                sock_str,
                reset,
                width = inner - 11
            ));
        }

        // Engine locks
        if !self.engine_locks.is_empty() {
            let lock_str = self.engine_locks.join(", ");
            let lock_display = if lock_str.len() > inner - 10 {
                format!("{}...", &lock_str[..inner - 13])
            } else {
                lock_str
            };
            lines.push(format!(
                "{}║  Locked: {:<width$}║{}",
                rarity_color,
                lock_display,
                reset,
                width = inner - 10
            ));
        }

        // Corruption
        if let Some(ref corr) = self.corruption {
            lines.push(format!(
                "{}║  \x1b[35m[CORRUPT] {:<width$}{}║{}",
                rarity_color,
                corr.chars().take(inner - 12).collect::<String>(),
                rarity_color,
                reset,
                width = 0
            ));
        }

        lines.push(format!("{}╚{}╝{}", rarity_color, "═".repeat(width), reset));
        lines
    }

    /// Socket a gem into this item. Returns Err if no space.
    pub fn socket_gem(&mut self, gem: Gem) -> Result<(), &'static str> {
        if self.socketed_gems.len() < self.socket_count as usize {
            self.socketed_gems.push(gem);
            Ok(())
        } else {
            Err("No empty sockets available")
        }
    }

    /// Unsocket a gem by index. Returns the gem if successful.
    pub fn unsocket_gem(&mut self, idx: usize) -> Option<Gem> {
        if idx < self.socketed_gems.len() {
            Some(self.socketed_gems.remove(idx))
        } else {
            None
        }
    }

    /// Get support gems that are linked to the skill gem at index 0.
    /// Linked gems are those with index < socket_links.
    pub fn linked_supports(&self) -> Vec<&Gem> {
        self.socketed_gems
            .iter()
            .take(self.socket_links as usize + 1)
            .filter(|g| matches!(g.gem_type, GemType::Support))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_generation_produces_valid_items() {
        for seed in 0..20u64 {
            let item = Item::generate(seed);
            assert!(!item.name.is_empty());
            assert!(!item.base_type.is_empty());
        }
    }

    #[test]
    fn item_rarity_from_magnitude() {
        assert_eq!(Rarity::from_magnitude(5), Rarity::Common);
        assert_eq!(Rarity::from_magnitude(1500), Rarity::Legendary);
        assert_eq!(Rarity::from_magnitude(999999), Rarity::Beyond);
    }
}
