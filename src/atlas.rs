//! Infinite Atlas endgame system.
//!
//! The atlas is a procedurally generated grid of zones.
//! Completing a zone reveals adjacent zones.
//! Every 10 zones cleared, an engine-themed conqueror spawns.
//! At depth 100: The Algorithm. The chaos pipeline itself. You have to destabilize it.

use crate::chaos_pipeline::chaos_roll_verbose;
use serde::{Deserialize, Serialize};

// ─── ZONE MODIFIER ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZoneModifier {
    /// All Collatz chains start from even numbers
    CollatzEven,
    /// All Mandelbrot samples from inside the set (all outputs negative)
    MandelbrotInside,
    /// Enemies have double engine count in their chains
    EnemyDoubleEngines,
    /// Items drop with +2 sockets
    BonusSockets,
    /// DEX and STR swap for all calculations (gravity reversed)
    GravityReversed,
    /// All engine outputs are squared (negatives become positive!)
    SquaredOutputs,
    /// Lorenz attractor uses different constants (sigma=20, rho=50)
    LorenzAmplified,
    /// All chaos rolls use 2 fewer engines (min 2)
    ReducedEngines,
    /// Gold drops multiplied x3
    GoldRush,
    /// Player and enemy HP halved at zone entry
    BloodPact,
    /// Critical hit threshold is 50 (very easy to crit)
    HairTrigger,
    /// Engine order is shuffled every round
    EngineShuffle,
    /// Injuries don't heal between fights in this zone
    OpenWounds,
    /// All stat bonuses from equipment are doubled
    EmpoweredGear,
}

impl ZoneModifier {
    pub fn name(&self) -> &'static str {
        match self {
            ZoneModifier::CollatzEven => "Collatz: Even Start",
            ZoneModifier::MandelbrotInside => "Mandelbrot: Interior Sampling",
            ZoneModifier::EnemyDoubleEngines => "Enemy Double Engines",
            ZoneModifier::BonusSockets => "+2 Item Sockets",
            ZoneModifier::GravityReversed => "Gravity Reversed",
            ZoneModifier::SquaredOutputs => "Squared Engine Outputs",
            ZoneModifier::LorenzAmplified => "Lorenz Amplified",
            ZoneModifier::ReducedEngines => "Reduced Engine Count",
            ZoneModifier::GoldRush => "Gold Rush x3",
            ZoneModifier::BloodPact => "Blood Pact (half HP)",
            ZoneModifier::HairTrigger => "Hair Trigger (crit at 50)",
            ZoneModifier::EngineShuffle => "Engine Shuffle",
            ZoneModifier::OpenWounds => "Open Wounds (injuries persist)",
            ZoneModifier::EmpoweredGear => "Empowered Gear x2",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ZoneModifier::CollatzEven => {
                "All Collatz chains start from even numbers (halving immediately -- less volatile)"
            }
            ZoneModifier::MandelbrotInside => {
                "Mandelbrot samples from inside the set only -- all Mandelbrot outputs negative"
            }
            ZoneModifier::EnemyDoubleEngines => "Enemies use twice as many engines in their chains",
            ZoneModifier::BonusSockets => "All dropped items have +2 bonus sockets",
            ZoneModifier::GravityReversed => {
                "Force and Luck swap. Precision and Cunning swap. Physics disagrees."
            }
            ZoneModifier::SquaredOutputs => {
                "All engine outputs squared. Negatives become positive. Nothing is safe."
            }
            ZoneModifier::LorenzAmplified => {
                "Lorenz attractor with sigma=20, rho=50. The butterfly has grown."
            }
            ZoneModifier::ReducedEngines => "All chains use 2 fewer engines (minimum 2)",
            ZoneModifier::GoldRush => "Gold drops from all sources multiplied x3",
            ZoneModifier::BloodPact => "Both player and enemies enter at 50% HP",
            ZoneModifier::HairTrigger => "Critical hits trigger at roll > 50 (instead of 90)",
            ZoneModifier::EngineShuffle => {
                "Engine order randomized every round. The chain mutates."
            }
            ZoneModifier::OpenWounds => {
                "Body part injuries do not recover between fights in this zone"
            }
            ZoneModifier::EmpoweredGear => "All equipment stat bonuses doubled",
        }
    }

    pub fn is_beneficial(&self) -> bool {
        matches!(
            self,
            ZoneModifier::BonusSockets
                | ZoneModifier::GoldRush
                | ZoneModifier::HairTrigger
                | ZoneModifier::EmpoweredGear
        )
    }

    pub fn color(&self) -> &'static str {
        if self.is_beneficial() {
            "\x1b[32m"
        } else {
            "\x1b[31m"
        }
    }

    pub fn generate(seed: u64) -> Self {
        let modifiers = [
            ZoneModifier::CollatzEven,
            ZoneModifier::MandelbrotInside,
            ZoneModifier::EnemyDoubleEngines,
            ZoneModifier::BonusSockets,
            ZoneModifier::GravityReversed,
            ZoneModifier::SquaredOutputs,
            ZoneModifier::LorenzAmplified,
            ZoneModifier::ReducedEngines,
            ZoneModifier::GoldRush,
            ZoneModifier::BloodPact,
            ZoneModifier::HairTrigger,
            ZoneModifier::EngineShuffle,
            ZoneModifier::OpenWounds,
            ZoneModifier::EmpoweredGear,
        ];
        let idx = (seed % modifiers.len() as u64) as usize;
        modifiers[idx].clone()
    }
}

