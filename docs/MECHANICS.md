# CHAOS RPG — Deep Mechanics Reference

This document covers the full mathematical and game-design underpinning of every system in CHAOS RPG. If you want to understand exactly why something happened, this is the reference.

---

## The Chaos Pipeline

The chaos pipeline is the mathematical heart of the game. Every number that matters — damage, healing, loot quality, enemy stats, skill check results — passes through a chain of real mathematical algorithms. The output of each stage feeds into the next.

### The Full Chain

```
Input seed (u64)
  │
  ▼
Stage 1: Lorenz Attractor
  Runs 20 iterations of the Lorenz differential equations:
  dx/dt = σ(y − x)        σ = 10.0
  dy/dt = x(ρ − z) − y    ρ = 28.0
  dz/dt = xy − βz         β = 8/3
  Output: x coordinate after 20 steps (−30..30 range)

  ▼
Stage 2: Mandelbrot Escape
  Tests c = (lorenz_x / 30) + (seed_fractional)i
  Runs max_iter = 100 iterations of z → z² + c
  Output: normalized escape depth (0..1)

  ▼
Stage 3: Zeta Sum
  Computes partial Riemann zeta ζ(s) at s = 2 + (mandelbrot_output)
  Approximated as Σ 1/n^s for n = 1..50
  Output: normalized zeta value

  ▼
Stage 4: Collatz Depth
  Applies Collatz sequence starting from seed-derived integer
  Counts steps to reach 1
  Output: normalized step count

  ▼
Stage 5: Fibonacci Normalizer
  Uses the F(n) / F(n+1) convergence toward 1/φ to normalize
  Output: value centered at 0 with range ±1

  ▼
Stage 6: Euler Product
  Evaluates Euler product formula convergence rate
  Output: small correction factor

  ▼
Final Output: ChaosRollResult
  final_value: f64  (typically −100..100, not capped)
  crit_flag:   bool (|final_value| > 85)
  catastrophe: bool (catastrophic failure — value inverted)
  engine_id:   u8   (which engine dominated this roll)
  trace:        Vec<(engine_name, output)>
```

### Critical Hits and Catastrophes

A roll with `|final_value| > 85` is a **critical hit**. The multiplier applied depends on class:
- Base: ×1.5 damage
- Mage passive: +ENTROPY/10 bonus damage on spell crits
- Berserker passive: double attack on physical crits below 30% HP

A **catastrophe** (`final_value < -95`) means the roll failed completely — an attack that should have connected deals 0 damage, a spell backfires, etc. Catastrophes contribute to the Misery Index.

### Seeding

The seed mutates at each stage via:
```
seed = seed × 6364136223846793005 + 1442695040888963407
```
This is the LCG used internally, chosen for its period and avalanche properties. The floor seed is further mutated by floor number × 31337 at each descent, ensuring each floor feels different even in seeded runs.

### Corruption's Effect on the Pipeline

Corruption stacks accumulate at 1 per kill. Every 50 stacks:
- Stage 1 (Lorenz): σ parameter shifts by +0.5
- Stage 3 (Zeta): evaluation point shifts further from Re(s)=2
- Stage 5 (Fibonacci): convergence target mutates

By corruption stack 8+, the engine is producing outputs that bear no mathematical relationship to the original. This is intentional.

---

## Combat System

### Round Order

1. Player chooses action
2. Player action resolves (damage, healing, spell, flee attempt)
3. Enemy counterattacks if still alive
4. Tick all active status effects
5. Passive ability fires (class-dependent)
6. Check win/loss conditions

### Damage Formula

**Physical attack:**
```
base_dmg = 5 + force/5 + precision/10
roll = chaos_roll(force × 0.01, seed)
raw_dmg = base_dmg + (roll.final_value × force / 200)
crit_mult = 1.5 if roll.is_critical else 1.0
final_dmg = raw_dmg × crit_mult
```

