# Proof Engine — API Boundary Contract

This document defines everything the proof-engine rendering frontend must support to fully replace the bracket-lib graphical frontend of CHAOS RPG. It is the integration contract between the two systems.

---

## 1. Core Types the Renderer Must Read

All types are from `chaos-rpg-core`. The engine never imports them directly — the game frontend imports both and bridges them.

### Character / Player
```
Character {
    name, class (CharacterClass), background (Background), difficulty (Difficulty),
    stats: StatBlock { vitality, force, mana, cunning, precision, luck, entropy },
    hp, max_hp, xp, level, gold,
    inventory: Vec<Item>, equipped: HashMap<String, Item>,
    spells: Vec<Spell>,
    passive_nodes: Vec<usize>,    // allocated passive tree nodes
    status_effects: Vec<StatusEffect>,
    boon: Option<Boon>,
    run_stats: RunStats,
    misery_state: MiseryState,
    body: Body,                   // per-part HP for BodyChart screen
}
```

### Enemy
```
Enemy {
    name, tier (EnemyTier: Minion/Elite/Champion/Boss/Abomination),
    hp, max_hp, base_damage, attack_modifier, chaos_level,
    xp_reward, gold_reward,
    ascii_sprite: &'static str,
    seed, special_ability, floor_ability (FloorAbility),
}
```

### Combat State / Outcomes
```
CombatAction: Attack | HeavyAttack | Defend | UseSpell(usize) | UseItem(usize) | Flee | Taunt
CombatOutcome: PlayerWon | PlayerDied | PlayerFled | Ongoing
CombatState { player_hp, enemy_hp, round, ... }
ChaosRollResult { final_value: f64, chain: Vec<ChainStep>, game_value: i64 }
ChainStep { engine_name, input, output, seed_used }
```

### Floor / World
```
Floor { rooms: Vec<Room>, current_room: usize, seed }
Room { room_type (RoomType), description, env_effect (EnvEffect), floor, seed, visited }
RoomType: Combat | Treasure | Shop | Shrine | Trap | Boss | Portal | Empty | ChaosRift | CraftingBench
```

### Items
```
Item { name, base_type, material, adjective, rarity (Rarity), modifiers: Vec<StatModifier>,
       is_weapon, volatility, durability, max_durability }
Rarity: Common | Uncommon | Rare | Epic | Legendary | Artifact | Chaos
StatModifier { stat: String, value: i64 }
```

### Spells
```
Spell { name, element, mana_cost, damage_formula, ... }
```

### Bosses
```
boss_id: u8  (1-12 unique bosses)
boss_name(id) -> &'static str
```

### Nemesis
```
NemesisRecord { name, floor_killed_on, kill_count, abilities: Vec<String>, is_promoted }
```

### Achievements
```
AchievementStore { entries: Vec<Achievement> }
Achievement { id, name, description, unlocked, rarity }
```

### Run History
```
RunHistory { records: Vec<RunRecord> }
RunRecord { class, level, floor, cause_of_death, score, timestamp, narrative_events }
```

### Scores / Leaderboard
```
ScoreEntry { name, class, score, floor, level, timestamp }
LeaderboardRow { rank, name, score, floor }
```

### Config
```
ChaosConfig { audio, leaderboard, ... }
```

### Power Tier
```
PowerTier { tier_number: u32, name: &'static str }
```

### Misery System
```
MiseryState { misery_index, spite, defiance, milestone_count }
```

---

## 2. Core Functions the Renderer Calls (via game frontend)

