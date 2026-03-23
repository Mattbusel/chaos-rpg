//! Item generation — every item is a chaos-rolled catastrophe.

use crate::chaos_pipeline::{chaos_roll_verbose, roll_stat};
use serde::{Deserialize, Serialize};

// ─── Tables ──────────────────────────────────────────────────────────────────

const BASE_TYPES: &[&str] = &[
    // Weapons (indices 0-9 = is_weapon: true)
    "Sword",
    "Greatsword",
    "Staff",
    "Wand",
    "Bow",
    "Crossbow",
    "Dagger",
    "Scythe",
    "Paradox Blade",
    "Death Equation",
    // Non-weapons (indices 10+)
    "Shield",
    "Helm",
    "Armor",
    "Ring",
    "Amulet",
    "Boots",
    "Gloves",
    "Cape",
    "Chaos Crystal",
    "Prime Shard",
    "Fractal Lens",
    "Null Field",
    "Theorem",
    "Singularity",
    "Mysterious Object",
    "Error",
    "Placeholder That Became Real",
];

const MATERIALS: &[&str] = &[
    "wood",
    "iron",
    "steel",
    "mithril",
    "diamond",
    "antimatter",
    "condensed screaming",
    "crystallized luck",
    "frozen time",
    "solidified math",
    "sadness",
    "weaponized optimism",
    "dark matter",
    "suspicious cheese",
    "the concept of sharpness",
    "recycled prayers",
    "bottled lightning",
    "crystallized entropy",
    "prime-factored obsidian",
    "eigenstate alloy",
    "superposition glass",
    "deterministic void",
    "asymptotic silk",
    "non-euclidean bone",
    "compressed infinity",
    "decompiled soul",
    "Turing-complete leather",
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
    "heals enemies on hit",
    "teleports wielder randomly each turn",
    "attracts bees",
    "doubles all chaos rolls while equipped",
    "makes you invisible but also blind",
    "screams",
    "is slightly damp",
    "occasionally becomes a different item",
    "deals damage to you equal to damage dealt to enemy",
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
    "rerolls one chaos die per combat (unverified)",
    "all critical hits deal 3× damage instead of 2×",
    "immune to status effects (while conscious)",
    "each kill permanently raises max HP by 5",
    "spell costs become negative: mana regenerates on cast",
    "20% chance to phase through incoming attacks",
    "combo attacks deal double the normal combo bonus",
    "grants a vision of your next enemy's weakness (blurry)",
    "exists in superposition: both useful and useless until observed",
    "the prime factorization of its damage is always prime",
    "laughs at the concept of defense stats",
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

// ─── Item ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub base_type: String,
    pub damage_or_defense: i64,
    pub stat_modifiers: Vec<StatModifier>,
    pub special_effect: String,
    pub rarity: Rarity,
    pub is_weapon: bool,
    pub value: i64,
}

impl Item {
    pub fn generate(seed: u64) -> Self {
        let base_idx = (seed % BASE_TYPES.len() as u64) as usize;
        let mat_idx = ((seed.wrapping_mul(1234567)) % MATERIALS.len() as u64) as usize;
        let adj_idx = ((seed.wrapping_mul(9876543)) % ADJECTIVES.len() as u64) as usize;

        let base_type = BASE_TYPES[base_idx].to_string();
        let material = MATERIALS[mat_idx];
        let adjective = ADJECTIVES[adj_idx];
        let name = format!("{} {} {}", material, base_type, adjective);

        let is_weapon = base_idx <= 9; // indices 0-9 are weapons

        // Damage/defense — fully unbounded chaos roll mapped to a wide range
        let dmg_seed = seed.wrapping_add(111);
        let dmg_roll = chaos_roll_verbose(dmg_seed as f64 * 1e-9, dmg_seed);
        let damage_or_defense =
            (dmg_roll.final_value * 500.0) as i64 + roll_stat(-200, 200, seed.wrapping_add(222));

        // 0-3 stat modifiers
        let n_mods = (seed.wrapping_mul(17) % 4) as usize;
        let mut stat_modifiers = Vec::new();
        let mut total_magnitude = damage_or_defense.abs();

        for i in 0..n_mods {
            let mod_seed = seed.wrapping_add(333 + i as u64 * 77);
            let stat_idx = (mod_seed % STAT_NAMES.len() as u64) as usize;
            let roll = chaos_roll_verbose(mod_seed as f64 * 1e-9, mod_seed);
            let val =
                (roll.final_value * 2000.0) as i64 + roll_stat(-500, 500, mod_seed.wrapping_add(1));
            total_magnitude += val.abs();
            stat_modifiers.push(StatModifier {
                stat: STAT_NAMES[stat_idx].to_string(),
                value: val,
            });
        }

        let effect_idx = ((seed.wrapping_mul(55555)) % SPECIAL_EFFECTS.len() as u64) as usize;
        let special_effect = SPECIAL_EFFECTS[effect_idx].to_string();

        let rarity = Rarity::from_magnitude(total_magnitude);
        let value = roll_stat(1, 10000, seed.wrapping_add(9999));

        Item {
            name,
            base_type,
            damage_or_defense,
            stat_modifiers,
            special_effect,
            rarity,
            is_weapon,
            value,
        }
    }

    /// Generate a starting item biased toward the given class stat
    pub fn generate_for_class(seed: u64, _class_bias: u64) -> Self {
        // For now just generate normally with the seed
        Self::generate(seed)
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

        lines.push(format!("{}╚{}╝{}", rarity_color, "═".repeat(width), reset));
        lines
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
