//! Combat visual orchestration system.
//!
//! Manages all visual elements in combat: entity rendering, animations,
//! damage numbers, spell effects, status icons, and victory/death sequences.
//! Everything renders through `ui_render` and `engine.spawn_glyph`.

use proof_engine::prelude::*;
use crate::state::GameState;
use crate::theme::{Theme, THEMES};
use crate::ui_render;

use chaos_rpg_core::character::{CharacterClass, StatusEffect};
use chaos_rpg_core::enemy::EnemyTier;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Constants
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Player entity screen position (left side of arena).
const PLAYER_POS: Vec3 = Vec3::new(-5.0, 0.5, 0.0);
/// Enemy entity screen position (right side of arena).
const ENEMY_POS: Vec3 = Vec3::new(5.0, 0.5, 0.0);

/// Maximum damage numbers visible at once.
const MAX_DAMAGE_NUMBERS: usize = 12;
/// Maximum combat log entries shown.
const MAX_LOG_ENTRIES: usize = 5;
/// Maximum status icon orbit slots.
const MAX_STATUS_ICONS: usize = 8;
/// Combo counter display duration.
const COMBO_DISPLAY_TIME: f32 = 3.0;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Combat Visual State
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Tracks all visual elements active during combat.
pub struct CombatVisualState {
    // ── Damage numbers ──
    pub damage_numbers: Vec<DamageNumber>,

    // ── Animations ──
    pub active_anim: Option<CombatAnimation>,
    pub anim_timer: f32,

    // ── Combo system ──
    pub combo_count: u32,
    pub combo_timer: f32,

    // ── Turn indicator ──
    pub is_player_turn: bool,
    pub turn_pulse: f32,

    // ── Target selection ──
    pub target_cursor: usize,
    pub target_pulse: f32,

    // ── Action menu ──
    pub action_cursor: usize,
    pub menu_open: bool,

    // ── Entity recoil ──
    pub player_recoil: f32,
    pub enemy_recoil: f32,
    pub player_recoil_dir: f32,
    pub enemy_recoil_dir: f32,

    // ── Victory / death ──
    pub victory_timer: f32,
    pub death_timer: f32,
    pub death_scatter: Vec<ScatterGlyph>,

    // ── Effectiveness text ──
    pub effectiveness_timer: f32,
    pub effectiveness_text: String,

    // ── Boss phase transition ──
    pub boss_phase_timer: f32,
    pub boss_phase: u8,

    // ── Spell cast animation ──
    pub spell_cast_timer: f32,
    pub spell_element_color: Vec4,
    pub spell_rune_chars: Vec<char>,

    // ── Defend animation ──
    pub defend_timer: f32,
    pub defend_blocked: bool,

    // ── Item use animation ──
    pub item_use_timer: f32,
    pub item_char: char,

    // ── Global time accumulator for cyclic animations ──
    pub time: f32,
}

impl CombatVisualState {
    pub fn new() -> Self {
        Self {
            damage_numbers: Vec::new(),
            active_anim: None,
            anim_timer: 0.0,
            combo_count: 0,
            combo_timer: 0.0,
            is_player_turn: true,
            turn_pulse: 0.0,
            target_cursor: 0,
            target_pulse: 0.0,
            action_cursor: 0,
            menu_open: true,
            player_recoil: 0.0,
            enemy_recoil: 0.0,
            player_recoil_dir: 1.0,
            enemy_recoil_dir: -1.0,
            victory_timer: 0.0,
            death_timer: 0.0,
            death_scatter: Vec::new(),
            effectiveness_timer: 0.0,
            effectiveness_text: String::new(),
            boss_phase_timer: 0.0,
            boss_phase: 0,
            spell_cast_timer: 0.0,
            spell_element_color: Vec4::new(0.5, 0.5, 1.0, 1.0),
            spell_rune_chars: vec!['*', '+', 'x', '~', '^'],
            defend_timer: 0.0,
            defend_blocked: false,
            item_use_timer: 0.0,
            item_char: '+',
            time: 0.0,
        }
    }