```rust
// Combat
resolve_action(player, enemy, action, combat_state, seed) -> (CombatOutcome, Vec<CombatEvent>, ChaosRollResult)

// World generation
generate_floor(floor_num, seed) -> Floor
room_enemy(room, floor_num, seed) -> Enemy

// Enemy generation
generate_enemy(floor_num, seed) -> Enemy

// Boss pool
boss_pool_for_floor(floor_num) -> Vec<u8>
random_unique_boss(floor_num, seed) -> u8
boss_name(boss_id) -> &'static str

// Items
generate_item(floor_num, seed) -> Item
item_effect(item, player) -> StatDelta

// Chaos pipeline
chaos_roll_verbose(seed, engines, n_steps) -> ChaosRollResult
destiny_roll(seed) -> ChaosRollResult

// Scores
save_score(entry) -> ()
load_scores() -> Vec<ScoreEntry>
load_misery_scores() -> Vec<ScoreEntry>

// Achievements
AchievementStore::check_and_unlock(summary: RunSummary) -> Vec<String>  // newly unlocked names

// Nemesis
save_nemesis(record) / load_nemesis() / clear_nemesis()

// Daily leaderboard
submit_score(url, entry) -> Result<()>
fetch_scores(url, date) -> Result<Vec<LeaderboardRow>>

// Skill checks
perform_skill_check(player, skill_type, difficulty) -> (bool, ChaosRollResult)
```

---

## 3. Screens the Renderer Must Implement

All screens from the `AppScreen` enum in `graphical/src/main.rs`:

| Screen | Key Game State Needed | Key Visual Concerns |
|---|---|---|
| `Title` | save_exists, theme_idx | Chaos field background, logo particle assembly (90-frame timer), menu navigation |
| `Tutorial` | tutorial_slide (1-N) | Multi-slide content, navigation arrows |
| `ModeSelect` | mode_cursor | 3 modes: Story / Infinite / Daily |
| `CharacterCreation` | cc_class, cc_bg, cc_diff, cc_name | Class ASCII art, stat preview, name text input |
| `BoonSelect` | boon_options[3], boon_cursor | 3-column boon cards |
| `FloorNav` | floor, floor_num, player | Room grid map, room icons, player position, environmental descriptions |
| `RoomView` | room_event | Room description, pending item/spell preview, gold/HP deltas |
| `Combat` | player, enemy, combat_state, last_roll, is_boss_fight | HP bars, enemy sprite, action menu, chaos trace panel, particles, screen shake, spell beam, status ambient |
| `Shop` | shop_items, shop_heal_cost, shop_cursor, player | Item listing with costs, heal option |
| `Crafting` | craft_phase, craft_item_cursor, craft_op_cursor, craft_message | Item selection, 8 operation menu, operation animation (4 types) |
| `CharacterSheet` | player, char_tab (0-4) | 5 tabs: Stats / Inventory / Effects / Lore / Log |
| `BodyChart` | player.body | Per-body-part HP visualization |
| `PassiveTree` | player.passive_nodes, passive_scroll | 820+ node grid, allocated/available/locked states |
| `GameOver` | player, floor_num, last_recap_text | Death sequence cinematic, score display, recap text |
| `Victory` | player, floor_num | Victory cinematic |
| `Scoreboard` | loaded scores | Top scores table, misery scores |
| `Achievements` | achievements, achievement_scroll, achievement_filter | Filterable (All/Unlocked/Locked), rarity badges |
| `RunHistory` | run_history, history_scroll | Run list, narrative events |
| `DailyLeaderboard` | daily_rows, daily_status, daily_submitted | Remote leaderboard table, submit button |
| `Bestiary` | bestiary_scroll, bestiary_selected | Enemy encyclopedia, combat stats |
| `Codex` | codex_scroll, codex_selected | Lore/item/spell encyclopedia |
| `Settings` | config | Audio, visual, gameplay toggles |

---

## 4. Input Map Per Screen

### Global (all screens)
| Key | Action |
|---|---|
| `T` | Cycle theme (Title only) |

### Title
| Key | Action |
|---|---|
| `↑/↓` | Move menu cursor |
| `Enter` | Select menu item |
| `L` | Load saved game |
| `T` | Cycle theme |
| `Q` | Quit |
| `J` | Go to Achievements |
| `H` | Go to Run History |
| `B` | Go to Bestiary |
| `X` | Go to Codex |
| `D` | Go to Daily Leaderboard |
| `O` | Settings |
| `?/F1` | Tutorial |

