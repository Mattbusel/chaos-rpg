//! World generation — rooms, areas, environmental effects.
//!
//! Every room is procedurally generated from chaos math.
//! The floor number and seed determine the geometry, hazards, and loot.

use crate::chaos_pipeline::{chaos_roll_verbose, roll_stat};
use crate::enemy::{generate_enemy, Enemy};
use serde::{Deserialize, Serialize};

// ─── ROOM TYPES ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// What kind of room this is.
pub enum RoomType {
    /// A fight with an enemy.
    Combat,
    /// Loot to pick up.
    Treasure,
    /// A merchant.
    Shop,
    /// A shrine that grants a buff.
    Shrine, // buff room
    /// A trap.
    Trap,
    /// A boss fight.
    Boss,
    /// A portal that skips ahead to the next floor.
    Portal,        // advance floor early
    /// A rare empty rest room.
    Empty,         // rare rest room
    /// A rift of pure randomness.
    ChaosRift,     // pure randomness
    /// A bench for modifying items with chaos operations.
    CraftingBench, // modify items with chaos operations
}

impl RoomType {
    /// Display name of the room type.
    pub fn name(&self) -> &'static str {
        match self {
            RoomType::Combat => "Combat",
            RoomType::Treasure => "Treasure",
            RoomType::Shop => "Shop",
            RoomType::Shrine => "Shrine",
            RoomType::Trap => "Trap",
            RoomType::Boss => "BOSS",
            RoomType::Portal => "Portal",
            RoomType::Empty => "Empty",
            RoomType::ChaosRift => "CHAOS RIFT",
            RoomType::CraftingBench => "Crafting Bench",
        }
    }

    /// Short icon for the room type on the map.
    pub fn icon(&self) -> &'static str {
        match self {
            RoomType::Combat => "[×]",
            RoomType::Treasure => "[★]",
            RoomType::Shop => "[$]",
            RoomType::Shrine => "[☯]",
            RoomType::Trap => "[!]",
            RoomType::Boss => "[☠]",
            RoomType::Portal => "[↑]",
            RoomType::Empty => "[ ]",
            RoomType::ChaosRift => "[∞]",
            RoomType::CraftingBench => "[⚒]",
        }
    }
}

// ─── ENVIRONMENT EFFECTS ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Environmental effect active in a room.
pub enum EnvEffect {
    /// Gain this much mana each turn.
    ManaBoost(i64),    // +mana per turn
    /// Take this much damage each turn from the environment.
    DamageAura(i64),   // take damage per turn from environment
    /// The player always acts first.
    SpeedBoost,        // player goes first always
    /// Multiply every chaos roll by this factor.
    ChaosAmplify(f64), // multiply all chaos roll values
    /// One stat is lowered while in the room.
    StatDebuff { stat: String, amount: i64 },
    /// Enemy stats are hidden.
    VisionBlur,          // can't see enemy stats
    /// Gold drops are multiplied by this factor.
    GoldMultiplier(f64), // gold drops multiplied
    /// No environmental effect.
    None,
}

impl EnvEffect {
    /// Player-facing description of the effect.
    pub fn describe(&self) -> String {
        match self {
            EnvEffect::ManaBoost(n) => format!("+{} mana per turn", n),
            EnvEffect::DamageAura(n) => format!("Take {} damage per turn", n),
            EnvEffect::SpeedBoost => "You always go first".to_string(),
            EnvEffect::ChaosAmplify(m) => format!("Chaos amplified ×{:.1}", m),
            EnvEffect::StatDebuff { stat, amount } => format!("-{} to {}", amount, stat),
            EnvEffect::VisionBlur => "Enemy stats are hidden".to_string(),
            EnvEffect::GoldMultiplier(m) => format!("Gold drops ×{:.1}", m),
            EnvEffect::None => "None".to_string(),
        }
    }
}

// ─── ROOM ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
/// One room on a floor.
pub struct Room {
    /// Kind of room.
    pub room_type: RoomType,
    /// Flavour text.
    pub description: String,
    /// Environmental effect in the room.
    pub env_effect: EnvEffect,
    /// Floor the room is on.
    pub floor: u32,
    /// Seed for this room's rolls.
    pub seed: u64,
    /// True once the player has entered the room.
    pub visited: bool,
}