    /// Reset for a new combat encounter.
    pub fn reset(&mut self) {
        self.damage_numbers.clear();
        self.active_anim = None;
        self.anim_timer = 0.0;
        self.combo_count = 0;
        self.combo_timer = 0.0;
        self.is_player_turn = true;
        self.target_cursor = 0;
        self.action_cursor = 0;
        self.menu_open = true;
        self.player_recoil = 0.0;
        self.enemy_recoil = 0.0;
        self.victory_timer = 0.0;
        self.death_timer = 0.0;
        self.death_scatter.clear();
        self.effectiveness_timer = 0.0;
        self.boss_phase_timer = 0.0;
        self.boss_phase = 0;
        self.spell_cast_timer = 0.0;
        self.defend_timer = 0.0;
        self.item_use_timer = 0.0;
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Sub-structures
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A floating damage number with physics.
pub struct DamageNumber {
    pub value: i64,
    pub position: Vec3,
    pub velocity: Vec3,
    pub age: f32,
    pub lifetime: f32,
    pub is_crit: bool,
    pub is_heal: bool,
    pub color: Vec4,
    pub scale: f32,
    pub shake_offset: f32,
}

impl DamageNumber {
    pub fn new(value: i64, origin: Vec3, is_crit: bool, is_heal: bool, color: Vec4) -> Self {
        let spread = ((value as f32).abs() * 0.01).clamp(0.2, 1.5);
        let vx = (pseudo_rand_f32(value as u64) - 0.5) * spread;
        let vy = 2.0 + pseudo_rand_f32(value as u64 ^ 0xDEAD) * 1.5;
        Self {
            value,
            position: origin,
            velocity: Vec3::new(vx, vy, 0.0),
            age: 0.0,
            lifetime: if is_crit { 1.8 } else { 1.2 },
            is_crit,
            is_heal,
            color,
            scale: if is_crit { 0.6 } else { 0.35 },
            shake_offset: 0.0,
        }
    }

    /// Tick physics: gravity + fade.
    pub fn update(&mut self, dt: f32) {
        self.age += dt;
        // Gravity
        self.velocity.y -= 4.5 * dt;
        self.position += self.velocity * dt;
        // Crit shake
        if self.is_crit && self.age < 0.5 {
            self.shake_offset = (self.age * 40.0).sin() * 0.08 * (1.0 - self.age * 2.0);
        } else {
            self.shake_offset = 0.0;
        }
    }

    pub fn alive(&self) -> bool {
        self.age < self.lifetime
    }

    pub fn alpha(&self) -> f32 {
        let fade_start = self.lifetime * 0.6;
        if self.age > fade_start {
            1.0 - ((self.age - fade_start) / (self.lifetime - fade_start))
        } else {
            1.0
        }
    }
}

/// A glyph scattering outward during a death animation.
pub struct ScatterGlyph {
    pub character: char,
    pub position: Vec3,
    pub velocity: Vec3,
    pub color: Vec4,
    pub age: f32,
    pub lifetime: f32,
    pub rotation_speed: f32,
}

impl ScatterGlyph {
    pub fn update(&mut self, dt: f32) {
        self.age += dt;
        self.velocity.y -= 3.0 * dt; // gravity
        self.position += self.velocity * dt;
    }

    pub fn alive(&self) -> bool {
        self.age < self.lifetime
    }

    pub fn alpha(&self) -> f32 {
        let fade = self.lifetime * 0.5;
        if self.age > fade {
            1.0 - ((self.age - fade) / (self.lifetime - fade))
        } else {
            1.0
        }
    }
}

/// Types of combat animations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CombatAnimation {
    MeleeAttack,
    HeavyAttack,
    SpellCast,
    Defend,
    ItemUse,
    EnemyAttack,
    Flee,
    Taunt,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Pseudo-random helper (deterministic, no external crate needed for visuals)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn pseudo_rand_f32(seed: u64) -> f32 {
    let v = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((v >> 33) as f32) / (u32::MAX as f32)
}

fn pseudo_rand_f32_idx(seed: u64, idx: usize) -> f32 {
    pseudo_rand_f32(seed ^ (idx as u64).wrapping_mul(2654435761))
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// UPDATE
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Tick all combat visual state. Called from `screens::combat::update`.
pub fn update(vis: &mut CombatVisualState, _state: &GameState, dt: f32) {
    vis.time += dt;

    // Tick damage numbers
    vis.damage_numbers.iter_mut().for_each(|d| d.update(dt));
    vis.damage_numbers.retain(|d| d.alive());
    if vis.damage_numbers.len() > MAX_DAMAGE_NUMBERS {
        vis.damage_numbers.drain(0..vis.damage_numbers.len() - MAX_DAMAGE_NUMBERS);
    }

    // Tick recoil decay
    vis.player_recoil *= (1.0 - 8.0 * dt).max(0.0);
    vis.enemy_recoil *= (1.0 - 8.0 * dt).max(0.0);

    // Tick animation timers
    if vis.anim_timer > 0.0 {
        vis.anim_timer = (vis.anim_timer - dt).max(0.0);
        if vis.anim_timer <= 0.0 {
            vis.active_anim = None;
        }
    }

    // Tick combo
    if vis.combo_timer > 0.0 {
        vis.combo_timer = (vis.combo_timer - dt).max(0.0);
        if vis.combo_timer <= 0.0 {
            vis.combo_count = 0;
        }
    }

    // Turn pulse
    vis.turn_pulse = (vis.time * 3.0).sin() * 0.5 + 0.5;

    // Target pulse
    vis.target_pulse = (vis.time * 4.0).sin() * 0.5 + 0.5;

    // Effectiveness text
    if vis.effectiveness_timer > 0.0 {
        vis.effectiveness_timer = (vis.effectiveness_timer - dt).max(0.0);
    }

    // Boss phase transition
    if vis.boss_phase_timer > 0.0 {
        vis.boss_phase_timer = (vis.boss_phase_timer - dt).max(0.0);
    }

    // Spell cast
    if vis.spell_cast_timer > 0.0 {
        vis.spell_cast_timer = (vis.spell_cast_timer - dt).max(0.0);
    }

    // Defend
    if vis.defend_timer > 0.0 {
        vis.defend_timer = (vis.defend_timer - dt).max(0.0);
    }

    // Item use
    if vis.item_use_timer > 0.0 {
        vis.item_use_timer = (vis.item_use_timer - dt).max(0.0);
    }

    // Victory
    if vis.victory_timer > 0.0 {
        vis.victory_timer += dt; // counts up
    }

    // Death scatter
    vis.death_scatter.iter_mut().for_each(|s| s.update(dt));
    vis.death_scatter.retain(|s| s.alive());

    // Death timer
    if vis.death_timer > 0.0 {
        vis.death_timer += dt;
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// EVENT TRIGGERS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Trigger a damage number floating up from a position.
pub fn trigger_damage(vis: &mut CombatVisualState, value: i64, at_player: bool, is_crit: bool, theme: &Theme) {
    let origin = if at_player { PLAYER_POS } else { ENEMY_POS };
    let color = if is_crit {
        theme.gold
    } else if value < 0 {
        // Healing
        theme.success
    } else {
        theme.danger
    };
    let is_heal = value < 0;
    let dn = DamageNumber::new(value.abs(), origin + Vec3::new(0.0, 1.5, 0.0), is_crit, is_heal, color);
    vis.damage_numbers.push(dn);

    // Recoil
    if at_player {
        vis.player_recoil = 0.4;
        vis.player_recoil_dir = -1.0;
    } else {
        vis.enemy_recoil = 0.4;
        vis.enemy_recoil_dir = 1.0;
    }

    // Combo
    if !at_player && !is_heal {
        vis.combo_count += 1;
        vis.combo_timer = COMBO_DISPLAY_TIME;
    }
}

/// Trigger a melee attack animation.
pub fn trigger_melee_attack(vis: &mut CombatVisualState) {
    vis.active_anim = Some(CombatAnimation::MeleeAttack);
    vis.anim_timer = 0.6;
}

/// Trigger a heavy attack animation.
pub fn trigger_heavy_attack(vis: &mut CombatVisualState) {
    vis.active_anim = Some(CombatAnimation::HeavyAttack);
    vis.anim_timer = 0.8;
}

/// Trigger a spell cast animation with element color.
pub fn trigger_spell_cast(vis: &mut CombatVisualState, element_color: Vec4) {
    vis.active_anim = Some(CombatAnimation::SpellCast);
    vis.anim_timer = 1.0;
    vis.spell_cast_timer = 1.0;
    vis.spell_element_color = element_color;
    vis.spell_rune_chars = vec!['*', '+', 'x', '~', '^', 'o', '@', '#'];
}

/// Trigger defend animation.
pub fn trigger_defend(vis: &mut CombatVisualState, blocked: bool) {
    vis.active_anim = Some(CombatAnimation::Defend);
    vis.anim_timer = 0.5;
    vis.defend_timer = 0.5;
    vis.defend_blocked = blocked;
}

/// Trigger item use animation.
pub fn trigger_item_use(vis: &mut CombatVisualState, item_char: char) {
    vis.active_anim = Some(CombatAnimation::ItemUse);
    vis.anim_timer = 0.7;
    vis.item_use_timer = 0.7;
    vis.item_char = item_char;
}

/// Trigger effectiveness popup.
pub fn trigger_effectiveness(vis: &mut CombatVisualState, text: &str) {
    vis.effectiveness_timer = 1.5;
    vis.effectiveness_text = text.to_string();
}

/// Trigger enemy death with scatter.
pub fn trigger_enemy_death(vis: &mut CombatVisualState, enemy_name: &str, tier: &EnemyTier) {
    let (chars, color) = enemy_visual_data(enemy_name, tier);
    let glyph_count = tier_glyph_count(tier);
    vis.death_scatter.clear();

    for i in 0..glyph_count {
        let angle = (i as f32 / glyph_count as f32) * std::f32::consts::TAU;
        let speed = 2.0 + pseudo_rand_f32_idx(i as u64, i) * 3.0;
        let ch = chars[i % chars.len()];
        vis.death_scatter.push(ScatterGlyph {
            character: ch,
            position: ENEMY_POS,
            velocity: Vec3::new(angle.cos() * speed, angle.sin() * speed + 1.5, 0.0),
            color,
            age: 0.0,
            lifetime: 1.5 + pseudo_rand_f32_idx(i as u64, i + 7) * 0.5,
            rotation_speed: (pseudo_rand_f32_idx(i as u64, i + 3) - 0.5) * 10.0,
        });
    }

    vis.victory_timer = 0.001; // start counting
}

/// Trigger player death.
pub fn trigger_player_death(vis: &mut CombatVisualState) {
    vis.death_timer = 0.001; // start counting
    vis.death_scatter.clear();
    let chars = vec!['#', '@', '%', '&', '*'];
    let color = Vec4::new(0.9, 0.2, 0.15, 1.0);
    for i in 0..10 {
        let angle = (i as f32 / 10.0) * std::f32::consts::TAU;
        let speed = 1.5 + pseudo_rand_f32_idx(i as u64, i) * 2.0;
        vis.death_scatter.push(ScatterGlyph {
            character: chars[i % chars.len()],
            position: PLAYER_POS,
            velocity: Vec3::new(angle.cos() * speed, angle.sin() * speed + 1.0, 0.0),
            color,
            age: 0.0,
            lifetime: 2.0,
            rotation_speed: 5.0,
        });
    }
}

/// Trigger boss phase transition.
pub fn trigger_boss_phase(vis: &mut CombatVisualState, new_phase: u8) {
    vis.boss_phase = new_phase;
    vis.boss_phase_timer = 1.5;
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Master
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Render all combat visuals. Called from `screens::combat::render`.
pub fn render(vis: &CombatVisualState, state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    // Background arena grid
    render_arena_floor(vis, engine, theme);

    // Player entity
    if let Some(ref player) = state.player {
        render_player_entity(vis, engine, player.class, state.display_player_hp, theme);
    }

    // Enemy entity
    if let Some(ref enemy) = state.enemy {
        render_enemy_entity(vis, engine, &enemy.name, &enemy.tier, state.display_enemy_hp, theme);
    }

    // Turn indicator
    render_turn_indicator(vis, engine, theme);

    // Active animations
    render_animations(vis, engine, theme);

    // Damage numbers
    render_damage_numbers(vis, engine);

    // Death scatter
    render_death_scatter(vis, engine);

    // Status effect icons on player
    if let Some(ref player) = state.player {
        render_status_icons(vis, engine, &player.status_effects, PLAYER_POS, theme);
    }

    // Effectiveness text
    render_effectiveness(vis, engine, theme);

    // Combo counter
    render_combo(vis, engine, theme);

    // Victory overlay
    if vis.victory_timer > 0.0 {
        render_victory_overlay(vis, state, engine, theme);
    }

    // Boss phase transition flash
    if vis.boss_phase_timer > 0.0 {
        render_boss_phase_transition(vis, engine, theme);
    }

    // Target selection highlight
    render_target_highlight(vis, engine, theme);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Arena Floor
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_arena_floor(vis: &CombatVisualState, engine: &mut ProofEngine, theme: &Theme) {
    let grid_chars = ['.', '+', '.', '+'];
    let cols = 30;
    let rows = 4;
    let start_y = -2.5;
    let start_x = -8.0;
    let sp = 0.55;

    for row in 0..rows {
        for col in 0..cols {
            let x = start_x + col as f32 * sp;
            let y = start_y - row as f32 * sp;
            let wave = ((x * 0.5 + vis.time * 0.3).sin() * 0.5 + 0.5) * 0.05;
            let fade = 0.06 + wave;
            let ch = grid_chars[(row + col) % grid_chars.len()];
            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(x, y, 0.0),
                scale: Vec2::splat(0.2),
                color: Vec4::new(
                    theme.muted.x * fade * 4.0,
                    theme.muted.y * fade * 4.0,
                    theme.muted.z * fade * 4.0,
                    fade,
                ),
                emission: fade * 0.3,
                layer: RenderLayer::Background,
                ..Default::default()
            });
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Player Entity
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Get class-specific formation and characters.
fn class_formation(class: CharacterClass) -> (Vec<(f32, f32)>, Vec<char>, Vec4) {
    match class {
        // Warrior/Berserker: shield formation
        CharacterClass::Berserker | CharacterClass::Warlord => {
            let positions = shield_formation(10);
            let chars = vec!['>', '<', '!', '#', '|', '|', '#', '!', '<', '>'];
            let color = Vec4::new(0.85, 0.2, 0.15, 1.0);
            (positions, chars, color)
        }
        // Mage: star/pentagram formation
        CharacterClass::Mage | CharacterClass::Chronomancer | CharacterClass::Runesmith => {
            let positions = star_formation(12);
            let chars = vec!['*', '+', 'x', '~', '^', 'o', '*', '+', 'x', '~', '^', 'o'];
            let color = Vec4::new(0.4, 0.3, 0.95, 1.0);
            (positions, chars, color)
        }
        // Rogue/Thief: crescent formation
        CharacterClass::Thief | CharacterClass::Trickster => {
            let positions = crescent_formation(8);
            let chars = vec!['.', '~', '-', '\'', '`', ':', '.', '~'];
            let color = Vec4::new(0.5, 0.5, 0.5, 1.0);
            (positions, chars, color)
        }
        // Cleric/Paladin: cross formation
        CharacterClass::Paladin => {
            let positions = cross_formation(10);
            let chars = vec!['+', '|', '-', '+', '|', '-', '+', '|', '-', '+'];
            let color = Vec4::new(0.9, 0.85, 0.4, 1.0);
            (positions, chars, color)
        }
        // Ranger: arrow formation
        CharacterClass::Ranger => {
            let positions = arrow_formation(9);
            let chars = vec!['/', '\\', '|', '>', '<', '/', '\\', '|', '>'];
            let color = Vec4::new(0.3, 0.8, 0.2, 1.0);
            (positions, chars, color)
        }
        // Necromancer: skull pattern
        CharacterClass::Necromancer => {
            let positions = circle_formation(10, 1.0);
            let chars = vec!['#', '+', 'x', '.', 'o', '#', '+', 'x', '.', 'o'];
            let color = Vec4::new(0.3, 0.7, 0.3, 1.0);
            (positions, chars, color)
        }
        // Alchemist: flask/bubble pattern
        CharacterClass::Alchemist => {
            let positions = bubble_formation(8);
            let chars = vec!['~', 'o', 'O', '.', '~', 'o', 'O', '.'];
            let color = Vec4::new(0.7, 0.5, 0.9, 1.0);
            (positions, chars, color)
        }
        // VoidWalker: scattered/ethereal
        CharacterClass::VoidWalker => {
            let positions = scatter_formation(10);
            let chars = vec![' ', '.', '~', ' ', '.', '~', ' ', '.', '~', ' '];
            let color = Vec4::new(0.6, 0.2, 0.8, 1.0);
            (positions, chars, color)
        }
        // Default fallback
        _ => {
            let positions = circle_formation(8, 0.8);
            let chars = vec!['o', '+', 'o', '+', 'o', '+', 'o', '+'];
            let color = Vec4::new(0.7, 0.7, 0.7, 1.0);
            (positions, chars, color)
        }
    }
}

fn render_player_entity(
    vis: &CombatVisualState,
    engine: &mut ProofEngine,
    class: CharacterClass,
    hp_pct: f32,
    theme: &Theme,
) {
    let (positions, chars, base_color) = class_formation(class);
    let recoil_offset = vis.player_recoil * vis.player_recoil_dir;

    // Breathing animation
    let breath = (vis.time * 1.5).sin() * 0.03;
    let hp_dim = 0.5 + hp_pct * 0.5; // dim when low hp

    for (i, &(dx, dy)) in positions.iter().enumerate() {
        let ch = chars[i % chars.len()];
        if ch == ' ' { continue; }

        // Per-glyph wobble
        let wobble_x = (vis.time * 2.0 + i as f32 * 0.7).sin() * 0.02;
        let wobble_y = (vis.time * 1.8 + i as f32 * 1.1).cos() * 0.02;

        let x = PLAYER_POS.x + dx + recoil_offset + wobble_x;
        let y = PLAYER_POS.y + dy + breath + wobble_y;

        let color = Vec4::new(
            base_color.x * hp_dim,
            base_color.y * hp_dim,
            base_color.z * hp_dim,
            base_color.w,
        );

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x, y, 0.0),
            scale: Vec2::splat(0.35),
            color,
            emission: 0.4 + vis.turn_pulse * 0.2 * if vis.is_player_turn { 1.0 } else { 0.0 },
            glow_color: Vec3::new(base_color.x, base_color.y, base_color.z),
            glow_radius: 0.5,
            layer: RenderLayer::Entity,
            ..Default::default()
        });
    }

    // Player name below entity
    let name_label = class.name();
    ui_render::small(engine, name_label, PLAYER_POS.x - 1.0, PLAYER_POS.y - 2.0, theme.primary);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Enemy Entity
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn tier_glyph_count(tier: &EnemyTier) -> usize {
    match tier {
        EnemyTier::Minion => 5,
        EnemyTier::Elite => 10,
        EnemyTier::Champion => 16,
        EnemyTier::Boss => 22,
        EnemyTier::Abomination => 30,
    }
}

fn enemy_visual_data(name: &str, tier: &EnemyTier) -> (Vec<char>, Vec4) {
    // Build chars from name + decorations
    let mut chars: Vec<char> = name.chars().take(8).collect();
    let decorations = match tier {
        EnemyTier::Minion => vec!['.', '~'],
        EnemyTier::Elite => vec!['#', '+', 'x'],
        EnemyTier::Champion => vec!['#', '@', '%', '&'],
        EnemyTier::Boss => vec!['#', '@', '%', '&', '*', '^'],
        EnemyTier::Abomination => vec!['#', '@', '%', '&', '*', '^', '!', '='],
    };
    chars.extend(decorations);

    // Color by name hash for element tint
    let hash: u64 = name.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    let r = 0.5 + pseudo_rand_f32(hash) * 0.5;
    let g = 0.2 + pseudo_rand_f32(hash ^ 0xFF) * 0.5;
    let b = 0.2 + pseudo_rand_f32(hash ^ 0xFFFF) * 0.5;
    let color = Vec4::new(r, g, b, 1.0);

    (chars, color)
}

fn render_enemy_entity(
    vis: &CombatVisualState,
    engine: &mut ProofEngine,
    name: &str,
    tier: &EnemyTier,
    hp_pct: f32,
    theme: &Theme,
) {
    let (chars, base_color) = enemy_visual_data(name, tier);
    let glyph_count = tier_glyph_count(tier);
    let recoil_offset = vis.enemy_recoil * vis.enemy_recoil_dir;

    // Formation: rings
    let rings = ((glyph_count as f32).sqrt() as usize).max(1);
    let mut placed = 0;

    for ring in 0..rings {
        let r = (ring + 1) as f32 * 0.4;
        let count_in_ring = ((ring + 1) * 4).min(glyph_count - placed);
        for i in 0..count_in_ring {
            if placed >= glyph_count { break; }
            let base_angle = (i as f32 / count_in_ring as f32) * std::f32::consts::TAU;
            // Idle wobble
            let wobble = (vis.time * 1.2 + placed as f32 * 0.9).sin() * 0.04;
            let angle = base_angle + wobble;

            let x = ENEMY_POS.x + angle.cos() * r + recoil_offset;
            let y = ENEMY_POS.y + angle.sin() * r;

            let hp_dim = 0.5 + hp_pct * 0.5;
            let ch = chars[placed % chars.len()];
            if ch == ' ' { placed += 1; continue; }

            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(x, y, 0.0),
                scale: Vec2::splat(0.3),
                color: Vec4::new(
                    base_color.x * hp_dim,
                    base_color.y * hp_dim,
                    base_color.z * hp_dim,
                    base_color.w,
                ),
                emission: 0.3 + (1.0 - hp_pct) * 0.3, // glow more when damaged
                glow_color: Vec3::new(base_color.x, base_color.y, base_color.z),
                glow_radius: 0.3,
                layer: RenderLayer::Entity,
                ..Default::default()
            });
            placed += 1;
        }
    }

    // Enemy name label
    let display_name: String = name.chars().take(16).collect();
    let name_x = ENEMY_POS.x - (display_name.len() as f32 * 0.12);
    ui_render::small(engine, &display_name, name_x, ENEMY_POS.y - 2.0, theme.danger);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — HP/MP Bars
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Render a detailed HP bar using block characters for smooth fill.
pub fn render_hp_bar(
    engine: &mut ProofEngine,
    x: f32,
    y: f32,
    width: f32,
    ratio: f32,
    ghost_ratio: f32,
    theme: &Theme,
    scale: f32,
) {
    let block_chars = [' ', '\u{258F}', '\u{258E}', '\u{258D}', '\u{258C}', '\u{258B}', '\u{258A}', '\u{2589}', '\u{2588}'];
    let sp = scale * 0.85;
    let total_cells = (width / sp) as usize;
    let fill_exact = ratio.clamp(0.0, 1.0) * total_cells as f32;
    let full_cells = fill_exact as usize;
    let partial = ((fill_exact - full_cells as f32) * 8.0) as usize;

    let ghost_exact = ghost_ratio.clamp(0.0, 1.0) * total_cells as f32;
    let ghost_cells = ghost_exact as usize;

    let fill_color = theme.hp_color(ratio);
    let ghost_color = Vec4::new(fill_color.x * 0.4, fill_color.y * 0.4, fill_color.z * 0.4, 0.6);

    for i in 0..total_cells {
        let cx = x + i as f32 * sp;
        let (ch, color, em) = if i < full_cells {
            ('\u{2588}', fill_color, 0.4)
        } else if i == full_cells && partial > 0 {
            (block_chars[partial], fill_color, 0.3)
        } else if i < ghost_cells {
            ('\u{2591}', ghost_color, 0.1)
        } else {
            ('\u{2591}', theme.muted, 0.05)
        };
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(cx, y, 0.0),
            scale: Vec2::splat(scale),
            color,
            emission: em,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}

/// Render an MP bar (always blue).
pub fn render_mp_bar(
    engine: &mut ProofEngine,
    x: f32,
    y: f32,
    width: f32,
    ratio: f32,
    theme: &Theme,
    scale: f32,
) {
    ui_render::bar(engine, x, y, width, ratio, theme.mana, theme.muted, scale);
}

/// Render an XP bar.
pub fn render_xp_bar(
    engine: &mut ProofEngine,
    x: f32,
    y: f32,
    width: f32,
    ratio: f32,
    theme: &Theme,
    scale: f32,
) {
    ui_render::bar(engine, x, y, width, ratio, theme.xp, theme.muted, scale);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Status Effect Icons
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn status_icon(effect: &StatusEffect) -> (char, Vec4) {
    match effect {
        StatusEffect::Burning(_) => ('*', Vec4::new(1.0, 0.4, 0.0, 1.0)),
        StatusEffect::Frozen(_) => ('#', Vec4::new(0.3, 0.7, 1.0, 1.0)),
        StatusEffect::Poisoned(_) => ('~', Vec4::new(0.2, 0.9, 0.1, 1.0)),
        StatusEffect::Stunned(_) => ('!', Vec4::new(1.0, 1.0, 0.0, 1.0)),
        StatusEffect::Cursed(_) => ('x', Vec4::new(0.6, 0.0, 0.6, 1.0)),
        StatusEffect::Blessed(_) => ('+', Vec4::new(1.0, 0.95, 0.6, 1.0)),
        StatusEffect::Shielded(_) => ('O', Vec4::new(0.5, 0.8, 1.0, 1.0)),
        StatusEffect::Enraged(_) => ('!', Vec4::new(1.0, 0.1, 0.1, 1.0)),
        StatusEffect::Regenerating(_) => ('+', Vec4::new(0.2, 1.0, 0.4, 1.0)),
        StatusEffect::Phasing(_) => ('~', Vec4::new(0.5, 0.3, 0.9, 1.0)),
        StatusEffect::Empowered(_) => ('^', Vec4::new(1.0, 0.8, 0.2, 1.0)),
        StatusEffect::Fracture(_) => ('/', Vec4::new(0.9, 0.1, 0.9, 1.0)),
        StatusEffect::Resonance(_) => ('=', Vec4::new(0.3, 0.9, 0.9, 1.0)),
        StatusEffect::PhaseLock(_) => ('#', Vec4::new(0.7, 0.7, 0.7, 1.0)),
        _ => ('.', Vec4::new(0.5, 0.5, 0.5, 1.0)),
    }
}

fn render_status_icons(
    vis: &CombatVisualState,
    engine: &mut ProofEngine,
    effects: &[StatusEffect],
    center: Vec3,
    _theme: &Theme,
) {
    let orbit_radius = 1.8;
    let count = effects.len().min(MAX_STATUS_ICONS);

    for (i, effect) in effects.iter().take(count).enumerate() {
        let (ch, color) = status_icon(effect);
        let angle = (i as f32 / count.max(1) as f32) * std::f32::consts::TAU + vis.time * 0.5;
        let x = center.x + angle.cos() * orbit_radius;
        let y = center.y + 1.5 + angle.sin() * orbit_radius * 0.3; // flattened ellipse

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x, y, 0.0),
            scale: Vec2::splat(0.25),
            color,
            emission: 0.6,
            glow_color: Vec3::new(color.x, color.y, color.z),
            glow_radius: 0.3,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Damage Numbers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_damage_numbers(vis: &CombatVisualState, engine: &mut ProofEngine) {
    for dn in &vis.damage_numbers {
        let alpha = dn.alpha();
        let text = if dn.is_heal {
            format!("+{}", dn.value)
        } else {
            format!("{}", dn.value)
        };

        let color = Vec4::new(dn.color.x, dn.color.y, dn.color.z, alpha);
        let scale = dn.scale;

        ui_render::text(
            engine,
            &text,
            dn.position.x + dn.shake_offset,
            dn.position.y,
            color,
            scale,
            if dn.is_crit { 0.9 } else { 0.5 },
        );

        // Crit marker
        if dn.is_crit && dn.age < 0.5 {
            ui_render::text(
                engine,
                "CRIT!",
                dn.position.x - 0.4,
                dn.position.y + 0.4,
                Vec4::new(1.0, 0.85, 0.0, alpha * 0.8),
                0.2,
                0.8,
            );
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Combat Animations
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_animations(vis: &CombatVisualState, engine: &mut ProofEngine, theme: &Theme) {
    match vis.active_anim {
        Some(CombatAnimation::MeleeAttack) => render_melee_attack(vis, engine, theme),
        Some(CombatAnimation::HeavyAttack) => render_heavy_attack(vis, engine, theme),
        Some(CombatAnimation::SpellCast) => render_spell_cast(vis, engine),
        Some(CombatAnimation::Defend) => render_defend_anim(vis, engine, theme),
        Some(CombatAnimation::ItemUse) => render_item_use_anim(vis, engine, theme),
        Some(CombatAnimation::EnemyAttack) => render_enemy_attack_anim(vis, engine, theme),
        Some(CombatAnimation::Taunt) => render_taunt_anim(vis, engine, theme),
        _ => {}
    }
}

fn render_melee_attack(vis: &CombatVisualState, engine: &mut ProofEngine, theme: &Theme) {
    let progress = 1.0 - (vis.anim_timer / 0.6).clamp(0.0, 1.0);
    // Weapon glyph swings in arc from player to enemy
    let arc_t = progress;
    let start = PLAYER_POS;
    let end = ENEMY_POS;

    // Arc path: lerp x, parabolic y
    let x = start.x + (end.x - start.x) * arc_t;
    let y = start.y + (end.y - start.y) * arc_t + (arc_t * (1.0 - arc_t)) * 3.0;

    let weapon_char = '/';
    let rotation = arc_t * std::f32::consts::PI;

    engine.spawn_glyph(Glyph {
        character: weapon_char,
        position: Vec3::new(x, y, 0.0),
        scale: Vec2::splat(0.5),
        rotation,
        color: theme.primary,
        emission: 0.8,
        glow_color: Vec3::new(theme.primary.x, theme.primary.y, theme.primary.z),
        glow_radius: 0.8,
        layer: RenderLayer::Particle,
        ..Default::default()
    });

    // Trail: afterimages
    for trail in 1..5 {
        let trail_t = (arc_t - trail as f32 * 0.05).max(0.0);
        let tx = start.x + (end.x - start.x) * trail_t;
        let ty = start.y + (end.y - start.y) * trail_t + (trail_t * (1.0 - trail_t)) * 3.0;
        let fade = 1.0 - trail as f32 * 0.25;

        engine.spawn_glyph(Glyph {
            character: weapon_char,
            position: Vec3::new(tx, ty, 0.0),
            scale: Vec2::splat(0.4),
            rotation: trail_t * std::f32::consts::PI,
            color: Vec4::new(theme.primary.x, theme.primary.y, theme.primary.z, fade * 0.5),
            emission: 0.3 * fade,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
}

fn render_heavy_attack(vis: &CombatVisualState, engine: &mut ProofEngine, theme: &Theme) {
    let progress = 1.0 - (vis.anim_timer / 0.8).clamp(0.0, 1.0);
    let start = PLAYER_POS;
    let end = ENEMY_POS;

    // Heavy attack: slower, bigger weapon, ground slam feel
    let wind_up = if progress < 0.4 { progress / 0.4 } else { 1.0 };
    let swing = if progress >= 0.4 { (progress - 0.4) / 0.6 } else { 0.0 };

    let x = start.x + (end.x - start.x) * swing;
    let y = start.y + (end.y - start.y) * swing + wind_up * 2.0 * (1.0 - swing);

    let weapon_char = '#';
    engine.spawn_glyph(Glyph {
        character: weapon_char,
        position: Vec3::new(x, y, 0.0),
        scale: Vec2::splat(0.7),
        rotation: swing * std::f32::consts::TAU,
        color: theme.danger,
        emission: 1.0,
        glow_color: Vec3::new(theme.danger.x, theme.danger.y, theme.danger.z),
        glow_radius: 1.2,
        layer: RenderLayer::Particle,
        ..Default::default()
    });

    // Impact particles at end
    if swing > 0.8 {
        let burst = (swing - 0.8) / 0.2;
        for i in 0..8 {
            let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
            let dist = burst * 1.5;
            let px = end.x + angle.cos() * dist;
            let py = end.y + angle.sin() * dist;
            engine.spawn_glyph(Glyph {
                character: '*',
                position: Vec3::new(px, py, 0.0),
                scale: Vec2::splat(0.25),
                color: Vec4::new(1.0, 0.8, 0.2, 1.0 - burst),
                emission: 0.6,
                layer: RenderLayer::Particle,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }
    }
}

fn render_spell_cast(vis: &CombatVisualState, engine: &mut ProofEngine) {
    let progress = 1.0 - (vis.spell_cast_timer / 1.0).clamp(0.0, 1.0);
    let color = vis.spell_element_color;

    // Phase 1: rune circle growing around caster (0.0 - 0.5)
    if progress < 0.6 {
        let circle_t = progress / 0.6;
        let radius = 0.5 + circle_t * 1.5;
        let rune_count = vis.spell_rune_chars.len();

        for (i, &ch) in vis.spell_rune_chars.iter().enumerate() {
            let angle = (i as f32 / rune_count as f32) * std::f32::consts::TAU + vis.time * 2.0;
            let x = PLAYER_POS.x + angle.cos() * radius;
            let y = PLAYER_POS.y + angle.sin() * radius;

            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(x, y, 0.0),
                scale: Vec2::splat(0.3),
                color: Vec4::new(color.x, color.y, color.z, circle_t),
                emission: 0.8,
                glow_color: Vec3::new(color.x, color.y, color.z),
                glow_radius: 0.5,
                layer: RenderLayer::Particle,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }
    }

    // Phase 2: projectile travels to enemy (0.5 - 1.0)
    if progress >= 0.4 {
        let proj_t = ((progress - 0.4) / 0.6).clamp(0.0, 1.0);
        let x = PLAYER_POS.x + (ENEMY_POS.x - PLAYER_POS.x) * proj_t;
        let y = PLAYER_POS.y + (ENEMY_POS.y - PLAYER_POS.y) * proj_t;

        engine.spawn_glyph(Glyph {
            character: '*',
            position: Vec3::new(x, y, 0.0),
            scale: Vec2::splat(0.5),
            color,
            emission: 1.2,
            glow_color: Vec3::new(color.x, color.y, color.z),
            glow_radius: 1.0,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });

        // Sparkle trail
        for trail in 1..4 {
            let tt = (proj_t - trail as f32 * 0.08).max(0.0);
            let tx = PLAYER_POS.x + (ENEMY_POS.x - PLAYER_POS.x) * tt;
            let ty = PLAYER_POS.y + (ENEMY_POS.y - PLAYER_POS.y) * tt;
            let fade = 1.0 - trail as f32 * 0.3;
            engine.spawn_glyph(Glyph {
                character: '.',
                position: Vec3::new(tx, ty, 0.0),
                scale: Vec2::splat(0.3),
                color: Vec4::new(color.x, color.y, color.z, fade * 0.6),
                emission: 0.4,
                layer: RenderLayer::Particle,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }
    }
}

fn render_defend_anim(vis: &CombatVisualState, engine: &mut ProofEngine, theme: &Theme) {
    let progress = 1.0 - (vis.defend_timer / 0.5).clamp(0.0, 1.0);
    let shield_scale = 0.3 + progress * 0.4;
    let glow = if vis.defend_blocked { 1.0 } else { 0.5 };

    // Shield glyph in front of player
    let shield_chars = ['[', '|', ']'];
    for (i, &ch) in shield_chars.iter().enumerate() {
        let x = PLAYER_POS.x + 1.0 + (i as f32 - 1.0) * 0.4;
        let y = PLAYER_POS.y;

        let color = if vis.defend_blocked {
            Vec4::new(0.3, 0.8, 1.0, 1.0 - progress * 0.5)
        } else {
            Vec4::new(theme.muted.x, theme.muted.y, theme.muted.z, 0.8 - progress * 0.3)
        };

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x, y, 0.0),
            scale: Vec2::splat(shield_scale),
            color,
            emission: glow * (1.0 - progress),
            glow_color: Vec3::new(0.3, 0.7, 1.0),
            glow_radius: if vis.defend_blocked { 1.0 } else { 0.3 },
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

fn render_item_use_anim(vis: &CombatVisualState, engine: &mut ProofEngine, theme: &Theme) {
    let progress = 1.0 - (vis.item_use_timer / 0.7).clamp(0.0, 1.0);

    // Item glyph floats from bottom-right to player
    let start = Vec3::new(6.0, -4.0, 0.0);
    let end = PLAYER_POS;
    let x = start.x + (end.x - start.x) * progress;
    let y = start.y + (end.y - start.y) * progress + (progress * (1.0 - progress)) * 2.0;

    engine.spawn_glyph(Glyph {
        character: vis.item_char,
        position: Vec3::new(x, y, 0.0),
        scale: Vec2::splat(0.4),
        color: theme.success,
        emission: 0.7,
        glow_color: Vec3::new(0.2, 1.0, 0.4),
        glow_radius: 0.6,
        layer: RenderLayer::Particle,
        ..Default::default()
    });

    // Sparkle trail
    for i in 0..3 {
        let tt = (progress - i as f32 * 0.1).max(0.0);
        let sx = start.x + (end.x - start.x) * tt;
        let sy = start.y + (end.y - start.y) * tt + (tt * (1.0 - tt)) * 2.0;
        engine.spawn_glyph(Glyph {
            character: '.',
            position: Vec3::new(sx, sy, 0.0),
            scale: Vec2::splat(0.2),
            color: Vec4::new(0.4, 1.0, 0.6, 0.5 - i as f32 * 0.15),
            emission: 0.3,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
}

fn render_enemy_attack_anim(vis: &CombatVisualState, engine: &mut ProofEngine, theme: &Theme) {
    let progress = 1.0 - (vis.anim_timer / 0.6).clamp(0.0, 1.0);
    let start = ENEMY_POS;
    let end = PLAYER_POS;

    let x = start.x + (end.x - start.x) * progress;
    let y = start.y + (end.y - start.y) * progress;

    engine.spawn_glyph(Glyph {
        character: 'X',
        position: Vec3::new(x, y, 0.0),
        scale: Vec2::splat(0.45),
        color: theme.danger,
        emission: 0.7,
        glow_color: Vec3::new(theme.danger.x, theme.danger.y, theme.danger.z),
        glow_radius: 0.6,
        layer: RenderLayer::Particle,
        ..Default::default()
    });
}

fn render_taunt_anim(vis: &CombatVisualState, engine: &mut ProofEngine, theme: &Theme) {
    let progress = 1.0 - (vis.anim_timer / 0.6).clamp(0.0, 1.0);
    // Exclamation marks burst out from player
    let count = 6;
    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        let dist = progress * 2.0;
        let x = PLAYER_POS.x + angle.cos() * dist;
        let y = PLAYER_POS.y + angle.sin() * dist;

        engine.spawn_glyph(Glyph {
            character: '!',
            position: Vec3::new(x, y, 0.0),
            scale: Vec2::splat(0.35),
            color: Vec4::new(theme.warn.x, theme.warn.y, theme.warn.z, 1.0 - progress),
            emission: 0.6 * (1.0 - progress),
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Turn Indicator
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_turn_indicator(vis: &CombatVisualState, engine: &mut ProofEngine, theme: &Theme) {
    let target_pos = if vis.is_player_turn { PLAYER_POS } else { ENEMY_POS };
    let y = target_pos.y + 2.5;
    let x = target_pos.x;
    let pulse = vis.turn_pulse;

    // Arrow pointing down
    let arrow_chars = ['V', '|'];
    for (i, &ch) in arrow_chars.iter().enumerate() {
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x, y + i as f32 * 0.4, 0.0),
            scale: Vec2::splat(0.3),
            color: Vec4::new(theme.accent.x, theme.accent.y, theme.accent.z, 0.6 + pulse * 0.4),
            emission: 0.5 + pulse * 0.3,
            glow_color: Vec3::new(theme.accent.x, theme.accent.y, theme.accent.z),
            glow_radius: 0.4,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Target Selection Highlight
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_target_highlight(vis: &CombatVisualState, engine: &mut ProofEngine, theme: &Theme) {
    if !vis.is_player_turn { return; }

    // Pulsing bracket highlight around enemy
    let pulse = vis.target_pulse;
    let x = ENEMY_POS.x;
    let y = ENEMY_POS.y;

    let bracket_offset = 1.8 + pulse * 0.2;
    let alpha = 0.4 + pulse * 0.3;

    // Left bracket
    engine.spawn_glyph(Glyph {
        character: '[',
        position: Vec3::new(x - bracket_offset, y, 0.0),
        scale: Vec2::splat(0.4),
        color: Vec4::new(theme.accent.x, theme.accent.y, theme.accent.z, alpha),
        emission: 0.3 + pulse * 0.2,
        layer: RenderLayer::UI,
        ..Default::default()
    });

    // Right bracket
    engine.spawn_glyph(Glyph {
        character: ']',
        position: Vec3::new(x + bracket_offset, y, 0.0),
        scale: Vec2::splat(0.4),
        color: Vec4::new(theme.accent.x, theme.accent.y, theme.accent.z, alpha),
        emission: 0.3 + pulse * 0.2,
        layer: RenderLayer::UI,
        ..Default::default()
    });
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Death Scatter
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_death_scatter(vis: &CombatVisualState, engine: &mut ProofEngine) {
    for sg in &vis.death_scatter {
        let alpha = sg.alpha();
        engine.spawn_glyph(Glyph {
            character: sg.character,
            position: sg.position,
            scale: Vec2::splat(0.3),
            rotation: sg.age * sg.rotation_speed,
            color: Vec4::new(sg.color.x, sg.color.y, sg.color.z, alpha),
            emission: 0.5 * alpha,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Effectiveness Text
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_effectiveness(vis: &CombatVisualState, engine: &mut ProofEngine, theme: &Theme) {
    if vis.effectiveness_timer <= 0.0 { return; }
    let alpha = (vis.effectiveness_timer / 1.5).clamp(0.0, 1.0);
    let y = 2.0 + (1.5 - vis.effectiveness_timer) * 0.5;

    let color = Vec4::new(theme.gold.x, theme.gold.y, theme.gold.z, alpha);
    ui_render::text_centered(engine, &vis.effectiveness_text, y, color, 0.5, 0.8);

    // Exclamation particles
    let count = 4;
    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU + vis.time * 3.0;
        let dist = 1.0 + (1.5 - vis.effectiveness_timer) * 0.5;
        let px = angle.cos() * dist;
        let py = y + angle.sin() * dist * 0.3;

        engine.spawn_glyph(Glyph {
            character: '!',
            position: Vec3::new(px, py, 0.0),
            scale: Vec2::splat(0.2),
            color: Vec4::new(1.0, 0.9, 0.2, alpha * 0.7),
            emission: 0.4,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Combo Counter
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_combo(vis: &CombatVisualState, engine: &mut ProofEngine, theme: &Theme) {
    if vis.combo_count < 2 { return; }

    let x = 7.0;
    let y = 4.0;

    // Scale grows at milestones
    let base_scale = 0.5;
    let milestone_bonus = if vis.combo_count >= 100 {
        0.4
    } else if vis.combo_count >= 50 {
        0.3
    } else if vis.combo_count >= 25 {
        0.2
    } else if vis.combo_count >= 10 {
        0.1
    } else {
        0.0
    };
    let scale = base_scale + milestone_bonus;

    // Shake at milestones
    let shake = if vis.combo_timer > COMBO_DISPLAY_TIME - 0.3 && milestone_bonus > 0.0 {
        (vis.time * 30.0).sin() * 0.1
    } else {
        0.0
    };

    let text = format!("{}x", vis.combo_count);
    let color = if vis.combo_count >= 25 {
        theme.gold
    } else if vis.combo_count >= 10 {
        theme.warn
    } else {
        theme.accent
    };

    ui_render::text(engine, &text, x + shake, y, color, scale, 0.7);
    ui_render::small(engine, "COMBO", x - 0.3, y - 0.5, theme.dim);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Victory Overlay
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_victory_overlay(vis: &CombatVisualState, state: &GameState, engine: &mut ProofEngine, theme: &Theme) {
    let t = vis.victory_timer;
    if t <= 0.0 { return; }

    let alpha = (t / 0.5).clamp(0.0, 1.0);

    // "VICTORY" text fades in
    if t > 0.3 {
        let victory_alpha = ((t - 0.3) / 0.5).clamp(0.0, 1.0);
        let color = Vec4::new(theme.gold.x, theme.gold.y, theme.gold.z, victory_alpha);
        ui_render::text_centered(engine, "VICTORY!", 2.0, color, 0.8, 1.0);
    }

    // XP/Gold float up
    if t > 0.8 {
        if let Some(ref enemy) = state.enemy {
            let reward_alpha = ((t - 0.8) / 0.5).clamp(0.0, 1.0);
            let float_y = 0.5 + (t - 0.8) * 0.3;

            let xp_text = format!("+{} XP", enemy.xp_reward);
            let gold_text = format!("+{} Gold", enemy.gold_reward);

            ui_render::text_centered(
                engine,
                &xp_text,
                float_y,
                Vec4::new(theme.xp.x, theme.xp.y, theme.xp.z, reward_alpha),
                0.4,
                0.6,
            );
            ui_render::text_centered(
                engine,
                &gold_text,
                float_y - 0.6,
                Vec4::new(theme.gold.x, theme.gold.y, theme.gold.z, reward_alpha),
                0.4,
                0.6,
            );
        }
    }

    // Particle burst
    if t > 0.2 && t < 2.0 {
        let burst_t = (t - 0.2) / 1.8;
        for i in 0..12 {
            let angle = (i as f32 / 12.0) * std::f32::consts::TAU + t * 0.5;
            let dist = burst_t * 4.0;
            let px = ENEMY_POS.x + angle.cos() * dist;
            let py = ENEMY_POS.y + angle.sin() * dist;
            let particle_alpha = alpha * (1.0 - burst_t);

            engine.spawn_glyph(Glyph {
                character: '*',
                position: Vec3::new(px, py, 0.0),
                scale: Vec2::splat(0.2),
                color: Vec4::new(theme.gold.x, theme.gold.y, theme.gold.z, particle_alpha),
                emission: 0.5 * particle_alpha,
                layer: RenderLayer::Particle,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Boss Phase Transition
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn render_boss_phase_transition(vis: &CombatVisualState, engine: &mut ProofEngine, theme: &Theme) {
    let t = vis.boss_phase_timer;
    if t <= 0.0 { return; }

    let flash_alpha = if t > 1.0 { (t - 1.0) / 0.5 } else { 0.0 };

    // Screen flash overlay
    if flash_alpha > 0.0 {
        for ix in -15..15 {
            for iy in -8..8 {
                engine.spawn_glyph(Glyph {
                    character: ' ',
                    position: Vec3::new(ix as f32 * 0.6, iy as f32 * 0.6, 0.0),
                    scale: Vec2::splat(0.6),
                    color: Vec4::new(1.0, 1.0, 1.0, flash_alpha * 0.3),
                    emission: flash_alpha,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }
    }

    // Phase text
    let phase_text = format!("PHASE {}", vis.boss_phase);
    let text_alpha = (1.0 - t / 1.5).clamp(0.0, 1.0);
    let color = Vec4::new(theme.danger.x, theme.danger.y, theme.danger.z, text_alpha);
    ui_render::text_centered(engine, &phase_text, 0.0, color, 0.7, 0.9);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Combat Log
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Render the scrolling combat log at the bottom of the screen.
pub fn render_combat_log(
    engine: &mut ProofEngine,
    log: &[String],
    collapsed: bool,
    theme: &Theme,
) {
    if collapsed { return; }

    let start = log.len().saturating_sub(MAX_LOG_ENTRIES);
    let base_y = -4.3;
    let line_height = 0.4;

    // Background tint for readability
    for i in 0..MAX_LOG_ENTRIES {
        let y = base_y - i as f32 * line_height;
        for col in 0..40 {
            let x = -8.0 + col as f32 * 0.25;
            engine.spawn_glyph(Glyph {
                character: ' ',
                position: Vec3::new(x, y, 0.0),
                scale: Vec2::splat(0.25),
                color: Vec4::new(0.0, 0.0, 0.0, 0.3),
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }
    }

    for (i, msg) in log[start..].iter().enumerate() {
        let y = base_y - i as f32 * line_height;
        let truncated: String = msg.chars().take(55).collect();

        // Color-code by content
        let color = if msg.contains("damage") || msg.contains("hit") {
            theme.danger
        } else if msg.contains("heal") || msg.contains("regen") {
            theme.success
        } else if msg.contains("defend") || msg.contains("block") {
            theme.mana
        } else if msg.contains("cast") || msg.contains("spell") {
            theme.accent
        } else {
            theme.dim
        };

        // Fade older entries
        let age_fade = 1.0 - (i as f32 / MAX_LOG_ENTRIES as f32) * 0.3;
        let faded = Vec4::new(color.x * age_fade, color.y * age_fade, color.z * age_fade, age_fade);

        ui_render::small(engine, &truncated, -8.0, y, faded);
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Action Menu
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Action labels and keybinds.
const ACTIONS: [(&str, &str, char); 5] = [
    ("Attack", "A", '/'),
    ("Heavy",  "H", '#'),
    ("Defend", "D", 'O'),
    ("Flee",   "F", '>'),
    ("Taunt",  "T", '!'),
];

/// Render action menu as floating glyph cards.
pub fn render_action_menu(
    vis: &CombatVisualState,
    engine: &mut ProofEngine,
    theme: &Theme,
) {
    let base_x = -7.5;
    let base_y = -3.2;
    let card_width = 2.8;

    for (i, &(label, key, icon)) in ACTIONS.iter().enumerate() {
        let x = base_x + i as f32 * card_width;
        let y = base_y;
        let is_selected = i == vis.action_cursor;

        // Card background (bracket chars)
        let bg_alpha = if is_selected { 0.3 } else { 0.1 };
        let border_color = if is_selected { theme.selected } else { theme.dim };

        // Left bracket
        engine.spawn_glyph(Glyph {
            character: '[',
            position: Vec3::new(x - 0.3, y, 0.0),
            scale: Vec2::splat(0.35),
            color: Vec4::new(border_color.x, border_color.y, border_color.z, bg_alpha + 0.3),
            emission: if is_selected { 0.4 } else { 0.1 },
            layer: RenderLayer::UI,
            ..Default::default()
        });

        // Right bracket
        engine.spawn_glyph(Glyph {
            character: ']',
            position: Vec3::new(x + 1.6, y, 0.0),
            scale: Vec2::splat(0.35),
            color: Vec4::new(border_color.x, border_color.y, border_color.z, bg_alpha + 0.3),
            emission: if is_selected { 0.4 } else { 0.1 },
            layer: RenderLayer::UI,
            ..Default::default()
        });

        // Icon glyph
        let icon_color = if is_selected { theme.accent } else { theme.primary };
        engine.spawn_glyph(Glyph {
            character: icon,
            position: Vec3::new(x + 0.1, y, 0.0),
            scale: Vec2::splat(0.35),
            color: icon_color,
            emission: if is_selected { 0.6 } else { 0.3 },
            layer: RenderLayer::UI,
            ..Default::default()
        });

        // Label text
        let label_color = if is_selected { theme.selected } else { theme.primary };
        ui_render::small(engine, label, x + 0.5, y, label_color);

        // Keybind hint
        let key_text = format!("[{}]", key);
        ui_render::text(engine, &key_text, x + 0.3, y - 0.35, theme.dim, 0.2, 0.2);
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Formation generators
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn shield_formation(count: usize) -> Vec<(f32, f32)> {
    let mut positions = Vec::with_capacity(count);
    // Vertical line with slight curve
    for i in 0..count {
        let t = i as f32 / (count - 1).max(1) as f32;
        let y = (t - 0.5) * 2.0;
        let x = -(y * y) * 0.3; // concave shield
        positions.push((x, y));
    }
    positions
}

fn star_formation(count: usize) -> Vec<(f32, f32)> {
    let mut positions = Vec::with_capacity(count);
    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        let r = if i % 2 == 0 { 1.0 } else { 0.5 };
        positions.push((angle.cos() * r, angle.sin() * r));
    }
    positions
}

fn crescent_formation(count: usize) -> Vec<(f32, f32)> {
    let mut positions = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / count as f32;
        let angle = (t - 0.5) * std::f32::consts::PI * 1.2;
        let r = 0.8;
        positions.push((angle.cos() * r - 0.3, angle.sin() * r));
    }
    positions
}

fn cross_formation(count: usize) -> Vec<(f32, f32)> {
    let mut positions = Vec::with_capacity(count);
    let arm = count / 4;
    // Vertical
    for i in 0..arm {
        let t = (i as f32 / arm as f32 - 0.5) * 2.0;
        positions.push((0.0, t));
    }
    // Horizontal
    for i in 0..arm {
        let t = (i as f32 / arm as f32 - 0.5) * 2.0;
        positions.push((t, 0.0));
    }
    // Fill remaining with center cluster
    while positions.len() < count {
        let idx = positions.len();
        let r = 0.2;
        let angle = (idx as f32) * 2.4;
        positions.push((angle.cos() * r, angle.sin() * r));
    }
    positions
}

fn arrow_formation(count: usize) -> Vec<(f32, f32)> {
    let mut positions = Vec::with_capacity(count);
    // V-shape pointing right
    let half = count / 2;
    for i in 0..half {
        let t = i as f32 / half as f32;
        positions.push((t * 1.0, t * 0.6));
        positions.push((t * 1.0, -t * 0.6));
    }
    // Tip
    if positions.len() < count {
        positions.push((1.2, 0.0));
    }
    positions.truncate(count);
    positions
}

fn circle_formation(count: usize, radius: f32) -> Vec<(f32, f32)> {
    let mut positions = Vec::with_capacity(count);
    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        positions.push((angle.cos() * radius, angle.sin() * radius));
    }
    positions
}

fn bubble_formation(count: usize) -> Vec<(f32, f32)> {
    let mut positions = Vec::with_capacity(count);
    // Scattered bubbles
    for i in 0..count {
        let seed = (i as u64).wrapping_mul(2654435761);
        let x = (pseudo_rand_f32(seed) - 0.5) * 2.0;
        let y = (pseudo_rand_f32(seed ^ 0xBEEF) - 0.5) * 2.0;
        positions.push((x, y));
    }
    positions
}

fn scatter_formation(count: usize) -> Vec<(f32, f32)> {
    let mut positions = Vec::with_capacity(count);
    for i in 0..count {
        let seed = (i as u64).wrapping_mul(6364136223846793005);
        let x = (pseudo_rand_f32(seed) - 0.5) * 2.5;
        let y = (pseudo_rand_f32(seed ^ 0xCAFE) - 0.5) * 2.5;
        positions.push((x, y));
    }
    positions
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Turn Order Display
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Render turn order icons along the top.
pub fn render_turn_order(
    vis: &CombatVisualState,
    engine: &mut ProofEngine,
    player_name: &str,
    enemy_name: &str,
    theme: &Theme,
) {
    let base_x = -2.0;
    let y = 5.0;

    ui_render::small(engine, "TURN:", base_x - 1.5, y, theme.dim);

    // Player icon
    let p_alpha = if vis.is_player_turn { 1.0 } else { 0.4 };
    let p_color = Vec4::new(theme.primary.x, theme.primary.y, theme.primary.z, p_alpha);
    engine.spawn_glyph(Glyph {
        character: '@',
        position: Vec3::new(base_x, y, 0.0),
        scale: Vec2::splat(0.3),
        color: p_color,
        emission: if vis.is_player_turn { 0.5 } else { 0.1 },
        layer: RenderLayer::UI,
        ..Default::default()
    });

    // Arrow
    ui_render::small(engine, ">", base_x + 0.5, y, theme.dim);

    // Enemy icon
    let e_alpha = if !vis.is_player_turn { 1.0 } else { 0.4 };
    let e_color = Vec4::new(theme.danger.x, theme.danger.y, theme.danger.z, e_alpha);
    engine.spawn_glyph(Glyph {
        character: 'X',
        position: Vec3::new(base_x + 1.0, y, 0.0),
        scale: Vec2::splat(0.3),
        color: e_color,
        emission: if !vis.is_player_turn { 0.5 } else { 0.1 },
        layer: RenderLayer::UI,
        ..Default::default()
    });
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Minimap
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Render a small combat minimap in the corner.
pub fn render_minimap(
    vis: &CombatVisualState,
    engine: &mut ProofEngine,
    theme: &Theme,
) {
    let cx = 7.5;
    let cy = -3.5;
    let size = 1.2;

    // Border
    let border_chars = ['-', '|', '+'];
    // Top/bottom
    for i in 0..5 {
        let x = cx - size + i as f32 * 0.5;
        engine.spawn_glyph(Glyph {
            character: '-',
            position: Vec3::new(x, cy + size, 0.0),
            scale: Vec2::splat(0.15),
            color: theme.dim,
            emission: 0.1,
            layer: RenderLayer::UI,
            ..Default::default()
        });
        engine.spawn_glyph(Glyph {
            character: '-',
            position: Vec3::new(x, cy - size, 0.0),
            scale: Vec2::splat(0.15),
            color: theme.dim,
            emission: 0.1,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }

    // Player dot
    engine.spawn_glyph(Glyph {
        character: '@',
        position: Vec3::new(cx - 0.5, cy, 0.0),
        scale: Vec2::splat(0.2),
        color: theme.primary,
        emission: 0.4,
        layer: RenderLayer::UI,
        ..Default::default()
    });

    // Enemy dot
    engine.spawn_glyph(Glyph {
        character: 'X',
        position: Vec3::new(cx + 0.5, cy, 0.0),
        scale: Vec2::splat(0.2),
        color: theme.danger,
        emission: 0.4,
        layer: RenderLayer::UI,
        ..Default::default()
    });
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// RENDER — Tooltip System
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Render a tooltip box at a position.
pub fn render_tooltip(
    engine: &mut ProofEngine,
    text: &str,
    x: f32,
    y: f32,
    theme: &Theme,
) {
    let lines: Vec<&str> = text.split('\n').collect();
    let max_width = lines.iter().map(|l| l.len()).max().unwrap_or(0);
    let box_width = max_width as f32 * 0.22 + 0.4;
    let box_height = lines.len() as f32 * 0.35 + 0.3;

    // Background
    let cols = (box_width / 0.22) as usize;
    let rows = (box_height / 0.35) as usize;
    for row in 0..rows {
        for col in 0..cols {
            let bx = x + col as f32 * 0.22;
            let by = y - row as f32 * 0.35;
            engine.spawn_glyph(Glyph {
                character: ' ',
                position: Vec3::new(bx, by, 0.0),
                scale: Vec2::splat(0.22),
                color: Vec4::new(0.05, 0.05, 0.1, 0.85),
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }
    }

    // Border top/bottom
    for col in 0..cols {
        let bx = x + col as f32 * 0.22;
        engine.spawn_glyph(Glyph {
            character: '-',
            position: Vec3::new(bx, y + 0.15, 0.0),
            scale: Vec2::splat(0.15),
            color: theme.border,
            emission: 0.2,
            layer: RenderLayer::Overlay,
            ..Default::default()
        });
    }

    // Text
    for (i, line) in lines.iter().enumerate() {
        ui_render::text(engine, line, x + 0.1, y - 0.15 - i as f32 * 0.35, theme.primary, 0.2, 0.3);
    }
}