### CharacterCreation
| Key | Action |
|---|---|
| `←/→` | Cycle class |
| `↑/↓` | Move section cursor |
| `Enter` | Confirm / begin name input |
| `Escape` | Back to mode select |
| `R` | Re-roll stats |

### Combat
| Key | Action |
|---|---|
| `A` | Attack |
| `H` | Heavy Attack |
| `D` | Defend |
| `S1-S9` | Use Spell N |
| `I1-I9` | Use Item N |
| `F` | Flee |
| `T` | Taunt |
| `C` | Toggle character sheet |
| `V` | Toggle chaos viz overlay |
| `Z` | Toggle combat log collapse |
| `Space` | Continue (after result) |

### FloorNav
| Key | Action |
|---|---|
| `↑/↓/←/→` | Navigate rooms |
| `Enter` | Enter room |
| `C` | Character sheet |
| `P` | Passive tree |
| `M` | Map |

### CharacterSheet (tabs)
| Key | Action |
|---|---|
| `1-5` | Switch tab |
| `Escape` | Back |

### Shop
| Key | Action |
|---|---|
| `↑/↓` | Move cursor |
| `Enter/B` | Buy |
| `Escape` | Leave shop |

### Crafting
| Key | Action |
|---|---|
| Phase SelectItem: `↑/↓` | Move item cursor |
| Phase SelectItem: `Enter` | Select item → go to SelectOp |
| Phase SelectOp: `↑/↓` | Move op cursor |
| Phase SelectOp: `Enter` | Apply operation |
| `Escape` | Back / exit crafting |
| `/` | Toggle item filter |

### PassiveTree
| Key | Action |
|---|---|
| `↑/↓` | Scroll |
| `Enter` | Allocate node |
| `Escape` | Back |

### All meta screens (Achievements, Bestiary, Codex, RunHistory, DailyLeaderboard, Scoreboard)
| Key | Action |
|---|---|
| `↑/↓` | Scroll |
| `Enter` | Select / drill-down |
| `Escape` | Back to title |
| `F` (Achievements) | Cycle filter |

---

## 5. Visual Effects Inventory

All visual effects from `graphical/src/main.rs` and associated modules that must be reimplemented using proof-engine primitives:

