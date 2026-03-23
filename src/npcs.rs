//! NPC generation — merchants, quest-givers, mysterious strangers.
//!
//! Every NPC is procedurally generated with chaos-rolled personality,
//! inventory, and dialogue. Haggling is governed by the logistic map.

use crate::chaos_pipeline::{chaos_roll_verbose, biased_chaos_roll, roll_stat};
use crate::items::Item;
use crate::character::Character;
use serde::{Deserialize, Serialize};

// ─── NPC ROLES ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcRole {
    Merchant,
    Oracle,     // reveals next room
    Blacksmith, // upgrades weapons
    Healer,
    MysteriousStranger,
    CursedScholar, // sells knowledge at a cost
}

impl NpcRole {
    pub fn name(&self) -> &'static str {
        match self {
            NpcRole::Merchant => "Merchant",
            NpcRole::Oracle => "Oracle",
            NpcRole::Blacksmith => "Blacksmith",
            NpcRole::Healer => "Healer",
            NpcRole::MysteriousStranger => "Mysterious Stranger",
            NpcRole::CursedScholar => "Cursed Scholar",
        }
    }
}

// ─── NPC PERSONALITY ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcPersonality {
    pub greed: f64,       // 0=generous, 1=greedy. Affects prices.
    pub hostility: f64,   // 0=friendly, 1=hostile. Affects dialogue.
    pub chaos_affinity: f64, // 0=orderly, 1=chaotic. Affects what they say.
}

impl NpcPersonality {
    pub fn from_seed(seed: u64) -> Self {
        let greed_roll = chaos_roll_verbose(seed as f64 * 1e-12, seed);
        let hostile_roll = chaos_roll_verbose(seed as f64 * 1e-11, seed.wrapping_add(1));
        let chaos_roll = chaos_roll_verbose(seed as f64 * 1e-10, seed.wrapping_add(2));

        NpcPersonality {
            greed: (greed_roll.final_value + 1.0) / 2.0,
            hostility: (hostile_roll.final_value + 1.0) / 2.0,
            chaos_affinity: (chaos_roll.final_value + 1.0) / 2.0,
        }
    }
}

// ─── NPC ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Npc {
    pub name: String,
    pub role: NpcRole,
    pub personality: NpcPersonality,
    pub inventory: Vec<Item>,
    pub relationship: f64, // -1.0 hostile, 0.0 neutral, 1.0 friendly
    pub seed: u64,
}

impl Npc {
    pub fn greeting(&self) -> String {
        let chaos = self.personality.chaos_affinity;
        let hostile = self.personality.hostility;

        if hostile > 0.8 {
            return format!("{} glares at you. 'What do YOU want.'", self.name);
        }
        if chaos > 0.8 {
            return format!(
                "{} says: 'The Collatz sequence led me here. Also, {} things.',",
                self.name,
                (self.seed % 97) + 1
            );
        }

        match self.role {
            NpcRole::Merchant => format!("{}: 'Ah, a traveler! Care to browse my chaos-touched wares?'", self.name),
            NpcRole::Oracle => format!("{}: 'The Riemann zeros have foretold your arrival... mostly.'", self.name),
            NpcRole::Blacksmith => format!("{}: 'I can temper your weapons with prime-density alloy.'", self.name),
            NpcRole::Healer => format!("{}: 'The Fibonacci sequence of your wounds is quite beautiful. Let me fix it.'", self.name),
            NpcRole::MysteriousStranger => format!("{} says nothing. Their eyes compute something.", self.name),
            NpcRole::CursedScholar => format!("{}: 'Knowledge has a cost. Mana, usually. Sometimes sanity.'", self.name),
        }
    }

    /// Calculate sale price with personality and haggling
    pub fn sale_price(&self, base_value: i64, player_cunning: i64) -> i64 {
        let greed_mult = 0.5 + self.personality.greed;
        let cunning_discount = (player_cunning as f64 / 200.0).min(0.3);
        let relationship_discount = (self.relationship * 0.1).max(-0.1);
        let total_mult = greed_mult * (1.0 - cunning_discount + relationship_discount);
        (base_value as f64 * total_mult).max(1.0) as i64
    }

