#![allow(dead_code, unused_variables, unused_imports, unused_mut, unused_parens, named_arguments_used_positionally)]

//! Game logic for [CHAOS RPG](https://github.com/Mattbusel/chaos-rpg): every
//! dice roll in the game is a chain of real math engines, and this crate is
//! where that happens.
//!
//! ```
//! use chaos_rpg_core::chaos_pipeline::chaos_roll_verbose;
//!
//! // Same input and seed, same chain, every time.
//! let roll = chaos_roll_verbose(0.5, 666);
//! for step in &roll.chain {
//!     println!("{:<24} {:>6.3} -> {:>6.3}", step.engine_name, step.input, step.output);
//! }
//! assert!((-1.0..=1.0).contains(&roll.final_value));
//! let d20 = roll.as_d20();
//! assert!((1..=20).contains(&d20));
//! ```
//!
//! Run `cargo run -p chaos-rpg-core --example roll -- 666` in the repository
//! to print a full attack roll and destiny roll.
//!
//! Where to look:
//!
//! - [`chaos_pipeline`]: [`chaos_roll_verbose`](chaos_pipeline::chaos_roll_verbose),
//!   [`destiny_roll`](chaos_pipeline::destiny_roll) and
//!   [`ChaosRollResult`](chaos_pipeline::ChaosRollResult), the rolls behind everything.
//! - [`math_engines`]: the 10 engines (Lorenz attractor, Fourier harmonic, prime
//!   density sieve, Riemann zeta partial sum, Fibonacci golden spiral,
//!   Mandelbrot escape, logistic map, Euler's totient, Collatz chain, modular
//!   exponentiation hash).
//! - [`character`], [`combat`], [`enemy`], [`world`]: the game itself.
//!
//! Rendering is the job of the frontend crates:
//! [`chaos-rpg`](https://crates.io/crates/chaos-rpg) (terminal),
//! [`chaos-rpg-graphical`](https://crates.io/crates/chaos-rpg-graphical) and
//! [`chaos-rpg-proof`](https://crates.io/crates/chaos-rpg-proof).

pub mod audio_events;
pub mod audio_synth;
pub mod power_tier;
pub mod misery_system;
pub mod run_stats;
pub mod legacy_system;
pub mod achievement_system;
pub mod achievements;
pub mod run_history;
pub mod chaos_config;
pub mod daily_leaderboard;
pub mod io_util;
pub mod atlas;
pub mod body;
pub mod bosses;
pub mod chaos_pipeline;
pub mod character;
pub mod combat;
pub mod crafting;
pub mod dialogue;
pub mod dungeon;
pub mod enemy;
pub mod enemy_ai;
pub mod faction_system;
pub mod factions;
pub mod items;
pub mod loot_system;
pub mod magic;
pub mod math_engines;
pub mod nemesis;
pub mod npcs;
pub mod passive_tree;
pub mod quest;
pub mod recipes;
pub mod save_system;
pub mod scoreboard;
pub mod skill_checks;
pub mod skill_tree;
pub mod spells;
pub mod status_effects;
pub mod world;
pub mod world_map;
pub mod weather_system;
pub mod economy;
pub mod skill_tree_v2;
pub mod inventory_system;
pub mod combat_simulator;
pub mod npc_generator;
pub mod party_system;
pub mod class_system;
pub mod dungeon_generator_v2;
pub mod dialogue_system;
pub mod map_generator;
pub mod merchant_system;
pub mod crafting_system_v2;
pub mod magic_system;
pub mod relationship_system;
pub mod trap_system;
pub mod time_system;
pub mod crafting_system;
pub mod bestiary;
pub mod quest_system;
pub mod combat_system;
pub mod character_creator;
pub mod dungeon_generator;
pub mod loot_table;
pub mod lore;
pub mod character_lore;
pub mod player_bestiary;
pub mod codex_progress;