### Particle Emitters
| Event | Description |
|---|---|
| `emit_death_explosion` | 40 radial burst particles (☠ × + · * # ! ▓ ▒ ░ █ ▄), angled 360°, physics: friction 0.90, gravity 0.04 |
| `emit_level_up_fountain` | 30 upward fountain particles (★ ✦ + · ↑ ▲), gold/white |
| `emit_crit_burst` | 16 spark ring (✦), bright yellow |
| `emit_hit_sparks` | 8-16 small sparks (·), color matches damage type |
| `emit_loot_sparkle` | 12 slow-orbiting sparkles (✦ · * +) around drop point |
| `emit_status_ambient` | Per-frame ambient by status: burn=orange sparks, freeze=blue flakes, poison=green bubbles, bleed=red drips, regen=green + |
| `emit_stun_orbit` | 2 orbiting stars (★ ✦), golden, rotating around entity |
| `emit_room_ambient` | Per-room-type: combat=red haze, treasure=gold sparkle, shrine=blue upward, chaos_rift=glitching chars, boss=pulsing purple/red |
| `emit_boss_entrance_burst` | Boss-specific entrance particles (Mirror: symmetric split, Fibonacci: golden spiral, Committee: 5 converging clusters, Algorithm: ring explosion, generic: radial ☠) |

### Screen Shake
```
hit_shake: u32        // frames of outer-border shake on big crits
player_flash: u32     // red border flash on player panel when hit
enemy_flash: u32      // colored border flash on enemy panel
enemy_flash_col       // color varies by attack type
```

### Spell Beam
```
spell_beam: u32           // frames of beam animation between player and enemy
spell_beam_col: (u8,u8,u8)  // beam color = spell element color
```

### Ghost HP Bars
```
ghost_player_hp / ghost_player_timer   // previous HP fraction lingers visually
ghost_enemy_hp / ghost_enemy_timer
```

### Smooth HP/MP Display
```
display_player_hp: f32   // lerped 0.0-1.0, not instant snap
display_enemy_hp: f32
display_mp: f32
```

### Color Grade Push System
```
ColorGrade { tint, saturation, contrast, vignette, ... }
// Applied on: crit = red flash, death = full desaturate, floor transition = black fade
```

### Tile Effects
```
TileEffects   // per-tile visual effects (see tile_effects.rs)
```

### Weather
```
Weather { weather_type (WeatherType), intensity, ... }
// rain, snow, ash, aurora, etc. — overlay effects on the floor navigation screen
```

### Death Sequence
```
DeathSeq   // multi-phase death cinematic (see death_seq.rs)
// Phase 1: player entity breaks apart
// Phase 2: tiles fall
// Phase 3: camera tilts
// Phase 4: death screen fades in
```

### Combat Animations
```
CombatAnim { weapon_kind, spell_element, status_kind, ... }
// Weapon kinds: slash, thrust, crush, cleave
// Spell elements: fire, ice, lightning, void, heal
// Status kinds: burn, freeze, poison, bleed, stun
// Each has unique particle and flash behavior
```

### Nemesis Reveal Cinematic
```
NemesisReveal  // see nemesis_reveal.rs
// Boss entrance where the player's killer returns as nemesis
// Uses unique boss-entrance animation + nemesis name/stats reveal
```

### Achievement Banner
```
AchievementBanner { text, rarity (BannerRarity) }
// Slides in from top, displays for ~180 frames, slides out
// Color coded by rarity
```

### Floor Transition Overlay
```
floor_transition_timer: u32    // black fade + floor number text
floor_transition_floor: u32
```

### Boss Entrance Animation
```
boss_entrance_timer: u32
boss_entrance_name: String
// Particle burst (boss-specific) + name text reveal
// Timer: 60 frames total
```

### Crafting Operation Animations
```
craft_anim_timer: u32
craft_anim_type: u8  // 1=reforge 2=corrupt 3=shatter 4=imbue
// Each has a distinct particle/color effect
```

### Title Logo Assembly
```
title_logo_timer: u32  // 90-frame timer, logo particles fly in from edges to form text
```

### Chaos Field Background
```
ChaosField  // see chaos_field.rs
// 2D field of animated mathematical symbols filling the background
// Driven by chaos engine functions
```

### Text Effects
```
text_effects.rs  // floating damage numbers, ribbon text, etc.
```

### UI Overlay System
```
ui_overlay.rs  // HUD elements, status overlays
```

### Chaos Engine Visualization Overlay
```
chaos_viz_open: bool  // toggle in combat showing engine chain trace as visual bars
```

---

## 6. Architecture Notes

### What stays in core/ (UNCHANGED)
All game logic. The engine NEVER imports from core. The game frontend imports from BOTH.

### Migration Target: graphical/ → graphical-proof/
The new frontend replaces the bracket-lib frontend as the default graphical binary.

The old graphical/ is kept as `graphical-legacy/` for fallback.

### Cargo.toml workspace addition
```toml
[workspace]
members = [
    "core",
    "audio",
    "terminal",
    "graphical",           # legacy (bracket-lib)
    "graphical-proof",     # NEW (proof-engine)
    "web",
]
```

### Running targets
```bash
cargo run --release -p chaos-rpg             # terminal
cargo run --release -p chaos-rpg-graphical   # legacy graphical
cargo run --release -p chaos-rpg-proof       # NEW proof-engine graphical
```

---

## 7. Performance Targets

| Metric | Target |
|---|---|
| Frame rate | 60 fps constant |
| Glyph count | 5000+ at 60 fps |
| Particle count | 2000+ at 60 fps |
| Force fields active | 20+ simultaneous |
| Post-processing | <4ms total pipeline |
| RAM | <100 MB |
| Startup to first frame | <2s |
