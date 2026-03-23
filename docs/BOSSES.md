# CHAOS RPG — Boss Reference

All 12 unique bosses, their mechanics, the math behind them, and how to beat them.

Bosses begin appearing from Floor 5 and grow progressively more complex. Each one is designed to punish a specific build archetype — if you've been abusing one stat or strategy, there is a boss here waiting for you.

---

## How Bosses Work

Standard enemies use the normal `resolve_action()` combat loop. Bosses bypass it entirely — each has a fully custom fight function that implements their unique mechanics. This means:
- Boss fights have unique win conditions (not just "reduce HP to 0")
- Some bosses don't have HP at all
- Some fights are puzzles masquerading as combat
- Boss rewards scale with your current floor: `reward = base + floor × multiplier`

Bosses appear in a **Boss room** (`[B]`). Gauntlet encounters (multiple bosses with no rest) occur on certain milestone floors.

---

## 1. THE MIRROR
**First appears:** Floor 5
**Archetype punished:** High-stat mirror builds
**Reward:** 800 base XP, 200 base gold

```
  ╔═══════╗
  ║  YOU  ║
  ║ (x_x) ║
  ╚═══════╝
```

### Mechanic
The Mirror is an exact copy of you — same maximum HP, same force stat, same mana, same spells. It uses a **divergent chaos seed**, meaning its rolls don't mirror yours even though its stats do.

The Mirror does **not** copy your passive ability. This is the asymmetry you must exploit.

### Fight Details
- The Mirror attacks using your own spell list (random selection each turn) when it rolls a success
- On failed rolls it uses physical attacks based on your force stat
- Its damage is calculated from the **Lorenz engine using your mana as the bias**
- You attack using your own chaos seed chain

### How to Win
1. High Vitality beats it — the Mirror has your max HP but can't regenerate
2. Class passives that trigger on your attacks (Berserker frenzy, Paladin regen) give you the edge
3. Defend reduces damage from your own reflected spells
4. High Entropy characters: The Mirror also has your entropy, but can't use your entropy-scaling passives

### How to Lose
Spellcaster builds with low HP and no defense are particularly vulnerable — your own spells hit hard.

---

## 2. THE ACCOUNTANT
**First appears:** Floor 10
**Archetype punished:** Glass cannons, high damage dealers
**Reward:** 600 base XP, 300 base gold

```
  ┌─────────┐
  │ LEDGER  │
  │ $ $ $ $ │
  └─────────┘
```

### Mechanic
The Accountant has no HP. It does not attack. For exactly 5 rounds, it silently records:
- Your **lifetime total damage dealt** (all previous runs and this run combined)
- Damage you deal **during this fight**

After round 5, it sends you **THE BILL** — a single unavoidable damage hit equal to your recorded damage, minus any Defend reductions.

### Fight Details
- Each Defend action before the bill reduces the multiplier by 20% (max 80% reduction with 4 defends)
- Attacking during the 5 rounds adds to the bill
- The bill formula: `bill = (lifetime_dmg + fight_dmg) × (1 - defend_reduction)`
- Minimum damage: 1 (the bill always costs something)

### How to Win
1. Defend every single round (80% reduction)
2. Don't attack — every point of damage you deal adds to the bill
3. High Vitality characters can survive even a large bill

### How to Lose
High-damage builds with low HP that have been fighting for many floors will receive astronomical bills. The Accountant specifically punishes Berserkers and Mages who have been dealing huge damage all run.

---

## 3. THE FIBONACCI HYDRA
**First appears:** Floor 15
**Archetype punished:** Single-target focused builds
**Reward:** 1,000 base XP, 250 base gold (1,200 XP for perfect clear)

```
  {o} {o}
  /\/\/\
  \  /
   \/
```

### Mechanic
The Fibonacci Hydra splits when killed. It follows the Fibonacci sequence for head counts: 1, 1, 2, 3, 5, 8, 13...

Each child head has `1/φ ≈ 61.8%` of its parent's HP (the golden ratio reciprocal). The swarm grows larger with each generation.

### Fight Details
- Generation 1: 1 head
- Generation 2: 1 head
- Generation 3: 2 heads
- Generation 4: 3 heads
- Generation 5: 5 heads
- ...continues until either you or the Hydra are eliminated
- If you reach 10 total splits, the Hydra collapses under its own weight (you win)
- HP per head: `base_hp / φ^generation_index`

**Perfect Clear Bonus:** Defeating an entire generation in one round (without using Defend) triggers an early-victory condition — the Hydra cannot split further and you win with +200 bonus XP.