// ─── ZONE ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub depth: u32,
    pub modifiers: Vec<ZoneModifier>,
    pub zone_type: ZoneType,
    pub cleared: bool,
    pub revealed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneType {
    Combat,
    Boss,
    Crafting,
    Conqueror,         // engine-themed boss every 10 clears
    TheFinalAlgorithm, // depth 100
    NpcHub,
    SecretAnomaly,
}

impl ZoneType {
    pub fn name(self) -> &'static str {
        match self {
            ZoneType::Combat => "Combat",
            ZoneType::Boss => "Boss",
            ZoneType::Crafting => "Crafting Bench",
            ZoneType::Conqueror => "CONQUEROR",
            ZoneType::TheFinalAlgorithm => "THE ALGORITHM",
            ZoneType::NpcHub => "Hub",
            ZoneType::SecretAnomaly => "??? Anomaly",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            ZoneType::Combat => "[x]",
            ZoneType::Boss => "[B]",
            ZoneType::Crafting => "[C]",
            ZoneType::Conqueror => "[!]",
            ZoneType::TheFinalAlgorithm => "[∞]",
            ZoneType::NpcHub => "[H]",
            ZoneType::SecretAnomaly => "[?]",
        }
    }
}

impl Zone {
    pub fn generate(id: u32, x: i32, y: i32, depth: u32, seed: u64) -> Self {
        // Zone type determined by depth and seed
        let zone_type = if depth == 100 {
            ZoneType::TheFinalAlgorithm
        } else if depth.is_multiple_of(10) && depth > 0 {
            ZoneType::Conqueror
        } else {
            let roll = seed % 100;
            match roll {
                0..=4 => ZoneType::SecretAnomaly, // 5% secret
                5..=14 => ZoneType::Crafting,     // 10% crafting
                15..=19 => ZoneType::NpcHub,      // 5% hub
                20..=29 => ZoneType::Boss,        // 10% boss
                _ => ZoneType::Combat,            // 70% combat
            }
        };

        // 1-3 zone modifiers, chaos-rolled
        let n_mods = 1 + (seed.wrapping_mul(7) % 3) as usize;
        let mut modifiers = Vec::new();
        for i in 0..n_mods {
            let mod_seed = seed.wrapping_add(i as u64 * 99991);
            modifiers.push(ZoneModifier::generate(mod_seed));
        }

        // Secret anomaly zones are only revealed by high Luck
        let revealed = zone_type != ZoneType::SecretAnomaly;

        Zone {
            id,
            x,
            y,
            depth,
            modifiers,
            zone_type,
            cleared: false,
            revealed,
        }
    }
}

// ─── ATLAS ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atlas {
    pub zones: Vec<Zone>,
    pub zones_cleared: u32,
    pub current_zone: u32,
    pub seed: u64,
}

impl Atlas {
    pub fn new(seed: u64) -> Self {
        let mut atlas = Atlas {
            zones: Vec::new(),
            zones_cleared: 0,
            current_zone: 0,
            seed,
        };
        // Generate starting zone + 3 initial neighbors
        atlas.generate_zone(0, 0, 0, 1);
        atlas.generate_neighbors(0);
        atlas
    }

    fn generate_zone(&mut self, x: i32, y: i32, parent_depth: u32, branch: u32) -> u32 {
        let id = self.zones.len() as u32;
        let depth = parent_depth + 1;
        let zone_seed = self
            .seed
            .wrapping_add(id as u64 * 31337)
            .wrapping_add(x as u64 * 999983)
            .wrapping_add(y as u64 * 1000003)
            .wrapping_add(branch as u64);
        let zone = Zone::generate(id, x, y, depth, zone_seed);
        self.zones.push(zone);
        id
    }