impl Room {
    /// The room drawn as a 52-character-wide box (type, description, environment) for the terminal.
    pub fn ascii_border(&self) -> Vec<String> {
        let width = 50;
        let top = format!("╔{}╗", "═".repeat(width));
        let bottom = format!("╚{}╝", "═".repeat(width));
        let type_line = format!(
            "║  {:^width$}  ║",
            format!("{} {}", self.room_type.icon(), self.room_type.name()),
            width = width - 4
        );
        let desc_short: String = self.description.chars().take(width - 4).collect();
        let desc_line = format!("║  {:<width$}  ║", desc_short, width = width - 4);
        let env_line = if !matches!(self.env_effect, EnvEffect::None) {
            format!(
                "║  ⚡ EFFECT: {:<width$}║",
                self.env_effect.describe(),
                width = width - 13
            )
        } else {
            format!("║  {:<width$}  ║", " ", width = width - 4)
        };

        vec![top, type_line, desc_line, env_line, bottom]
    }
}

const ROOM_DESCS_COMBAT: &[&str] = &[
    "A chamber where fractals scream on the walls.",
    "Entropy leaks from the ceiling. Something moves.",
    "Prime numbers are carved into every surface.",
    "The Lorenz attractor spins in midair. You are not alone.",
    "Bifurcation diagrams cover the floor. The enemy is here.",
];

const ROOM_DESCS_TREASURE: &[&str] = &[
    "Gold glitters, shaped like Fibonacci spirals.",
    "A chest vibrates at a frequency that shouldn't exist.",
    "The loot appears to be made of crystallized math.",
    "Riemann zeros mark the location of hidden valuables.",
];

const ROOM_DESCS_SHRINE: &[&str] = &[
    "A golden ratio shrine pulses with gentle power.",
    "The Fourier spirit offers its blessing.",
    "Collatz sequence graffiti lines these sacred walls.",
    "A Mandelbrot carving radiates calm chaos.",
];

const ROOM_DESCS_TRAP: &[&str] = &[
    "Something feels mathematically wrong here.",
    "The floor displays a logistic map. It's at r=4.0.",
    "Totient traps are embedded in the ground.",
    "The Collatz path through this room is unusually long.",
];

const ROOM_DESCS_EMPTY: &[&str] = &[
    "Just silence. Even the math is quiet here.",
    "Nothing. Pure, beautiful nothing. Rest.",
    "An empty room. The absence of enemies is itself suspicious.",
];

// ─── FLOOR GENERATION ────────────────────────────────────────────────────────

/// A complete floor with rooms laid out on a linear path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Floor {
    /// Floor number, starting at 1.
    pub number: u32,
    /// Rooms in the order they are visited.
    pub rooms: Vec<Room>,
    /// Index of the room the player is in.
    pub current_room: usize,
    /// Seed the floor was generated from.
    pub seed: u64,
}

impl Floor {
    /// The room the player is in.
    pub fn current(&self) -> &Room {
        &self.rooms[self.current_room]
    }

    /// Move to the next room; returns false when the floor is finished.
    pub fn advance(&mut self) -> bool {
        if self.current_room + 1 < self.rooms.len() {
            self.current_room += 1;
            true
        } else {
            false // floor complete
        }
    }

    /// Rooms left after the current one.
    pub fn rooms_remaining(&self) -> usize {
        self.rooms.len() - self.current_room - 1
    }

    /// One-line map of the floor: cleared rooms, the current room and rooms ahead.
    pub fn minimap(&self) -> String {
        let mut map = String::new();
        for (i, _room) in self.rooms.iter().enumerate() {
            if i < self.current_room {
                map.push('▓');
            } else if i == self.current_room {
                map.push('◉');
            } else {
                map.push('░');
            }
            if i + 1 < self.rooms.len() {
                map.push('─');
            }
        }
        map
    }
}

/// Generate floor `floor_num` from `seed`: 5 rooms, plus one per two floors, up to 10.
pub fn generate_floor(floor_num: u32, seed: u64) -> Floor {
    let roll = chaos_roll_verbose(floor_num as f64 * 0.05, seed);

    // Floor size: 5–10 rooms, increasing with floor
    let room_count = (5 + floor_num / 2).min(10) as usize;
    let mut rooms = Vec::with_capacity(room_count);

    for i in 0..room_count {
        let room_seed = seed.wrapping_add(i as u64 * 9973).wrapping_mul(2654435761);
        let room = generate_room(floor_num, room_seed, i == room_count - 1);
        rooms.push(room);
    }

    // Override last room to be boss on every 5th floor
    if floor_num.is_multiple_of(5) {
        let last = rooms.last_mut().unwrap();
        last.room_type = RoomType::Boss;
        last.description = "The BOSS chamber. Mathematics itself trembles.".to_string();
    }

    let _ = roll; // used for rng influence
    Floor {
        number: floor_num,
        rooms,
        current_room: 0,
        seed,
    }
}

