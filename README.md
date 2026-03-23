# CHAOS RPG

> *Where Math Goes To Die*

A terminal ASCII roguelike where **every single outcome** -- character creation, combat, items, spells, skill checks -- is determined by chaining 4-10 real mathematical algorithms together. The output of one feeds into the next. The result is a game where you can roll a character that is literally God incarnate, or the reanimated corpse of the weakest being to ever exist. Both outcomes are mathematically valid.

[![CI](https://github.com/Mattbusel/chaos-rpg/actions/workflows/ci.yml/badge.svg)](https://github.com/Mattbusel/chaos-rpg/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## Quick Start

```bash
git clone https://github.com/Mattbusel/chaos-rpg
cd chaos-rpg
cargo run --release
```

A pre-built Windows binary is available at `dist/chaos-rpg.exe` if you do not have Rust installed.

**Requirements:** Rust stable 1.70+, any terminal with ANSI color support, 80x24 minimum size.

| Platform | Status | Notes |
|----------|--------|-------|
| Windows | Supported | Use Windows Terminal for best results |
| Linux | Primary | Full color, all features |
| macOS | Supported | Same as Linux |

---

## Seeded Runs

```bash
CHAOS_SEED=666 cargo run --release
```

Same seed produces the same character stats, enemies, and loot every time. Share cursed seeds.

---

## The 10 Sacred Algorithms

Every roll chains 4-10 of these in sequence. The full chain is printed after every action.

| # | Algorithm | Why it's chaotic |
|---|-----------|-----------------|
| 1 | **Lorenz Attractor** | The butterfly effect. sigma=10, rho=28, beta=8/3. Tiny input change produces massive output change. |
| 2 | **Fourier Harmonic Series** | Sums N sinusoids with chaotic phase shifts. Constructive interference spikes values; destructive interference zeros them. |
| 3 | **Prime Density Sieve** | Counts primes in a window, compares to Prime Number Theorem prediction. Deviations drive the output. |
| 4 | **Riemann Zeta Partial Sum** | Partial sums of zeta(s) on the critical line. The imaginary part oscillates wildly. |
| 5 | **Fibonacci Golden Spiral** | Maps through phi=(1+sqrt(5))/2. The golden ratio is irrational in the deepest sense -- it never repeats. |
| 6 | **Mandelbrot Escape Velocity** | Escape iteration count near the Mandelbrot boundary. Points inside the set return negative values (cursed). |
| 7 | **Logistic Map Bifurcation** | x_{n+1} = r*x*(1-x) at r~3.9. Period doubling to infinity. Fully chaotic regime. |
| 8 | **Euler's Totient Function** | phi(n)/n is wildly irregular. Primes give near-1 ratios; highly composite numbers give low ratios. |
| 9 | **Collatz Conjecture Chain** | 3n+1. Some numbers orbit to insane altitudes before collapsing. The altitude ratio creates huge swings. |
| 10 | **Modular Exponentiation Hash** | a^b mod p. The avalanche effect of modular arithmetic. Smoothly varying inputs produce pseudo-random outputs. |

---

## Inline Engine Trace

After every combat action, the full algorithm chain is printed automatically:

```
  +--------------------------------------------------------+
  | ATTACK (6 engines)                                     |
  |  Lorenz Attractor    +0.847  [##############      ]   |
  |    -> The butterfly snaps its wings. Chaos amplified.  |
  |  Prime Density       -0.213  [########            ]   |
  |    -> The primes thin out here. Fortune wavers.        |
  |  Collatz Chain       +0.931  [################    ]   |
  |    -> 27 took 111 steps to fall. So does this hit.     |
  |  Riemann Zeta        +0.612  [############        ]   |
  |    -> A zero of the zeta function aligns. Power flows. |
  |  Fibonacci Spiral    -0.044  [#####               ]   |
  |    -> The golden ratio finds no resonance here.        |
  |  Modular Hash        +0.778  [##############      ]   |
  |    -> The avalanche completes. Output locked.          |
  +-- CRITICAL HIT: 1,847 damage -------------------------+

  Fractal Imp: Euler -> Lorenz -> Collatz [missed, -12 HP]
```

---

## Character Classes

8 classes, each with a unique passive ability. Stats are fully unbounded -- FORCE can be -4,000 or 99,999. Both are mathematically reachable.

| Class | Role | Passive |
|-------|------|---------|
| **Mage** | Bends chaos through pure mathematical will | Arcane Overflow: critical spells deal ENTROPY/10 bonus damage |
| **Berserker** | Channels pain into exponential power | Blood Frenzy: below 30% HP, +40% damage and attack twice on crit |
| **Ranger** | Reads prime number patterns in nature | Prime Sight: PRECISION/20 bonus accuracy on every attack |
| **Thief** | Exploits logistic map phase transitions | Chaos Dodge: CUNNING/200 + 10% chance to dodge incoming hits |
| **Necromancer** | Death is not an end -- it's a variable | Death Drain: on kill, absorb 8% of enemy max HP as your own |
| **Alchemist** | The logistic map of chemical chaos | Transmutation: items and potions grant 50% more effect |
| **Paladin** | A divine constant in a chaotic universe | Divine Regen: regenerate (3 + VIT/20) HP at start of each round |
| **VoidWalker** | Exists between the Mandelbrot boundary and everywhere else | Phase Shift: 15% + LCK/400 chance to phase-dodge any attack |

---

## Backgrounds

Pick a background at character creation for a flat stat bonus (or penalty).

| Background | Effect |
|-----------|--------|
| Scholar | +15 MANA, +10 ENTROPY |
| Wanderer | +15 LUCK, +10 PRECISION |
| Gladiator | +15 FORCE, +10 VITALITY |
| Outcast | +15 CUNNING, +10 ENTROPY |
| Merchant | +15 CUNNING, +10 LUCK |
| Cultist | +20 MANA, +20 ENTROPY, -10 VITALITY |
| Exile | +20 CUNNING, +10 ENTROPY, -10 MANA |
| Oracle | +20 LUCK, +10 MANA, -15 FORCE |

---

## Boons

At the start of each run, three boons are randomly offered. Pick one. They are permanent.

| Boon | Effect |
|------|--------|
| Blood Pact | +50 max HP. Take 2 HP damage entering each room. |
| Chaos Blessing | +10 Luck. Chaos rolls biased in your favor. |
| Gold Vein | Start with 200 gold. |
| Scholar's Gift | Start with 3 extra chaos-generated spells. |
| Warrior's Blessing | +20 Force, +15 Vitality. |
| Lucky Birth | +30 Luck. |
| Entropic Soul | 2x Entropy and Mana, half Vitality. |
| Crystal Skin | Start with an 80 HP shield. |
| Math Savant | All spell damage x1.75. |
| Void Touched | All stats x1.5. |
| Prime Blood | Each kill: +1 to your highest stat. |
| Shadow Start | Start at 50% HP. All XP x3. |

---

## Difficulty

| Difficulty | Enemy Damage | Gold | XP | Score Multiplier |
|-----------|-------------|------|----|-----------------|
| Easy | 70% | 130% | 80% | x1 |
| Normal | 100% | 100% | 100% | x2 |
| Brutal | 140% | 75% | 120% | x4 |
| Chaos | 200% | 50% | 200% | x10 |

---

## Body System

Every character has 12 independent body parts, each with its own HP pool.

```
  HEAD         [  82/  82] ok
  TORSO        [ 220/ 220] ok
  LEFT ARM     [  61/  61] ok
  RIGHT ARM    [  61/  61] [FRACTURED] Force -8, Precision -5
  LEFT LEG     [  74/  74] ok
  RIGHT LEG    [  74/  74] [SHATTERED] Speed penalty
  LEFT HAND    [  45/  45] ok
  RIGHT HAND   [  45/  45] ok
  LEFT FOOT    [  38/  38] ok
  RIGHT FOOT   [  38/  38] ok
  LEFT EYE     [  18/  18] ok
  RIGHT EYE    [   0/  18] [SEVERED] Precision -20
```

- Attacks hit a weighted random location. Head gets fewer hits but crits route there more often.
- Injuries have five severities: **Bruised -> Fractured -> Shattered -> Severed -> Mathematically Absent**
- Injuries apply stat penalties (arm injuries reduce Force, eye injuries reduce Precision, leg injuries reduce flee chance)
- Body parts can drop below 0 HP into negative territory -- **cursed** parts drain HP each turn
- Losing your head is instant death regardless of total HP
- Armor equips to specific body slots and reduces damage to that part only

---

## Passive Skill Tree

82 nodes arranged around 8 class starting positions. Path of Exile style.

- **Stat nodes** -- bonus is chaos-rolled at allocation time. You don't know if you get +3 Force or +47 Force until you spend the point.
- **Engine nodes** -- permanently modify how a specific chaos engine behaves for your character (ForcePositive, Volatile, BoundaryMagnet, DoubleOutput, etc.)
- **Synergy clusters** -- 3 weak nodes that together activate a powerful bonus
- **Keystones** -- 4 build-defining choices at the far edge of the tree:
  - **Chaos Immunity** -- immune to all negative chaos effects, lose all positive ones too
  - **Glass Cannon Infinite** -- unlimited damage ceiling, 1 HP cap
  - **Mathematical Certainty** -- rolls always return 0.5, no crits, no disasters
  - **Entropy Inversion** -- all negative rolls become positive; all positive rolls become negative

Use `P` in game to open the ASCII tree navigator. `WASD` to move cursor, `E` to allocate.

---

## Gem and Socket System

Items can have up to 4 sockets. Sockets can be linked. Gems slot into sockets and modify combat.

**Skill gems** activate new moves in combat. **Support gems** modify linked skills.

```
  [S]--[S]  [S]     <- socket layout (-- = linked)
```

- Linked supports apply to the skill in the same link group
- Items can have engine locks: a specific chaos engine is permanently forced into or out of that item's roll chain
- Sockets and links are chaos-rolled on item generation -- rare items tend to have more

---

## Crafting

Six operations at any Crafting bench (found in Crafting zones and some NPC hubs):

| Operation | Effect | Cost |
|-----------|--------|------|
| **Reforge** | Reroll all stat modifiers (keeps base type) | 120 gold |
| **Augment** | Add one chaos-rolled stat modifier | 60 gold |
| **Annul** | Remove one random stat modifier | 80 gold |
| **Corrupt** | Destiny roll the entire item (can add unique implicits, double stats, or brick it) | 150 gold |
| **Fuse** | Attempt to add a socket link (chaos-rolled, fails possible) | 100 gold |
| **Engine Lock** | Permanently lock an engine into/out of this item's rolls | 200+ gold |

Corrupted items cannot be crafted further.

**Corruption implicits** include: ExtraEngines, DoublePrimary, TripleStat, SelfAware, StatMirror, CritStatusProc, GoldSyphon, and MathematicalError (bricked).

---

## Factions

Three factions, each aligned with a mathematical philosophy.

| Faction | Philosophy | Pact (Trusted+) |
|---------|-----------|-----------------|
| **Order of Convergence** | "Chaos is a disease. We are the cure." | Chain length -1 (min 3). More predictable rolls. |
| **Cult of Divergence** | "The ceiling is a suggestion. We remove suggestions." | Chain length +2. Higher ceiling, lower floor. |
| **Watchers of the Boundary** | "The boundary between order and chaos is where everything interesting happens." | Crit threshold lowered to 65. Near-zero rolls trigger bonus effects. |

- Rep tiers: Hostile / Neutral / Recognized / Trusted / Exalted
- Gaining rep with Order or Cult penalizes the other (-1/3 of the gain)
- Watchers are politically neutral
- Each faction offers quests scaled to your floor with gold and rep rewards
- Vendor dialogue changes based on your standing

---

## Deep Status Effects

Six additional ailments beyond the standard 11. These interact directly with the chaos pipeline itself.

| Effect | Badge | Description |
|--------|-------|-------------|
| Fracture | [FRC] | Some rolls use only 1 engine (extreme volatility in both directions) |
| Resonance | [RES] | Roll N output feeds roll N+1 input (streak amplification) |
| Phase Lock | [PLK] | All rolls use the same seed (good run = invincible, bad run = stuck) |
| Dimensional Bleed | [DLB] | Enemies use your own stat biases against you |
| Recursive | [RCV] | Each engine runs twice -- chaos depth doubled |
| Nullified | [NUL] | All rolls return 0.0 -- base stats only, no crits, no math |

---

## Standard Status Effects

| Effect | Badge | Description |
|--------|-------|-------------|
| Burning | [FIRE] | Takes 8 HP damage per round |
| Poisoned | [PSN] | Takes 3 HP damage per round |
| Stunned | [STN] | Skips next action |
| Cursed | [CRS] | Reduced effectiveness on all rolls |
| Blessed | [BLS] | Bonus to all rolls |
| Shielded | [SHD] | Absorbs incoming damage |
| Enraged | [RAG] | Bonus damage, reduced defense |
| Frozen | [FRZ] | Movement and action restricted |
| Regenerating | [REG] | Heals each round based on Vitality |
| Phasing | [PHS] | Chance to avoid attacks entirely |
| Empowered | [EMP] | All damage amplified |

---

## Combo System

Consecutive attacks build a combo streak. Each hit in a streak adds +20% damage, capped at x2.5. Heavy Attack consumes the streak as a finisher multiplier and resets the counter.

---

## The Infinite Atlas

The endgame. A procedurally generated map of zones stretching to depth 100 and beyond.

- Every zone has 1-3 **zone modifiers** (chaos-rolled) that alter the rules of that zone
- Clear a zone to reveal adjacent zones. High Luck reveals secret anomaly zones.
- Every 10 clears, an **engine-themed Conqueror** spawns -- one for each of the 10 algorithms
- At depth 100: **The Algorithm** -- the chaos pipeline itself. It has no HP. You must destabilize it by manipulating its engine chain until output exceeds input threshold.

**Zone modifiers include:**

| Modifier | Effect |
|----------|--------|
| Collatz Even | Chains start from even numbers -- less volatile |
| Mandelbrot Inside | All Mandelbrot outputs negative |
| Enemy Double Engines | Enemies use twice as many engines |
| Bonus Sockets | All drops have +2 sockets |
| Gravity Reversed | Force/Luck swap; Precision/Cunning swap |
| Squared Outputs | All engine outputs squared (negatives become positive) |
| Lorenz Amplified | sigma=20, rho=50. The butterfly has grown. |
| Reduced Engines | All chains use 2 fewer engines (min 2) |
| Gold Rush | Gold drops x3 |
| Blood Pact | Player and enemies start at 50% HP |
| Hair Trigger | Crits at roll > 50 instead of 90 |
| Engine Shuffle | Engine order randomized every round |
| Open Wounds | Injuries do not recover between fights |
| Empowered Gear | All equipment stat bonuses doubled |

**Daily Seed:** Each day has a globally fixed seed. Everyone playing on the same day gets the same map. Track your best score.

---

## Power Level Tiers

Your character's total stat sum determines your power tier.

| Stat Total | Tier |
|-----------|------|
| Below -1000 | **ABYSSAL** -- "The math has forsaken you. You exist only through spite." |
| -1000 to -300 | **DAMNED** -- "The algorithms hate you specifically. Keep going." |
| -300 to -1 | **CURSED** -- "Even rats pity you. Negative stats are technically valid." |
| 0 to 99 | **Mortal** -- "Statistically average. The Logistic Map is neutral on you." |
| 100 to 299 | **Awakened** -- "The prime numbers notice you. That is an improvement." |
| 300 to 599 | **Champion** -- "The Lorenz attractor bends in your favor." |
| 600 to 999 | **Legendary** -- "The Riemann zeros align. You are an anomaly." |
| 1000 to 2999 | **Transcendent** -- "The Mandelbrot boundary recognizes your face." |
| 3000 to 9999 | **GODLIKE** -- "You ARE the chaos engine. The math screams." |
| 10000+ | **BEYOND MATH** -- "ERROR: STAT OVERFLOW. YOU HAVE BROKEN THE ALGORITHM." |

---

## Color Themes

| Theme | Description |
|-------|-------------|
| Classic | Standard ANSI terminal colors |
| Neon | Bright electric cyberpunk |
| Blood | Deep reds and dark tones |
| Void | Purple and shadow |
| Monochrome | Grayscale only |

---

## Combat Controls

| Key | Action |
|-----|--------|
| `A` | Attack (builds combo) |
| `H` | Heavy Attack (consumes combo streak) |
| `D` | Defend |
| `T` | Taunt |
| `F` | Flee |
| `S1`-`S9` | Cast spell from known spells list |
| `I1`-`I9` | Use item from inventory |
| `P` | Open passive skill tree |
| `B` | View body chart |
| `?` | Show last chaos engine trace |

---

## Game Modes

**Story Mode** -- 20-30 rooms with a procedurally generated quest. The final boss rolls its power level via the Destiny Roll (all 10 engines). It might be weaker than a floor-1 rat. It might be a god.

**Infinite Mode** -- Endless floors. Enemy difficulty scales with depth. Score is tracked and saved to `~/.chaos_rpg_scores.json`.

**Daily Seed** -- Fixed global seed for the current date. Same map for everyone. Race for the high score.

---

## See Also

- [MATH_MANIFESTO.md](MATH_MANIFESTO.md) -- Deep dive into every algorithm

---

## License

MIT -- see [LICENSE](LICENSE) for details.
