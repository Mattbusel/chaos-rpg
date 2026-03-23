//! The Chaos Pipeline — chaining mathematical engines to produce destiny.
//!
//! This is the core randomness engine of CHAOS RPG. Every roll threads
//! through 4–10 algorithms, each feeding its output into the next.
//! The result is deterministic chaos: same seed = same fate.

use crate::math_engines::{
    collatz_chain, euler_totient, fibonacci_golden_spiral, fourier_harmonic, logistic_map,
    lorenz_attractor, mandelbrot_escape, modular_exp_hash, prime_density_sieve,
    riemann_zeta_partial, MathEngine, ALL_ENGINES, ENGINE_NAMES,
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

    /// Inline combat trace — shows the engine chain with narrative flavor.
    /// Printed automatically after every combat action so players always see
    /// *which algorithms conspired to produce this result*.
    pub fn combat_trace_lines(&self, action: &str, outcome: &str) -> Vec<String> {
        let w = 56usize; // inner width
        let reset = "\x1b[0m";
        let dim = "\x1b[2m";

        // Pick border color from result
        let border = if self.is_critical() {
            "\x1b[93m" // bright yellow
        } else if self.is_catastrophe() {
            "\x1b[91m" // bright red
        } else if self.final_value > 0.0 {
            "\x1b[32m" // green
        } else {
            "\x1b[37m" // white
        };

        let result_icon = if self.is_critical() {
            "★ CRITICAL"
        } else if self.is_catastrophe() {
            "☠ CATASTROPHE"
        } else if self.is_success() {
            "✓ SUCCESS"
        } else {
            "✗ FAILURE"
        };

        // Header line: ┌─ ACTION (N engines) ──────┐
        let engine_count = self.chain.len();
        let header_label = format!(" {} ({} engines) ", action, engine_count);
        let dash_right = w.saturating_sub(header_label.len() + 2);
        let mut lines = Vec::new();
        lines.push(format!(
            "  {}┌─{}{}─┐{}",
            border, header_label, "─".repeat(dash_right), reset
        ));

        for step in &self.chain {
            // Bar: 14-char filled/empty
            let bar_fill = ((step.output.clamp(-1.0, 1.0) + 1.0) / 2.0 * 14.0) as usize;
            let bar = format!("[{}{}]", "█".repeat(bar_fill), "░".repeat(14 - bar_fill));
            let sign = if step.output >= 0.0 { "+" } else { "" };
            let val_str = format!("{}{:.4}", sign, step.output);

            // Engine line
            let engine_line = format!(
                "  {}│  {:<24} {}  {}  {}│{}",
                border, step.engine_name, val_str, bar,
                " ".repeat(w.saturating_sub(24 + val_str.len() + 16 + 6)),
                reset
            );
            lines.push(engine_line);

            // Flavor line — narrative explanation, dimmed
            let flavor = engine_combat_flavor(step.engine_name, step.output);
            lines.push(format!("  {}{}│   ↳ {:<width$}│{}", dim, border, flavor, reset, width = w.saturating_sub(6)));
        }

        // Separator + outcome
        lines.push(format!("  {}├{}┤{}", border, "─".repeat(w + 2), reset));

        let outcome_line = format!("{} — {}", result_icon, outcome);
        lines.push(format!(
            "  {}│  {:<width$}│{}",
            border, outcome_line, reset,
            width = w
        ));
        lines.push(format!("  {}└{}┘{}", border, "─".repeat(w + 2), reset));

        lines
    }

    /// Compact single-line summary of the chain for enemy rolls.
    /// "Lorenz→Logistic→Prime (+0.42) dealt 14 damage"
    pub fn enemy_trace_line(&self, enemy_name: &str, outcome: &str) -> String {
        let reset = "\x1b[0m";
        let dim = "\x1b[2m";
        let chain_str: String = self
            .chain
            .iter()
            .map(|s| engine_short_name(s.engine_name))
            .collect::<Vec<_>>()
            .join("→");
        let sign = if self.final_value >= 0.0 { "+" } else { "" };
        format!(
            "  {}⚡ {} [{}{}  {:.3}] {}{}",
            dim, enemy_name, chain_str, reset, self.final_value, outcome, reset
        )
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

/// Narrative flavor text for each engine based on its output value.
/// These are shown in the combat trace to explain what each algorithm did.
fn engine_combat_flavor(name: &str, output: f64) -> &'static str {
    match name {
        "Lorenz Attractor" => {
            if output > 0.6 { "butterfly cascades — tiny input became enormous output" }
            else if output < -0.6 { "butterfly reverses — attractor pulls toward negative basin" }
            else { "Lorenz orbit stabilizes near the saddle point" }
        }
        "Fourier Harmonic" => {
            if output > 0.6 { "harmonics align constructively — wave peaks reinforce" }
            else if output < -0.6 { "destructive interference — harmonics cancel to near zero" }
            else { "mixed harmonics — partial cancellation, partial amplification" }
        }
        "Prime Density Sieve" => {
            if output > 0.5 { "prime-dense window — actual density exceeds Li(x) prediction" }
            else if output < -0.5 { "prime desert — gap exceeds PNT prediction, density low" }
            else { "prime density near logarithmic integral — average region" }
        }
        "Riemann Zeta Partial" => {
            if output > 0.5 { "zeta oscillation peaks — far from a nontrivial zero" }
            else if output < -0.5 { "near a Riemann zero — the critical line destabilizes" }
            else { "zeta partial sum in mid-oscillation — moderate chaos" }
        }
        "Fibonacci Golden Spiral" => {
            if output > 0.5 { "golden angle aligns — φ spiral constructive phase" }
            else if output < -0.5 { "irrational rotation inverts — φ² gap produces minimum" }
            else { "golden ratio distributes evenly — no clustering, no void" }
        }
        "Mandelbrot Escape" => {
            if output > 0.5 { "boundary region — high escape velocity, outside the set" }
            else if output < -0.5 { "INSIDE THE SET — orbit never escapes, cursed outcome" }
            else { "seahorse valley boundary — fractal edge, maximum sensitivity" }
        }
        "Logistic Map" => {
            if output > 0.5 { "bifurcation cascade — period-doubling amplifies" }
            else if output < -0.5 { "chaotic orbit collapses to low attractor" }
            else { "logistic map at r≈3.9 — fully chaotic, unpredictable" }
        }
        "Euler's Totient" => {
            if output > 0.5 { "prime n — φ(n)/n near 1, maximum coprime ratio" }
            else if output < -0.5 { "highly composite n — many small primes, minimum ratio" }
            else { "mixed factorization — φ(n)/n near the 6/π² average" }
        }
        "Collatz Chain" => {
            if output > 0.5 { "short stopping time — rapid convergence to 1" }
            else if output < -0.5 { "long Collatz path — 27-type orbit, thousands of steps" }
            else { "moderate chain length — neither cursed nor blessed" }
        }
        "Modular Exp Hash" => {
            if output > 0.5 { "cryptographic avalanche locks high — a^b mod p peaks" }
            else if output < -0.5 { "hash avalanche collapses — discrete log pulls toward zero" }
            else { "modular exponentiation distributes uniformly" }
        }
        _ => "unknown engine produces chaos"
    }
}

/// Abbreviated engine names for the compact enemy trace line.
fn engine_short_name(name: &str) -> &'static str {
    match name {
        "Lorenz Attractor"        => "Lorenz",
        "Fourier Harmonic"        => "Fourier",
        "Prime Density Sieve"     => "Prime",
        "Riemann Zeta Partial"    => "Riemann",
        "Fibonacci Golden Spiral" => "Fibonacci",
        "Mandelbrot Escape"       => "Mandelbrot",
        "Logistic Map"            => "Logistic",
        "Euler's Totient"         => "Totient",
        "Collatz Chain"           => "Collatz",
        "Modular Exp Hash"        => "ModExp",
        _                         => "??",
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
        let engine_idx = (seed
            .wrapping_mul(2654435761)
            .wrapping_add(i as u64 * 1234567891)) as usize
            % ALL_ENGINES.len();
        let engine_seed = seed
            .wrapping_add(i as u64 * 9999991)
            .wrapping_mul(6364136223846793005);
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
        let engine_seed = seed
            .wrapping_mul(2654435761u64.wrapping_add(i as u64))
            .wrapping_add(i as u64 * 9876543211);
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
    ChaosRollResult {
        final_value: value,
        chain,
        game_value,
    }
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
            let engine_seed = seed
                .wrapping_mul(self.seed_modifier)
                .wrapping_add(i as u64 * 777777777);
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
        ChaosRollResult {
            final_value: value,
            chain,
            game_value,
        }
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
        assert!(
            positive_count >= 18,
            "Positive bias should mostly give positive results"
        );
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