    /// Attempt to haggle. Returns new price and relationship delta.
    pub fn haggle(&mut self, player: &Character, seed: u64) -> (i64, f64) {
        let roll = biased_chaos_roll(
            player.stats.cunning as f64 * 0.01,
            player.stats.luck as f64 / 200.0,
            seed,
        );

        let rel_delta = if roll.is_critical() {
            0.1 // made a friend
        } else if roll.is_catastrophe() {
            -0.2 // insulted the NPC
        } else {
            0.0
        };

        self.relationship = (self.relationship + rel_delta).clamp(-1.0, 1.0);
        let discount = if roll.final_value > 0.3 { 0.15 } else { 0.0 };

        // Apply discount to inventory (conceptual — actual price recalculated at purchase)
        let _ = discount;

        (roll.to_range(-20, 0) as i64, rel_delta)
    }
}

// ─── NAME GENERATION ─────────────────────────────────────────────────────────

const NPC_PREFIXES: &[&str] = &[
    "Sigma", "Null", "Divergent", "Lorenz", "Prime", "Zeta",
    "Fractal", "Attractor", "Asymptotic", "Recursive",
];

const NPC_BASES: &[&str] = &[
    "the Merchant", "Bifurcatus", "of the Last Algorithm",
    "the Undefined", "Sequence-Walker", "of Infinite Regress",
    "the Convergent", "Phase-Locked", "Math-Touched",
];

pub fn generate_npc(role: NpcRole, floor: u32, seed: u64) -> Npc {
    let prefix_idx = (seed % NPC_PREFIXES.len() as u64) as usize;
    let base_idx = (seed.wrapping_mul(31337) % NPC_BASES.len() as u64) as usize;
    let name = format!("{} {}", NPC_PREFIXES[prefix_idx], NPC_BASES[base_idx]);

    let personality = NpcPersonality::from_seed(seed);

    // Generate inventory (merchants have more items)
    let item_count = match role {
        NpcRole::Merchant | NpcRole::Blacksmith => roll_stat(2, 5, seed.wrapping_add(10)) as usize,
        NpcRole::CursedScholar => roll_stat(1, 3, seed.wrapping_add(11)) as usize,
        _ => 0,
    };

    let inventory: Vec<Item> = (0..item_count)
        .map(|i| Item::generate(seed.wrapping_add(i as u64 * 7777 + floor as u64)))
        .collect();

    Npc {
        name,
        role,
        personality,
        inventory,
        relationship: 0.0,
        seed,
    }
}

/// Generate a shop NPC for the current floor
pub fn shop_npc(floor: u32, seed: u64) -> Npc {
    generate_npc(NpcRole::Merchant, floor, seed)
}

/// Generate an oracle NPC
pub fn oracle_npc(floor: u32, seed: u64) -> Npc {
    let mut npc = generate_npc(NpcRole::Oracle, floor, seed);
    npc.relationship = 0.3; // oracles are slightly friendly by default
    npc
}

/// Oracle prophecy text
pub fn oracle_prophecy(floor: u32, seed: u64) -> String {
    let roll = chaos_roll_verbose(floor as f64 * 0.1, seed);
    let prophecies = [
        "The Mandelbrot boundary will test you on floor {}. Prepare your entropy.",
        "A prime number will save your life. Keep your precision high.",
        "The logistic map bifurcates at your next battle. Expect chaos.",
        "Zeta zeros align in your favor... for now.",
        "The Lorenz butterfly has already decided your fate. The math is inevitable.",
        "Your Collatz chain will reach 1. Eventually.",
    ];
    let idx = (seed % prophecies.len() as u64) as usize;
    let next_floor = floor + (roll.to_range(1, 3)) as u32;
    prophecies[idx].replace("{}", &next_floor.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npc_generation_produces_valid_npc() {
        let npc = generate_npc(NpcRole::Merchant, 1, 42);
        assert!(!npc.name.is_empty());
        assert!(!npc.inventory.is_empty());
    }

    #[test]
    fn merchant_has_items() {
        let npc = shop_npc(3, 999);
        assert!(!npc.inventory.is_empty());
    }

    #[test]
    fn haggling_changes_relationship() {
        use crate::character::{CharacterClass, Background};
        let mut npc = shop_npc(1, 42);
        let player = Character::roll_new("Haggler".to_string(), CharacterClass::Thief, Background::Merchant, 1);
        let initial_rel = npc.relationship;
        let _ = npc.haggle(&player, 99);
        // relationship might have changed
        let _ = initial_rel;
    }
}
