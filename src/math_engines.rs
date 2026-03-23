//! The 10 Sacred Mathematical Algorithms of CHAOS RPG.
//!
//! Every function takes `(input: f64, seed: u64) -> f64`.
//! Output is in the range [-1.0, 1.0] unless explicitly noted.
//! Chain them together. Watch reality unravel.

use std::f64::consts::PI;

// ─── 1. LORENZ ATTRACTOR ─────────────────────────────────────────────────────
/// Simulates a mini Lorenz step. The butterfly effect in one function.
/// σ=10, ρ=28, β=8/3 — the classic strange attractor parameters.
pub fn lorenz_attractor(input: f64, seed: u64) -> f64 {
    let sigma = 10.0_f64;
    let rho = 28.0_f64;
    let beta = 8.0_f64 / 3.0_f64;
    let dt = 0.01_f64;

    // Seed perturbs initial y, z
    let seed_f = (seed as f64).sin() * 0.5 + 0.5;
    let x = input * PI;
    let y = seed_f * 20.0 - 10.0;
    let z = seed_f * 30.0;

    let dx = sigma * (y - x);
    let dy = x * (rho - z) - y;
    let dz = x * y - beta * z;

    let nx = x + dx * dt;
    let ny = y + dy * dt;
    let nz_raw = z + dz * dt;
    let _nz = nz_raw; // suppress unused warning

    // Normalize x output to [-1, 1]
    let raw = nx + ny * 0.01;
    (raw.sin() * 0.5 + (raw * 0.1).tanh() * 0.5).clamp(-1.0, 1.0)
}

// ─── 2. FOURIER HARMONIC ─────────────────────────────────────────────────────
/// Sums N harmonic sinusoids with seed-derived phase offsets.
/// The more harmonics, the more complex the interference pattern.
pub fn fourier_harmonic(input: f64, seed: u64) -> f64 {
    let harmonics = 8u64;
    let mut sum = 0.0_f64;
    let mut weight_sum = 0.0_f64;

    for k in 1..=harmonics {
        let phase =
            ((seed.wrapping_mul(k).wrapping_add(k * 31337)) as f64 * 1e-10).fract() * 2.0 * PI;
        let freq = k as f64;
        let amplitude = 1.0 / freq; // 1/f spectrum
        sum += amplitude * (freq * input * PI + phase).sin();
        weight_sum += amplitude;
    }

    (sum / weight_sum).clamp(-1.0, 1.0)
}

// ─── 3. PRIME DENSITY SIEVE ──────────────────────────────────────────────────
/// Counts primes near input*seed region using a mini sieve.
/// Output encodes prime density as chaos signal.
pub fn prime_density_sieve(input: f64, seed: u64) -> f64 {
    let base = ((input.abs() * 1000.0) as u64).wrapping_add(seed % 10_000) % 50_000 + 2;
    let window = 64usize;
    let start = base as usize;

    // Mini sieve of Eratosthenes over the window
    let mut sieve = vec![true; window];
    sieve[0] = false; // base itself: skip 0-index
    let limit = ((start + window) as f64).sqrt() as usize + 1;
    for p in 2..=limit {
        if p < start {
            let first_mult = if start.is_multiple_of(p) {
                start
            } else {
                start + p - (start % p)
            };
            let offset = first_mult - start;
            let mut j = offset;
            while j < window {
                sieve[j] = false;
                j += p;
            }
        } else if p >= start && p - start < window {
            let idx = p - start;
            if sieve[idx] {
                let mut j = idx + p;
                while j < window {
                    sieve[j] = false;
                    j += p;
                }
            }
        }
    }

    let prime_count = sieve.iter().filter(|&&b| b).count() as f64;
    let density = prime_count / window as f64;

    // Li(x) approximation for expected density
    let x = (start + window / 2) as f64;
    let expected = if x > 1.0 { 1.0 / x.ln() } else { 0.5 };
    let deviation = (density - expected) / expected.max(0.001);

    deviation.tanh().clamp(-1.0, 1.0)
}

// ─── 4. RIEMANN ZETA PARTIAL ─────────────────────────────────────────────────
/// Computes partial sum of ζ(s) on the critical line s = 0.5 + it.
/// The imaginary part of the zeta function evaluated at t = |input|*seed.
pub fn riemann_zeta_partial(input: f64, seed: u64) -> f64 {
    let t = input.abs() * ((seed % 1000) as f64 + 1.0) * 0.1;
    let s_real = 0.5_f64;
    let terms = 50u64;

    let mut real_sum = 0.0_f64;
    let mut imag_sum = 0.0_f64;

    for n in 1..=terms {
        let n_f = n as f64;
        // n^(-s) = n^(-0.5) * e^(-it*ln(n))
        let magnitude = n_f.powf(-s_real);
        let phase = -t * n_f.ln();
        real_sum += magnitude * phase.cos();
        imag_sum += magnitude * phase.sin();
    }

    // Return normalized imaginary part (zeros of zeta are mysterious)
    let magnitude = (real_sum * real_sum + imag_sum * imag_sum).sqrt();
    if magnitude > 0.0 {
        (imag_sum / magnitude.max(0.001)).tanh()
    } else {
        0.0
    }
}

