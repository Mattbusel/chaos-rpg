//! The Chaos Pipeline — chaining mathematical engines to produce destiny.
//!
//! This is the core randomness engine of CHAOS RPG. Every roll threads
//! through 4–10 algorithms, each feeding its output into the next.
//! The result is deterministic chaos: same seed = same fate.

use crate::math_engines::{
    ALL_ENGINES, ENGINE_NAMES, MathEngine,
    lorenz_attractor, fourier_harmonic, prime_density_sieve, riemann_zeta_partial,
    fibonacci_golden_spiral, mandelbrot_escape, logistic_map, euler_totient,
    collatz_chain, modular_exp_hash,
};

/// A single step in the chaos chain
#[derive(Debug, Clone)]
pub struct ChainStep {
    pub engine_name: &'static str,
    pub input: f64,
    pub output: f64,
    pub seed_used: u64,
}

/// The verbose result of a chaos roll — includes full chain trace
#[derive(Debug, Clone)]
pub struct ChaosRollResult {
    pub final_value: f64,
    pub chain: Vec<ChainStep>,
    pub game_value: i64, // mapped to a usable integer range
}

impl ChaosRollResult {
    /// Map final_value [-1, 1] to [min, max]
    pub fn to_range(&self, min: i64, max: i64) -> i64 {
        let t = (self.final_value + 1.0) / 2.0; // [0, 1]
        let range = (max - min) as f64;
        (min as f64 + t * range).round() as i64
    }

    /// Interpret the result as a d20
    pub fn as_d20(&self) -> u8 {
        self.to_range(1, 20) as u8
    }

    /// Interpret as a percentage [0, 100]
    pub fn as_percent(&self) -> u8 {
        self.to_range(0, 100) as u8
    }

    /// True if the final value is in the top quartile (success)
    pub fn is_success(&self) -> bool {
        self.final_value > 0.5
    }

    /// True if the final value is in the top 10% (critical)
    pub fn is_critical(&self) -> bool {
        self.final_value > 0.8
    }

    /// True if the final value is in the bottom 10% (catastrophic failure)
    pub fn is_catastrophe(&self) -> bool {
        self.final_value < -0.8
    }

    /// Generate verbose display lines for skill check UI
    pub fn display_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push("  ╔═══════════════════════════════════════════╗".to_string());
        lines.push("  ║         CHAOS ENGINE CHAIN TRACE          ║".to_string());
        lines.push("  ╠═══════════════════════════════════════════╣".to_string());

        for (i, step) in self.chain.iter().enumerate() {
            let bar_len = ((step.output + 1.0) / 2.0 * 20.0) as usize;
            let bar = "█".repeat(bar_len) + &"░".repeat(20 - bar_len);
            lines.push(format!(
                "  ║ {:2}. {:22} {:+.4} ║",
                i + 1,
                step.engine_name,
                step.output
            ));
            lines.push(format!("  ║     [{}] ║", bar));
        }

        lines.push("  ╠═══════════════════════════════════════════╣".to_string());

        let result_icon = if self.is_critical() {
            "★ CRITICAL"
        } else if self.is_catastrophe() {
            "☠ CATASTROPHE"
        } else if self.is_success() {
            "✓ SUCCESS"
        } else {
            "✗ FAILURE"
        };

        lines.push(format!(
            "  ║  RESULT: {:+.6}  {}      ║",
            self.final_value, result_icon
        ));
        lines.push("  ╚═══════════════════════════════════════════╝".to_string());
        lines
    }
}

/// Chain N randomly-selected engines, verbose output
pub fn chaos_roll_verbose(input: f64, seed: u64) -> ChaosRollResult {
    // Determine chain length (4-8) from seed
    let chain_len = (seed % 5 + 4) as usize; // 4..=8
    let mut chain = Vec::with_capacity(chain_len);
    let mut value = input;

    for i in 0..chain_len {
        // Pick engine deterministically from seed+position
        let engine_idx = (seed.wrapping_mul(2654435761).wrapping_add(i as u64 * 1234567891))
            as usize
            % ALL_ENGINES.len();
        let engine_seed = seed.wrapping_add(i as u64 * 9999991).wrapping_mul(6364136223846793005);
        let engine = ALL_ENGINES[engine_idx];
        let output = engine(value, engine_seed);

        chain.push(ChainStep {
            engine_name: ENGINE_NAMES[engine_idx],
            input: value,
            output,
            seed_used: engine_seed,
        });

        value = output;
    }

    let game_value = ((value + 1.0) / 2.0 * 100.0).round() as i64;

    ChaosRollResult {
        final_value: value,
        chain,
        game_value,
    }
}

/// Run ALL 10 engines in sequence — the destiny roll. Used for character creation.
pub fn destiny_roll(input: f64, seed: u64) -> ChaosRollResult {
    let engines: &[(MathEngine, &str)] = &[
        (lorenz_attractor, "Lorenz Attractor"),
        (fourier_harmonic, "Fourier Harmonic"),
        (prime_density_sieve, "Prime Density Sieve"),
        (riemann_zeta_partial, "Riemann Zeta Partial"),
        (fibonacci_golden_spiral, "Fibonacci Golden Spiral"),
        (mandelbrot_escape, "Mandelbrot Escape"),
        (logistic_map, "Logistic Map"),
        (euler_totient, "Euler's Totient"),
        (collatz_chain, "Collatz Chain"),
        (modular_exp_hash, "Modular Exp Hash"),
    ];

    let mut chain = Vec::with_capacity(10);
    let mut value = input;

    for (i, (engine, name)) in engines.iter().enumerate() {
        let engine_seed =
            seed.wrapping_mul(2654435761u64.wrapping_add(i as u64)).wrapping_add(i as u64 * 9876543211);
        let output = engine(value, engine_seed);
        chain.push(ChainStep {
            engine_name: name,
            input: value,
            output,
            seed_used: engine_seed,
        });
        value = output;
    }

    let game_value = ((value + 1.0) / 2.0 * 100.0).round() as i64;
    ChaosRollResult { final_value: value, chain, game_value }
}

