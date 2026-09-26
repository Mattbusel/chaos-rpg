<p align="center">
  <img src="https://raw.githubusercontent.com/Mattbusel/chaos-rpg/master/assets/banner.png" alt="CHAOS RPG: a roguelike where every roll is a chain of real math" width="100%"/>
</p>

# CHAOS RPG

**A roguelike where every hit, heal and loot drop is decided by chaining real math (the Lorenz attractor, the Mandelbrot set, the Collatz sequence) instead of a random number.**

<p align="center">
  <img src="https://raw.githubusercontent.com/Mattbusel/chaos-rpg/master/assets/proof-frontend.gif" alt="CHAOS RPG running: title screen, character creation, then auto-pilot clears rooms and wins a fight while the combat log shows the math chain behind each roll" width="100%"/>
  <br/><sub>The Proof Engine frontend, recorded today from a hidden window at real speed (one cut): new run, character creation, then auto-pilot (Z) clears a shrine and two fights.</sub>
</p>

## Install

| You have | Run this |
|---|---|
| **Windows** (PowerShell) | `irm https://raw.githubusercontent.com/Mattbusel/chaos-rpg/master/install.ps1 \| iex` |
| **Windows** (Scoop) | `scoop bucket add mattbusel https://github.com/Mattbusel/scoop-bucket` then `scoop install mattbusel/chaos-rpg` |
| **macOS / Linux** (Homebrew) | `brew install mattbusel/tap/chaos-rpg` |
| **macOS / Linux** (script) | `curl -fsSL https://raw.githubusercontent.com/Mattbusel/chaos-rpg/master/install.sh \| sh` |
| **Rust**, prebuilt | `cargo binstall chaos-rpg-graphical` (also `chaos-rpg`, `chaos-rpg-proof`) |
| **Rust**, from source | `cargo install chaos-rpg-graphical` |
| Nothing, just a zip | [Latest release](https://github.com/Mattbusel/chaos-rpg/releases/latest): Windows `.zip`, macOS and Linux `.tar.gz`, with `SHA256SUMS.txt` |
| A browser account | [mattbusel.itch.io/chaos-rpg](https://mattbusel.itch.io/chaos-rpg) |

Every method gives you the same three programs:

| Program | What it is |
|---|---|
| `chaos-rpg-graphical` | The game in its own window. **Start here.** |
| `chaos-rpg-proof` | The same game on [Proof Engine](https://github.com/Mattbusel/proof-engine) (preview, shown in the GIF above). |
| `chaos-rpg` | The same game in your terminal. Works over SSH, no GPU. |

The binaries are not code signed. If you download the zip by hand, Windows SmartScreen may say "unknown publisher": click **More info**, then **Run anyway**. On macOS, right-click the binary and choose **Open** the first time.

## Play in 3 steps

1. **Start it.** Run `chaos-rpg-graphical` (or double-click it). You land on the title screen.
2. **Roll a character.** Pick **New Game**, then a mode (Story is 10 floors and a final boss), a class, a background and a boon. Your stats come from a 10-engine "destiny roll", so the same class can be a god or a corpse.
3. **Descend.** On the floor map press **Enter** to walk into the next room, fight with **A** (attack), **H** (heavy), **D** (defend) or a spell number. Press **Z** to let auto-pilot play, and **V** in combat to watch the math behind each roll.

Want the same run twice? The terminal version takes a seed:

```bash
CHAOS_SEED=666 chaos-rpg          # macOS / Linux
$env:CHAOS_SEED=666; chaos-rpg    # Windows PowerShell
```

Each program answers `--help` and `--version`. Settings (music, difficulty tweaks, visuals) live in `chaos_config.toml` next to the program; see [Configuration](#configuration).

## Results: the math is real

Every combat action prints the chain of engines that produced it. This frame is from the recording above: the hero's attack went Mandelbrot `0.05 -> 1.00`, logistic map `1.00 -> 0.15`, Euler's totient `0.15 -> -0.42`, and landed on **-0.417, a miss**.

<p align="center">
  <img src="https://raw.githubusercontent.com/Mattbusel/chaos-rpg/master/assets/combat-trace.png" alt="Combat screen with the chaos pipeline trace in the combat log" width="85%"/>
</p>

The pipeline is a plain library, so you can run it yourself. This is the real output of `cargo run -p chaos-rpg-core --example roll -- 666` from a clone of this repository:

```text
Attack roll, seed 666
  Logistic Map              0.500 ->  0.374
  Euler's Totient           0.374 -> -0.126
  Collatz Chain            -0.126 -> -0.657
  Modular Exp Hash         -0.657 -> -0.409
  Lorenz Attractor         -0.409 -> -0.064
  final -0.064  d20 10  (miss)

Destiny roll (all 10 engines, used for character creation), seed 666
  Lorenz Attractor          0.500 ->  0.191
  Fourier Harmonic          0.191 -> -0.383
  Prime Density Sieve      -0.383 ->  0.433
  Riemann Zeta Partial      0.433 -> -0.698
  Fibonacci Golden Spiral  -0.698 -> -0.691
  Mandelbrot Escape        -0.691 ->  1.000
  Logistic Map              1.000 ->  0.303
  Euler's Totient           0.303 -> -0.176
  Collatz Chain            -0.176 -> -0.613
  Modular Exp Hash         -0.613 ->  0.609
  final 0.609  game value 80
```

Run it again with the same seed and you get exactly the same numbers. That is why seeded runs are reproducible.

<details>
<summary><b>Screenshots of the stable frontend (chaos-rpg-graphical)</b></summary>

<table>
<tr>
<td><img src="https://raw.githubusercontent.com/Mattbusel/chaos-rpg/master/docs/screenshots/title-void.png" width="420" alt="Title screen: VOID PROTOCOL theme"/><br/><sub>Title screen, VOID PROTOCOL theme</sub></td>
<td><img src="https://raw.githubusercontent.com/Mattbusel/chaos-rpg/master/docs/screenshots/title-emerald.png" width="420" alt="Title screen: EMERALD ENGINE theme"/><br/><sub>Title screen, EMERALD ENGINE theme, chaos field background</sub></td>
</tr>
<tr>
<td><img src="https://raw.githubusercontent.com/Mattbusel/chaos-rpg/master/docs/screenshots/character-sheet.png" width="420" alt="Character sheet: Stats tab"/><br/><sub>Character sheet: stat bars, run info, faction standings</sub></td>
<td><img src="https://raw.githubusercontent.com/Mattbusel/chaos-rpg/master/docs/screenshots/body-chart.png" width="420" alt="Body condition: 13-part injury system"/><br/><sub>Body condition: 13-part injury system</sub></td>
</tr>
<tr>
<td colspan="2"><img src="https://raw.githubusercontent.com/Mattbusel/chaos-rpg/master/docs/screenshots/game-over.png" width="860" alt="Run summary on death"/><br/><sub>Full run summary on death: damage dealt, final events, combat log</sub></td>
</tr>
</table>

</details>

---

## Build from Source

Requires Rust 1.75+ from [rustup.rs](https://rustup.rs). Proof Engine comes from crates.io, so one clone is enough:

```bash
git clone https://github.com/Mattbusel/chaos-rpg
cd chaos-rpg

cargo run --release -p chaos-rpg-graphical  # bracket-lib frontend (stable)
cargo run --release -p chaos-rpg-proof      # Proof Engine frontend (preview)
cargo run --release -p chaos-rpg            # terminal frontend
```

**Seeded runs:**
```bash
CHAOS_SEED=666 cargo run --release -p chaos-rpg         # Linux/macOS
$env:CHAOS_SEED=666; cargo run --release -p chaos-rpg   # Windows PowerShell
```
Same seed = same character stats, same enemies, same loot, every time.

---

## The Chaos Pipeline

This is the math under every number in the game (`core/src/chaos_pipeline.rs`, `core/src/math_engines.rs`).

There are 10 engines. Each takes a value in [-1, 1] plus a seed and returns a new value in [-1, 1]:

| Engine | What it computes |
|---|---|
| Lorenz Attractor | steps the Lorenz system (sigma 10, rho 28, beta 8/3) |
| Fourier Harmonic | a sum of seeded harmonics |
| Prime Density Sieve | prime density around a seeded integer |
| Riemann Zeta Partial | a partial sum of zeta on the critical line |
| Fibonacci Golden Spiral | Fibonacci ratios and the golden angle |
| Mandelbrot Escape | escape time of z -> z^2 + c |
| Logistic Map | iterates r x (1 - x) in the chaotic regime |
| Euler's Totient | phi(n) / n for a seeded n |
| Collatz Chain | stopping time of the Collatz sequence |
| Modular Exp Hash | modular exponentiation as a hash |

```
input (-1..1), seed
  |
  v  chaos_roll_verbose: pick 4 to 8 engines from the seed,
  |  feed each engine's output into the next one
  v
ChaosRollResult
  .final_value     -1.0 .. 1.0
  .chain           every step: engine, input, output (shown in the combat log)
  .game_value      0 .. 100
  .as_d20()        1 .. 20
  .is_success()    final_value > 0.5
  .is_critical()   final_value > 0.8
  .is_catastrophe() final_value < -0.8
```

- **Destiny roll** (character creation) runs all 10 engines in a fixed order, which is why the same class can roll wildly different stats.
- **Biased rolls** blend the chain's result toward a bias, so stats and difficulty tilt the odds without removing the chaos.
- **Corruption:** every 50 kills is one corruption stage (up to 8). Each stage mutates the Lorenz, logistic and Mandelbrot engines, and every second stage adds one more engine to the chain. From 400 kills on, a roll has a 5% to 20% chance to backfire and flip negative.
- **EngineLock** crafting locks an engine into an item's rolls.

Same input and seed always give the same chain, so seeded runs are fully reproducible.

**Chaos Engine Visualizer:** Press **`[V]`** during combat to open a live overlay showing the full engine chain - engine name, raw input, output, delta, and a magnitude bar for each step in the current roll.

---

## Three Frontends, One Game

<details>
<summary>What each frontend has</summary>

All frontends share the same core library (`chaos-rpg-core`), so the game plays the same everywhere.

### Proof Engine frontend (preview)

Built on [Proof Engine](https://github.com/Mattbusel/proof-engine), a mathematical rendering engine written from scratch in Rust.

- **PBR Lighting**: per-room presets (combat red, shrine blue, boss spotlight), per-entity point lights, attack/crit/spell flash lights, status effect lights (burn flicker, freeze steady, poison pulse, stun strobe), floor-depth ambient scaling (warm → cold → void)
- **Shader Graph**: 5 per-theme presets (VOID chromatic+vignette, BLOOD contrast+red, EMERALD CRT+green, SOLAR warm+bloom, GLACIAL desat+blue), floor-depth visual degradation (clean → grain → distortion → VHS), corruption glitch effects, 6 boss-specific shader overrides (Null progressive strip, Paradox hue inversion, Algorithm glitch storm)
- **12 Boss Visuals**: Mirror symmetry line, Accountant gold coins, Fibonacci Hydra golden spiral, Eigenstate form flicker, Taxman gold drain, Null progressive blackout, Ouroboros cycle ring, Collatz live sequence, Committee vote indicators, Recursion stack bar, Paradox reality inversion, Algorithm Reborn 3-phase chaos takeover
- **Cinematics**: 5-phase death sequence, 3-phase victory celebration, 12 unique boss entrance sequences, floor transitions, level-up gold pillar, achievement unlocks (Common→Omega), misery milestones, corruption milestones, nemesis reveal
- **Weather**: digital rain (floors 1-10), compute pulses (11-25), static noise (26-50), ash storms (51-75), electrical storms with lightning (76-99), void snow (100+), boss overrides
- **AI Systems**: 6 steering archetypes, behavior trees (Accountant, Committee), GOAP (Algorithm Reborn), utility AI with logistic scoring
- **Terrain**: isometric noise-based floor map, room-type elevation, epoch-specific glyph sets
- **Economy**: supply/demand pricing, faction treasuries, reputation discounts
- **Dialogue**: Archivist reputation-reactive greetings, boss combat dialogue, Mathematician Fragment codex trees with typewriter and emotion tints
- **Modding**: script hooks (11 event types), mod.toml manifests, hot-reload
- **Replay**: automatic recording, playback with speed control, ghost runs
- **5 Save Slots**: visual state persistence, cloud sync ready
- **Debug Tools**: profiler, field visualizer, inspector, console with 20+ commands

### Graphical (bracket-lib OpenGL)
- Fullscreen OpenGL window at **160×80 tiles**
- Animated HP/MP bars, 5 color themes, chaos field background
- The stable, recommended way to play

### Terminal (ratatui)
- Runs in any terminal emulator (80×24 minimum)
- Full color, keyboard driven, works over SSH

| Theme | Vibe |
|-------|------|
| **VOID PROTOCOL** | Deep space violet/indigo, electric blue |
| **BLOOD PACT** | Gothic crimson on near-black, ember accents |
| **EMERALD ENGINE** | Matrix green circuit geometry |
| **SOLAR FORGE** | Amber/gold alchemical heat |
| **GLACIAL ABYSS** | Crystalline ice blue, zero-precision cold |

</details>

---

## Game Systems

<details>
<summary>Modes, classes, combat, bosses, crafting, achievements and more</summary>

### Game Modes

| Mode | Description |
|------|-------------|
| **Story** | 10 floors, structured narrative, final boss. Recommended for new players. |
| **Infinite** | Floors never end. Score goes on the leaderboard. Enemies scale exponentially. |
| **Daily Seed** | Same dungeon for everyone today. Resets at UTC midnight. Score submits to the global leaderboard. |

### Title Screen Keys

| Key | Opens |
|-----|-------|
| `[N]` | New game |
| `[C]` | Continue saved run |
| `[T]` | Cycle color theme |
| `[?]` | Tutorial (5 slides) |
| `[J]` | Achievements (175 total) |
| `[A]` | Achievements browser (terminal) |
| `[H]` | Run history |
| `[D]` | Daily leaderboard |
| `[E]` | Bestiary (graphical) |
| `[X]` | Codex (graphical) |

### Character Creation

Pick a **name**, **class**, **background**, and **difficulty**. Stats are rolled by chaining a Destiny Roll → Lorenz attractor → full chaos pipeline. Same class, different character every time.

In the graphical version: press **`[N]`** on the character creation screen to type your character's name (up to 20 characters). Leave blank to default to "Hero".

**12 Classes:**

| Class | Passive Ability |
|-------|----------------|
| **Mage** | Critical spells deal ENTROPY/10 bonus damage |
| **Berserker** | Below 30% HP: +40% damage, attack twice on crit |
| **Ranger** | PRECISION/20 bonus accuracy every attack |
| **Thief** | CUNNING/200 + 10% dodge on incoming hits |
| **Necromancer** | On kill: absorb 8% of enemy max HP |
| **Alchemist** | Items and potions grant 50% more effect |
| **Paladin** | Regenerate (3 + VIT/20) HP at turn start |
| **VoidWalker** | ENTROPY% chance to phase through any attack |
| **Warlord** | Commands soldiers; morale affects damage |
| **Trickster** | Illusion attacks; confusion skills |
| **Runesmith** | Inscribes weapons mid-combat for effects |
| **Chronomancer** | Manipulates action order; time-dilation spells |

**8 Backgrounds:**

| Background | Starting Bonus |
|------------|---------------|
| **Scholar** | +15 MANA, +10 ENTROPY |
| **Wanderer** | +15 LUCK, +10 PRECISION |
| **Gladiator** | +15 FORCE, +10 VITALITY |
| **Outcast** | +15 CUNNING, +10 ENTROPY |
| **Merchant** | +15 CUNNING, +10 LUCK |
| **Cultist** | +20 MANA, +20 ENTROPY, -10 VITALITY |
| **Exile** | +20 CUNNING, +10 ENTROPY, -10 MANA |
| **Oracle** | +20 LUCK, +10 MANA, -15 FORCE |

**4 Difficulties:** Easy (0.7× enemy damage, score ×1) → Normal (score ×2) → Brutal (1.4×, score ×4) → CHAOS (2×, all random effects maximized, score ×10)

Stats can be **negative**. A character with -40 force genuinely fights at a disadvantage - but the Misery System compensates.

### The Boon System

After character creation, choose one of three randomly offered permanent bonuses. These range from stat boosts to starting items to passive abilities. Only one per run. The graphical selection screen shows full boon details in a right-side panel.

### Combat

**Round structure:**
1. Player chooses action
2. Player action resolves (damage = force × chaos_roll / 50)
3. Enemy counterattacks (if alive)
4. Status effects tick (burn, freeze, stun, bleed, poison)
5. Class passive fires

**Actions:**

| Key | Action |
|-----|--------|
| `A` | Attack - basic melee, scales with force |
| `H` | Heavy Attack - more damage, lower accuracy |
| `D` | Defend - reduce incoming damage this round |
| `T` | Taunt - force enemy to attack you |
| `F` | Flee - luck+cunning roll to escape |
| `1-8` | Cast spell - costs mana, chaos-powered |
| `Q-O` | Use item from inventory |
| `V` | Toggle Chaos Engine Visualizer overlay |

### Auto-Pilot

Press **`[Z]`** on the floor map to activate Auto-Pilot. The AI handles all navigation and combat decisions automatically, pausing at:
- Item pickup prompts
- Shop / Crafting screens
- Game Over / Victory

When active, the floor map shows **"◆ AUTO-PILOT  [Z] to stop"** - press `[Z]` again to resume manual control. The terminal version shows **[AUTO PILOT ON - Z to stop]** in the status line.

### Power Tiers

The sum of all 7 stats determines your Power Tier, displayed next to your name. 40 tiers from THE VOID to OMEGA. OMEGA-tier characters render with animated rainbow text. Negative-tier characters have glitch, static, and inversion effects on their display.

### The Misery System

Suffering accumulates into the Misery Index. The worse things go, the more powerful the compensations become.

**Misery milestones:**
- **5,000**: Spite resource unlocks. Enemies randomly miss you (up to 25%).
- **10,000**: Defiance activates. Near-death amplifies power.
- **25,000**: Cosmic Joke. The game acknowledges what's happening.
- **50,000**: Transcendent Misery. Suffering becomes direct power.
- **100,000**: Published Failure. Immortalized in the Hall of Misery.

**Underdog Multiplier:** Negative total stats give a logarithmic XP and score bonus. A character at -200 total stats earns roughly ×2.3 XP per kill.

**Spite Actions:** Spend Spite points on revenge abilities - Spite Strike (+50% damage), Bitter Endurance (absorb one hit), Chaos Spite (invert an enemy roll).

### The Passive Skill Tree

~820 nodes organized as 8 class-specific rings × 5 levels, connected by bridge clusters. Earn passive points from leveling up and defeating bosses. Start adjacent to your class entry node and traverse outward. Keystones at the tree's deep nodes provide game-changing abilities.

Press **`[P]`** on the character sheet to view the full tree. Press **`[N]`** anywhere to auto-allocate all pending points.

### The Nemesis System

If you flee from an enemy or barely survive a fight, that enemy may be promoted to Nemesis:
- Gains 50–200% HP bonus
- Gains +25% damage
- Gains a unique ability based on the encounter (fire kills → Immolation, spell kills → Spell Reflection)
- Returns on a later floor titled "Slayer of [your name]"
- Killing a Nemesis gives 3× XP, 3× gold, and a Legacy achievement

### The 12 Unique Bosses

Each boss targets a specific build archetype:

| Boss | Floor | Counter |
|------|-------|---------|
| **The Mirror** | 5+ | Copies your exact stats - exploit your class passive |
| **The Accountant** | 10+ | Sends you a bill based on lifetime damage dealt - defend repeatedly |
| **The Fibonacci Hydra** | 15+ | Splits on death following Fibonacci sequence - burst damage or survive 10 splits |
| **The Eigenstate** | 15+ | Superposition of 1 HP (instant kill) or 10,000 HP (no attack) - Taunt to reveal |
| **The Taxman** | 20+ | Taxes your gold every round at escalating rates - kill it fast |
| **The Null** | 25+ | Nullifies the chaos pipeline - base stats only, status effects still work |
| **The Ouroboros** | 30+ | Heals from damage, remembers attack patterns - vary your attacks |
| **The Collatz Titan** | 35+ | HP follows Collatz sequence - force it into powers of 2 |
| **The Committee** | 40+ | 5 members vote on whether your attack resolves - majority rules |
| **The Recursion** | 50+ | Deals damage equal to all damage dealt this fight - burst or die |
| **The Paradox** | 75+ | Inverts defense stats - high Vitality becomes a liability |
| **The Algorithm Reborn** | 100 | The dungeon itself, fully aware - adapts to your playstyle across 3 phases |

Full boss strategies: [docs/BOSSES.md](https://github.com/Mattbusel/chaos-rpg/blob/master/docs/BOSSES.md)

### Crafting

At Crafting Bench rooms, nine operations are available. The graphical version shows the selected item's current modifiers in a right-side detail panel.

| Key | Operation | Effect |
|-----|-----------|--------|
| `1` | **Reforge** | Chaos-reroll all stat modifiers from scratch |
| `2` | **Augment** | Add one new chaos-rolled modifier |
| `3` | **Annul** | Remove one random modifier |
| `4` | **Corrupt** | Choose risk tier (Safe/Risky/Reckless) - unpredictable chaos effect |
| `5` | **Fuse** | Double all values and upgrade rarity tier |
| `6` | **EngineLock** | Lock a chaos engine into the item permanently |
| `7` | **Shatter** | Destroy the item and scatter its modifiers to other items |
| `8` | **Imbue** | Grant the item 3 charges (bonus effect on use) |
| `9` | **Repair** | Restore item durability to maximum (costs gold) |

**Item Filter:** Press **`/`** in the item selection screen to type-ahead filter by name or rarity. Clear the filter with `Escape`.

### Achievements

175 achievements across 7 rarity tiers, tracked persistently across all runs.

- **Graphical:** Press **`[J]`** on the title screen
- **Terminal:** Press **`[A]`** on the title screen - shows unlocked/locked tabs, rarity colors, progress bars, filter by All/Unlocked/Locked

| Tier | Examples |
|------|---------|
| **Common** | Reach floor 5, deal first kill |
| **Uncommon** | Win on Hard, reach floor 15 |
| **Rare** | Beat a Nemesis, win with negative stats |
| **Epic** | Win on Chaos difficulty, floor 100 |
| **Legendary** | Complete all class masteries |
| **Mythic** | Floor 500, 10,000 total kills |
| **Omega** | Transcendent feats across the entire run history |

Unlocked achievements display full name, description, and unlock date. Locked achievements show `???` until earned.

### Bestiary

Press **`[E]`** on the graphical title screen to open the Bestiary. Every enemy you have encountered is recorded with:
- Times seen, times killed, times it killed you
- Maximum damage dealt
- Lore snippet

The bestiary shows 66 entries at once on the full 160×80 canvas and scrolls with `↑↓`.

### Codex

Press **`[X]`** on the graphical title screen to open the Codex - a lore encyclopedia covering world lore, item descriptions, spell origins, and narrative fragments discovered during runs. Entries unlock as you encounter the corresponding content.

### Run Narrative

After each run, a procedurally generated narrative is recorded in the run history - a short story summarizing the key moments of your run (first kill, near-death moments, nemesis encounters, final fate). Press **`[H]`** on the title screen and select a run to read its full narrative.

### Run History

Press **`[H]`** on the title screen to view a scrollable table of all past runs. The graphical version shows 62 runs at once with extended columns: class, floor, score, kills, mode, difficulty, gold, power tier, and cause of death.

### Daily Leaderboard

Press **`[D]`** on the title screen to view the daily leaderboard. Daily Seed mode uses the same dungeon for all players on a given UTC day. After completing a Daily Seed run, your score is submitted to the global leaderboard automatically.

- Today's seed is shown at the top of the screen
- Your personal best for today is shown below the seed
- The global rankings are listed by rank, name, class, floor, score, and kills
- Press **`[R]`** to refresh the remote scores

### Audio

All sound is synthesized procedurally at startup - no audio files. Every SFX (attacks, spells, level-ups, death) and music loop (menu, exploration, combat, boss, cursed floor) is built from oscillators, ADSR envelopes, and filters seeded from the current floor.

**Music vibes** (set in `chaos_config.toml`):
- `chill` (default) - ambient, low-tempo
- `classic` - heavier, more intense
- `minimal` - sparse, almost silent
- `off` - no music, SFX only

Audio degrades gracefully - if no audio device is found, the game runs silently.

### Saving, Scoring, and Legacy

**Per-run save:** Auto-saves between floors to the same folder as the executable

**Scoreboard:** Top scores saved locally
```
score = kills × floor × difficulty_multiplier × chaos_bonus × underdog_multiplier
```

**Hall of Misery:** Separate leaderboard for `misery_index × floor × underdog_mult`

**Legacy system:**
- 175 persistent achievements across all runs across 7 rarity tiers
- Full run history with per-run statistics and procedural narratives
- Character graveyard with procedurally generated epitaphs
- Hall of Misery historical records

</details>

---

## Configuration

<details>
<summary>Every setting in chaos_config.toml</summary>

`chaos_config.toml` is read from the same folder as the executable on startup. All fields are optional - defaults are used for anything not specified.

```toml
[audio]
# Music vibe: chill (default) | classic | minimal | off
music_vibe = "chill"
# Master music volume (0.0 = silent, 1.0 = default, 2.0 = double)
music_volume = 1.0
# Master SFX volume
sfx_volume = 1.0

[display]
# Multiply particle drift speed (1.0 = default)
particle_speed_mult = 1.0
# Override kill-linger frame count (0 = engine default ~45)
kill_linger_frames = 0
# Halve all visual timings (same as FAST_MODE=1 env var)
fast_mode = false

[gameplay]
# Bonus gold at run start (0 = none)
starting_gold_bonus = 0
# Scale all enemy HP and damage (1.0 = normal, 2.0 = double)
difficulty_modifier = 1.0
# Force a specific seed for Infinite mode (0 = random)
infinite_seed_override = 0
# Disable mechanics
disable_hunger = false
disable_nemesis = false
disable_corruption = false
# Extra inventory slots (0-20)
extra_inventory_slots = 0
# XP multiplier bonus (0.0 = none, 1.0 = double XP)
xp_multiplier = 0.0

[leaderboard]
url = "https://chaos-rpg-leaderboard.mfletcherdev.workers.dev"
submit_daily = true
fetch_on_open = true

[meta]
# Override player name shown in leaderboard submissions
player_name = ""
```

</details>

---

## Room Types

<details>
<summary>The room icons on the floor map</summary>

| Icon | Room | What Happens |
|------|------|-------------|
| `[x]` | Combat | Enemy encounter. No free escape. |
| `[*]` | Treasure | Free item - sometimes cursed. |
| `[$]` | Shop | Buy items, spells, healing with gold. Graphical version shows your inventory in a right-side panel. |
| `[~]` | Shrine | Stat bonuses or healing. |
| `[!]` | Trap | Unavoidable damage or debuff. |
| `[B]` | Boss | Unique boss encounter. |
| `[^]` | Portal | Skip to a later floor. Risk vs reward. |
| `[8]` | Chaos Rift | Pure chaos event. Anything can happen. |
| `[c]` | Crafting | Modify items at the bench. |

</details>

---

## Project Structure

<details>
<summary>Workspace layout</summary>

A Cargo workspace of six crates (`core`, `audio`, `terminal`, `graphical`, `graphical-proof`, `web`). The Proof Engine frontend uses **[Proof Engine](https://github.com/Mattbusel/proof-engine)** from crates.io.

```
chaos-rpg/
├── core/                         # chaos-rpg-core: all game logic
│   └── src/
│       ├── character.rs              12 classes, 8 backgrounds, stat rolling, passives
│       ├── combat.rs                 round resolution, chaos-powered damage
│       ├── chaos_pipeline.rs         10-engine mathematical chain
│       ├── bosses.rs                 12 unique bosses with custom mechanics
│       ├── passive_tree.rs           ~820-node skill tree
│       ├── misery_system.rs          Misery Index, Spite, Defiance
│       ├── achievement_system.rs     175 achievements, 7 rarity tiers
│       └── ... (77 source files)
│
├── graphical-proof/              # chaos-rpg-proof: PROOF ENGINE FRONTEND
│   └── src/
│       ├── main.rs                   ProofGame impl, game loop, screen dispatch
│       ├── state.rs                  150+ field game state
│       ├── theme.rs                  5 themes with engine shader properties
│       ├── lighting.rs               PBR room/combat/boss/floor lighting
│       ├── shader_presets.rs         theme/floor/corruption/boss/status compositing
│       ├── cinematics.rs             boss entrances, death/victory, milestones
│       ├── enemy_ai.rs               steering, behavior trees, GOAP, utility AI
│       ├── weather_system.rs         10 weather types, floor-reactive
│       ├── terrain_map.rs            isometric noise terrain, room elevation
│       ├── game_economy.rs           supply/demand, faction treasuries
│       ├── dialogue_system.rs        Archivist, boss, NPC, codex dialogues
│       ├── mod_system.rs             mod loader, 11 hooks, hot-reload
│       ├── replay_system.rs          recording, playback, ghost runs
│       ├── save_upgrade.rs           5 slots, visual snapshots, cloud sync
│       ├── effects/boss_visuals.rs   12 unique boss visual overlays
│       ├── scenes/chaos_field.rs     2000+ glyph mathematical background
│       ├── screens/                  19 fully implemented screens
│       └── ... (54 source files)
│
├── graphical/                    # chaos-rpg-graphical: bracket-lib frontend
├── terminal/                     # chaos-rpg: terminal frontend
├── audio/                        # chaos-rpg-audio: procedural synthesis
├── web/                          # web frontend: macroquad
│
├── src/                          # older single-crate version of the game (not a workspace member)
├── dist/                         # older release builds (now only in git history; see Releases)
│
├── server/                       # Cloudflare Worker leaderboard
└── docs/                         # Guides, mechanics, boss docs
```

</details>

---

## Contributing

Issues and pull requests welcome. For large changes, open an issue first.

The chaos pipeline parameters (Lorenz σ/ρ/β, Mandelbrot max_iter, bifurcation r range) are intentionally tuned - changes there have cascading effects on all game balance. Test thoroughly.

---

## Further Reading

- [docs/GETTING_STARTED.md](https://github.com/Mattbusel/chaos-rpg/blob/master/docs/GETTING_STARTED.md) - first run walkthrough, stat explanations, survival tips
- [docs/MECHANICS.md](https://github.com/Mattbusel/chaos-rpg/blob/master/docs/MECHANICS.md) - full mathematical breakdown of every system
- [docs/BOSSES.md](https://github.com/Mattbusel/chaos-rpg/blob/master/docs/BOSSES.md) - all 12 bosses, their mechanics, and how to beat them
- [docs/LORE.md](https://github.com/Mattbusel/chaos-rpg/blob/master/docs/LORE.md) - the full lore of The Proof: epochs, factions, engines, bosses, bestiary, items, Fragments
