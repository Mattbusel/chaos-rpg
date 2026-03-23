//! Enemy generation via chaos algorithms.
//!
//! Every enemy is procedurally generated — name, stats, abilities, loot.
//! Floor number and seed determine what horrors await.

use crate::chaos_pipeline::{roll_stat, chaos_roll_verbose};
use serde::{Deserialize, Serialize};

// ─── TIERS ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnemyTier {
    Minion,
    Elite,
    Champion,
    Boss,
    Abomination,
}

impl EnemyTier {
    pub fn name(&self) -> &'static str {
        match self {
            EnemyTier::Minion => "Minion",
            EnemyTier::Elite => "Elite",
            EnemyTier::Champion => "Champion",
            EnemyTier::Boss => "Boss",
            EnemyTier::Abomination => "ABOMINATION",
        }
    }

    pub fn hp_multiplier(&self) -> f64 {
        match self {
            EnemyTier::Minion => 0.5,
            EnemyTier::Elite => 1.0,
            EnemyTier::Champion => 2.0,
            EnemyTier::Boss => 4.5,
            EnemyTier::Abomination => 12.0,
        }
    }

    pub fn xp_multiplier(&self) -> u64 {
        match self {
            EnemyTier::Minion => 1,
            EnemyTier::Elite => 3,
            EnemyTier::Champion => 8,
            EnemyTier::Boss => 25,
            EnemyTier::Abomination => 100,
        }
    }

    pub fn gold_multiplier(&self) -> f64 {
        match self {
            EnemyTier::Minion => 0.5,
            EnemyTier::Elite => 1.2,
            EnemyTier::Champion => 3.0,
            EnemyTier::Boss => 8.0,
            EnemyTier::Abomination => 30.0,
        }
    }
}

// ─── ENEMY STRUCT ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub name: String,
    pub tier: EnemyTier,
    pub hp: i64,
    pub max_hp: i64,
    pub base_damage: i64,
    pub attack_modifier: i64,
    pub chaos_level: f64,   // feeds into chaos rolls as input
    pub xp_reward: u64,
    pub gold_reward: i64,
    pub ascii_sprite: &'static str,
    pub seed: u64,
    pub special_ability: Option<&'static str>,
}

impl Enemy {
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    pub fn hp_percent(&self) -> f64 {
        (self.hp as f64 / self.max_hp as f64).clamp(0.0, 1.0)
    }

    pub fn hp_bar(&self, width: usize) -> String {
        let filled = ((self.hp_percent() * width as f64) as usize).min(width);
        let bar = "█".repeat(filled) + &"░".repeat(width - filled);
        format!("[{}] {}/{}", bar, self.hp, self.max_hp)
    }

    pub fn tier_color(&self) -> &'static str {
        match self.tier {
            EnemyTier::Minion => "\x1b[37m",
            EnemyTier::Elite => "\x1b[32m",
            EnemyTier::Champion => "\x1b[36m",
            EnemyTier::Boss => "\x1b[33m",
            EnemyTier::Abomination => "\x1b[35m",
        }
    }
}

// ─── NAME & SPRITE TABLES ────────────────────────────────────────────────────

const MINION_NAMES: &[&str] = &[
    "Fractal Imp", "Entropy Sprite", "Logic Wraith", "Sigma Rat", "Null Bat",
    "Zeta Goblin", "Drift Slime", "Collatz Shade", "Prime Leech", "Fourier Gnat",
    "Divergence Tick", "Singularity Moth",
];

const ELITE_NAMES: &[&str] = &[
    "Lorenz Stalker", "Mandelbrot Hound", "Riemann Specter", "Golden Asp",
    "Logistic Troll", "Euler Revenant", "Modular Knight", "Fibonacci Worm",
    "Cantor Serpent", "Phase-Space Wraith",
];

const CHAMPION_NAMES: &[&str] = &[
    "The Divergence", "Attractor Beast", "Phase-Lock Horror", "Sieve Colossus",
    "Spiral Tyrant", "Zeta Construct", "Totient Golem", "Chaos Shepherd",
    "The Bifurcation", "Orbit Breaker",
];

const BOSS_NAMES: &[&str] = &[
    "Lord Mandelbrot", "The Strange Attractor", "Absolute Uncertainty",
    "The Null Hypothesis", "Infinite Regress", "Bifurcation King",
    "Grand Collatz", "The Omega Point",
];

const ABOMINATION_NAMES: &[&str] = &[
    "THE HEAT DEATH", "RECURSIVE INFINITY", "GODELS DAEMON", "THE P!=NP PROOF",
];

const SPECIAL_ABILITIES: &[&str] = &[
    "Bifurcation Strike: Hits twice",
    "Entropic Drain: Steals HP",
    "Logic Shatter: Reduces MANA",
    "Chaos Strike: Unpredictable damage",
    "Temporal Stun: Skips your next turn",
    "Prime Curse: Lowers all stats by 3",
    "Lorenz Phase: Doubles next attack",
];

// Sprites stored as static strings
const SPRITE_IMP: &str =
    "  (o_o)\n  /||\\\n  d  b";

const SPRITE_WRAITH: &str =
    "  ~*~\n  \\|/\n   V";

