# Getting Started with CHAOS RPG

> *Where Math Goes To Die*

This guide walks you through your first run. No prior roguelike experience required — but if you die in the first three rooms, that's working as intended.

---

## Installation

### Pre-built Binaries (Recommended)

Download from the [GitHub Releases page](https://github.com/Mattbusel/chaos-rpg/releases). Two frontends are available:

| Binary | Platform | Description |
|--------|----------|-------------|
| `chaos-rpg-terminal-windows.exe` | Windows | Full-featured TUI (ratatui) |
| `chaos-rpg-terminal-linux` | Linux | Same, for Linux |
| `chaos-rpg-terminal-macos` | macOS | Same, for macOS |
| `chaos-rpg-graphical-windows.exe` | Windows | OpenGL window with themes |
| `chaos-rpg-graphical-linux` | Linux | Same, for Linux |
| `chaos-rpg-graphical-macos` | macOS | Same, for macOS |

**Windows:** Double-click the .exe. Use [Windows Terminal](https://aka.ms/terminal) for best color rendering in the TUI version.

**Linux / macOS:** `chmod +x chaos-rpg-terminal-linux && ./chaos-rpg-terminal-linux`

**macOS "unidentified developer" error:** Right-click → Open, or run `xattr -d com.apple.quarantine ./chaos-rpg-terminal-macos`.

### Build from Source

Requires Rust 1.75+ from [rustup.rs](https://rustup.rs).

```bash
git clone https://github.com/Mattbusel/chaos-rpg
cd chaos-rpg
cargo run --release -p chaos-rpg           # terminal frontend
cargo run --release -p chaos-rpg-graphical # graphical frontend
```

---

## Choosing a Frontend

**Terminal (ratatui):** Runs in your existing terminal. Colorful TUI panels with the full chaos pipeline trace visible during combat. Works over SSH. Keyboard driven.

**Graphical (bracket-lib):** Opens a fullscreen OpenGL window. Bigger text, animated bars, five color themes selectable with `[T]` on the title screen. Requires a GPU with OpenGL 3.3+.

Both frontends are 100% feature-equivalent — same game logic, same saves, same scoreboard.

---

## Your First Run

### 1. Select a Game Mode

You'll be asked to choose between three modes:

- **Story Mode** — 10 floors with a structured narrative arc and a final boss. The recommended starting point. Finite, completable.
- **Infinite Mode** — Floors never end. The math compounds harder with each descent. This is where leaderboard scores come from.
- **Daily Seed** — Everyone in the world plays the identical dungeon today. Resets at UTC midnight. Great for comparing runs.

**Start with Story Mode.** It teaches all the systems before the difficulty spikes.

### 2. Character Creation

Three choices define your character:

#### Class
Your class determines your passive ability — a permanent bonus that reshapes your combat style.

| Class | Passive | Playstyle |
|-------|---------|-----------|
| **Mage** | Critical spells deal ENTROPY/10 bonus damage | Spell-heavy, wants high mana + entropy |
| **Berserker** | Below 30% HP: +40% damage, double attack on crit | High-risk, rewards staying near death |
| **Ranger** | PRECISION/20 accuracy bonus every attack | Consistent damage dealer, wants precision |
| **Thief** | CUNNING/200 + 10% dodge chance | Evasive, wants cunning stat investment |
| **Necromancer** | On kill: absorb 8% of enemy max HP | Sustain through kills, snowballs hard |
| **Alchemist** | Items and potions grant 50% more effect | Makes every item and shop purchase better |
| **Paladin** | Regenerate (3 + VIT/20) HP at turn start | Durable, self-healing, slow but reliable |
| **VoidWalker** | ENTROPY% chance to phase through attacks | High entropy stat = near-invincible |
| **Warlord** | Commands soldiers; morale affects damage | Party-synergy build |
| **Trickster** | Illusion skills; confusion attacks | Debuff-focused, unpredictable |
| **Runesmith** | Inscribes weapons mid-combat | Item-synergy, crafting-focused |
| **Chronomancer** | Manipulates action order; time-dilation | Advanced — alters combat timing |

**Recommended for first run: Paladin** (forgiving) or **Necromancer** (powerful once you start killing things).

#### Background
Backgrounds shift your starting stat distribution:

- **Scholar** — Bonus mana and cunning. Suits spellcasters.
- **Wanderer** — Balanced. No weaknesses. Safe choice.
- **Gladiator** — Bonus force and vitality. More HP and damage.
- **Outcast** — Bonus entropy and luck. Chaotic, high-variance.

#### Difficulty
- **Normal** — Standard enemy scaling. Manageable.
- **Hard** — Enemies hit harder and have more HP.
- **Chaos** — Exponential scaling. Not recommended until you've completed Story Mode.

**Stats are random.** The chaos pipeline rolls them fresh every time, so the same class can produce vastly different characters. If your stats look disastrous, that's also valid — the Misery system (see below) rewards suffering.

### 3. Choose Your Boon

After character creation, three random boons appear. Pick one. This is a free permanent buff. Examples:
- +15 to a random stat
- Start with a rare item
- Begin with 2 known spells
- Gain 20% bonus XP for the entire run

Read all three carefully — some are dramatically better than others for your class.

---

## Navigating the Floor

The floor is a linear sequence of rooms. After entering a room and resolving its event, you advance to the next.

### Room Types

| Icon | Name | What Happens |
|------|------|-------------|
| `[×]` | Combat | Enemy encounter. Must fight or flee. |
| `[★]` | Treasure | Free item. May be cursed. |
| `[$]` | Shop | Buy items, spells, or healing with gold. |
| `[~]` | Shrine | Stat bonuses or healing. Usually good. |
| `[!]` | Trap | Takes damage or applies a debuff. Unavoidable. |
| `[B]` | Boss | Unique boss encounter. See BOSSES.md. |
| `[^]` | Portal | Skip ahead to a later floor. High risk/reward. |
| `[∞]` | Chaos Rift | Pure chaos engine event. Anything goes. |
| `[⚒]` | Crafting Bench | Modify items. Six operations available. |

### Floor Map
The minimap at the bottom of the floor navigation screen shows every room:
- Highlighted cell = current room you're about to enter
- Dim cells = rooms already completed
- Colored cells ahead = upcoming room types

### Descending
Once you've cleared all rooms on a floor, `[D]` becomes available to descend. Each new floor is harder — enemies have more HP and deal more damage, scaling exponentially.

---

## Combat Basics

### The Round Structure
1. You choose an action
2. Your action resolves first
3. The enemy counterattacks (if still alive)
4. Status effects tick (burn, freeze, stun, bleed, poison)
5. Your passive ability fires

### Actions

| Key | Action | Notes |
|-----|--------|-------|
| `A` | Attack | Standard melee. Scales with force stat. |
| `H` | Heavy Attack | More damage, worse accuracy. |
| `D` | Defend | Reduces incoming damage this round. Use when low HP. |
| `T` | Taunt | Forces enemy to attack you. Certain interactions require it. |
| `F` | Flee | Luck-based escape roll. Fails often on deep floors. |
| `1–8` | Cast Spell | Uses mana. Each spell has unique chaos math. |
| `Q–O` | Use Item | Items from inventory slots Q through O. |

### Damage Formula

Every attack runs through the chaos pipeline:

```
damage = (base_force_damage) × chaos_roll_output / 50
```

The chaos roll is produced by chaining: seed → Lorenz attractor → Mandelbrot escape depth → bifurcation map → Fibonacci normalizer → final value. This means identical stats can produce wildly different damage values — a feature, not a bug.

### Reading the Combat Log
The log panel shows every action and its outcome. Lines prefixed with `··` are the internal engine trace — you can see which mathematical transform produced each result.

### When to Flee
- When below 20% HP with no healing items
- When the enemy has a damage spike ability you can't survive
- Note: Fleeing a fight contributes to the Misery Index (see below) and may promote that enemy to Nemesis

### Status Effects

| Effect | What it Does |
|--------|-------------|
| **Burn** | Deals fire damage each turn |
| **Freeze** | Reduces your speed/action chance |
| **Stun** | Skip your action this turn |
| **Bleed** | HP drains slowly each turn |
| **Poison** | Escalating DoT — gets worse over time |
| **Corruption** | Permanent stack; alters chaos pipeline over time |
| **Defiance** | Stacks built from suffering; unlocks Misery abilities |

---

## The Misery System

Bad runs get better the worse they go. The Misery system tracks all the suffering your character accumulates:

- **Misery Index** — grows when you take damage, fail to flee, get headshots, face corrupt enemies, etc.
- **Spite** — resource unlocked at 5,000 Misery. Spend it to power special revenge abilities.
- **Defiance** — stat unlocked at 10,000 Misery. Accumulates through near-death moments.
- **Underdog Multiplier** — if your total stat sum is negative, you gain a logarithmic XP and score bonus. The worse your character, the more rewarding each kill.

At high Misery Index milestones:
- **5,000**: The Spite resource unlocks. Enemies start pitying you (chance to miss you entirely).
- **10,000**: Defiance state activates. Near-death becomes powerful.
- **25,000**: Cosmic Joke. The game acknowledges what you're going through.
- **50,000**: Transcendent Misery. The chaos pipeline inverts — suffering becomes power.
- **100,000**: Published Failure. Your run is memorialized in the Hall of Misery.

---

## Gold and the Shop

Gold drops from enemies and treasure rooms. In the shop:
- `[H]` buys a healing potion (+40 HP)
- `[1]–[4]` buy items from the shop inventory
- Items show their stat modifiers — read them before buying
- Gold carried over between floors, but doesn't survive death

---

## Crafting

Crafting Bench rooms let you modify items. Six operations:

| Operation | Effect | Notes |
|-----------|--------|-------|
| **Reforge** | Randomize ALL stat modifiers | Chaos-reroll everything |
| **Augment** | Add one new random modifier | Increases item complexity |
| **Annul** | Remove one random modifier | Remove a negative roll |
| **Corrupt** | Unpredictable chaos effect | Can double values, flip them, add sockets, change item type |
| **Fuse** | Double all values + upgrade rarity | Expensive but powerful |
| **EngineLock** | Lock a chaos engine signature into the item | Advanced, costs gold |

---

## Stats Reference

All seven stats feed into character power:

| Stat | What It Does |
|------|-------------|
| **Force** | Melee damage. Primary combat stat. |
| **Vitality** | Max HP. Also contributes to defense. |
| **Mana** | Spell power and mana pool. |
| **Cunning** | Dodge chance, skill check success, trap avoidance. |
| **Precision** | Attack accuracy, crit rate, debuff duration. |
| **Entropy** | Chaos engine weight. More entropy = more extreme rolls. |
| **Luck** | Loot quality, flee success, pity chance. |

Stats can be **negative**. A character with -40 force is genuinely fighting at a disadvantage — but the Underdog multiplier compensates with XP and score bonuses.

---

## Power Tiers

Your total stat sum determines your Power Tier, from THE VOID (the lowest conceivable) to ΩMEGA (mathematical divinity). The tier affects how your stats and name are displayed — ΩMEGA tier characters glow with animated rainbow text.

The full tier table is in [MECHANICS.md](MECHANICS.md).

---

## Saving and Scores

- The game saves automatically between floors
- On death, your run is written to `~/.chaos_rpg/scores.json`
- Cross-run legacy data (achievements, graveyard, Hall of Misery) is saved to `~/.chaos_rpg/legacy.json`
- View the live scoreboard from the main menu — top 20 runs by score, plus a separate Hall of Misery leaderboard for suffering-maximization runs

---

## First Run Tips

1. **Don't panic about stats.** Even terrible stats can win through the Misery system's compensation mechanics.
2. **Defend when low HP.** One Defend action can mean the difference between survival and death.
3. **Save gold for the shop.** Healing potions are always available and often the right buy.
4. **Read your boon carefully.** A stat boon for a stat you already have high is often less useful than an item or XP boon.
5. **The combat log tells you everything.** Watch it during fights — it shows engine crits, catastrophes, and passive activations.
6. **Flee strategically.** Fleeing costs Misery Index but saves HP. On deep floors, discretion is better than dying.
7. **Visit crafting rooms.** Annulling a negative modifier off an item is always free value.
8. **Boss rooms are optional in Infinite Mode.** If you're not ready, you can skip (though you miss significant loot and XP).
