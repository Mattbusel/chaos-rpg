//! Spell generation — mathematically cursed magic for everyone.

use crate::chaos_pipeline::{chaos_roll_verbose, roll_stat};
use serde::{Deserialize, Serialize};

const VERBS: &[&str] = &[
    "Invoke",
    "Summon",
    "Cast",
    "Unleash",
    "Whisper",
    "Detonate",
    "Caress",
    "Insult",
    "Become",
    "Unbecome",
    "Calculate",
    "Ferment",
    "Yeet",
    "Gently Place",
    "Aggressively Suggest",
];

const NOUNS: &[&str] = &[
    "Fire",
    "Ice",
    "Bees",
    "Gravity",
    "Time",
    "Math",
    "Regret",
    "Fractal",
    "Nothing",
    "Everything",
    "A Sandwich",
    "Screaming",
    "Silence",
    "Friendship",
    "Taxes",
    "The Concept of Damage",
    "A Smaller Spell",
];

const MODIFIERS: &[&str] = &[
    "of Doom",
    "Gently",
    "With Extreme Prejudice",
    "(But Worse)",
    "Recursively",
    "In Reverse",
    "Twice",
    "At Great Personal Cost",
    "For Free",
    "By Accident",
];

const SIDE_EFFECTS: &[&str] = &[
    "caster takes equal damage",
    "caster is teleported 1d6 rooms away",
    "gravity reverses for 3 turns",
    "all NPCs in the area become temporarily hostile",
    "nothing visible happens but your stats secretly change",
    "the screen shakes violently",
    "a new NPC spawns and is immediately confused",
    "the spell targets a random entity",
    "the spell learns from this experience and grows slightly stronger",
    "you age 10 years",
    "you age -10 years (become younger)",
    "a door appears",
    "a door disappears",
    "nearby enemies are mildly embarrassed",
    "No visible side effect (suspicious)",
    "you gain 1 gold",
    "caster is briefly on fire (cosmetic only)",
];

