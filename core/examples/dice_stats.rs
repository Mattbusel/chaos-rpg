//! How fair is a chaos d20? Rolls one d20 per seed for N seeds and prints the
//! count of each face next to what a fair die would give, plus a chi-square
//! statistic (19 degrees of freedom; a fair die lands under 30.1 about 95% of
//! the time).
//!
//! Run with `cargo run -p chaos-rpg-core --example dice_stats -- 100000`.

use chaos_rpg_core::chaos_pipeline::chaos_roll_verbose;

fn main() {
    let n: u64 = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(100_000);
    let mut counts = [0u64; 21];
    for seed in 0..n {
        counts[chaos_roll_verbose(0.5, seed).as_d20() as usize] += 1;
    }
    let expected = n as f64 / 20.0;
    let mut chi2 = 0.0;
    println!("face  count   share   fair");
    for (face, &c) in counts.iter().enumerate().skip(1) {
        chi2 += (c as f64 - expected).powi(2) / expected;
        println!(
            "{face:>4}  {c:>6}  {:>5.2}%  5.00%",
            100.0 * c as f64 / n as f64
        );
    }
    println!("{n} rolls, chi-square {chi2:.1} (fair die: under 30.1 in 95% of samples)");
}
