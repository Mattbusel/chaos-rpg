//! Combat HUD — heads-up display for all combat information.
//!
//! Renders player info, enemy info, action buttons, inventory quick-access,
//! floor indicator, combo meter, turn order, minimap, and tooltips.
//! All rendering goes through `ui_render` and `engine.spawn_glyph`.

use proof_engine::prelude::*;
use crate::state::GameState;
use crate::theme::{Theme, THEMES};
use crate::ui_render;
use crate::combat_visuals::{self, CombatVisualState};

use chaos_rpg_core::character::{CharacterClass, StatusEffect};
use chaos_rpg_core::enemy::EnemyTier;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Constants
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Left edge of the screen for panels.
const LEFT_EDGE: f32 = -8.5;
/// Right edge of the screen for panels.
const RIGHT_EDGE: f32 = 4.0;
/// Top of screen.
const TOP_Y: f32 = 5.0;
/// Bottom action bar Y position.
const ACTION_BAR_Y: f32 = -3.5;
/// Quick slot Y position.
const QUICK_SLOT_Y: f32 = -4.0;
/// HP bar width in world units.
const BAR_WIDTH: f32 = 4.0;
/// Bar scale (glyph size).
const BAR_SCALE: f32 = 0.28;
/// Combo meter X position.
const COMBO_METER_X: f32 = 8.0;
/// Combo meter height.
const COMBO_METER_HEIGHT: f32 = 6.0;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// HUD State
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Persistent HUD state (interpolation targets, tooltip tracking).
pub struct CombatHudState {
    /// Smooth HP display for player.
    pub display_player_hp: f32,
    /// Smooth HP display for enemy.
    pub display_enemy_hp: f32,
    /// Smooth MP display.
    pub display_mp: f32,
    /// Ghost bar tracking for player HP.
    pub ghost_player_hp: f32,
    pub ghost_player_timer: f32,
    /// Ghost bar tracking for enemy HP.
    pub ghost_enemy_hp: f32,
    pub ghost_enemy_timer: f32,
    /// Active tooltip text (empty = no tooltip).
    pub tooltip_text: String,
    pub tooltip_x: f32,
    pub tooltip_y: f32,
    /// Quick slot items (up to 3).
    pub quick_slots: [Option<QuickSlotItem>; 3],
    /// Combo meter fill (0.0 - 1.0).
    pub combo_fill: f32,
    pub combo_fill_target: f32,
    /// Animated border phase.
    pub border_phase: f32,
}

/// A quick-slot item display.
pub struct QuickSlotItem {
    pub name: String,
    pub icon: char,
    pub count: u32,
}

impl CombatHudState {
    pub fn new() -> Self {
        Self {
            display_player_hp: 1.0,
            display_enemy_hp: 1.0,
            display_mp: 0.0,
            ghost_player_hp: 1.0,
            ghost_player_timer: 0.0,
            ghost_enemy_hp: 1.0,
            ghost_enemy_timer: 0.0,
            tooltip_text: String::new(),
            tooltip_x: 0.0,
            tooltip_y: 0.0,
            quick_slots: [None, None, None],
            combo_fill: 0.0,
            combo_fill_target: 0.0,
            border_phase: 0.0,
        }
    }