// ─── 5. FIBONACCI GOLDEN SPIRAL ──────────────────────────────────────────────
/// Maps input through the golden ratio spiral.
/// Uses φ = (1+√5)/2 as the fundamental constant of growth.
pub fn fibonacci_golden_spiral(input: f64, seed: u64) -> f64 {
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0; // ≈ 1.6180339887

    // Find the Nth Fibonacci number scaled by seed
    let n = ((seed % 30) + 5) as u32;
    let fib_n = {
        // Binet's formula: F(n) = (φ^n - ψ^n) / √5
        let psi = (1.0 - 5.0_f64.sqrt()) / 2.0;
        let fib = (phi.powi(n as i32) - psi.powi(n as i32)) / 5.0_f64.sqrt();
        fib.round()
    };

    // Modulate input by golden angle
    let golden_angle = 2.0 * PI * (1.0 - 1.0 / phi);
    let spiral_val = (input * phi * golden_angle + fib_n * 0.001).sin();

    // Add golden ratio harmonics
    let harmonic = ((input * phi).fract() * 2.0 - 1.0) * 0.3;

    (spiral_val * 0.7 + harmonic).clamp(-1.0, 1.0)
}

// ─── 6. MANDELBROT ESCAPE ────────────────────────────────────────────────────
/// Measures escape velocity from the Mandelbrot set at a seed-derived point.
/// Higher iteration count = closer to the boundary = more chaos.
pub fn mandelbrot_escape(input: f64, seed: u64) -> f64 {
    let max_iter = 64u32;

    // Map seed+input to a point near the Mandelbrot boundary
    let seed_angle = (seed as f64 * 2.3999632297286573e-9 * PI * 2.0).fract() * 2.0 * PI;
    let cr = -0.7269 + input.cos() * 0.1 + seed_angle.cos() * 0.05;
    let ci = 0.1889 + input.sin() * 0.1 + seed_angle.sin() * 0.05;

    let mut zr = 0.0_f64;
    let mut zi = 0.0_f64;
    let mut iter = 0u32;

    while iter < max_iter && zr * zr + zi * zi < 4.0 {
        let zr_new = zr * zr - zi * zi + cr;
        zi = 2.0 * zr * zi + ci;
        zr = zr_new;
        iter += 1;
    }

    // Smooth coloring formula
    let smooth_iter = if iter < max_iter {
        iter as f64 - (zr * zr + zi * zi).ln().ln() / 2.0_f64.ln()
    } else {
        max_iter as f64
    };

    let normalized = smooth_iter / max_iter as f64;
    (normalized * 2.0 - 1.0).clamp(-1.0, 1.0)
}

// ─── 7. LOGISTIC MAP ─────────────────────────────────────────────────────────
/// Iterates the logistic map x_{n+1} = r*x_n*(1-x_n).
/// At r≈3.9 this system is fully chaotic — period doubling to infinity.
pub fn logistic_map(input: f64, seed: u64) -> f64 {
    // r in chaotic regime [3.57, 4.0], shifted by seed
    let r = 3.57 + ((seed % 1000) as f64 / 1000.0) * 0.43;
    let mut x = (input.abs() + 0.1).fract();
    if x <= 0.0 || x >= 1.0 {
        x = 0.3;
    }

    // Burn-in 50 iterations (transient removal)
    for _ in 0..50 {
        x = r * x * (1.0 - x);
    }

    // Collect 10 more and average for stability
    let mut avg = 0.0;
    for _ in 0..10 {
        x = r * x * (1.0 - x);
        avg += x;
    }
    avg /= 10.0;

    (avg * 2.0 - 1.0).clamp(-1.0, 1.0)
}

// ─── 8. EULER'S TOTIENT ──────────────────────────────────────────────────────
/// Computes φ(n) / n, the ratio of integers coprime to n.
/// Uses seed-derived n. The more prime-like n is, the higher the ratio.
pub fn euler_totient(input: f64, seed: u64) -> f64 {
    let n = ((input.abs() * 997.0) as u64)
        .wrapping_add(seed % 9973)
        .wrapping_add(2);
    let n = (n % 9999) + 2; // keep in [2, 10000]

    let mut result = n;
    let mut m = n;
    let mut p = 2u64;
    while p * p <= m {
        if m.is_multiple_of(p) {
            while m.is_multiple_of(p) {
                m /= p;
            }
            result -= result / p;
        }
        p += 1;
    }
    if m > 1 {
        result -= result / m;
    }

    let ratio = result as f64 / n as f64;
    // Expected value of φ(n)/n ~ 6/π² ≈ 0.608
    let baseline = 6.0 / (PI * PI);
    let deviation = (ratio - baseline) / baseline;

    deviation.tanh().clamp(-1.0, 1.0)
}