/// Bias a chaos roll toward positive or negative outcomes.
/// bias: 1.0 = full positive, -1.0 = full negative, 0.0 = pure chaos.
pub fn biased_chaos_roll(input: f64, bias: f64, seed: u64) -> ChaosRollResult {
    let raw = chaos_roll_verbose(input, seed);
    let bias = bias.clamp(-1.0, 1.0);
    // Blend raw result with bias
    let blended = raw.final_value * (1.0 - bias.abs()) + bias * bias.abs();
    let blended = blended.clamp(-1.0, 1.0);

    // Rebuild with blended final value
    let mut result = raw;
    result.final_value = blended;
    result.game_value = ((blended + 1.0) / 2.0 * 100.0).round() as i64;
    result
}

/// Quick single-engine roll for fast lookups
pub fn quick_roll(seed: u64, stat_modifier: i32) -> i64 {
    let engine_idx = (seed % ALL_ENGINES.len() as u64) as usize;
    let raw = ALL_ENGINES[engine_idx](seed as f64 * 1e-9, seed);
    let base = ((raw + 1.0) / 2.0 * 20.0 + 1.0) as i64;
    (base + stat_modifier as i64).max(1)
}

/// Roll a stat value in [min, max] with given seed
pub fn roll_stat(min: i64, max: i64, seed: u64) -> i64 {
    let result = chaos_roll_verbose(seed as f64 * 1.618033988749895e-13, seed);
    result.to_range(min, max)
}

/// Roll a damage value with attacker's relevant stat as bias
pub fn roll_damage(base_damage: i64, attacker_stat: i64, seed: u64) -> i64 {
    // Stat above 50 biases toward more damage
    let bias = (attacker_stat as f64 - 50.0) / 100.0;
    let result = biased_chaos_roll(seed as f64 * 1e-10, bias, seed);
    let multiplier = (result.final_value + 1.5).max(0.1); // [0.1, 2.5]
    let dmg = (base_damage as f64 * multiplier).round() as i64;
    dmg.max(1)
}

/// The Chaos Pipeline struct — configure and execute custom chains
pub struct ChaosPipeline {
    pub engines: Vec<(MathEngine, &'static str)>,
    pub seed_modifier: u64,
}

impl ChaosPipeline {
    pub fn new() -> Self {
        Self {
            engines: Vec::new(),
            seed_modifier: 1,
        }
    }

    pub fn add_engine(&mut self, engine: MathEngine, name: &'static str) -> &mut Self {
        self.engines.push((engine, name));
        self
    }

    pub fn with_seed_modifier(&mut self, modifier: u64) -> &mut Self {
        self.seed_modifier = modifier;
        self
    }

    pub fn execute(&self, input: f64, seed: u64) -> ChaosRollResult {
        let mut chain = Vec::with_capacity(self.engines.len());
        let mut value = input;

        for (i, (engine, name)) in self.engines.iter().enumerate() {
            let engine_seed =
                seed.wrapping_mul(self.seed_modifier).wrapping_add(i as u64 * 777777777);
            let output = engine(value, engine_seed);
            chain.push(ChainStep {
                engine_name: name,
                input: value,
                output,
                seed_used: engine_seed,
            });
            value = output;
        }

        let game_value = ((value + 1.0) / 2.0 * 100.0).round() as i64;
        ChaosRollResult { final_value: value, chain, game_value }
    }
}

impl Default for ChaosPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chaos_roll_verbose_in_range() {
        for seed in [0u64, 1, 42, 9999, u64::MAX / 2] {
            let result = chaos_roll_verbose(0.5, seed);
            assert!(result.final_value >= -1.0 && result.final_value <= 1.0);
            assert!(!result.chain.is_empty());
            assert!(result.game_value >= 0 && result.game_value <= 100);
        }
    }

    #[test]
    fn destiny_roll_uses_all_10_engines() {
        let result = destiny_roll(0.5, 42);
        assert_eq!(result.chain.len(), 10);
    }

    #[test]
    fn bias_positive_shifts_result() {
        let mut positive_count = 0;
        for seed in 0..20u64 {
            let biased = biased_chaos_roll(0.0, 1.0, seed);
            if biased.final_value > 0.0 {
                positive_count += 1;
            }
        }
        assert!(positive_count >= 18, "Positive bias should mostly give positive results");
    }

    #[test]
    fn roll_stat_in_range() {
        for seed in 0..50u64 {
            let val = roll_stat(1, 100, seed);
            assert!(val >= 1 && val <= 100, "roll_stat out of range: {}", val);
        }
    }

    #[test]
    fn pipeline_executes_custom_chain() {
        let mut p = ChaosPipeline::new();
        p.add_engine(lorenz_attractor, "Lorenz Attractor");
        p.add_engine(logistic_map, "Logistic Map");
        let result = p.execute(0.5, 42);
        assert_eq!(result.chain.len(), 2);
    }
}