**Spell damage:**
```
base_dmg = spell.base_power + mana/10
roll = chaos_roll(mana × 0.01, seed)
raw_dmg = base_dmg × (1 + roll.final_value/100)
entropy_bonus = entropy/10 if crit (Mage passive)
final_dmg = raw_dmg + entropy_bonus
```

**Heavy attack:**
```
Same as physical but base_dmg × 1.4, accuracy roll also scaled
```

### Defense Calculation

When a player chooses Defend:
```
damage_reduction = vitality/3 + force/5
incoming = max(1, raw_dmg − damage_reduction)
```

### Flee Mechanics

```
flee_chance = (luck × 0.005 + cunning × 0.003).clamp(0.05, 0.85)
roll = chaos_roll(flee_chance, seed + 9999)
success = roll.final_value > 0
```
Failed flee attempts add to Misery Index and may trigger Nemesis promotion.

### The Hunger System

On floors 50+, if you go 3+ rooms without killing an enemy:
- The Hunger activates
- After 5 dry rooms you take escalating HP damage
- Forces aggressive play at deep floors

---

## The Ten Chaos Engines

Each engine is characterized by different distribution properties. The game selects an engine based on the current seed's modular value:

| ID | Engine | Distribution | Notes |
|----|--------|--------------|-------|
| 0 | **Linear** | Uniform | Predictable baseline |
| 1 | **Lorenz** | Butterfly-shaped bimodal | Most common; high variance |
| 2 | **Zeta** | Heavy tails | Rare catastrophic outliers |
| 3 | **Collatz** | Step-distribution | Unpredictable lengths |
| 4 | **Mandelbrot** | Fractal edge density | High density near 0 |
| 5 | **Fibonacci** | Convergent | Very stable near mean |
| 6 | **Euler** | Smooth bell | Low variance |
| 7 | **SharpEdge** | Extreme bimodal | Either very high or very low |
| 8 | **Orbit** | Sinusoidal | Cyclic patterns |
| 9 | **Recursive** | Self-similar | Produces "streaks" |

**Engine Locks:** Via crafting, you can lock a specific engine signature into an item. That item's activations always use the locked engine, giving you predictable (or deliberately exploitable) rolls.

---

## Stats and Leveling

### Stat Rolls at Character Creation

```
destiny_roll → Lorenz(20 steps) → Mandelbrot(depth) → raw_value
stat = base_class_value + raw_value × difficulty_scale
```
Each of the 7 stats gets an independent roll. Stats can be negative if the pipeline produces deeply negative outputs.

### Leveling

```
xp_to_next_level = 100 × level² × difficulty_multiplier
level_up: all stats + chaos_pipeline_bonus(floor, seed)
```
Stat gains on level-up are also chaos-rolled — you might gain 15 force or 0.

### Stat Total and Power Tier

The sum of all 7 stats determines your Power Tier:

| Range | Tier | Effect |
|-------|------|--------|
| < -500 | THE VOID | Pure black display |
| -500 to -300 | ABSOLUTE ZERO | Frozen text |
| -300 to -200 | NEGATIVE INFINITY | Fading text |
| -200 to -150 | STATISTICAL ERROR | Static |
| -150 to -100 | BELOW THE FLOOR | Glitch effect |
| -100 to -75 | VOID-TOUCHED | Inverted |
| -75 to -50 | CURSED | Dark rainbow |
| -50 to -30 | BROKEN | Flash |
| -30 to -15 | DEFECTIVE | Pulse |
| -15 to -5 | BELOW AVERAGE | Normal dim |
| -5 to 5 | MORTAL | Normal |
| 5 to 15 | ABOVE AVERAGE | Normal |
| 15 to 30 | NOTABLE | Normal |
| 30 to 50 | ADEPT | Normal |
| 50 to 75 | CAPABLE | Accent color |
| 75 to 100 | STRONG | Normal |
| 100 to 150 | POWERFUL | Pulse |
| 150 to 200 | EXCEPTIONAL | Pulse |
| 200 to 300 | ELITE | Rainbow |
| 300 to 500 | LEGENDARY | Rainbow |
| 500 to 750 | MYTHIC | Fast Rainbow |
| 750 to 1000 | TRANSCENDENT | Fast Rainbow |
| 1000 to 2000 | DIVINE | Gold Flash |
| 2000 to 5000 | ASCENDANT | Bold White Flash |
| 5000 to 10000 | OMNIPOTENT | Full Flash |
| 10000 to 50000 | ABSOLUTE | Full Flash |
| 50000+ | ΩMEGA | Full animated rainbow |