// ─── 9. COLLATZ CHAIN ────────────────────────────────────────────────────────
/// Measures the Collatz stopping time for a seed-derived number.
/// The 3n+1 conjecture: simple rule, insane complexity.
pub fn collatz_chain(input: f64, seed: u64) -> f64 {
    let start = ((input.abs() * 9973.0) as u64)
        .wrapping_add(seed)
        .wrapping_add(3);
    let mut n = (start % 100_000) + 3;
    let mut steps = 0u64;
    let max_steps = 2000u64;

    while n != 1 && steps < max_steps {
        if n.is_multiple_of(2) {
            n /= 2;
        } else {
            n = n.saturating_mul(3).saturating_add(1);
        }
        steps += 1;
    }

    // Normalize against expected stopping time (empirically ~100 for numbers <100k)
    let normalized = (steps as f64 / 500.0 - 1.0).tanh();
    normalized.clamp(-1.0, 1.0)
}

// ─── 10. MODULAR EXPONENTIATION HASH ─────────────────────────────────────────
/// Computes a^b mod m where all three are derived from input+seed.
/// Exploits the avalanche effect of modular arithmetic for pseudo-randomness.
pub fn modular_exp_hash(input: f64, seed: u64) -> f64 {
    // Derive a, b, m from input and seed
    let base_val = ((input * 1e9) as i64).unsigned_abs().wrapping_add(seed);
    let a = (base_val % 997).max(2); // prime base
    let b = ((seed.wrapping_mul(2654435761)) % 65537) + 1; // prime exponent region
    let m = 1_000_003u64; // large prime modulus

    // Fast modular exponentiation
    let mut result = 1u64;
    let mut base = a % m;
    let mut exp = b;

    while exp > 0 {
        if exp % 2 == 1 {
            result = (result * base) % m;
        }
        exp /= 2;
        base = (base * base) % m;
    }

    // Mix with input's fractional part for smoothness
    let hash_normalized = result as f64 / m as f64; // [0, 1)
    let frac_mix = input.fract().abs();
    let mixed = hash_normalized * 0.9 + frac_mix * 0.1;

    (mixed * 2.0 - 1.0).clamp(-1.0, 1.0)
}

// ─── ENGINE REGISTRY ─────────────────────────────────────────────────────────
pub type MathEngine = fn(f64, u64) -> f64;

pub const ALL_ENGINES: [MathEngine; 10] = [
    lorenz_attractor,
    fourier_harmonic,
    prime_density_sieve,
    riemann_zeta_partial,
    fibonacci_golden_spiral,
    mandelbrot_escape,
    logistic_map,
    euler_totient,
    collatz_chain,
    modular_exp_hash,
];

pub const ENGINE_NAMES: [&str; 10] = [
    "Lorenz Attractor",
    "Fourier Harmonic",
    "Prime Density Sieve",
    "Riemann Zeta Partial",
    "Fibonacci Golden Spiral",
    "Mandelbrot Escape",
    "Logistic Map",
    "Euler's Totient",
    "Collatz Chain",
    "Modular Exp Hash",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_engines_in_range() {
        for (i, engine) in ALL_ENGINES.iter().enumerate() {
            for seed in [0u64, 1, 42, 999, 1234567890] {
                for input in [-1.0f64, -0.5, 0.0, 0.3, 0.7, 1.0] {
                    let result = engine(input, seed);
                    assert!(
                        result >= -1.0 && result <= 1.0,
                        "Engine {} ({}) out of range: input={}, seed={}, result={}",
                        i,
                        ENGINE_NAMES[i],
                        input,
                        seed,
                        result
                    );
                }
            }
        }
    }

    #[test]
    fn engines_are_deterministic() {
        for engine in ALL_ENGINES.iter() {
            let r1 = engine(0.42, 12345);
            let r2 = engine(0.42, 12345);
            assert_eq!(r1, r2);
        }
    }

    #[test]
    fn different_seeds_give_different_results() {
        let mut all_same = true;
        for engine in ALL_ENGINES.iter() {
            let r1 = engine(0.5, 0);
            let r2 = engine(0.5, 999999);
            if (r1 - r2).abs() > 1e-10 {
                all_same = false;
                break;
            }
        }
        assert!(
            !all_same,
            "All engines returned identical results for different seeds"
        );
    }
}