### How to Win
1. Focus on clearing full generations quickly
2. AOE spell damage helps (each cast counts as a separate attack against the current head)
3. Survive 10 splits total — even if you can't keep up, endurance wins
4. High force/precision builds can delete heads before counterattack matters

### How to Lose
Defend-heavy builds that don't deal enough damage will be overwhelmed before reaching 10 splits.

---

## 4. THE EIGENSTATE
**First appears:** Floor 15
**Archetype punished:** High-luck characters
**Reward:** 700 base XP, 180 base gold

```
  ?╔══╗?
  ║A?B║
  ?╚══╝?
```

### Mechanic
The Eigenstate exists in quantum superposition between two forms:
- **Form A**: Massive HP (`500 + floor × 100`), no attack capability
- **Form B**: 1 HP, instant-kill attack (deals `max_hp + 1` damage)

The form is determined each round by a chaos roll **biased by your Luck — inverted**. Higher Luck actually *increases* the chance of encountering the deadly Form B.

### Fight Details
- The form is hidden until you commit to an action
- `[T]` Taunt reveals the form safely — the Eigenstate counterattacks lightly but you see what you're facing before committing
- `[A]` Attack commits without revealing — great for Form A, lethal for Form B
- `[D]` Defend survives Form B's one-shot by converting it to `(max_hp + 1 - vitality×2)` damage

### How to Win
1. Use Taunt every round — the small chip damage is always preferable to Form B's instant kill
2. If you have high Vitality, the Defend option becomes viable even against Form B
3. Low-luck characters can read Form A more reliably through aggressive play

### How to Lose
High-luck characters who attack blindly. The probability inversion is the trap — your luck works against you here.

---

## 5. THE TAXMAN
**First appears:** Floor 20
**Archetype punished:** Gold hoarders
**Reward:** Variable (see below)

```
  [TAX]
  (>.<)
  /|$|\
  d   b
```

### Mechanic
The Taxman levies a tax on your gold based on combat duration. Each round you don't kill it, the tax rate increases. The Taxman has standard combat HP but its damage ignores your defense, instead draining your gold directly.

### Fight Details
- Turn 1: 5% of gold as damage (converted to HP loss if broke)
- Turn 2: 10%
- Turn 3: 20%
- Turn 4+: 40% per round
- Physical damage: moderate, but the gold drain is the real threat
- Reward: `base_xp + (gold_taxed / 2)` — you recover half of what was taxed if you win

### How to Win
1. Burst it down fast — every round of delay costs 40% of your gold
2. High damage builds and spell setups shine here
3. Being broke (0 gold) makes the gold drain meaningless — but the physical damage still kills you

### How to Lose
Gold-heavy Merchant builds or runs with lots of carried gold will be completely drained before winning.

---

## 6. THE NULL
**First appears:** Floor 25
**Archetype punished:** Chaos-scalers, entropy builds
**Reward:** 900 base XP, 220 base gold

```
  [ NULL ]
  |      |
  | 0.00 |
  [______]
```

### Mechanic
The Null nullifies the chaos pipeline itself. While fighting The Null, all your chaos rolls return exactly `0.0`. No crits, no catastrophes, no variance — pure, boring, deterministic combat.

Your damage becomes exactly: `base_damage_value` with no roll applied.