    fn generate_neighbors(&mut self, zone_id: u32) {
        let (x, y, depth) = {
            let z = &self.zones[zone_id as usize];
            (z.x, z.y, z.depth)
        };
        // Generate 2-3 branching paths forward
        let n_paths = 2 + (zone_id % 2); // 2 or 3 paths
        for i in 0..n_paths {
            let nx = x + (i as i32 - 1);
            let ny = y + 1;
            // Don't generate if zone already exists at this position
            if !self.zones.iter().any(|z| z.x == nx && z.y == ny) {
                self.generate_zone(nx, ny, depth, i);
            }
        }
    }

    /// Complete current zone and reveal neighbors.
    pub fn clear_zone(&mut self, zone_id: u32) {
        if let Some(z) = self.zones.iter_mut().find(|z| z.id == zone_id) {
            z.cleared = true;
        }
        self.zones_cleared += 1;
        self.generate_neighbors(zone_id);

        // Reveal secret anomalies based on Luck (called externally with luck value)
    }

    /// Reveal secret anomaly zones based on player Luck
    pub fn try_reveal_secrets(&mut self, luck: i64, seed: u64) {
        let roll = chaos_roll_verbose(luck as f64 * 0.01, seed);
        if roll.final_value > 0.7 {
            for zone in self.zones.iter_mut() {
                if !zone.revealed && zone.zone_type == ZoneType::SecretAnomaly {
                    zone.revealed = true;
                }
            }
        }
    }