---

## The Misery System

The Misery System is a second-chance/anti-frustration mechanism that converts accumulated suffering into power. It runs in parallel with normal progression.

### Misery Sources

Each event adds to the Misery Index:

| Source | Misery Added |
|--------|-------------|
| Damage taken | +damage_amount × 0.1 |
| Spell backfire | +50 |
| Headshot received | +80 |
| Flee failed | +100 |
| Enemy critically hits you | +60 |
| Catastrophic roll | +40 |
| Death (before graveyard) | +500 |
| Corruption tick | +25 per stack |

### Misery Milestones

| Index | Milestone | Effect |
|-------|-----------|--------|
| 1,000 | A Rough Start | Flavor text appears |
| 5,000 | Spite Unlocked | Gain Spite resource; enemies start pitying you (random miss chance) |
| 10,000 | Defiance Born | Defiance state activates; near-death amplifies power |
| 25,000 | Cosmic Joke | Combat flavor lines appear; chaos pipeline quirks |
| 50,000 | Transcendent Misery | Suffering inverts; Misery becomes direct damage bonus |
| 100,000 | Published Failure | Immortalized in the Hall of Misery |

### Underdog Multiplier

If your total stat sum is negative:
```
underdog_mult = 1 + log2(|stat_sum| + 1) × 0.15
```
Applied to XP and score. A character with -200 total stats gets approximately ×2.3 XP and score.

### Spite Actions

Spite points (unlocked at 5,000 Misery) can be spent:

| Action | Cost | Effect |
|--------|------|--------|
| Spite Strike | 100 | Next attack deals +50% damage |
| Bitter Endurance | 200 | Absorb next hit for 0 damage |
| Rage Surge | 300 | Temporary +20 force for 3 turns |
| Chaos Spite | 500 | Fully inverts next enemy roll |

### Enemy Pity Chance

At Misery 5,000+, enemies have a chance to miss you entirely:
```
pity_chance = min(0.25, (misery_index - 5000) / 200000)
```
At maximum pity (Misery 50,000+), enemies miss 25% of the time unconditionally.

---

## The Passive Skill Tree

The passive skill tree contains approximately 820 nodes organized in 8 class-specific rings plus a bridge cluster connecting shared nodes.

### Structure

- **8 Class Rings** × **5 Levels** = 40 major clusters
- **Bridge Clusters** connecting classes at each ring level
- Total: ~820 nodes (at time of writing)

### Node Types

| Type | Description |
|------|-------------|
| **Small** | Minor stat bonus (+2–5 to a stat) |
| **Notable** | Significant bonus or unique mechanic |
| **Keystone** | Major passive ability (game-changing) |
| **Bridge** | Connects class clusters |

### Allocating Nodes

Passive points are earned:
- 1 per level-up
- Bonus points from defeating bosses
- Bonus points from certain shrine events

You start adjacent to your class's entry node and must traverse the tree. Reaching a keystone in another class's section is possible but expensive in points.

### Example Keystones

- **Void Resonance** (VoidWalker tree): Every dodge has a 5% chance to deal full attack damage back
- **Blood Covenant** (Necromancer tree): Kills restore 20% max HP instead of 8%
- **Force Calculus** (Berserker tree): Force × Vitality contributes to damage formula
- **Entropy Cascade** (Mage tree): Crits trigger another chaos roll for a chain reaction

