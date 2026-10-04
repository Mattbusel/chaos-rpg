//! Game-logic hot paths, no window: a chaos roll, a destiny roll, a whole
//! fight resolved turn by turn, and floor generation.
//!
//! The same file builds against 2.2.1; the changelog numbers compare the two
//! releases on one machine. Run with `cargo bench -p chaos-rpg-core`.

use chaos_rpg_core::chaos_pipeline::{chaos_roll_verbose, destiny_roll};
use chaos_rpg_core::character::{Background, Character, CharacterClass, Difficulty};
use chaos_rpg_core::combat::{resolve_action, CombatAction, CombatOutcome, CombatState};
use chaos_rpg_core::enemy::generate_enemy;
use chaos_rpg_core::world::generate_floor;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn bench(c: &mut Criterion) {
    let mut seed = 0u64;
    c.bench_function("chaos_roll_verbose", |b| {
        b.iter(|| {
            seed = seed.wrapping_add(1);
            black_box(chaos_roll_verbose(0.5, seed).final_value)
        })
    });
    c.bench_function("destiny_roll", |b| {
        b.iter(|| {
            seed = seed.wrapping_add(1);
            black_box(destiny_roll(0.5, seed).final_value)
        })
    });
    c.bench_function("fight_to_the_end_floor_5", |b| {
        b.iter(|| {
            seed = seed.wrapping_add(1);
            let mut hero = Character::roll_new(
                "Bench".into(),
                CharacterClass::Mage,
                Background::Scholar,
                seed,
                Difficulty::Normal,
            );
            let mut enemy = generate_enemy(5, seed);
            let mut state = CombatState::new(seed);
            let mut turns = 0;
            loop {
                let (_, outcome) = resolve_action(&mut hero, &mut enemy, CombatAction::Attack, &mut state);
                turns += 1;
                if !matches!(outcome, CombatOutcome::Ongoing) || turns >= 200 {
                    break;
                }
            }
            black_box(turns)
        })
    });
    c.bench_function("generate_floor", |b| {
        b.iter(|| {
            seed = seed.wrapping_add(1);
            black_box(generate_floor(5, seed).rooms.len())
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