const SPRITE_KNIGHT: &str =
    "  /-\\\n (X X)\n  |=|\n  / \\";

const SPRITE_GOLEM: &str =
    "  [#]\n  |#|\n /###\\";

const SPRITE_BOSS: &str =
    " /=====\\\n|(O) (O)|\n|  ~~~  |\n \\=====/\n  ||||";

const SPRITE_ABOMINATION: &str =
    "##############\n# [UNDEFINED] #\n# (x_INFINITY)#\n##############";

// ─── GENERATION ──────────────────────────────────────────────────────────────

pub fn generate_enemy(floor: u32, seed: u64) -> Enemy {
    let tier = determine_tier(floor, seed);
    let floor_scale = 1.0 + (floor as f64 - 1.0) * 0.18;

    let base_hp = roll_stat(15, 50, seed.wrapping_add(20));
    let base_dmg = roll_stat(3, 15, seed.wrapping_add(10));

    let hp = ((base_hp as f64 * floor_scale * tier.hp_multiplier()) as i64).max(1);
    let base_damage = ((base_dmg as f64 * floor_scale) as i64).max(1);
    let attack_modifier = roll_stat(0, 5 + floor as i64 / 2, seed.wrapping_add(30)) as i64;
    let chaos_level = (seed.wrapping_add(floor as u64) as f64 * 1e-13).fract();

    let xp = (15 + floor as u64 * 8) * tier.xp_multiplier();
    let gold = ((3.0 + floor as f64 * 2.0) * tier.gold_multiplier()) as i64;

    let (name, ascii_sprite) = pick_name_sprite(&tier, seed);
    let special_ability = pick_ability(&tier, seed);

    Enemy {
        name,
        tier,
        hp,
        max_hp: hp,
        base_damage,
        attack_modifier,
        chaos_level,
        xp_reward: xp,
        gold_reward: gold,
        ascii_sprite,
        seed,
        special_ability,
    }
}

fn determine_tier(floor: u32, seed: u64) -> EnemyTier {
    if floor % 5 == 0 {
        if floor >= 10 && seed % 3 == 0 {
            return EnemyTier::Abomination;
        }
        return EnemyTier::Boss;
    }

    let roll = chaos_roll_verbose(floor as f64 * 0.1, seed).final_value;
    match roll {
        r if r > 0.65 => EnemyTier::Champion,
        r if r > 0.2 => EnemyTier::Elite,
        _ => EnemyTier::Minion,
    }
}

fn pick_name_sprite(tier: &EnemyTier, seed: u64) -> (String, &'static str) {
    match tier {
        EnemyTier::Minion => {
            let idx = (seed % MINION_NAMES.len() as u64) as usize;
            let sprite = if seed % 2 == 0 { SPRITE_IMP } else { SPRITE_WRAITH };
            (MINION_NAMES[idx].to_string(), sprite)
        }
        EnemyTier::Elite => {
            let idx = (seed % ELITE_NAMES.len() as u64) as usize;
            (ELITE_NAMES[idx].to_string(), SPRITE_KNIGHT)
        }
        EnemyTier::Champion => {
            let idx = (seed % CHAMPION_NAMES.len() as u64) as usize;
            (CHAMPION_NAMES[idx].to_string(), SPRITE_GOLEM)
        }
        EnemyTier::Boss => {
            let idx = (seed % BOSS_NAMES.len() as u64) as usize;
            (BOSS_NAMES[idx].to_string(), SPRITE_BOSS)
        }
        EnemyTier::Abomination => {
            let idx = (seed % ABOMINATION_NAMES.len() as u64) as usize;
            (ABOMINATION_NAMES[idx].to_string(), SPRITE_ABOMINATION)
        }
    }
}

fn pick_ability(tier: &EnemyTier, seed: u64) -> Option<&'static str> {
    match tier {
        EnemyTier::Boss | EnemyTier::Abomination | EnemyTier::Champion => {
            let idx = (seed % SPECIAL_ABILITIES.len() as u64) as usize;
            Some(SPECIAL_ABILITIES[idx])
        }
        EnemyTier::Elite if seed % 2 == 0 => {
            let idx = (seed.wrapping_add(7) % SPECIAL_ABILITIES.len() as u64) as usize;
            Some(SPECIAL_ABILITIES[idx])
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_valid_stats() {
        for floor in [1u32, 3, 5, 10, 15] {
            for seed in [0u64, 42, 999] {
                let e = generate_enemy(floor, seed);
                assert!(e.hp > 0, "floor={} seed={} hp={}", floor, seed, e.hp);
                assert!(e.base_damage > 0);
                assert!(e.xp_reward > 0);
            }
        }
    }

    #[test]
    fn floor_5_spawns_boss() {
        for seed in 0..10u64 {
            let e = generate_enemy(5, seed);
            assert!(
                matches!(e.tier, EnemyTier::Boss | EnemyTier::Abomination),
                "Floor 5 should spawn boss"
            );
        }
    }

    #[test]
    fn hp_bar_works() {
        let e = generate_enemy(1, 42);
        let bar = e.hp_bar(20);
        assert!(bar.contains('/'));
    }
}