---

## The Nemesis System

### Promotion Rules

An enemy may be promoted to Nemesis when:
1. You flee from a fight with it
2. You win a fight with < 10% HP remaining
3. A specific enemy kills you

### Nemesis Properties

A promoted nemesis:
- Gains 50–200% HP bonus depending on how dangerous they were
- Gains +25% damage bonus
- Gains a unique ability based on the encounter type:
  - **Fire kills**: Nemesis gains Immolation (burns on contact)
  - **Spell kills**: Nemesis gains Spell Reflection
  - **Near-miss promotions**: Nemesis gains Vengeance (extra damage when you're below 50% HP)
- Appears on a future floor with the title "Slayer of [your name]"

### Nemesis Rewards

Killing your Nemesis:
- 3× normal XP
- 3× normal gold
- Legacy achievement unlock
- Graveyard record updated

---

## The Legacy System

Cross-run data persists in `~/.chaos_rpg/legacy.json`.

### Achievements (34 total)

Achievements unlock across runs and do not reset. Examples:

| Achievement | Requirement |
|-------------|------------|
| First Blood | Complete your first combat |
| The Floor | Reach floor 10 |
| Negative Hero | Win a run with negative total stats |
| Misery Tourist | Reach 10,000 Misery Index in one run |
| Nemesis Slayer | Kill a nemesis enemy |
| Corruption Master | Reach corruption stage 8 |
| Mathematical Perfection | Deal 10,000+ damage in a single hit |
| The Void | Reach THE VOID power tier |
| Omega Ascension | Reach ΩMEGA power tier |
| Spite Machine | Spend 5,000 Spite in one run |

### The Graveyard

Every character who dies is added to the graveyard with:
- Name, class, level, floor reached
- Cause of death
- Key stats at time of death
- A procedurally generated epitaph based on the run's highlights

### Hall of Misery

Separate leaderboard tracking the highest Misery Index scores:
```
misery_score = (misery_index × floor_reached × underdog_multiplier) as u64
```

---

## World Generation

### Floor Generation

Each floor is generated from:
```
floor_seed = base_seed × LCG + floor_number × 31337
```

Room count: 8–15 rooms per floor, based on floor number.
Room type distribution shifts as floors deepen:
- Early floors: mostly Combat, Treasure, occasional Shop
- Mid floors: Boss rooms start appearing, Crafting increases
- Deep floors (50+): Chaos Rifts become common, Cursed Floors activate

### Cursed Floors

Every 13th floor is a cursed floor. On cursed floors:
- All chaos engine outputs are inverted (high becomes low, low becomes high)
- Enemy attack values are multiplied by 1.5
- Chaos Rift rooms appear more frequently
- Visual indicator shown in the header

### Enemy Scaling

```
enemy_hp = (base_hp + floor × 12) × difficulty_multiplier
enemy_damage = (base_dmg + floor × 3) × difficulty_multiplier
enemy_level = max(1, floor / 5)
```

Difficulty multipliers:
- Normal: ×1.0
- Hard: ×1.5
- Chaos: ×(1 + floor × 0.1) [exponential]

Floor Abilities unlock at certain depths, giving enemies special combat behaviors:
- Floor 10+: Regeneration
- Floor 20+: Spell casting
- Floor 30+: Summoning
- Floor 50+: Nemesis-tier stats

---

## Scoring

```
base_score = kills × 10 + floor_reached × 100 + gold_earned × 0.5
difficulty_mult = 1.0 (Normal) | 1.5 (Hard) | 2.5 (Chaos)
chaos_bonus = corruption_stacks × 5 + max_power_tier_value × 10
underdog_bonus = underdog_multiplier (if negative stats)
final_score = (base_score × difficulty_mult + chaos_bonus) × underdog_bonus
```

Scores are stored locally in `~/.chaos_rpg/scores.json`, top 20 per difficulty.