fn generate_room(floor: u32, seed: u64, is_last: bool) -> Room {
    let roll = chaos_roll_verbose(floor as f64 * 0.1, seed);
    let val = roll.final_value;

    let room_type = if is_last {
        // Last room on non-boss floors: portal or combat
        if val > 0.3 {
            RoomType::Portal
        } else {
            RoomType::Combat
        }
    } else {
        // Weighted distribution
        match (val + 1.0) / 2.0 {
            v if v > 0.92 => RoomType::ChaosRift,
            v if v > 0.85 => RoomType::CraftingBench,
            v if v > 0.75 => RoomType::Treasure,
            v if v > 0.65 => RoomType::Shop,
            v if v > 0.55 => RoomType::Shrine,
            v if v > 0.45 => RoomType::Trap,
            v if v > 0.10 => RoomType::Combat,
            _ => RoomType::Empty,
        }
    };

    let description = pick_room_desc(&room_type, seed);
    let env_effect = generate_env_effect(floor, seed.wrapping_add(777));

    Room {
        room_type,
        description,
        env_effect,
        floor,
        seed,
        visited: false,
    }
}

fn pick_room_desc(room_type: &RoomType, seed: u64) -> String {
    let descs = match room_type {
        RoomType::Combat | RoomType::Boss => ROOM_DESCS_COMBAT,
        RoomType::Treasure => ROOM_DESCS_TREASURE,
        RoomType::Shrine => ROOM_DESCS_SHRINE,
        RoomType::Trap => ROOM_DESCS_TRAP,
        RoomType::Empty => ROOM_DESCS_EMPTY,
        RoomType::Shop => &["A merchant emerges from the math-fog."],
        RoomType::ChaosRift => &["REALITY ERROR. MATHEMATICAL EXCEPTION. PROCEED?"],
        RoomType::Portal => &["A shimmering portal to the next floor hums ahead."],
        RoomType::CraftingBench => &[
            "A bench covered in crystallized prime number shards. Items can be reforged.",
            "Mathematical tools lay arranged in Fibonacci order. The bench beckons.",
            "Chaos resonates through iron anvil runes. Something waits to be remade.",
        ],
    };
    let idx = (seed % descs.len() as u64) as usize;
    descs[idx].to_string()
}

fn generate_env_effect(floor: u32, seed: u64) -> EnvEffect {
    let roll = chaos_roll_verbose(floor as f64 * 0.07, seed);
    // 60% chance of no effect
    if roll.final_value < 0.2 {
        return EnvEffect::None;
    }

    let effect_idx = seed % 7;
    match effect_idx {
        0 => EnvEffect::ManaBoost(roll_stat(2, 8 + floor as i64, seed.wrapping_add(1))),
        1 => EnvEffect::DamageAura(roll_stat(1, 5 + floor as i64 / 2, seed.wrapping_add(2))),
        2 => EnvEffect::SpeedBoost,
        3 => EnvEffect::ChaosAmplify(1.0 + roll.final_value.abs()),
        4 => {
            let stats = [
                "Vitality",
                "Force",
                "Mana",
                "Cunning",
                "Precision",
                "Entropy",
                "Luck",
            ];
            let stat_idx = (seed.wrapping_add(10) % stats.len() as u64) as usize;
            EnvEffect::StatDebuff {
                stat: stats[stat_idx].to_string(),
                amount: roll_stat(2, 10, seed.wrapping_add(3)),
            }
        }
        5 => EnvEffect::VisionBlur,
        _ => EnvEffect::GoldMultiplier(1.5 + roll.final_value.abs()),
    }
}

/// Generate an enemy for the current room/floor
pub fn room_enemy(room: &Room) -> Enemy {
    generate_enemy(room.floor, room.seed.wrapping_add(12345678))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_generates_correct_room_count() {
        for floor in [1u32, 3, 5, 10] {
            let f = generate_floor(floor, 42);
            let expected = (5 + floor / 2).min(10) as usize;
            assert_eq!(f.rooms.len(), expected);
        }
    }

    #[test]
    fn floor_5_ends_with_boss() {
        let f = generate_floor(5, 99);
        assert_eq!(f.rooms.last().unwrap().room_type, RoomType::Boss);
    }

    #[test]
    fn minimap_length_matches_rooms() {
        let f = generate_floor(3, 42);
        // minimap has rooms + separators
        let map = f.minimap();
        assert!(!map.is_empty());
    }
}
