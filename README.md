# CHAOS RPG

> *Where Math Goes To Die*

A roguelike where **every single outcome** — character creation, combat, items, spells, skill checks, loot, world generation — is determined by chaining 4–10 real mathematical algorithms together. The output of one feeds into the next. You can roll a character that is literally God incarnate, or the reanimated corpse of the weakest being to ever exist. Both are mathematically valid.

[![CI](https://github.com/Mattbusel/chaos-rpg/actions/workflows/ci.yml/badge.svg)](https://github.com/Mattbusel/chaos-rpg/actions/workflows/ci.yml)
[![Release](https://github.com/Mattbusel/chaos-rpg/actions/workflows/release.yml/badge.svg)](https://github.com/Mattbusel/chaos-rpg/actions/workflows/release.yml)

---

## Download & Run (No Rust Required)

Pre-built binaries are attached to every [GitHub Release](https://github.com/Mattbusel/chaos-rpg/releases).

| Frontend | Platform | How to run |
|----------|----------|-----------|
| **Terminal** (ratatui TUI) | Windows | Double-click `chaos-rpg-terminal-windows.exe` |
| **Terminal** (ratatui TUI) | Linux | `chmod +x chaos-rpg-terminal-linux && ./chaos-rpg-terminal-linux` |
| **Terminal** (ratatui TUI) | macOS | `chmod +x chaos-rpg-terminal-macos && ./chaos-rpg-terminal-macos` |
| **Graphical** (OpenGL window) | Windows | Double-click `chaos-rpg-graphical-windows.exe` |
| **Graphical** (OpenGL window) | Linux | `chmod +x chaos-rpg-graphical-linux && ./chaos-rpg-graphical-linux` |
| **Graphical** (OpenGL window) | macOS | `chmod +x chaos-rpg-graphical-macos && ./chaos-rpg-graphical-macos` |

> **Windows tip:** Use [Windows Terminal](https://aka.ms/terminal) for the best color experience on the TUI frontend.

> **macOS tip:** If you get "unidentified developer", right-click → Open, or run `xattr -d com.apple.quarantine ./chaos-rpg-terminal-macos`.

---

## Build from Source

### Requirements

- [Rust](https://rustup.rs/) stable 1.75+
- Windows Terminal / any ANSI terminal (80×24 minimum for TUI)
- OpenGL 3.3+ capable GPU (for graphical frontend only)

### Clone and build

```bash
git clone https://github.com/Mattbusel/chaos-rpg
cd chaos-rpg
```

#### Terminal frontend (ratatui TUI)

```bash
cargo run --release -p chaos-rpg
```

#### Graphical frontend (bracket-lib OpenGL window)

```bash
cargo run --release -p chaos-rpg-graphical
```

#### Build everything at once

```bash
cargo build --release --workspace
```

---

## Seeded Runs

```bash
# Linux / macOS
CHAOS_SEED=666 cargo run --release -p chaos-rpg

# Windows (PowerShell)
$env:CHAOS_SEED=666; cargo run --release -p chaos-rpg

# Windows (cmd.exe)
set CHAOS_SEED=666 && cargo run --release -p chaos-rpg
```

Same seed → same character stats, same enemies, same loot, every time. Share cursed seeds.

---

## Visual Themes (Graphical Frontend)

Press **`[T]`** on the title screen to cycle through five hand-crafted color palettes:

| Theme | Vibe |
|-------|------|
| **VOID PROTOCOL** | Deep space violet/indigo with electric blue highlights |
| **BLOOD PACT** | Gothic crimson on near-black, ember orange accents |
| **EMERALD ENGINE** | Matrix green circuit-board geometry on dark canvas |
| **SOLAR FORGE** | Amber/gold desert heat, alchemical fire |
| **GLACIAL ABYSS** | Crystalline ice blue, absolute zero precision |

All gradient HP/mana bars, double-line borders, animated cursors, and panel titles adapt to the active theme.

---

## Tutorial

### Character Creation

Pick a class, background, and difficulty. Stats are rolled by chaining a Destiny Roll → Lorenz Attractor → full chaos pipeline. The same class can produce wildly different stat spreads on each run.

**Classes:**

| Class | Passive Ability |
|-------|----------------|
| Mage | Critical spells deal ENTROPY/10 bonus damage |
| Berserker | Below 30% HP: +40% damage, attack twice on crit |
| Ranger | PRECISION/20 bonus accuracy on every attack |
| Thief | CUNNING/200 + 10% dodge chance on incoming hits |
| Necromancer | On kill: absorb 8% of enemy max HP |
| Alchemist | Items and potions grant 50% more effect |
| Paladin | Regenerate (3 + VIT/20) HP at start of each round |
| VoidWalker | ENTROPY% chance to phase through any attack |
| Warlord | Commands a party of soldiers; morale affects stats |
| Trickster | Illusion-based skills; confusion attacks |
| Runesmith | Inscribes weapons mid-combat for bonus effects |
| Chronomancer | Manipulates action order; time-dilation spells |

**Backgrounds:** Scholar (+mana/cunning), Wanderer (balanced), Gladiator (+force/vitality), Outcast (+entropy/luck)

**Difficulties:** Normal → Hard → Chaos (exponential enemy scaling)

### The Floor

Each floor is a procedurally generated map of rooms. Navigate with arrow keys or WASD.

| Symbol | Room Type | What happens |
|--------|-----------|--------------|
| `!` | Combat | Fight an enemy. No free escape. |
| `?` | Treasure | Free loot — sometimes cursed. |
| `+` | Rest | Heal HP. Always safe. |
| `$` | Shop | Spend gold on items and spells. |
| `>` | Stairs | Descend. Enemies scale harder. |
| `B` | Boss | Boss room. Very bad. Very rewarding. |
| `C` | Crafting | Combine items at a workbench. |
| `N` | NPC | Merchant, questgiver, or faction agent. |

### Combat

```
Round structure:
  1. You choose an action
  2. Your action resolves  (damage = force × chaos_roll / 50)
  3. Enemy counter-attacks (if still alive)
  4. Status effects tick   (burn, freeze, stun, bleed, etc.)
  5. Passive abilities fire (Paladin regen, Berserker frenzy, etc.)
```

**Combat actions:**

| Key | Action | Notes |
|-----|--------|-------|
| `A` | Attack | Basic attack. Scales with force stat. |
| `H` | Heal | Use a health item from inventory. |
| `D` | Defend | Raise shield; reduce incoming damage this round. |
| `F` | Flee | Luck + cunning roll to escape. Fails often on high floors. |
| `1–9` | Spell | Cast a known spell. Costs mana. Crits apply class bonuses. |
| `I` | Item | Use a non-healing item (bomb, scroll, etc.). |

### The Chaos Pipeline

Every number — damage, dodge, loot quality, enemy stats — runs through this chain:

```
Input seed
  → Lorenz Attractor      (chaotic differential equations)
  → Mandelbrot Escape     (fractal depth check)
  → Bifurcation Map       (logistic map iteration)
  → Entropy Mixer         (XOR + bit rotations)
  → Gaussian Normalizer
  → Final value           (usually −100..100, but not capped)
```

The **Engine Trace panel** in the TUI shows every step in real time during combat. Watch it spike into absurdity on critical hits.

### Corruption

Every kill adds 1 corruption stack. Every 50 stacks, the chaos pipeline gains a permanent mutation — shifted seeds, altered weights. By stage 4 you are no longer playing the same game. By stage 8, the math is unrecognizable.

### The Nemesis System

If you flee from an enemy or barely survive a fight, that enemy may be **promoted to Nemesis**:
- They remember how you fought
- They gain bonus HP, damage, and a unique ability based on that encounter
- They can reappear on later floors with a title ("Slayer of [your name]")
- Defeating a nemesis gives dramatically increased rewards

### Boss Gauntlets

Every 5 floors is a boss room. Every 10 floors is a gauntlet (multiple bosses, no rest between).

The 12 unique bosses include:

- **The Fractal King** — splits into smaller copies at 50% HP
- **Entropy Incarnate** — randomizes your stats each round
- **The Void Mirror** — copies your own stats and fights you with them
- **Mathbreaker** — forces integer overflow rolls; damage wraps around
- **The Recursive Lich** — heals for a fraction of all damage dealt to it
- *(and 7 more)*

### Factions & World Map

The world map spans multiple regions, each controlled by a faction. Reputation with each faction affects:
- Shop prices and available stock
- NPC dialogue options
- Whether guards let you pass or attack on sight
- Faction-exclusive quests and rewards

### Party System

Recruit NPCs as party members. Each has their own class, stats, and morale. Party morale affects combat performance — lead well or watch them flee.

### Items & Crafting

Items drop from enemies and treasure rooms. Combine them at a **Crafting Bench** room:

```
Health Potion + Chaos Shard  →  Overloaded Potion (heal + random chaos effect)
Iron Sword    + Fire Gem     →  Flameblade (+fire damage on attacks)
```

Full recipe list in [`MATH_MANIFESTO.md`](MATH_MANIFESTO.md).

### Audio

The game synthesizes all sound procedurally at startup — no audio files bundled. Every SFX (attacks, spells, level-up fanfares, death stingers) and music loop (menu, exploration, combat, boss, cursed floor) is built from oscillators, ADSR envelopes, and filters seeded from the current floor. Audio is optional — the game runs silently if no audio device is found.

### Saving & Scoreboard

The game auto-saves between floors. On death, your run is written to `~/.chaos-rpg/scores.json`. View the scoreboard from the main menu — top 20 runs, sorted by score.

Score = kills × floor_reached × difficulty_multiplier × chaos_bonus.

---

## Project Structure

```
chaos-rpg/
├── core/                    # chaos-rpg-core — all game logic (library crate, no UI)
│   └── src/
│       ├── character.rs         stat rolling, leveling, passive abilities
│       ├── combat.rs            hit resolution, round loop
│       ├── chaos_pipeline.rs    Lorenz → Mandelbrot → bifurcation chain
│       ├── enemy.rs             enemy generation + exponential floor scaling
│       ├── bosses.rs            12 unique bosses + gauntlet system
│       ├── nemesis.rs           nemesis promotion + memory system
│       ├── world.rs             floor + room procedural generation
│       ├── items.rs             item definitions
│       ├── spells.rs            spell generation
│       ├── passive_tree.rs      skill tree nodes
│       ├── crafting.rs          recipe system
│       ├── scoreboard.rs        score persistence
│       ├── faction_system.rs    faction reputation + world control
│       ├── party_system.rs      multi-character party management
│       ├── audio_events.rs      audio event enum + music state machine types
│       ├── audio_synth.rs       procedural oscillator/ADSR synthesizer
│       └── ...                  (40+ modules total)
│
├── audio/                   # chaos-rpg-audio — rodio playback backend
│   └── src/
│       ├── lib.rs               AudioSystem: mpsc channel + background thread
│       ├── sound_bank.rs        pre-generated WAV buffers for all SFX
│       └── music_system.rs      state-machine music transitions + looping
│
├── terminal/                # ratatui TUI frontend
│   └── src/
│       ├── main.rs              game loop, screen routing, thread-local audio
│       ├── ui.rs                menus, character sheet, floor nav (ANSI)
│       └── ratatui_screens.rs   full ratatui layout (combat, engine trace,
│                                Unicode portraits, HP bars, scoreboard)
│
├── graphical/               # bracket-lib CP437 OpenGL frontend
│   └── src/
│       ├── main.rs              game loop, all screens
│       ├── renderer.rs          double-box, gradient bars, animated helpers
│       ├── theme.rs             5 visual themes (VOID/BLOOD/EMERALD/SOLAR/GLACIAL)
│       ├── sprites.rs           CP437 sprite definitions
│       └── ui_overlay.rs        damage floats, tooltips, status banners
│
├── .github/workflows/
│   ├── ci.yml                   test + check on every push
│   └── release.yml              build both frontends × 3 platforms on tag
│
└── MATH_MANIFESTO.md        deep-dive on every algorithm used
```

---

## The Math

Full breakdown of every algorithm, why it was chosen, and what it means for gameplay: [`MATH_MANIFESTO.md`](MATH_MANIFESTO.md).

Short version: every number the game produces is a lie told by differential equations. The lie is consistent. The consistency is the game.

---

## Contributing

PRs welcome. Rules:
1. **No bare RNG** — all randomness must route through `chaos_pipeline::chaos_roll_verbose`.
2. **No UI code in `core/`** — only `io_util.rs` exceptions (ANSI constants + `prompt()`).
3. New bosses go in `core/src/bosses.rs`. New enemies in `core/src/enemy.rs`.
4. `cargo test --workspace` must pass before submitting.

---

MIT