    /// Get available next zones from current position
    pub fn available_zones(&self) -> Vec<&Zone> {
        let current = self.zones.iter().find(|z| z.id == self.current_zone);
        if let Some(curr) = current {
            let cx = curr.x;
            let cy = curr.y;
            self.zones
                .iter()
                .filter(|z| z.revealed && !z.cleared && (z.x - cx).abs() <= 1 && z.y == cy + 1)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// ASCII minimap of the atlas (shows nearby zones)
    pub fn render_minimap(&self, _width: usize) -> Vec<String> {
        let reset = "\x1b[0m";
        let mut lines = Vec::new();

        // Find current zone
        let Some(curr) = self.zones.iter().find(|z| z.id == self.current_zone) else {
            return lines;
        };

        // Show a window around current zone
        let view_range = 4i32;
        for dy in (-view_range..=view_range).rev() {
            let mut row = String::new();
            for dx in -view_range..=view_range {
                let wx = curr.x + dx;
                let wy = curr.y + dy;
                if let Some(zone) = self.zones.iter().find(|z| z.x == wx && z.y == wy) {
                    if zone.id == self.current_zone {
                        row.push_str("\x1b[97m@\x1b[0m");
                    } else if zone.cleared {
                        row.push_str("\x1b[90m.\x1b[0m");
                    } else if !zone.revealed {
                        row.push_str("\x1b[35m?\x1b[0m");
                    } else {
                        let col = match zone.zone_type {
                            ZoneType::Boss | ZoneType::Conqueror => "\x1b[31m",
                            ZoneType::Crafting => "\x1b[33m",
                            ZoneType::TheFinalAlgorithm => "\x1b[97m",
                            ZoneType::SecretAnomaly => "\x1b[95m",
                            ZoneType::NpcHub => "\x1b[36m",
                            _ => "\x1b[32m",
                        };
                        row.push_str(&format!(
                            "{}{}{}",
                            col,
                            zone.zone_type.icon().chars().next().unwrap_or('x'),
                            reset
                        ));
                    }
                } else {
                    row.push(' ');
                }
            }
            lines.push(row);
        }
        lines
    }
}

// ─── CONQUEROR ────────────────────────────────────────────────────────────────

/// Engine-themed boss, spawns every 10 atlas zones.
#[derive(Debug, Clone)]
pub struct Conqueror {
    pub engine: &'static str,
    pub name: &'static str,
    pub mechanic: &'static str,
    pub hp_behavior: &'static str,
}

pub const CONQUERORS: &[Conqueror] = &[
    Conqueror {
        engine: "Lorenz Attractor",
        name: "The Lorenz Conqueror",
        mechanic: "Attacks butterfly -- tiny hits that randomly spike to catastrophic damage",
        hp_behavior: "HP regenerates based on the Lorenz z-component. Unpredictable bursts.",
    },
    Conqueror {
        engine: "Collatz Chain",
        name: "The Collatz Conqueror",
        mechanic: "HP halves and triples unpredictably. You can't read its health bar.",
        hp_behavior: "Follows 3n+1 trajectory. Sometimes collapses fast; sometimes takes 100+ rounds.",
    },
    Conqueror {
        engine: "Mandelbrot",
        name: "The Mandelbrot Conqueror",
        mechanic: "Phases between vulnerable (outside set) and invulnerable (inside set) states",
        hp_behavior: "Phase determined by chaos roll. You have to hit it in its vulnerable state.",
    },
    Conqueror {
        engine: "Riemann Zeta",
        name: "The Zeta Conqueror",
        mechanic: "Attacks on the critical line. Every hit has unpredictable scaling.",
        hp_behavior: "HP pool is its partial zeta sum. Shrinks in oscillating waves.",
    },
    Conqueror {
        engine: "Prime Density",
        name: "The Prime Conqueror",
        mechanic: "Immune on prime-numbered rounds. Every prime round is a free hit for it.",
        hp_behavior: "Counts prime floors descended. High prime density = more frequent immunity.",
    },
    Conqueror {
        engine: "Fibonacci",
        name: "The Fibonacci Conqueror",
        mechanic: "Each attack is exactly phi times stronger than the last.",
        hp_behavior: "HP is a Fibonacci sequence. 1, 1, 2, 3, 5, 8... until collapse.",
    },
    Conqueror {
        engine: "Logistic Map",
        name: "The Logistic Conqueror",
        mechanic: "At r=3.9, fully chaotic. Attacks period-double until they bifurcate entirely.",
        hp_behavior: "HP at x_{n+1} = r*x*(1-x). Can self-destruct via bifurcation cascade.",
    },
    Conqueror {
        engine: "Euler Totient",
        name: "The Totient Conqueror",
        mechanic: "Armor scales with phi(n)/n ratio. Highly composite HP = low defense. Prime HP = near immune.",
        hp_behavior: "Totient function applied to HP each round. Wildly irregular recovery.",
    },
    Conqueror {
        engine: "Fourier",
        name: "The Fourier Conqueror",
        mechanic: "Sums sinusoidal attack waves. Constructive interference = devastating combo.",
        hp_behavior: "HP oscillates as harmonic series. Sometimes spikes; sometimes approaches zero.",
    },
    Conqueror {
        engine: "Modular Hash",
        name: "The Modular Conqueror",
        mechanic: "Small stat changes avalanche into massive outcomes. Unpredictable scaling.",
        hp_behavior: "HP mod p where p changes each round. Can jump from 1 to max unexpectedly.",
    },
];

pub fn conqueror_for_zone(zones_cleared: u32) -> &'static Conqueror {
    let idx = ((zones_cleared / 10 - 1) as usize) % CONQUERORS.len();
    &CONQUERORS[idx]
}

// ─── THE FINAL BOSS ───────────────────────────────────────────────────────────

/// Description of the final boss at atlas depth 100.
pub fn the_algorithm_description() -> Vec<&'static str> {
    vec![
        "THE ALGORITHM",
        "",
        "It has no stats. It has no HP. It IS the chaos pipeline.",
        "",
        "To win, you must destabilize it:",
        "  - Each action you take adds or removes an engine from its chain",
        "  - Get its chain to a state where output > input threshold",
        "  - The pipeline will collapse into itself",
        "",
        "You can brute-force it with enough power.",
        "Understanding the math wins it in under 10 rounds.",
        "",
        "The math is your only weapon here.",
    ]
}

// ─── DAILY SEED ───────────────────────────────────────────────────────────────

/// Generate the daily seed from the current date (YYYY * 10000 + MM * 100 + DD).
/// Same seed produces the same map, same character stats, same enemies for everyone on this day.
pub fn daily_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Seconds to days; then encode as YYYYMMDD via simple division
    let days_since_epoch = secs / 86400;
    // Derive a date-like number: use days * large prime for distribution
    days_since_epoch.wrapping_mul(6364136223846793005)
}

/// Return today's date string for display
pub fn daily_seed_label() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86400;
    // Approximate date from epoch (good enough for a seed label)
    // 2024-01-01 = day 19723 from Unix epoch
    let seed_num = days.wrapping_mul(6364136223846793005);
    format!("DAILY-{}", seed_num % 1_000_000)
}