    /// Reset for new combat.
    pub fn reset(&mut self) {
        self.display_player_hp = 1.0;
        self.display_enemy_hp = 1.0;
        self.display_mp = 0.0;
        self.ghost_player_hp = 1.0;
        self.ghost_player_timer = 0.0;
        self.ghost_enemy_hp = 1.0;
        self.ghost_enemy_timer = 0.0;
        self.tooltip_text.clear();
        self.combo_fill = 0.0;
        self.combo_fill_target = 0.0;
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// UPDATE
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Tick HUD interpolation and state.
pub fn update(hud: &mut CombatHudState, state: &GameState, vis: &CombatVisualState, dt: f32) {
    let lerp_speed = 1.0 - (1.0 - 0.08_f32).powf(dt * 60.0);

    // Smooth HP display
    if let Some(ref player) = state.player {
        let target_hp = player.current_hp as f32 / player.max_hp.max(1) as f32;
        // Detect HP change for ghost bar
        if (target_hp - hud.display_player_hp).abs() > 0.01 {
            hud.ghost_player_hp = hud.display_player_hp;
            hud.ghost_player_timer = 0.8;
        }
        hud.display_player_hp += (target_hp - hud.display_player_hp) * lerp_speed;

        let max_mp = state.max_mana();
        let target_mp = if max_mp > 0 {
            state.current_mana as f32 / max_mp as f32
        } else {
            0.0
        };
        hud.display_mp += (target_mp - hud.display_mp) * lerp_speed;
    }

    if let Some(ref enemy) = state.enemy {
        let target = enemy.hp as f32 / enemy.max_hp.max(1) as f32;
        if (target - hud.display_enemy_hp).abs() > 0.01 {
            hud.ghost_enemy_hp = hud.display_enemy_hp;
            hud.ghost_enemy_timer = 0.8;
        }
        hud.display_enemy_hp += (target - hud.display_enemy_hp) * lerp_speed;
    }

    // Ghost bar timers
    if hud.ghost_player_timer > 0.0 {
        hud.ghost_player_timer = (hud.ghost_player_timer - dt).max(0.0);
        if hud.ghost_player_timer <= 0.0 {
            hud.ghost_player_hp = hud.display_player_hp;
        }
    }
    if hud.ghost_enemy_timer > 0.0 {
        hud.ghost_enemy_timer = (hud.ghost_enemy_timer - dt).max(0.0);
        if hud.ghost_enemy_timer <= 0.0 {
            hud.ghost_enemy_hp = hud.display_enemy_hp;
        }
    }

    // Combo fill
    hud.combo_fill_target = (vis.combo_count as f32 / 100.0).clamp(0.0, 1.0);
    hud.combo_fill += (hud.combo_fill_target - hud.combo_fill) * lerp_speed;

    // Animated border
    hud.border_phase += dt * 0.5;
    if hud.border_phase > std::f32::consts::TAU {
        hud.border_phase -= std::f32::consts::TAU;
    }

    // Update quick slots from player inventory
    if let Some(ref player) = state.player {
        // Find first 3 consumable-like items
        let mut slot_idx = 0;
        for item in player.inventory.iter().take(20) {
            if slot_idx >= 3 { break; }
            let name = &item.name;
            let icon = if name.contains("Potion") || name.contains("potion") {
                '!'
            } else if name.contains("Scroll") || name.contains("scroll") {
                '?'
            } else if name.contains("Bomb") || name.contains("bomb") {
                '*'
            } else {
                continue; // skip non-consumables for quick slots
            };
            hud.quick_slots[slot_idx] = Some(QuickSlotItem {
                name: name.clone(),
                icon,
                count: 1,
            });
            slot_idx += 1;
        }
        // Clear unused slots
        for i in slot_idx..3 {
            hud.quick_slots[i] = None;
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Master HUD
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Render the full combat HUD.
pub fn render(hud: &CombatHudState, vis: &CombatVisualState, state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    // Floor/room indicator (top center)
    render_floor_indicator(state, engine, theme);

    // Player info panel (top-left)
    if let Some(ref player) = state.player {
        render_player_panel(hud, state, player, engine, theme);
    }

    // Enemy info panel (top-right)
    if let Some(ref enemy) = state.enemy {
        render_enemy_panel(hud, enemy, engine, theme);
    }

    // Action buttons (bottom)
    combat_visuals::render_action_menu(vis, engine, theme);

    // Quick slots (bottom-right)
    render_quick_slots(hud, engine, theme);

    // Combo meter (right side)
    render_combo_meter(hud, vis, engine, theme);

    // Turn order display
    if let (Some(ref player), Some(ref enemy)) = (&state.player, &state.enemy) {
        combat_visuals::render_turn_order(vis, engine, &player.name, &enemy.name, theme);
    }

    // Minimap
    combat_visuals::render_minimap(vis, engine, theme);

    // Combat log
    combat_visuals::render_combat_log(
        engine,
        &state.combat_log,
        state.combat_log_collapsed,
        theme,
    );

    // Tooltip (if any)
    if !hud.tooltip_text.is_empty() {
        combat_visuals::render_tooltip(engine, &hud.tooltip_text, hud.tooltip_x, hud.tooltip_y, theme);
    }

    // Animated border decoration
    render_animated_border(hud, engine, theme);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Floor Indicator
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_floor_indicator(state: &GameState, engine: &mut ProofEngine, theme: &Theme) {
    let header = if state.is_boss_fight {
        format!("Floor {} -- BOSS FIGHT", state.floor_num)
    } else {
        format!("Floor {} -- Combat", state.floor_num)
    };

    let header_color = if state.is_boss_fight { theme.danger } else { theme.heading };
    ui_render::heading_centered(engine, &header, TOP_Y, header_color);

    // Decorative divider line below header
    let divider = "------------------------------------";
    let div_color = Vec4::new(theme.dim.x, theme.dim.y, theme.dim.z, 0.4);
    ui_render::text_centered(engine, divider, TOP_Y - 0.5, div_color, 0.2, 0.1);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Player Info Panel
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_player_panel(
    hud: &CombatHudState,
    state: &GameState,
    player: &chaos_rpg_core::character::Character,
    engine: &mut ProofEngine,
    theme: &Theme,
) {
    let px = LEFT_EDGE;
    let py = 3.8;

    // Class icon
    let class_icon = class_icon_char(player.class);
    let class_color = class_icon_color(player.class);
    engine.spawn_glyph(Glyph {
        character: class_icon,
        position: Vec3::new(px, py, 0.0),
        scale: Vec2::splat(0.4),
        color: class_color,
        emission: 0.5,
        glow_color: Vec3::new(class_color.x, class_color.y, class_color.z),
        glow_radius: 0.3,
        layer: RenderLayer::UI,
        ..Default::default()
    });

    // Name and level
    let name_text = format!("{} Lv.{}", player.name, player.level);
    ui_render::body(engine, &name_text, px + 0.6, py, theme.heading);

    // Class name
    ui_render::small(engine, player.class.name(), px + 0.6, py - 0.45, theme.dim);

    // HP label + bar
    let hp_y = py - 0.9;
    ui_render::small(engine, "HP", px, hp_y, theme.muted);
    combat_visuals::render_hp_bar(
        engine,
        px + 0.8,
        hp_y,
        BAR_WIDTH,
        hud.display_player_hp,
        hud.ghost_player_hp,
        theme,
        BAR_SCALE,
    );
    let hp_text = format!("{}/{}", player.current_hp, player.max_hp);
    ui_render::small(engine, &hp_text, px + BAR_WIDTH + 1.2, hp_y, theme.hp_color(hud.display_player_hp));

    // MP bar (if applicable)
    let max_mp = state.max_mana();
    if max_mp > 0 {
        let mp_y = hp_y - 0.45;
        ui_render::small(engine, "MP", px, mp_y, theme.muted);
        ui_render::bar(engine, px + 0.8, mp_y, BAR_WIDTH, hud.display_mp, theme.mana, theme.muted, BAR_SCALE);
        let mp_text = format!("{}/{}", state.current_mana, max_mp);
        ui_render::small(engine, &mp_text, px + BAR_WIDTH + 1.2, mp_y, theme.mana);
    }

    // XP bar
    let xp_y = hp_y - if max_mp > 0 { 0.9 } else { 0.45 };
    let xp_for_next = xp_to_next_level(player.level);
    let xp_ratio = if xp_for_next > 0 {
        (player.xp as f32 % xp_for_next as f32) / xp_for_next as f32
    } else {
        0.0
    };
    ui_render::small(engine, "XP", px, xp_y, theme.muted);
    ui_render::bar(engine, px + 0.8, xp_y, BAR_WIDTH, xp_ratio, theme.xp, theme.muted, BAR_SCALE);

    // Status effects row
    let status_y = xp_y - 0.5;
    render_status_row(engine, &player.status_effects, px, status_y, theme);

    // Stats summary
    let stats_y = status_y - 0.45;
    let stats_text = format!("{} gold | {} kills", player.gold, player.kills);
    ui_render::text(engine, &stats_text, px, stats_y, theme.dim, 0.2, 0.2);
}

fn xp_to_next_level(level: u32) -> u64 {
    // Simple XP curve
    (level as u64 * level as u64 * 50) + 100
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Enemy Info Panel
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_enemy_panel(
    hud: &CombatHudState,
    enemy: &chaos_rpg_core::enemy::Enemy,
    engine: &mut ProofEngine,
    theme: &Theme,
) {
    let ex = RIGHT_EDGE;
    let ey = 3.8;

    // Enemy name
    let display_name: String = enemy.name.chars().take(18).collect();
    ui_render::body(engine, &display_name, ex, ey, theme.danger);

    // Tier stars
    let tier_count = tier_star_count(&enemy.tier);
    let stars: String = "*".repeat(tier_count);
    let tier_label = format!("{} {}", enemy.tier.name(), stars);
    ui_render::small(engine, &tier_label, ex, ey - 0.45, theme.dim);

    // Element icon (based on name hashing for visual variety)
    let element_icon = enemy_element_icon(&enemy.name);
    let element_color = enemy_element_color(&enemy.name);
    engine.spawn_glyph(Glyph {
        character: element_icon,
        position: Vec3::new(ex + 4.2, ey, 0.0),
        scale: Vec2::splat(0.35),
        color: element_color,
        emission: 0.5,
        glow_color: Vec3::new(element_color.x, element_color.y, element_color.z),
        glow_radius: 0.3,
        layer: RenderLayer::UI,
        ..Default::default()
    });

    // HP bar
    let hp_y = ey - 0.9;
    ui_render::small(engine, "HP", ex, hp_y, theme.muted);
    combat_visuals::render_hp_bar(
        engine,
        ex + 0.8,
        hp_y,
        BAR_WIDTH,
        hud.display_enemy_hp,
        hud.ghost_enemy_hp,
        theme,
        BAR_SCALE,
    );
    let hp_text = format!("{}/{}", enemy.hp, enemy.max_hp);
    ui_render::small(engine, &hp_text, ex + BAR_WIDTH + 1.2, hp_y, theme.hp_color(hud.display_enemy_hp));

    // Special ability indicator
    if let Some(ability) = enemy.special_ability {
        let ability_y = hp_y - 0.5;
        let ability_text = format!("Ability: {}", ability);
        ui_render::text(engine, &ability_text, ex, ability_y, theme.warn, 0.2, 0.2);
    }
}

fn tier_star_count(tier: &EnemyTier) -> usize {
    match tier {
        EnemyTier::Minion => 1,
        EnemyTier::Elite => 2,
        EnemyTier::Champion => 3,
        EnemyTier::Boss => 4,
        EnemyTier::Abomination => 5,
    }
}

fn enemy_element_icon(name: &str) -> char {
    let hash: u64 = name.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    let elements = ['*', '#', '~', '^', '+', 'o'];
    elements[(hash % elements.len() as u64) as usize]
}

fn enemy_element_color(name: &str) -> Vec4 {
    let hash: u64 = name.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    let colors = [
        Vec4::new(1.0, 0.4, 0.0, 1.0),   // fire
        Vec4::new(0.3, 0.7, 1.0, 1.0),   // ice
        Vec4::new(0.2, 0.9, 0.2, 1.0),   // poison
        Vec4::new(1.0, 1.0, 0.3, 1.0),   // lightning
        Vec4::new(0.6, 0.2, 0.8, 1.0),   // dark
        Vec4::new(0.9, 0.9, 0.9, 1.0),   // neutral
    ];
    colors[(hash % colors.len() as u64) as usize]
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Status Effect Row
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn status_icon_data(effect: &StatusEffect) -> (char, Vec4, &'static str) {
    match effect {
        StatusEffect::Burning(n) => ('*', Vec4::new(1.0, 0.4, 0.0, 1.0), "Burning"),
        StatusEffect::Frozen(n) => ('#', Vec4::new(0.3, 0.7, 1.0, 1.0), "Frozen"),
        StatusEffect::Poisoned(n) => ('~', Vec4::new(0.2, 0.9, 0.1, 1.0), "Poisoned"),
        StatusEffect::Stunned(n) => ('!', Vec4::new(1.0, 1.0, 0.0, 1.0), "Stunned"),
        StatusEffect::Cursed(n) => ('x', Vec4::new(0.6, 0.0, 0.6, 1.0), "Cursed"),
        StatusEffect::Blessed(n) => ('+', Vec4::new(1.0, 0.95, 0.6, 1.0), "Blessed"),
        StatusEffect::Shielded(n) => ('O', Vec4::new(0.5, 0.8, 1.0, 1.0), "Shielded"),
        StatusEffect::Enraged(n) => ('!', Vec4::new(1.0, 0.1, 0.1, 1.0), "Enraged"),
        StatusEffect::Regenerating(n) => ('+', Vec4::new(0.2, 1.0, 0.4, 1.0), "Regen"),
        StatusEffect::Phasing(n) => ('~', Vec4::new(0.5, 0.3, 0.9, 1.0), "Phasing"),
        StatusEffect::Empowered(n) => ('^', Vec4::new(1.0, 0.8, 0.2, 1.0), "Empowered"),
        StatusEffect::Fracture(n) => ('/', Vec4::new(0.9, 0.1, 0.9, 1.0), "Fracture"),
        StatusEffect::Resonance(n) => ('=', Vec4::new(0.3, 0.9, 0.9, 1.0), "Resonance"),
        StatusEffect::PhaseLock(n) => ('#', Vec4::new(0.7, 0.7, 0.7, 1.0), "PhaseLock"),
        _ => ('.', Vec4::new(0.5, 0.5, 0.5, 1.0), "Effect"),
    }
}

fn render_status_row(
    engine: &mut ProofEngine,
    effects: &[StatusEffect],
    x: f32,
    y: f32,
    _theme: &Theme,
) {
    for (i, effect) in effects.iter().take(8).enumerate() {
        let (ch, color, _label) = status_icon_data(effect);
        let sx = x + i as f32 * 0.4;

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(sx, y, 0.0),
            scale: Vec2::splat(0.25),
            color,
            emission: 0.4,
            glow_color: Vec3::new(color.x, color.y, color.z),
            glow_radius: 0.2,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Quick Slots
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_quick_slots(hud: &CombatHudState, engine: &mut ProofEngine, theme: &Theme) {
    let base_x = 5.5;
    let y = QUICK_SLOT_Y;

    ui_render::text(engine, "Items:", base_x - 1.5, y + 0.4, theme.dim, 0.2, 0.2);

    for (i, slot) in hud.quick_slots.iter().enumerate() {
        let sx = base_x + i as f32 * 1.2;

        // Slot border
        engine.spawn_glyph(Glyph {
            character: '[',
            position: Vec3::new(sx - 0.2, y, 0.0),
            scale: Vec2::splat(0.3),
            color: theme.dim,
            emission: 0.1,
            layer: RenderLayer::UI,
            ..Default::default()
        });
        engine.spawn_glyph(Glyph {
            character: ']',
            position: Vec3::new(sx + 0.5, y, 0.0),
            scale: Vec2::splat(0.3),
            color: theme.dim,
            emission: 0.1,
            layer: RenderLayer::UI,
            ..Default::default()
        });

        if let Some(item) = slot {
            // Item icon
            engine.spawn_glyph(Glyph {
                character: item.icon,
                position: Vec3::new(sx + 0.15, y, 0.0),
                scale: Vec2::splat(0.3),
                color: theme.accent,
                emission: 0.3,
                layer: RenderLayer::UI,
                ..Default::default()
            });
            // Count
            if item.count > 1 {
                let count_text = format!("{}", item.count);
                ui_render::text(engine, &count_text, sx + 0.35, y - 0.2, theme.dim, 0.15, 0.1);
            }
        } else {
            // Empty slot
            engine.spawn_glyph(Glyph {
                character: '-',
                position: Vec3::new(sx + 0.15, y, 0.0),
                scale: Vec2::splat(0.25),
                color: theme.muted,
                emission: 0.05,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }

        // Keybind hint
        let key = format!("{}", i + 1);
        ui_render::text(engine, &key, sx + 0.1, y - 0.35, theme.muted, 0.15, 0.1);
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Combo Meter
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_combo_meter(
    hud: &CombatHudState,
    vis: &CombatVisualState,
    engine: &mut ProofEngine,
    theme: &Theme,
) {
    if vis.combo_count == 0 { return; }

    let x = COMBO_METER_X;
    let base_y = -2.0;
    let total_cells = 20;
    let cell_height = COMBO_METER_HEIGHT / total_cells as f32;

    // Label
    ui_render::text(engine, "COMBO", x - 0.6, base_y + COMBO_METER_HEIGHT + 0.3, theme.dim, 0.2, 0.2);

    let filled = (hud.combo_fill * total_cells as f32) as usize;

    for i in 0..total_cells {
        let y = base_y + i as f32 * cell_height;
        let is_filled = i < filled;

        // Milestone markers
        let is_milestone = match i {
            2 => true,   // 10 hits
            5 => true,   // 25 hits
            10 => true,  // 50 hits
            19 => true,  // 100 hits
            _ => false,
        };

        let ch = if is_filled { '\u{2588}' } else { '\u{2591}' };
        let color = if is_filled {
            if i > 15 {
                theme.gold
            } else if i > 10 {
                theme.warn
            } else if i > 5 {
                theme.accent
            } else {
                theme.primary
            }
        } else {
            theme.muted
        };

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x, y, 0.0),
            scale: Vec2::splat(0.25),
            color,
            emission: if is_filled { 0.3 } else { 0.05 },
            layer: RenderLayer::UI,
            ..Default::default()
        });

        // Milestone tick mark
        if is_milestone {
            engine.spawn_glyph(Glyph {
                character: '-',
                position: Vec3::new(x - 0.3, y, 0.0),
                scale: Vec2::splat(0.15),
                color: theme.dim,
                emission: 0.1,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Animated Border
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_animated_border(hud: &CombatHudState, engine: &mut ProofEngine, theme: &Theme) {
    let border_chars = ['.', '+', '.', '+', '.'];
    let phase = hud.border_phase;

    // Top border
    for i in 0..35 {
        let x = -8.5 + i as f32 * 0.5;
        let y = 5.3;
        let idx = ((i as f32 + phase * 3.0) as usize) % border_chars.len();
        let pulse = ((phase + i as f32 * 0.1).sin() * 0.5 + 0.5) * 0.15;

        engine.spawn_glyph(Glyph {
            character: border_chars[idx],
            position: Vec3::new(x, y, 0.0),
            scale: Vec2::splat(0.15),
            color: Vec4::new(theme.border.x * 0.4, theme.border.y * 0.4, theme.border.z * 0.4, 0.2 + pulse),
            emission: pulse,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }

    // Bottom border
    for i in 0..35 {
        let x = -8.5 + i as f32 * 0.5;
        let y = -5.3;
        let idx = ((i as f32 + phase * 2.0) as usize) % border_chars.len();
        let pulse = ((phase + i as f32 * 0.12).cos() * 0.5 + 0.5) * 0.15;

        engine.spawn_glyph(Glyph {
            character: border_chars[idx],
            position: Vec3::new(x, y, 0.0),
            scale: Vec2::splat(0.15),
            color: Vec4::new(theme.border.x * 0.4, theme.border.y * 0.4, theme.border.z * 0.4, 0.2 + pulse),
            emission: pulse,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Class icon helpers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn class_icon_char(class: CharacterClass) -> char {
    match class {
        CharacterClass::Berserker => '>',
        CharacterClass::Mage => '*',
        CharacterClass::Ranger => '>',
        CharacterClass::Thief => '~',
        CharacterClass::Necromancer => '#',
        CharacterClass::Alchemist => 'o',
        CharacterClass::Paladin => '+',
        CharacterClass::VoidWalker => '.',
        CharacterClass::Warlord => '!',
        CharacterClass::Trickster => '?',
        CharacterClass::Runesmith => 'R',
        CharacterClass::Chronomancer => '@',
    }
}

fn class_icon_color(class: CharacterClass) -> Vec4 {
    match class {
        CharacterClass::Berserker => Vec4::new(0.85, 0.2, 0.15, 1.0),
        CharacterClass::Mage => Vec4::new(0.4, 0.3, 0.95, 1.0),
        CharacterClass::Ranger => Vec4::new(0.3, 0.8, 0.2, 1.0),
        CharacterClass::Thief => Vec4::new(0.5, 0.5, 0.5, 1.0),
        CharacterClass::Necromancer => Vec4::new(0.3, 0.7, 0.3, 1.0),
        CharacterClass::Alchemist => Vec4::new(0.7, 0.5, 0.9, 1.0),
        CharacterClass::Paladin => Vec4::new(0.9, 0.85, 0.4, 1.0),
        CharacterClass::VoidWalker => Vec4::new(0.6, 0.2, 0.8, 1.0),
        CharacterClass::Warlord => Vec4::new(0.8, 0.3, 0.2, 1.0),
        CharacterClass::Trickster => Vec4::new(0.6, 0.6, 0.3, 1.0),
        CharacterClass::Runesmith => Vec4::new(0.5, 0.4, 0.8, 1.0),
        CharacterClass::Chronomancer => Vec4::new(0.3, 0.6, 0.9, 1.0),
    }
}
