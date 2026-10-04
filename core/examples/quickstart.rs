//! The README snippet: one chaos roll and the engine chain behind it.
//!
//! Run with `cargo run -p chaos-rpg-core --example quickstart`.

use chaos_rpg_core::chaos_pipeline::chaos_roll_verbose;

fn main() {
    // Same input and seed, same chain, every time.
    let roll = chaos_roll_verbose(0.5, 666);
    for step in &roll.chain {
        println!("{:<24} {:>6.3} -> {:>6.3}", step.engine_name, step.input, step.output);
    }
    println!("d20 = {}", roll.as_d20());
}
