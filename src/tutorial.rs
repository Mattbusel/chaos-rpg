//! In-game tutorial and help system.
//!
//! Accessible via [H] at any time. Explains the chaos engine,
//! stats, combat, items, and scoring in player-friendly terms.

/// Multi-page tutorial content.
pub const PAGES: &[(&str, &str)] = &[
    (
        "Overview",
        "Every outcome in CHAOS RPG is determined by chaining 4-10 real \
mathematical algorithms. You will see them. You cannot control them. \
The same seed always produces the same game — share seeds with friends \
to compare fates.\n\
\n\
Set CHAOS_SEED=<number> to use a specific seed.",
    ),
    (
        "The 10 Algorithms",
        "1. Lorenz Attractor   — butterfly effect. Tiny input → huge output change.\n\
2. Fourier Harmonic   — harmonic interference. Can spike or zero out.\n\
3. Prime Density Sieve — primes vs. PNT prediction. Irregular density.\n\
4. Riemann Zeta Partial — critical-line oscillations. Near-zero = chaos.\n\
5. Fibonacci Spiral    — golden ratio φ. Most irrational number.\n\
6. Mandelbrot Escape   — inside the set = negative (cursed). Boundary = chaos.\n\
7. Logistic Map        — x_{n+1}=r·x·(1-x) at r≈3.9. Fully chaotic.\n\
8. Euler's Totient     — φ(n)/n deviation from 6/π². Wildly irregular.\n\
9. Collatz Chain       — 3n+1 stopping time. Some paths orbit for thousands of steps.\n\
10. Modular Exp Hash   — a^b mod prime. Cryptographic avalanche effect.",
    ),
    (
        "Stats",
        "All 7 stats are UNBOUNDED. They can be negative thousands or positive thousands.\n\
\n\
VITALITY  — Max HP and resistance\n\
FORCE     — Physical attack power\n\
MANA      — Spell power, magic resist\n\
CUNNING   — Crit chance, flee, traps\n\
PRECISION — Accuracy, ranged bonus\n\
ENTROPY   — Chaos amplifier on all rolls\n\
LUCK      — General fortune modifier\n\
\n\
A negative FORCE means picking up a sword makes it heavier for the universe.",
    ),
    (
        "Combat",
        "[A] Attack      — FORCE-based. Berserker rage bonus at low HP.\n\
[H] Heavy Attack — More damage, catastrophic miss risk.\n\
[D] Defend       — Reduces incoming damage this round.\n\
[T] Taunt        — Stun on crit. ENRAGE enemy on catastrophe.\n\
[F] Flee         — LUCK+CUNNING vs chaos. Failure = free enemy attack.\n\
[S] Spell        — Mage/scroll. Mana cost CAN be negative (gives mana).\n\
\n\
OVERFLOW: If damage exceeds 100,000 it wraps: 100,001 → 1 damage.\n\
This means killing blow can accidentally save the enemy.",
    ),
    (
        "Items & Spells",
        "Every item is fully procedural: material + base type + adjective.\n\
Damage/defense is UNBOUNDED — a sword with -500 damage heals enemies.\n\
Special effects are real and apply in combat.\n\
Rarity is computed AFTER generation from total stat magnitude.\n\
\n\
Spells have: damage (can be negative), mana cost (can be negative),\n\
AoE radius (negative = hits your party), scaling stat (not always INT),\n\
scaling factor (can be negative — dumber = stronger spell).",
    ),
    (
        "Scoring",
        "Score = stat_total + floor × 100 + level × 50 + kills × 10 + gold\n\
\n\
Infinite Mode: endless floors, score tracked, top 10 saved to disk.\n\
Story Mode: 10 floors with narrative events and a final boss.\n\
\n\
The final boss's power level is a Destiny Roll (all 10 engines).\n\
It might be weaker than the first rat. It might be a god.\n\
The math does not care about your expectations.",
    ),
];

/// Display all tutorial pages (called from ui::show_help)
pub fn full_tutorial() -> Vec<String> {
    let mut lines = Vec::new();
    for (title, content) in PAGES {
        lines.push(format!("\x1b[1m\x1b[36m=== {} ===\x1b[0m", title));
        for line in content.lines() {
            lines.push(format!("  {}", line));
        }
        lines.push(String::new());
    }
    lines
}
