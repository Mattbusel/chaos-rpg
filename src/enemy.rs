//! Enemy generation via chaos algorithms.
//!
//! Every enemy is procedurally generated — name, stats, abilities, loot.
//! Floor number and seed determine what horrors await.

use crate::chaos_pipeline::{roll_stat, chaos_roll_verbose, destiny_roll};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnemyTier {
    Minion,
    Elite,
    Champion,
    Boss,
    Abomination, // floor 10+ only
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
            EnemyTier::Boss => 4.0,
            EnemyTier::Abomination => 10.0,
        }
    }

    pub fn xp_multiplier(&self) -> u64 {
        match self {
            EnemyTier::Minion => 1,
            EnemyTier::Elite => 3,
            EnemyTier::Champion => 7,
            EnemyTier::Boss => 20,
            EnemyTier::Abomination => 100,
        }
    }

    pub fn gold_multiplier(&self) -> f64 {
        match self {
            EnemyTier::Minion => 0.5,
            EnemyTier::Elite => 1.0,
            EnemyTier::Champion => 2.5,
            EnemyTier::Boss => 6.0,
            EnemyTier::Abomination => 25.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyAbility {
    pub name: &'static str,
    pub description: &'static str,
    pub damage_bonus: i64,
    pub effect: AbilityEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbilityEffect {
    DoubleDamage,
    Lifesteal(i64),      // heals this % of damage dealt
    StatDrain(StatDrained),
    ChaosStrike,         // re-rolls with different seed
    Stun,                // player skips next turn
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatDrained {
    Force,
    Mana,
    Luck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub name: String,
    pub tier: EnemyTier,
    pub max_hp: i64,
    pub current_hp: i64,
    pub force: i64,
    pub resilience: i64,  // damage reduction
    pub speed: i64,       // initiative modifier
    pub xp_reward: u64,
    pub gold_reward: i64,
    pub ability: Option<EnemyAbility>,
    pub ascii_sprite: &'static str,
    pub seed: u64,
}

impl Enemy {
    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    pub fn take_damage(&mut self, amount: i64) {
        let actual = (amount - self.resilience).max(1);
        self.current_hp = (self.current_hp - actual).max(0);
    }

    pub fn hp_percent(&self) -> f64 {
        (self.current_hp as f64 / self.max_hp as f64).clamp(0.0, 1.0)
    }

    pub fn hp_bar(&self, width: usize) -> String {
        let filled = ((self.hp_percent() * width as f64) as usize).min(width);
        let bar = "█".repeat(filled) + &"░".repeat(width - filled);
        format!("[{}] {}/{}", bar, self.current_hp, self.max_hp)
    }
}

// ─── ENEMY ARCHETYPE POOLS ───────────────────────────────────────────────────

const MINION_NAMES: &[&str] = &[
    "Fractal Imp", "Entropy Sprite", "Logic Wraith", "Sigma Rat", "Null Bat",
    "Zeta Goblin", "Drift Slime", "Collatz Shade", "Prime Leech", "Fourier Gnat",
];

const ELITE_NAMES: &[&str] = &[
    "Lorenz Stalker", "Mandelbrot Hound", "Riemann Specter", "Golden Asp",
    "Logistic Troll", "Euler Revenant", "Modular Knight", "Fibonacci Worm",
];

const CHAMPION_NAMES: &[&str] = &[
    "The Divergence", "Attractor Beast", "Phase-Lock Horror", "Sieve Colossus",
    "Spiral Tyrant", "Zeta Construct", "Totient Golem", "Chaos Shepherd",
];

const BOSS_NAMES: &[&str] = &[
    "Lord Mandelbrot", "The Strange Attractor", "Absolute Uncertainty",
    "The Null Hypothesis", "Infinite Regress", "Bifurcation King",
];

const ABOMINATION_NAMES: &[&str] = &[
    "THE HEAT DEATH", "RECURSIVE INFINITY", "GÖDEL'S DAEMON", "THE P≠NP PROOF",
];

const MINION_SPRITES: &[&'static str] = &[
    "  (o_o)\n  /||\\\n  d  b",
    "  >::\n  /||\n   /\\",
    "  ~*~\n  \\|/\n   V",
];

const ELITE_SPRITES: &[&'static str] = &[
    "  /-\\\n (X X)\n  |=|\n  / \\",
    "  [#]\n  |#|\n /###\\",
];

const BOSS_SPRITES: &[&'static str] = &[
    " /=====\\\n| (O) (O) |\n|   WW   |\n \\======/\n   ||||",
    "  *****\n *     *\n*(>_<)*\n *     *\n  *****",
];

const ABOMINATION_SPRITE: &str =
    "██████████████\n█ ∞∞∞∞∞∞∞∞∞∞ █\n█ (UNDEFINED) █\n█ ∞∞∞∞∞∞∞∞∞∞ █\n██████████████";

const ABILITIES: &[EnemyAbility] = &[
    EnemyAbility {
        name: "Bifurcation Strike",
        description: "Hits twice with chaotic variance",
        damage_bonus: 5,
        effect: AbilityEffect::DoubleDamage,
    },
    EnemyAbility {
        name: "Entropic Drain",
        description: "Siphons life force",
        damage_bonus: 0,
        effect: AbilityEffect::Lifesteal(30),
    },
    EnemyAbility {
        name: "Logic Shatter",
        description: "Temporarily reduces your MANA",
        damage_bonus: 3,
        effect: AbilityEffect::StatDrain(StatDrained::Mana),
    },
    EnemyAbility {
        name: "Chaos Strike",
        description: "Unpredictable — could do anything",
        damage_bonus: 0,
        effect: AbilityEffect::ChaosStrike,
    },
    EnemyAbility {
        name: "Temporal Stun",
        description: "You lose your next action",
        damage_bonus: 2,
        effect: AbilityEffect::Stun,
    },
];

// ─── GENERATION ──────────────────────────────────────────────────────────────

/// Generate an enemy appropriate for the given floor
pub fn generate_enemy(floor: u32, seed: u64) -> Enemy {
    let tier = determine_tier(floor, seed);
    let floor_scale = 1.0 + (floor as f64 - 1.0) * 0.15;

    let base_force = roll_stat(5, 20, seed.wrapping_add(10));
    let base_hp = roll_stat(20, 60, seed.wrapping_add(20));
    let resilience = roll_stat(0, 5 + floor as i64 / 3, seed.wrapping_add(30));
    let speed = roll_stat(-5, 10, seed.wrapping_add(40));

    let force = ((base_force as f64 * floor_scale) as i64).max(1);
    let hp = ((base_hp as f64 * floor_scale * tier.hp_multiplier()) as i64).max(1);

    let xp = (20 + floor as u64 * 10) * tier.xp_multiplier();
    let gold = ((5.0 + floor as f64 * 3.0) * tier.gold_multiplier()) as i64;

    let (name, sprite) = pick_name_and_sprite(&tier, seed);

    // Bosses and champions always get an ability; elites sometimes
    let ability = match tier {
        EnemyTier::Boss | EnemyTier::Abomination | EnemyTier::Champion => {
            let idx = (seed % ABILITIES.len() as u64) as usize;
            Some(ABILITIES[idx].clone())
        }
        EnemyTier::Elite => {
            if seed % 2 == 0 {
                let idx = (seed.wrapping_add(7) % ABILITIES.len() as u64) as usize;
                Some(ABILITIES[idx].clone())
            } else {
                None
            }
        }
        EnemyTier::Minion => None,
    };

    Enemy {
        name,
        tier,
        max_hp: hp,
        current_hp: hp,
        force,
        resilience,
        speed,
        xp_reward: xp,
        gold_reward: gold,
        ability,
        ascii_sprite: sprite,
        seed,
    }
}

fn determine_tier(floor: u32, seed: u64) -> EnemyTier {
    // Boss every 5 floors
    if floor % 5 == 0 {
        if floor >= 10 && seed % 4 == 0 {
            return EnemyTier::Abomination;
        }
        return EnemyTier::Boss;
    }

    let roll = chaos_roll_verbose(floor as f64 * 0.1, seed).final_value;
    match roll {
        r if r > 0.7 => EnemyTier::Champion,
        r if r > 0.3 => EnemyTier::Elite,
        _ => EnemyTier::Minion,
    }
}

fn pick_name_and_sprite(tier: &EnemyTier, seed: u64) -> (String, &'static str) {
    match tier {
        EnemyTier::Minion => {
            let idx = (seed % MINION_NAMES.len() as u64) as usize;
            let sprite_idx = (seed % MINION_SPRITES.len() as u64) as usize;
            (MINION_NAMES[idx].to_string(), MINION_SPRITES[sprite_idx])
        }
        EnemyTier::Elite => {
            let idx = (seed % ELITE_NAMES.len() as u64) as usize;
            let sprite_idx = (seed % ELITE_SPRITES.len() as u64) as usize;
            (ELITE_NAMES[idx].to_string(), ELITE_SPRITES[sprite_idx])
        }
        EnemyTier::Champion => {
            let idx = (seed % CHAMPION_NAMES.len() as u64) as usize;
            let sprite_idx = (seed % ELITE_SPRITES.len() as u64) as usize;
            (CHAMPION_NAMES[idx].to_string(), ELITE_SPRITES[sprite_idx])
        }
        EnemyTier::Boss => {
            let idx = (seed % BOSS_NAMES.len() as u64) as usize;
            let sprite_idx = (seed % BOSS_SPRITES.len() as u64) as usize;
            (BOSS_NAMES[idx].to_string(), BOSS_SPRITES[sprite_idx])
        }
        EnemyTier::Abomination => {
            let idx = (seed % ABOMINATION_NAMES.len() as u64) as usize;
            (ABOMINATION_NAMES[idx].to_string(), ABOMINATION_SPRITE)
        }
    }
}

/// Generate a description of why this enemy exists
pub fn enemy_lore(enemy: &Enemy) -> String {
    let roll = destiny_roll(enemy.seed as f64 * 1e-12, enemy.seed);
    let chaos_value = roll.final_value;

    if chaos_value > 0.5 {
        format!(
            "Born from a {} collapse in the {} layer of mathematical reality.",
            if chaos_value > 0.8 { "catastrophic" } else { "minor" },
            ["Fourier", "Lorenz", "Mandelbrot", "Riemann"]
                [(enemy.seed % 4) as usize]
        )
    } else {
        format!(
            "A {}-tier entity summoned by {} divergence.",
            enemy.tier.name(),
            ["prime density", "logistic map", "collatz", "totient"]
                [(enemy.seed % 4) as usize]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enemy_generation_produces_valid_stats() {
        for floor in [1u32, 3, 5, 10, 15, 20] {
            for seed in [0u64, 42, 999] {
                let e = generate_enemy(floor, seed);
                assert!(e.max_hp > 0);
                assert!(e.force > 0);
                assert!(e.xp_reward > 0);
            }
        }
    }

    #[test]
    fn floor_5_always_boss() {
        for seed in 0..10u64 {
            let e = generate_enemy(5, seed);
            assert!(
                matches!(e.tier, EnemyTier::Boss | EnemyTier::Abomination),
                "Floor 5 should spawn boss, got {:?}",
                e.tier
            );
        }
    }
}
