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
| **Web/Native** (macroquad) | Windows | `chaos-rpg-web-windows.exe` |
| **Web** (WASM) | Browser | Serve `chaos-rpg-web.wasm` + `index.html` via any HTTP server |

> **Windows tip:** Use [Windows Terminal](https://aka.ms/terminal) for the best color experience on the TUI frontend. The old `cmd.exe` works but looks worse.

> **macOS tip:** If you get "unidentified developer", right-click → Open, or run `xattr -d com.apple.quarantine ./chaos-rpg-terminal-macos`.

A legacy single-binary is also kept at `dist/chaos-rpg.exe` in the repo for quick access.

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

#### Terminal frontend (ratatui TUI — recommended)

```bash
cargo run --release -p chaos-rpg
```

#### Graphical frontend (bracket-lib OpenGL window)

```bash
cargo run --release -p chaos-rpg-graphical
```

#### Web / macroquad frontend (native)

```bash
cargo run --release -p chaos-rpg-web
```

#### Web frontend → WASM

```bash
rustup target add wasm32-unknown-unknown
cargo build --release -p chaos-rpg-web --target wasm32-unknown-unknown
# Then serve target/wasm32-unknown-unknown/release/chaos-rpg-web.wasm + an index.html
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

## Tutorial

### First Launch

When you start the game you land on the **title screen**. Use arrow keys (TUI) or mouse (graphical/web) to navigate. Select **New Game**.

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
| `T` | Status | Open equipment/status screen mid-combat. |
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

Corruption stage and progress are shown on the character sheet once they begin.

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

### Leveling Up

Gain XP from combat. On level-up:
- All stats increase (amount based on chaos roll, not a fixed table)
- You learn a new spell
- You receive skill points for the **Passive Tree** (Path of Exile-style node grid)

### Items & Crafting

Items drop from enemies and treasure rooms. Combine them at a **Crafting Bench** room:

```
Health Potion + Chaos Shard  →  Overloaded Potion (heal + random chaos effect)
Iron Sword    + Fire Gem     →  Flameblade (+fire damage on attacks)
```

Full recipe list in [`MATH_MANIFESTO.md`](MATH_MANIFESTO.md).

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
│       ├── io_util.rs           ANSI constants + prompt() for non-UI use
│       └── ...                  (30+ modules total)
│
├── terminal/                # ratatui TUI frontend
│   └── src/
│       ├── main.rs              game loop, screen routing
│       ├── ui.rs                menus, character sheet, floor nav (ANSI)
│       └── ratatui_screens.rs   full ratatui layout (combat, engine trace,
│                                Unicode portraits, HP bars, scoreboard)
│
├── graphical/               # bracket-lib CP437 OpenGL frontend
│   └── src/
│       ├── main.rs              game loop, all 6 screens
│       ├── renderer.rs          titled-box and progress-bar helpers
│       ├── sprites.rs           CP437 sprite definitions (player + enemies)
│       └── ui_overlay.rs        damage floats, tooltips, status banners
│
├── web/                     # macroquad frontend — native desktop + WASM
│   └── src/
│       └── main.rs              full game loop, particle system,
│                                immediate-mode 2D UI, damage floats
│
├── assets/
│   ├── tilesets/                CP437 and custom PNG tilesets
│   ├── sprites/classes/         per-class player sprites
│   ├── sprites/enemies/         enemy sprites
│   └── fonts/                   bitmap fonts
│
├── dist/
│   └── chaos-rpg.exe            pre-built Windows binary (legacy)
│
├── .github/workflows/
│   ├── ci.yml                   test + check on every push
│   └── release.yml              build all 3 frontends × 3 platforms + WASM on tag
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

## License

MIT