const SCALING_STATS: &[&str] = &[
    "Vitality",
    "Force",
    "Mana",
    "Cunning",
    "Precision",
    "Entropy",
    "Luck",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spell {
    pub name: String,
    pub damage: i64,     // can be negative (heals enemies)
    pub mana_cost: i64,  // can be negative (gives mana)
    pub aoe_radius: i64, // 0=self, positive=area, negative=hits own party
    pub side_effect: String,
    pub scaling_stat: String,
    pub scaling_factor: f64, // can be negative
    pub engines_used: Vec<String>,
}

impl Spell {
    pub fn generate(seed: u64) -> Self {
        let verb_idx = (seed % VERBS.len() as u64) as usize;
        let noun_idx = ((seed.wrapping_mul(13337)) % NOUNS.len() as u64) as usize;
        let mod_idx = ((seed.wrapping_mul(77777)) % MODIFIERS.len() as u64) as usize;
        let name = format!(
            "{} {} {}",
            VERBS[verb_idx], NOUNS[noun_idx], MODIFIERS[mod_idx]
        );

        // Damage — chaos roll mapped to wide range
        let dmg_roll = chaos_roll_verbose(seed as f64 * 1e-9, seed.wrapping_add(1));
        let damage =
            (dmg_roll.final_value * 1000.0) as i64 + roll_stat(-500, 500, seed.wrapping_add(2));

        // Mana cost — can be negative!
        let mana_roll = chaos_roll_verbose(seed as f64 * 1e-9, seed.wrapping_add(3));
        let mana_cost =
            (mana_roll.final_value * 50.0) as i64 + roll_stat(-20, 60, seed.wrapping_add(4));

        // AoE
        let aoe_roll = chaos_roll_verbose(seed as f64 * 1e-9, seed.wrapping_add(5));
        let aoe_radius = (aoe_roll.final_value * 10.0) as i64;

        let fx_idx = ((seed.wrapping_mul(99999)) % SIDE_EFFECTS.len() as u64) as usize;
        let side_effect = SIDE_EFFECTS[fx_idx].to_string();

        let stat_idx = ((seed.wrapping_mul(31337)) % SCALING_STATS.len() as u64) as usize;
        let scaling_stat = SCALING_STATS[stat_idx].to_string();

        let scale_roll = chaos_roll_verbose(seed as f64 * 1e-9, seed.wrapping_add(6));
        let scaling_factor = scale_roll.final_value * 3.0; // can be negative

        let engines_used: Vec<String> = dmg_roll
            .chain
            .iter()
            .map(|s| s.engine_name.to_string())
            .collect();

        Spell {
            name,
            damage,
            mana_cost,
            aoe_radius,
            side_effect,
            scaling_stat,
            scaling_factor,
            engines_used,
        }
    }

    /// Calculate actual damage given the caster's stat value
    pub fn calc_damage(&self, stat_value: i64) -> i64 {
        let bonus = (stat_value as f64 * self.scaling_factor) as i64;
        self.damage + bonus
    }

    pub fn display_box(&self) -> Vec<String> {
        let cyan = "\x1b[36m";
        let reset = "\x1b[0m";
        let width = 44usize;
        let inner = width - 2;

        let mut lines = Vec::new();
        lines.push(format!("{}┌{}┐{}", cyan, "─".repeat(width), reset));

        let name_display = self.name.chars().take(inner - 2).collect::<String>();
        lines.push(format!(
            "{}│ ⚡ {:<width$}│{}",
            cyan,
            name_display,
            reset,
            width = inner - 4
        ));

        let dmg_sign = if self.damage >= 0 { "+" } else { "" };
        lines.push(format!(
            "{}│   Damage: {}{:<width$}│{}",
            cyan,
            dmg_sign,
            self.damage,
            reset,
            width = inner - 11
        ));

        let mana_sign = if self.mana_cost >= 0 { "" } else { "" };
        let mana_note = if self.mana_cost < 0 {
            " (GIVES mana!)"
        } else {
            ""
        };
        lines.push(format!(
            "{}│   Mana Cost: {}{}{:<w$}│{}",
            cyan,
            mana_sign,
            self.mana_cost,
            mana_note,
            reset,
            w = (inner - 14 - mana_note.len()).max(1)
        ));

        let aoe_desc = match self.aoe_radius.cmp(&0) {
            std::cmp::Ordering::Equal => "self only".to_string(),
            std::cmp::Ordering::Greater => format!("{} tile radius", self.aoe_radius),
            std::cmp::Ordering::Less => format!("hits your own party ({})", self.aoe_radius),
        };
        lines.push(format!(
            "{}│   AoE: {:<width$}│{}",
            cyan,
            aoe_desc,
            reset,
            width = inner - 9
        ));

        let scale_sign = if self.scaling_factor >= 0.0 {
            "×"
        } else {
            "×"
        };
        lines.push(format!(
            "{}│   Scales: {} {:.2}{:<w$}│{}",
            cyan,
            self.scaling_stat,
            scale_sign,
            self.scaling_factor,
            reset,
            w = (inner - self.scaling_stat.len() - 13).max(1)
        ));

        let fx = &self.side_effect;
        let fx_display = if fx.len() > inner - 14 {
            format!("{}...", &fx[..inner - 17])
        } else {
            fx.clone()
        };
        lines.push(format!(
            "{}│   Side FX: {:<w$}│{}",
            cyan,
            fx_display,
            reset,
            w = (inner - 13).max(1)
        ));

        // Engine chain display
        let engines_str: String = self
            .engines_used
            .iter()
            .take(4)
            .map(|e| e.split_whitespace().next().unwrap_or(e).to_string())
            .collect::<Vec<_>>()
            .join("→");
        lines.push(format!(
            "{}│   [{:<width$}]│{}",
            cyan,
            engines_str,
            reset,
            width = inner - 4
        ));

        lines.push(format!("{}└{}┘{}", cyan, "─".repeat(width), reset));
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spell_generation_valid() {
        for seed in 0..10u64 {
            let spell = Spell::generate(seed);
            assert!(!spell.name.is_empty());
            assert!(!spell.scaling_stat.is_empty());
        }
    }

    #[test]
    fn negative_mana_spell_exists_in_distribution() {
        let mut found_negative = false;
        for seed in 0..200u64 {
            let spell = Spell::generate(seed);
            if spell.mana_cost < 0 {
                found_negative = true;
                break;
            }
        }
        assert!(
            found_negative,
            "Should occasionally generate negative-cost spells"
        );
    }
}