### Fight Details
- The Null's own attacks use a simplified damage formula
- High-entropy builds lose the most — entropy becomes worthless at 0 chaos variance
- Status effects still tick
- Passive abilities still fire (they don't depend on chaos rolls)
- The Null can be "reactivated" by dealing exactly 42 damage in a single hit (causes a chaos surge)

### How to Win
1. Builds that rely on passives (Paladin, Necromancer) perform best
2. Pure force-based attackers are fine — base damage is all you need
3. Status effects (burn, bleed) bypass the nullification and deal full damage
4. Hit for exactly 42 damage to break the null state for one round

### How to Lose
Mage builds with high entropy and low base stats — when entropy is cancelled, their damage floor is very low.

---

## 7. THE OUROBOROS
**First appears:** Floor 30
**Archetype punished:** Passive regeneration builds
**Reward:** 950 base XP, 280 base gold

```
  ~~~>--
  |   (O)
  <~~~ /
```

### Mechanic
The Ouroboros heals for a percentage of all damage dealt to it. It also **remembers** damage patterns — attacks using the same damage value twice in a row are absorbed for 0 damage.

The Ouroboros loop: it heals → forces you to deal more damage → it heals more.

### Fight Details
- Regeneration: 8% of incoming damage converted to healing
- Pattern memory: consecutive same-value hits (within ±5) deal 0 damage
- The Ouroboros can enter a "complete loop" state (at full HP after taking damage) which triggers bonus attacks
- Duration mechanic: 15 round limit — if not dead in 15 rounds, it absorbs you (instant death)

### How to Win
1. Vary your damage values — alternate Attack/Heavy Attack/Spell to prevent pattern matching
2. High base damage that overwhelms the 8% regen matters at this floor depth
3. Burst damage strategies (Spite Strike, critical chains) can bypass the regen window
4. Don't Defend — reducing your own damage helps the Ouroboros more

### How to Lose
Paladin (relies on regen + steady attacks). The steady attack pattern is easily matched. The Ouroboros heals faster than you chip.

---

## 8. THE COLLATZ TITAN
**First appears:** Floor 35
**Archetype punished:** Math predictors
**Reward:** 1,100 base XP, 300 base gold

```
  ╔══════╗
  ║3n+1  ║
  ║  /2  ║
  ╚══════╝
```

### Mechanic
The Collatz Titan's HP follows the Collatz conjecture sequence. Starting from its current HP value:
- If HP is even: HP = HP / 2
- If HP is odd: HP = HP × 3 + 1

The Titan attacks you with damage equal to its current sequence step count. The sequence is mathematically proven to always reach 1 — but the path is unpredictable. Long chains mean many rounds of escalating damage.

### Fight Details
- Your attacks set the Titan's HP to any value you choose (not reduce — set)
- Setting to an even number compresses the sequence; odd numbers expand it
- Current sequence step = incoming damage that round
- The Titan dies when its sequence reaches 1 (not when HP hits 0)
- Optimal play: force it into a short Collatz sequence (e.g., HP = 4 → 2 → 1 in 2 steps)

### How to Win
1. Force HP values of 4, 8, 16, or any power of 2 — these reach 1 immediately through repeated halving
2. Avoid setting odd values unless you know the sequence length
3. Cunning stat helps predict which values lead to short paths

### How to Lose
Trying to reduce HP to 0 normally — the game ignores your damage and applies Collatz rules instead. Attack choices that set odd HP values inadvertently chain into long sequences.

---

## 9. THE COMMITTEE
**First appears:** Floor 40
**Archetype punished:** Everyone, via bureaucracy
**Reward:** 850 base XP, 350 base gold

```
  [A][B][C]
  [D] [E]
  COMMITTEE
```

### Mechanic
The Committee has 5 members. Each member votes on the actions taken that round. You must achieve a **majority vote** (3 of 5 in your favor) before your attack resolves. Members are persuaded through specific combat actions.

Each member responds to a different action type:
- **Member A** (The Chair): Persuaded by Taunting
- **Member B** (The Treasurer): Persuaded by accumulating gold
- **Member C** (The Secretary): Persuaded by having fewer than 3 status effects
- **Member D** (The Analyst): Persuaded by dealing > 50 damage in one hit
- **Member E** (The Swing Vote): Random each round — flip of a chaos coin

### Fight Details
- Without majority: attack deals 0 damage (Committee blocks)
- With majority (3+ votes): attack resolves normally
- The Committee counterattacks unanimously if you lose 3 consecutive votes
- Members can be "abstained" (removed from voting) temporarily by certain actions

### How to Win
1. Taunt every round (secures A's vote)
2. Maintain high gold (B's vote)
3. Clear status effects via Defend (C's vote)
4. Three secured votes means E's random flip doesn't matter

### How to Lose
Trying to out-damage the Committee without securing votes. All your damage bounces off bureaucratic procedure.

---

## 10. THE RECURSION
**First appears:** Floor 50
**Archetype punished:** High-kill, grinding builds
**Reward:** 1,400 base XP, 400 base gold

```
  ↻↻↻↻↻↻
  ↻ DMG ↻
  ↻↻↻↻↻↻
```

### Mechanic
The Recursion is a self-referential entity. It deals damage equal to the **total damage dealt in this fight so far** (by both sides). Every round, it adds all previous damage to its current attack.

Turn 1: deals 10 damage
Turn 2: deals 10 + 10 = 20 damage
Turn 3: deals 10 + 20 + 10 = 40 damage
Turn 4: deals 10 + 40 + 20 + 10 = 80 damage

This is exponential escalation. The Recursion's HP is low, but surviving to kill it requires tanking an increasingly catastrophic hit.

### Fight Details
- The Recursion has 150 + floor × 50 HP
- You must deal enough damage to kill it before the recursive damage sum kills you
- Your healing also adds to the damage pool (the Recursion counts it as "damage potential")
- VoidWalker phasing doesn't protect from recursive damage (it bypasses miss mechanics)

### How to Win
1. Burst it down in 1–2 rounds — the early rounds' damage is survivable
2. Berserker frenzy or Mage spell chains can delete it before recursion compounds
3. Defend the first round to reduce your damage contribution to the pool

### How to Lose
Tanky Paladin builds that plan to out-sustain — the Recursion's damage grows faster than any regen can handle.

---

## 11. THE PARADOX
**First appears:** Floor 75
**Archetype punished:** Stat-dumpers, builds with extreme stat disparities
**Reward:** 1,800 base XP, 500 base gold

```
  ∞ ≠ ∞
  ???
  ∅
```

### Mechanic
The Paradox violates mathematical consistency in your stat calculations. Specifically, it forces every stat to be treated as its **negative counterpart** for defense purposes — but not for offense.

Your Force of 200 still deals 200-based damage. But your Vitality of 200 now provides defense as if it were -200.

### Fight Details
- Offense stats remain positive: force, mana, entropy, precision still deal damage normally
- Defense stats are inverted: vitality and luck become anti-defenses
- High-Vitality builds take more damage than normal (the inverse protection backfires)
- The Paradox itself has stats that fluctuate between two contradictory values each turn
- `∞ HP` on even turns (completely immune to damage)
- Vulnerable only on odd turns

### How to Win
1. Attack only on odd turns — your damage on even turns is absorbed
2. Defend on even turns to survive the Paradox's own inverted attacks
3. Low-defense-stat builds (Berserker, VoidWalker) are surprisingly effective
4. Stack offense stats — they're not affected by the inversion

### How to Lose
Builds that invested everything in Vitality for defense. The Paradox turns your greatest strength into your greatest vulnerability.

---

## 12. THE ALGORITHM REBORN
**First appears:** Floor 100
**Archetype punished:** Everyone. Final boss of infinite mode.
**Reward:** 5,000 base XP, 2,000 base gold + Legacy achievement

```
  ▓▓▓▓▓▓▓▓
  ▓THE ALG▓
  ▓REBORN ▓
  ▓▓▓▓▓▓▓▓
```

### Mechanic
The Algorithm Reborn is the game itself turned against you. It has been watching every decision you've made. It knows your playstyle, your stats, your passive — and it has built counter-algorithms for each.

Phase 1 (HP 100%–50%): The Algorithm uses your **most-used combat action** against you. If you've been attacking more than defending, it mirrors attacks back.

Phase 2 (HP 50%–25%): The Algorithm identifies your **strongest stat** and nullifies it for the remainder of the fight.

Phase 3 (HP 25%–0%): Full chaos inversion — every chaos roll is inverted, your crits become catastrophes, your catastrophes become crits.

### Fight Details
- Phase 1 counter-mirror: `damage_reflected = (your_most_used_action_average × 0.8)`
- Phase 2 stat nullification: the highest stat in your character profile is reduced to 1 for all calculations
- Phase 3 inversion: `roll.final_value = -roll.final_value`; crit and catastrophe flags swap
- The Algorithm has `2,000 + floor × 200` HP
- Its own attacks scale with **your** level and floor depth

### How to Win
1. Phase 1: Change your combat pattern mid-fight. If you've been attacking, defend. Mix actions.
2. Phase 2: Accept the stat loss. Builds with multiple strong stats survive this better.
3. Phase 3: Inverted rolls mean your "catastrophes" are now crits. Embrace the inversion — attack aggressively.
4. High Misery Index builds get the Underdog bonus applied against The Algorithm — the harder your run was, the better your multiplier.

### Lore
The Algorithm Reborn is what the dungeon's mathematical substrate becomes after 100 floors of your interference. You didn't just fight through a dungeon — you corrupted it. The ALGORITHM REBORN is the dungeon's attempt to correct the corruption. Whether you kill it or it kills you, the corruption is complete.

---

## Boss Reward Scaling

All boss rewards scale with current floor:
```
xp_reward  = base_xp  + floor × 150
gold_reward = base_gold + floor × 30
```

On Floor 100, The Algorithm Reborn base rewards become:
```
xp  = 5000 + 100 × 150 = 20,000 XP
gold = 2000 + 100 × 30  = 5,000 gold
```

---

## Gauntlet Mode

On milestone floors (every 10th floor starting at floor 20), a **Gauntlet** spawns instead of a single boss:
- 3 consecutive boss fights with no rest between
- Same boss pool as the floor
- Reward multiplied by 1.5× per cleared boss
- Death during a gauntlet still ends the run

Completing a full gauntlet awards a Legacy achievement.
