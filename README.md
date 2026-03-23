# ⚡ CHAOS RPG ⚡

> *Where Math Goes To Die*

A terminal-based ASCII roguelike where **every single outcome** — character creation, combat, items, spells, skill checks — is determined by chaining 4–10 real mathematical algorithms together. The output of one feeds into the next. The result is a game where you can roll a character that is literally God incarnate, or the reanimated corpse of the weakest being to ever exist. Both outcomes are mathematically valid.

[![CI](https://github.com/Mattbusel/chaos-rpg/actions/workflows/ci.yml/badge.svg)](https://github.com/Mattbusel/chaos-rpg/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## What is this?

CHAOS RPG is a mathematically cursed roguelike. There are no lookup tables, no weighted random, no "balanced" encounter design. Every number that appears in the game passes through a pipeline of real mathematical chaos systems:

- Your sword's damage? **Lorenz Attractor → Riemann Zeta → Collatz Chain**
- Your character's power level? **All 10 algorithms in sequence**
- Whether you hit the enemy? **4–8 engines chained, results shown live**

You will watch the math happen. You cannot stop it.

---

## How to Build

```bash
git clone https://github.com/Mattbusel/chaos-rpg
cd chaos-rpg
cargo build --release
```

Binary lands at `target/release/chaos-rpg` (Linux/Mac) or `target/release/chaos-rpg.exe` (Windows).

**Requirements:** Rust stable (1.70+), any terminal with ANSI color support (80×24 minimum).

### Run directly

```bash
cargo run --release
```

### Platform notes

| Platform | Status | Notes |
|----------|--------|-------|
| Linux | ✅ Primary | Full color, all features |
| macOS | ✅ Supported | Same as Linux |
| Windows | ✅ Supported | Use Windows Terminal for best results |

---

## The 10 Sacred Algorithms

Every roll chains 4–10 of these. The output of each feeds into the next. The pipeline is shown to you every time.

| # | Algorithm | Why it's chaotic |
|---|-----------|-----------------|
| 1 | **Lorenz Attractor** | The butterfly effect. σ=10, ρ=28, β=8/3. Tiny input change → massive output change. |
| 2 | **Fourier Harmonic Series** | Sums N sinusoids with chaotic phase shifts. Constructive interference spikes values; destructive interference zeros them. |
| 3 | **Prime Density Sieve** | Counts primes in a window, compares to Prime Number Theorem prediction. Deviations drive the output. |
| 4 | **Riemann Zeta Partial Sum** | Partial sums of ζ(s) on the critical line. The imaginary part oscillates wildly. |
| 5 | **Fibonacci Golden Spiral** | Maps through φ=(1+√5)/2. The golden ratio is irrational in the deepest sense — it never repeats. |
| 6 | **Mandelbrot Escape Velocity** | Escape iteration count near the Mandelbrot boundary. Points inside the set return negative values (cursed). |
| 7 | **Logistic Map Bifurcation** | x_{n+1} = r·x·(1-x) at r≈3.9. Period doubling to infinity. Fully chaotic regime. |
| 8 | **Euler's Totient Function** | φ(n)/n is wildly irregular. Primes give near-1 ratios; highly composite numbers give low ratios. |
| 9 | **Collatz Conjecture Chain** | 3n+1. Some numbers orbit to insane altitudes before collapsing. The altitude ratio creates huge swings. |
| 10 | **Modular Exponentiation Hash** | aᵇ mod p. The avalanche effect of modular arithmetic. Smoothly varying inputs → pseudo-random outputs. |

---

## Game Modes

### Story Mode
20–30 rooms with a procedurally generated quest. Final boss rolls its power level via the **Destiny Roll** (all 10 engines). It might be weaker than the floor-1 rat. It might be a god. You will find out.

### Infinite Mode
Endless floors. Enemy engines increase with floor depth. Score tracked. Top 10 highscores saved to `~/.chaos_rpg_scores.json`.

### Quick Roll
Just character creation, over and over. For people who want to see what the chaos produces without the gameplay. Press any key to roll another.

---

## Character Classes

| Class | Role | Primary Stats |
|-------|------|--------------|
| **Mage** | Bends chaos through mathematical will | MANA, ENTROPY |
| **Berserker** | Channels pain into exponential power | VITALITY, FORCE |
| **Ranger** | Reads prime patterns in nature | PRECISION, LUCK |
| **Thief** | Exploits logistic map phase transitions | CUNNING, LUCK |

Stats are **completely unbounded**. Your FORCE can be -4,000. Your MANA can be 99,999. Both are mathematically reachable from character creation.

---

## Power Level Tiers

| Range | Title |
|-------|-------|
| < -50,000 | **THE ANTI-GOD** — "You are a negative singularity." |
| -50,000 to -5,000 | **COSMICALLY DOOMED** — "The universe wrote your obituary before you were born." |
| -5,000 to -1,000 | **LEGENDARY FAILURE** — "Bards sing of you. The songs are warnings." |
| -1,000 to -100 | **CURSED** — "Mirrors crack. Cats hiss. Your own shadow tries to leave." |
| -100 to 100 | **MORTAL** — "Unremarkable. The chaos has not yet noticed you." |
| 100 to 1,000 | **HEROIC** — "You might survive. Emphasis on 'might.'" |
| 1,000 to 5,000 | **LEGENDARY** — "The algorithm smiled upon you. (It was probably a glitch.)" |
| 5,000 to 10,000 | **MYTHICAL** — "Dragons ask YOU for autographs." |
| 10,000 to 50,000 | **DEMIGOD** — "Reality bends. Physics takes suggestions." |
| 50,000 to 99,999 | **GOD** — "You are the game. The game is your dream." |
| > 99,999 | **BEYOND** — "The math engines created something they don't understand." |

---

## Controls

| Key | Action |
|-----|--------|
| `A` | Attack |
| `S` | Cast spell |
| `I` | Use item |
| `F` | Flee |
| `T` | Talk |
| `C` | Character sheet |
| `H` | Help / tutorial |
| `Q` / `Esc` | Quit |

---

## Example Output

```
╔══════════════════════════════════════════════════╗
║                 FLOOR 7 — BEE ZONE               ║
╠══════════════════════════════════════════════════╣
║         MATHEMATICAL VOID DRAGON (Jr.)           ║
║              HP: ████████░░ 2,847/4,120          ║
╠══════════════════════════════════════════════════╣
║  ⚔ YOUR TURN                                     ║
║  HP: ██████░░░░░░░ 340/892   MP: ████████ 120   ║
║  [A]ttack  [S]pell  [I]tem  [F]lee  [T]alk      ║
╚══════════════════════════════════════════════════╝

  CHAOS ENGINE CHAIN TRACE:
    1. Lorenz Attractor      →  +0.7341
    2. Prime Density Sieve   →  -0.2819
    3. Collatz Chain         →  +0.9102
    4. Modular Exp Hash      →  -0.0447
    5. Fibonacci Spiral      →  +0.6634

  Result: 66/100   ★★★ HIT! 847 DAMAGE ★★★
```

---

## Seeded Runs

Set `CHAOS_SEED=<number>` to get a deterministic run. Same seed = same game, every time. Share cursed seeds with friends.

```bash
CHAOS_SEED=666 cargo run --release
```

---

## System Requirements

- Terminal: 80+ columns, 24+ rows
- ANSI color support (any modern terminal)
- Rust 1.70+ for building

---

## See Also

- [MATH_MANIFESTO.md](MATH_MANIFESTO.md) — Deep dive into every algorithm for nerds and masochists

---

## License

MIT — see [LICENSE](LICENSE) for details.
