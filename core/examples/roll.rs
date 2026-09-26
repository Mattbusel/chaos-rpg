//! Roll the chaos pipeline and print every step.
//!
//! ```text
//! cargo run -p chaos-rpg-core --example roll -- 666
//! ```

use chaos_rpg_core::chaos_pipeline::{chaos_roll_verbose, destiny_roll};

fn main() {
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(666);

    println!("Attack roll, seed {seed}");
    let roll = chaos_roll_verbose(0.5, seed);
    for step in &roll.chain {
        println!("  {:<24} {:>6.3} -> {:>6.3}", step.engine_name, step.input, step.output);
    }
    let verdict = if roll.is_critical() {
        "CRITICAL"
    } else if roll.is_catastrophe() {
        "CATASTROPHE"
    } else if roll.is_success() {
        "hit"
    } else {
        "miss"
    };
    println!("  final {:.3}  d20 {}  ({verdict})", roll.final_value, roll.as_d20());

    println!();
    println!("Destiny roll (all 10 engines, used for character creation), seed {seed}");
    let destiny = destiny_roll(0.5, seed);
    for step in &destiny.chain {
        println!("  {:<24} {:>6.3} -> {:>6.3}", step.engine_name, step.input, step.output);
    }
    println!("  final {:.3}  game value {}", destiny.final_value, destiny.game_value);
}
