//! Combat screen — full proof-engine showcase.
//!
//! Arena floor, entity formations, attack animations, damage numbers,
//! HP/MP ghost bars, status particles, screen shake, scrollable combat log,
//! boss overlays, and an action bar with mana-aware spell highlighting.

use proof_engine::prelude::*;
use proof_engine::audio::MusicVibe;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;
use chaos_rpg_core::combat::{resolve_action, CombatAction, CombatOutcome};
use chaos_rpg_core::character::{CharacterClass, StatusEffect};
use chaos_rpg_core::enemy::EnemyTier;
use chaos_rpg_core::spells::SpellSchool;
use chaos_rpg_core::world::RoomType;

// ─── Constants ───────────────────────────────────────────────────────────────

const PLAYER_CENTER: Vec3 = Vec3::new(-4.0, 0.0, 0.0);
const ENEMY_CENTER: Vec3 = Vec3::new(4.0, 0.0, 0.0);
const ARENA_Z: f32 = -1.0;
const ARENA_HALF_W: f32 = 8.0;
const ARENA_HALF_H: f32 = 4.0;

/// Pseudo-random float from integer seed, range [0, 1).
fn hash_f32(seed: u64) -> f32 {
    let h = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((h >> 33) as f32) / (u32::MAX as f32)
}

/// Pseudo-random float in [-1, 1) from integer seed.
fn hash_signed(seed: u64) -> f32 {
    hash_f32(seed) * 2.0 - 1.0
}

// ─── Formation geometry ──────────────────────────────────────────────────────

/// Generate diamond/ring formation offsets for `n` glyphs around origin.
fn diamond_offsets(n: usize, radius: f32) -> Vec<(f32, f32)> {
    let mut pts = Vec::with_capacity(n);
    if n == 0 {
        return pts;
    }
    // Center glyph
    pts.push((0.0, 0.0));
    if n == 1 {
        return pts;
    }
    // Concentric rings
    let mut ring = 1u32;
    while pts.len() < n {
        let perimeter = ring as usize * 4;
        let r = radius * ring as f32 * 0.35;
        for i in 0..perimeter {
            if pts.len() >= n {
                break;
            }
            let angle = (i as f32 / perimeter as f32) * std::f32::consts::TAU;
            // Diamond shape: use max-norm distortion
            let dx = angle.cos();
            let dy = angle.sin();
            let diamond = 1.0 / (dx.abs() + dy.abs()).max(0.01);
            pts.push((dx * r * diamond * 0.7, dy * r * diamond * 0.7));
        }
        ring += 1;
    }
    pts.truncate(n);
    pts
}

/// Get class-specific characters and color for player formation.
fn class_glyphs(class: &CharacterClass) -> (&[char], Vec4) {
    match class {
        CharacterClass::Mage => (
            &['*', '~', '^', 'o', '.'],
            Vec4::new(0.4, 0.5, 1.0, 1.0),
        ),
        CharacterClass::Berserker => (
            &['#', 'X', '+', '/', '\\'],
            Vec4::new(1.0, 0.3, 0.2, 1.0),
        ),
        CharacterClass::Ranger => (
            &['>', '-', '|', '.', '`'],
            Vec4::new(0.3, 0.85, 0.3, 1.0),
        ),
        CharacterClass::Thief => (
            &['.', ',', '\'', '`', '"'],
            Vec4::new(0.6, 0.6, 0.7, 1.0),
        ),
        CharacterClass::Necromancer => (
            &['$', '%', '&', '@', '!'],
            Vec4::new(0.6, 0.2, 0.8, 1.0),
        ),
        CharacterClass::Alchemist => (
            &['o', 'O', '0', '@', '*'],
            Vec4::new(0.9, 0.7, 0.1, 1.0),
        ),
        CharacterClass::Paladin => (
            &['+', 'T', '|', '#', '='],
            Vec4::new(1.0, 0.9, 0.4, 1.0),
        ),
        CharacterClass::VoidWalker => (
            &['.', ':', ';', '!', '?'],
            Vec4::new(0.5, 0.0, 0.8, 1.0),
        ),
        CharacterClass::Warlord => (
            &['#', '=', 'H', 'W', '+'],
            Vec4::new(0.8, 0.3, 0.1, 1.0),
        ),
        CharacterClass::Trickster => (
            &['?', '!', '~', '^', '`'],
            Vec4::new(0.2, 0.9, 0.7, 1.0),
        ),
        CharacterClass::Runesmith => (
            &['#', '=', '+', '*', 'R'],
            Vec4::new(0.9, 0.5, 0.2, 1.0),
        ),
        CharacterClass::Chronomancer => (
            &['@', 'o', '.', ':', '*'],
            Vec4::new(0.3, 0.8, 1.0, 1.0),
        ),
    }
}

/// Get glyph count for an enemy tier.
fn tier_glyph_count(tier: &EnemyTier) -> usize {
    match tier {
        EnemyTier::Minion => 10,
        EnemyTier::Elite => 20,
        EnemyTier::Champion => 30,
        EnemyTier::Boss | EnemyTier::Abomination => 50,
    }
}

/// Map a spell school to a particle color.
fn spell_school_color(school: &SpellSchool) -> Vec4 {
    match school {
        SpellSchool::Fire => Vec4::new(1.0, 0.4, 0.1, 1.0),
        SpellSchool::Ice => Vec4::new(0.3, 0.7, 1.0, 1.0),
        SpellSchool::Lightning => Vec4::new(1.0, 1.0, 0.3, 1.0),
        SpellSchool::Arcane => Vec4::new(0.7, 0.3, 1.0, 1.0),
        SpellSchool::Nature => Vec4::new(0.2, 0.9, 0.3, 1.0),
        SpellSchool::Shadow => Vec4::new(0.4, 0.1, 0.5, 1.0),
        SpellSchool::Chaos => Vec4::new(1.0, 0.0, 0.5, 1.0),
    }
}

// ─── Arena Floor ─────────────────────────────────────────────────────────────

/// Render a perspective grid of dim glyphs at z=ARENA_Z.
/// Floor pattern varies by room type from the current floor.
fn render_arena_floor(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = state.frame as f32 * 0.016;

    // Determine floor pattern from current room type
    let room_type = state
        .floor
        .as_ref()
        .map(|f| f.current().room_type.clone())
        .unwrap_or(RoomType::Combat);

    let (primary_char, secondary_char, accent_char) = match room_type {
        RoomType::Boss => ('#', '=', '+'),
        RoomType::Shop => ('.', '-', '|'),
        RoomType::Shrine => ('+', '.', '*'),
        RoomType::Trap => ('!', '.', '^'),
        RoomType::ChaosRift => ('~', '.', '*'),
        RoomType::Treasure => ('.', '-', '$'),
        RoomType::Portal => ('.', 'o', '@'),
        _ => ('.', '-', '|'), // Combat, Empty, CraftingBench
    };

    // Grid spacing
    let step_x = 1.2_f32;
    let step_y = 0.8_f32;
    let cols = (ARENA_HALF_W * 2.0 / step_x) as i32;
    let rows = (ARENA_HALF_H * 2.0 / step_y) as i32;

    for row in 0..rows {
        for col in 0..cols {
            let gx = -ARENA_HALF_W + col as f32 * step_x;
            let gy = -ARENA_HALF_H + row as f32 * step_y;

            // Distance from center for opacity falloff
            let dist = (gx * gx / (ARENA_HALF_W * ARENA_HALF_W)
                + gy * gy / (ARENA_HALF_H * ARENA_HALF_H))
                .sqrt();
            let alpha = (1.0 - dist).max(0.05) * 0.35;

            // Choose character based on position
            let ch = if row == 0 || row == rows - 1 {
                secondary_char
            } else if col % 6 == 0 {
                accent_char
            } else if col % 3 == 0 && row % 2 == 0 {
                secondary_char
            } else {
                primary_char
            };

            // Subtle wave animation
            let wave = (t * 0.5 + gx * 0.3 + gy * 0.2).sin() * 0.03;
            let floor_color = Vec4::new(
                theme.muted.x * 1.5,
                theme.muted.y * 1.5,
                theme.muted.z * 1.5,
                alpha + wave,
            );

            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(gx, gy, ARENA_Z),
                scale: Vec2::splat(0.25),
                color: floor_color,
                emission: 0.05,
                layer: RenderLayer::Background,
                ..Default::default()
            });
        }
    }

    // Arena border lines — top and bottom
    let border_chars = ['='; 1];
    let border_count = (ARENA_HALF_W * 2.0 / 0.6) as usize;
    for i in 0..border_count {
        let bx = -ARENA_HALF_W + i as f32 * 0.6;
        let pulse = ((t * 1.5 + i as f32 * 0.15).sin() * 0.2 + 0.5).max(0.0);
        let border_c = Vec4::new(
            theme.border.x * pulse,
            theme.border.y * pulse,
            theme.border.z * pulse,
            0.4,
        );
        for &by in &[-ARENA_HALF_H, ARENA_HALF_H] {
            engine.spawn_glyph(Glyph {
                character: border_chars[0],
                position: Vec3::new(bx, by, ARENA_Z + 0.1),
                scale: Vec2::splat(0.2),
                color: border_c,
                emission: 0.2,
                layer: RenderLayer::Background,
                ..Default::default()
            });
        }
    }
    // Left and right borders
    let border_rows = (ARENA_HALF_H * 2.0 / 0.5) as usize;
    for i in 0..border_rows {
        let by = -ARENA_HALF_H + i as f32 * 0.5;
        let pulse = ((t * 1.2 + i as f32 * 0.2).sin() * 0.2 + 0.5).max(0.0);
        let border_c = Vec4::new(
            theme.border.x * pulse,
            theme.border.y * pulse,
            theme.border.z * pulse,
            0.4,
        );
        for &bx in &[-ARENA_HALF_W, ARENA_HALF_W] {
            engine.spawn_glyph(Glyph {
                character: '|',
                position: Vec3::new(bx, by, ARENA_Z + 0.1),
                scale: Vec2::splat(0.2),
                color: border_c,
                emission: 0.2,
                layer: RenderLayer::Background,
                ..Default::default()
            });
        }
    }
}

// ─── Player Entity ───────────────────────────────────────────────────────────

/// Render the player formation at PLAYER_CENTER.
/// Diamond/ring shape using class-specific chars. Breathing animation on scale.
/// HP-linked cohesion: low HP causes glyphs to drift outward.
fn render_player_entity(state: &GameState, engine: &mut ProofEngine) {
    let player = match &state.player {
        Some(p) => p,
        None => return,
    };
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = state.frame as f32 * 0.016;

    let hp_frac = player.current_hp as f32 / player.max_hp.max(1) as f32;
    let (chars, base_color) = class_glyphs(&player.class);

    // Glyph count: 15 base, scales with level up to 30
    let glyph_count = (15 + (player.level as usize / 3).min(15)).min(30);
    let offsets = diamond_offsets(glyph_count, 1.8);

    // HP-linked drift: as HP drops, formation expands
    let drift_mult = 1.0 + (1.0 - hp_frac) * 0.5;

    // Breathing animation base
    let breath = (t * 1.2).sin() * 0.06 + 1.0;

    // Flash overlay when recently hit
    let flash_factor = state.player_flash.min(1.0);
    let flash_color = Vec4::new(1.0, 0.3, 0.2, 1.0);

    for (i, &(ox, oy)) in offsets.iter().enumerate() {
        let ch = chars[i % chars.len()];
        let dx = ox * drift_mult;
        let dy = oy * drift_mult;

        // Individual glyph jitter scaled by entropy (low HP = more jitter)
        let jitter_x = hash_signed((state.frame + i as u64) * 7919) * (1.0 - hp_frac) * 0.08;
        let jitter_y = hash_signed((state.frame + i as u64) * 6271) * (1.0 - hp_frac) * 0.08;

        let pos = Vec3::new(
            PLAYER_CENTER.x + dx + jitter_x,
            PLAYER_CENTER.y + dy + jitter_y,
            0.0,
        );

        // Color: blend base with theme heading, modulate by position in formation
        let ring_frac = ((ox * ox + oy * oy).sqrt() / 1.5).min(1.0);
        let mut color = Vec4::new(
            base_color.x * (1.0 - ring_frac * 0.3),
            base_color.y * (1.0 - ring_frac * 0.2),
            base_color.z * (1.0 - ring_frac * 0.1),
            0.7 + (1.0 - ring_frac) * 0.3,
        );

        // Apply hit flash
        if flash_factor > 0.0 {
            color = Vec4::new(
                color.x + (flash_color.x - color.x) * flash_factor,
                color.y + (flash_color.y - color.y) * flash_factor,
                color.z + (flash_color.z - color.z) * flash_factor,
                color.w,
            );
        }

        // Scale with breathing
        let s = 0.3 * breath * (1.0 - ring_frac * 0.15);

        engine.spawn_glyph(Glyph {
            character: ch,
            position: pos,
            scale: Vec2::splat(s),
            color,
            emission: 0.4 + (1.0 - ring_frac) * 0.4,
            glow_color: Vec3::new(base_color.x, base_color.y, base_color.z),
            glow_radius: 0.3 * (1.0 - ring_frac),
            layer: RenderLayer::Entity,
            life_function: Some(MathFunction::Breathing {
                rate: 1.2 + i as f32 * 0.02,
                depth: 0.04,
            }),
            ..Default::default()
        });
    }

    // Central class sigil — larger, brighter glyph at formation center
    let sigil = match player.class {
        CharacterClass::Mage => '@',
        CharacterClass::Berserker => '#',
        CharacterClass::Ranger => '>',
        CharacterClass::Thief => '&',
        CharacterClass::Necromancer => '$',
        CharacterClass::Alchemist => '%',
        CharacterClass::Paladin => '+',
        CharacterClass::VoidWalker => '?',
        CharacterClass::Warlord => 'W',
        CharacterClass::Trickster => '!',
        CharacterClass::Runesmith => 'R',
        CharacterClass::Chronomancer => '@',
    };

    let mut sigil_color = Vec4::new(
        base_color.x * 1.3,
        base_color.y * 1.3,
        base_color.z * 1.3,
        1.0,
    );
    if flash_factor > 0.0 {
        sigil_color = Vec4::new(
            sigil_color.x + (flash_color.x - sigil_color.x) * flash_factor,
            sigil_color.y + (flash_color.y - sigil_color.y) * flash_factor,
            sigil_color.z + (flash_color.z - sigil_color.z) * flash_factor,
            1.0,
        );
    }

    engine.spawn_glyph(Glyph {
        character: sigil,
        position: Vec3::new(PLAYER_CENTER.x, PLAYER_CENTER.y, 0.1),
        scale: Vec2::splat(0.45 * breath),
        color: sigil_color,
        emission: 0.8,
        glow_color: Vec3::new(base_color.x, base_color.y, base_color.z),
        glow_radius: 0.6,
        layer: RenderLayer::Entity,
        life_function: Some(MathFunction::Breathing { rate: 1.0, depth: 0.05 }),
        ..Default::default()
    });
}

// ─── Enemy Entity ────────────────────────────────────────────────────────────

/// Render the enemy formation at ENEMY_CENTER.
/// Uses enemy name characters, glyph count scales by tier.
fn render_enemy_entity(state: &GameState, engine: &mut ProofEngine) {
    let enemy = match &state.enemy {
        Some(e) => e,
        None => return,
    };
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = state.frame as f32 * 0.016;

    let hp_frac = enemy.hp as f32 / enemy.max_hp.max(1) as f32;
    let glyph_count = tier_glyph_count(&enemy.tier);
    let offsets = diamond_offsets(glyph_count, 2.0);

    // Use enemy name characters for formation
    let name_chars: Vec<char> = enemy.name.chars().filter(|c| !c.is_whitespace()).collect();
    let name_chars = if name_chars.is_empty() {
        vec!['?']
    } else {
        name_chars
    };

    // Enemy base color from theme danger, shift toward muted as HP drops
    let base_r = theme.danger.x * (0.5 + hp_frac * 0.5);
    let base_g = theme.danger.y * (0.4 + hp_frac * 0.3);
    let base_b = theme.danger.z * (0.5 + hp_frac * 0.5);

    let drift_mult = 1.0 + (1.0 - hp_frac) * 0.5;

    // Enemy idle animation: slower, menacing pulse
    let pulse = (t * 0.8).sin() * 0.04 + 1.0;

    // Flash overlay
    let flash_factor = state.enemy_flash.min(1.0);
    let flash_color = state.enemy_flash_color;

    for (i, &(ox, oy)) in offsets.iter().enumerate() {
        let ch = name_chars[i % name_chars.len()];
        let dx = ox * drift_mult;
        let dy = oy * drift_mult;

        let jitter_x = hash_signed((state.frame + i as u64 + 9999) * 4157) * (1.0 - hp_frac) * 0.1;
        let jitter_y = hash_signed((state.frame + i as u64 + 9999) * 3571) * (1.0 - hp_frac) * 0.1;

        let pos = Vec3::new(
            ENEMY_CENTER.x + dx + jitter_x,
            ENEMY_CENTER.y + dy + jitter_y,
            0.0,
        );

        let ring_frac = ((ox * ox + oy * oy).sqrt() / 1.5).min(1.0);
        let mut color = Vec4::new(
            base_r * (1.0 - ring_frac * 0.2),
            base_g * (1.0 - ring_frac * 0.3),
            base_b * (1.0 - ring_frac * 0.2),
            0.65 + (1.0 - ring_frac) * 0.35,
        );

        if flash_factor > 0.0 {
            color = Vec4::new(
                color.x + (flash_color.x - color.x) * flash_factor,
                color.y + (flash_color.y - color.y) * flash_factor,
                color.z + (flash_color.z - color.z) * flash_factor,
                color.w,
            );
        }

        let s = 0.28 * pulse * (1.0 - ring_frac * 0.1);

        engine.spawn_glyph(Glyph {
            character: ch,
            position: pos,
            scale: Vec2::splat(s),
            color,
            emission: 0.35 + (1.0 - ring_frac) * 0.3,
            glow_color: Vec3::new(base_r, base_g, base_b),
            glow_radius: 0.2 * (1.0 - ring_frac),
            layer: RenderLayer::Entity,
            life_function: Some(MathFunction::Sine {
                amplitude: 0.03,
                frequency: 0.6 + i as f32 * 0.01,
                phase: i as f32 * 0.5,
            }),
            ..Default::default()
        });
    }

    // Central enemy sigil
    let sigil = name_chars[0].to_uppercase().next().unwrap_or('?');
    let mut sigil_color = Vec4::new(base_r * 1.4, base_g * 1.4, base_b * 1.4, 1.0);
    if flash_factor > 0.0 {
        sigil_color = Vec4::new(
            sigil_color.x + (flash_color.x - sigil_color.x) * flash_factor,
            sigil_color.y + (flash_color.y - sigil_color.y) * flash_factor,
            sigil_color.z + (flash_color.z - sigil_color.z) * flash_factor,
            1.0,
        );
    }

    engine.spawn_glyph(Glyph {
        character: sigil,
        position: Vec3::new(ENEMY_CENTER.x, ENEMY_CENTER.y, 0.1),
        scale: Vec2::splat(0.5 * pulse),
        color: sigil_color,
        emission: 0.9,
        glow_color: Vec3::new(base_r, base_g, base_b),
        glow_radius: 0.7,
        layer: RenderLayer::Entity,
        life_function: Some(MathFunction::Sine {
            amplitude: 0.04,
            frequency: 0.5,
            phase: 0.0,
        }),
        ..Default::default()
    });
}

// ─── Attack Animation ────────────────────────────────────────────────────────

/// Spawn attack trail projectiles based on last_action_type.
///   0 = no anim, 1 = Attack, 2 = Heavy, 3 = Spell, 4 = Defend, 5 = Enemy attack
fn render_attack_animation(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = state.frame as f32 * 0.016;

    // Attack animations are driven by spell_beam_timer (used as general anim timer)
    let anim_t = state.spell_beam_timer;
    if anim_t <= 0.0 {
        return;
    }

    let progress = 1.0 - (anim_t / 0.6).min(1.0); // 0.6s animation duration normalized
    let action = state.last_action_type;

    match action {
        // ── Attack: '/' chars moving from player to enemy ──
        1 => {
            let trail_count = 8;
            for i in 0..trail_count {
                let frac = (progress - i as f32 * 0.06).clamp(0.0, 1.0);
                let x = PLAYER_CENTER.x + (ENEMY_CENTER.x - PLAYER_CENTER.x) * frac;
                let y = PLAYER_CENTER.y + (i as f32 - 3.5) * 0.08;
                let alpha = (1.0 - (progress - frac).abs() * 5.0).max(0.0) * anim_t * 2.0;

                engine.spawn_glyph(Glyph {
                    character: '/',
                    position: Vec3::new(x, y, 0.2),
                    velocity: Vec3::new(6.0, 0.0, 0.0),
                    scale: Vec2::splat(0.35),
                    color: Vec4::new(1.0, 0.9, 0.7, alpha),
                    emission: 0.7,
                    glow_color: Vec3::new(1.0, 0.8, 0.3),
                    glow_radius: 0.3,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    lifetime: 0.1,
                    ..Default::default()
                });
            }
        }

        // ── Heavy Attack: '>' chars, larger scale ──
        2 => {
            let trail_count = 8;
            for i in 0..trail_count {
                let frac = (progress - i as f32 * 0.04).clamp(0.0, 1.0);
                let x = PLAYER_CENTER.x + (ENEMY_CENTER.x - PLAYER_CENTER.x) * frac;
                let y_spread = (i as f32 - 3.5) * 0.12;
                let y = PLAYER_CENTER.y + y_spread;
                let alpha = (1.0 - (progress - frac).abs() * 4.0).max(0.0) * anim_t * 2.0;

                engine.spawn_glyph(Glyph {
                    character: '>',
                    position: Vec3::new(x, y, 0.2),
                    velocity: Vec3::new(8.0, 0.0, 0.0),
                    scale: Vec2::splat(0.45),
                    color: Vec4::new(1.0, 0.5, 0.1, alpha),
                    emission: 0.9,
                    glow_color: Vec3::new(1.0, 0.4, 0.0),
                    glow_radius: 0.5,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    lifetime: 0.1,
                    ..Default::default()
                });
            }
        }

        // ── Spell: '*' chars in a sine wave path with element color ──
        3 => {
            let spell_color = state.spell_beam_color;
            let trail_count = 8;
            for i in 0..trail_count {
                let frac = (progress - i as f32 * 0.05).clamp(0.0, 1.0);
                let x = PLAYER_CENTER.x + (ENEMY_CENTER.x - PLAYER_CENTER.x) * frac;
                let sine_y = (frac * std::f32::consts::TAU * 2.0).sin() * 0.8;
                let y = PLAYER_CENTER.y + sine_y;
                let alpha = (1.0 - (progress - frac).abs() * 4.0).max(0.0) * anim_t * 2.0;

                engine.spawn_glyph(Glyph {
                    character: '*',
                    position: Vec3::new(x, y, 0.2),
                    velocity: Vec3::new(5.0, sine_y * 3.0, 0.0),
                    scale: Vec2::splat(0.35),
                    color: Vec4::new(spell_color.x, spell_color.y, spell_color.z, alpha),
                    emission: 1.0,
                    glow_color: Vec3::new(spell_color.x, spell_color.y, spell_color.z),
                    glow_radius: 0.4,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    lifetime: 0.12,
                    ..Default::default()
                });
            }
        }

        // ── Defend: shield wall at player position ──
        4 => {
            let wall_height = 6;
            for i in 0..wall_height {
                let y = PLAYER_CENTER.y + (i as f32 - wall_height as f32 * 0.5) * 0.5;
                let shield_alpha = anim_t * 2.5;
                let pulse = ((t * 4.0 + i as f32 * 0.5).sin() * 0.15 + 0.85).max(0.0);

                engine.spawn_glyph(Glyph {
                    character: '\u{2588}', // full block
                    position: Vec3::new(PLAYER_CENTER.x + 1.0, y, 0.15),
                    scale: Vec2::new(0.4, 0.5),
                    color: Vec4::new(0.3, 0.6, 1.0, shield_alpha * pulse),
                    emission: 0.6,
                    glow_color: Vec3::new(0.2, 0.5, 1.0),
                    glow_radius: 0.5,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
            // Extra shield border characters
            for i in 0..3 {
                let x_off = -0.3 + i as f32 * 0.3;
                engine.spawn_glyph(Glyph {
                    character: '=',
                    position: Vec3::new(
                        PLAYER_CENTER.x + 1.0 + x_off,
                        PLAYER_CENTER.y + wall_height as f32 * 0.25 + 0.3,
                        0.15,
                    ),
                    scale: Vec2::splat(0.3),
                    color: Vec4::new(0.4, 0.7, 1.0, anim_t * 2.0),
                    emission: 0.5,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    lifetime: 0.15,
                    ..Default::default()
                });
            }
        }

        // ── Enemy attack: trail from enemy to player ──
        5 => {
            let trail_count = 8;
            for i in 0..trail_count {
                let frac = (progress - i as f32 * 0.05).clamp(0.0, 1.0);
                let x = ENEMY_CENTER.x + (PLAYER_CENTER.x - ENEMY_CENTER.x) * frac;
                let y = ENEMY_CENTER.y + hash_signed((state.frame + i as u64) * 1237) * 0.3;
                let alpha = (1.0 - (progress - frac).abs() * 5.0).max(0.0) * anim_t * 2.0;

                engine.spawn_glyph(Glyph {
                    character: '<',
                    position: Vec3::new(x, y, 0.2),
                    velocity: Vec3::new(-7.0, 0.0, 0.0),
                    scale: Vec2::splat(0.33),
                    color: Vec4::new(
                        theme.danger.x,
                        theme.danger.y * 0.6,
                        theme.danger.z * 0.4,
                        alpha,
                    ),
                    emission: 0.6,
                    glow_color: Vec3::new(theme.danger.x, 0.2, 0.1),
                    glow_radius: 0.3,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    lifetime: 0.1,
                    ..Default::default()
                });
            }
        }

        _ => {}
    }
}

// ─── Damage Numbers ──────────────────────────────────────────────────────────

/// Render floating damage text from recent combat events.
/// Crits: gold color, scale 0.5. Normal: white, scale 0.3.
/// Numbers drift upward and fade out.
fn render_damage_numbers(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    // Use the last few combat events to display damage numbers.
    // We use enemy_flash/player_flash timers as proxy for recency.
    let log_len = state.combat_log.len();
    if log_len == 0 {
        return;
    }

    // Show damage from the last 3 events if they are recent (flash timers > 0)
    let show_player_dmg = state.player_flash > 0.0;
    let show_enemy_dmg = state.enemy_flash > 0.0;

    if let Some(ref combat) = state.combat_state {
        // Last events from combat log
        let recent_start = combat.log.len().saturating_sub(3);
        for (idx, event) in combat.log[recent_start..].iter().enumerate() {
            let age = idx as f32 * 0.15; // stagger vertical positions
            match event {
                chaos_rpg_core::combat::CombatEvent::PlayerAttack { damage, is_crit } if show_enemy_dmg => {
                    let drift = state.enemy_flash * 1.5;
                    let alpha = state.enemy_flash.min(1.0);
                    let (scale, color) = if *is_crit {
                        (0.5, theme.gold)
                    } else {
                        (0.3, Vec4::new(1.0, 1.0, 1.0, 1.0))
                    };
                    let text = format!("{}", damage);
                    let tx = ENEMY_CENTER.x - 0.5;
                    let ty = ENEMY_CENTER.y + 1.5 + drift + age * 0.6;

                    ui_render::text(
                        engine,
                        &text,
                        tx,
                        ty,
                        Vec4::new(color.x, color.y, color.z, alpha),
                        scale,
                        0.8,
                    );

                    // Crit gets a star
                    if *is_crit {
                        engine.spawn_glyph(Glyph {
                            character: '*',
                            position: Vec3::new(tx - 0.4, ty, 0.3),
                            scale: Vec2::splat(0.4),
                            color: Vec4::new(1.0, 0.9, 0.2, alpha),
                            emission: 1.2,
                            glow_color: Vec3::new(1.0, 0.8, 0.0),
                            glow_radius: 0.5,
                            layer: RenderLayer::Particle,
                            blend_mode: BlendMode::Additive,
                            ..Default::default()
                        });
                    }
                }
                chaos_rpg_core::combat::CombatEvent::EnemyAttack { damage, is_crit } if show_player_dmg => {
                    let drift = state.player_flash * 1.5;
                    let alpha = state.player_flash.min(1.0);
                    let (scale, color) = if *is_crit {
                        (0.5, Vec4::new(1.0, 0.2, 0.1, 1.0))
                    } else {
                        (0.3, Vec4::new(1.0, 0.6, 0.5, 1.0))
                    };
                    let text = format!("{}", damage);
                    let tx = PLAYER_CENTER.x - 0.3;
                    let ty = PLAYER_CENTER.y + 1.5 + drift + age * 0.6;

                    ui_render::text(
                        engine,
                        &text,
                        tx,
                        ty,
                        Vec4::new(color.x, color.y, color.z, alpha),
                        scale,
                        0.8,
                    );

                    if *is_crit {
                        engine.spawn_glyph(Glyph {
                            character: '!',
                            position: Vec3::new(tx - 0.4, ty, 0.3),
                            scale: Vec2::splat(0.45),
                            color: Vec4::new(1.0, 0.1, 0.0, alpha),
                            emission: 1.5,
                            glow_color: Vec3::new(1.0, 0.0, 0.0),
                            glow_radius: 0.6,
                            layer: RenderLayer::Particle,
                            blend_mode: BlendMode::Additive,
                            ..Default::default()
                        });
                    }
                }
                chaos_rpg_core::combat::CombatEvent::SpellCast { name: _, damage, backfired } if show_enemy_dmg || show_player_dmg => {
                    let (target_x, flash) = if *backfired {
                        (PLAYER_CENTER.x, state.player_flash)
                    } else {
                        (ENEMY_CENTER.x, state.enemy_flash)
                    };
                    let drift = flash * 1.5;
                    let alpha = flash.min(1.0);
                    let text = format!("{}", damage);
                    let color = state.spell_beam_color;

                    ui_render::text(
                        engine,
                        &text,
                        target_x - 0.3,
                        0.0 + 1.5 + drift,
                        Vec4::new(color.x, color.y, color.z, alpha),
                        0.4,
                        1.0,
                    );
                }
                chaos_rpg_core::combat::CombatEvent::PlayerDefend { damage_reduced } if show_player_dmg => {
                    let alpha = state.player_flash.min(1.0);
                    let text = format!("-{}", damage_reduced);
                    ui_render::text(
                        engine,
                        &text,
                        PLAYER_CENTER.x + 0.5,
                        PLAYER_CENTER.y + 1.8,
                        Vec4::new(0.3, 0.6, 1.0, alpha),
                        0.35,
                        0.7,
                    );
                }
                chaos_rpg_core::combat::CombatEvent::PlayerHealed { amount } if show_player_dmg => {
                    let alpha = state.player_flash.min(1.0);
                    let text = format!("+{}", amount);
                    ui_render::text(
                        engine,
                        &text,
                        PLAYER_CENTER.x - 0.3,
                        PLAYER_CENTER.y + 1.8 + age * 0.5,
                        Vec4::new(0.2, 1.0, 0.4, alpha),
                        0.35,
                        0.6,
                    );
                }
                _ => {}
            }
        }
    }
}

// ─── HP / MP Bars ────────────────────────────────────────────────────────────

/// Render HP and MP bars with ghost bar effect.
fn render_hp_mp_bars(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    // ── Player HP Bar ──
    if let Some(ref player) = state.player {
        let px = -8.5;
        let py = 3.8;

        // Label + name
        ui_render::body(
            engine,
            &format!("{} ({})", player.name, player.class.name()),
            px,
            py,
            theme.heading,
        );

        let hp_pct = state.display_player_hp;
        let bar_x = px + 1.0;
        let bar_y = py - 0.6;
        let bar_w = 4.5;

        // Ghost bar behind the main bar (shows previous HP)
        if state.ghost_player_timer > 0.0 && state.ghost_player_hp > hp_pct {
            let ghost_alpha = (state.ghost_player_timer / 1.5).min(1.0);
            let ghost_color = Vec4::new(
                theme.hp_color(state.ghost_player_hp).x * 0.4,
                theme.hp_color(state.ghost_player_hp).y * 0.4,
                theme.hp_color(state.ghost_player_hp).z * 0.4,
                ghost_alpha * 0.6,
            );
            ui_render::bar(
                engine,
                bar_x,
                bar_y,
                bar_w,
                state.ghost_player_hp,
                ghost_color,
                theme.muted,
                0.3,
            );
        }

        // Main HP bar
        ui_render::small(engine, "HP", px, bar_y, theme.muted);
        ui_render::bar(
            engine,
            bar_x,
            bar_y,
            bar_w,
            hp_pct,
            theme.hp_color(hp_pct),
            theme.muted,
            0.3,
        );
        ui_render::small(
            engine,
            &format!("{}/{}", player.current_hp, player.max_hp),
            px + 5.8,
            bar_y,
            theme.hp_color(hp_pct),
        );

        // ── Player MP Bar ──
        let max_mp = state.max_mana();
        if max_mp > 0 {
            let mp_y = py - 1.2;
            ui_render::small(engine, "MP", px, mp_y, theme.muted);
            ui_render::bar(
                engine,
                bar_x,
                mp_y,
                bar_w,
                state.display_mp,
                theme.mana,
                theme.muted,
                0.3,
            );
            ui_render::small(
                engine,
                &format!("{}/{}", state.current_mana, max_mp),
                px + 5.8,
                mp_y,
                theme.mana,
            );
        }

        // Level / gold / kills
        ui_render::small(
            engine,
            &format!(
                "Lv.{} | {} gold | {} kills",
                player.level, player.gold, player.kills
            ),
            px,
            py - 1.8,
            theme.dim,
        );

        // ── Player Status Effects ──
        let status_y = py - 2.3;
        let mut sx = px;
        for effect in &player.status_effects {
            let tag = effect.badge();
            let tag_color = status_effect_color(effect, theme);
            ui_render::small(engine, tag, sx, status_y, tag_color);
            sx += tag.len() as f32 * 0.25 + 0.3;
        }
    }

    // ── Enemy HP Bar ──
    if let Some(ref enemy) = state.enemy {
        let ex = 1.5;
        let ey = 3.8;

        ui_render::body(engine, &enemy.name, ex, ey, theme.danger);

        let hp_pct = state.display_enemy_hp;
        let bar_x = ex + 1.0;
        let bar_y = ey - 0.6;
        let bar_w = 4.5;

        // Ghost bar
        if state.ghost_enemy_timer > 0.0 && state.ghost_enemy_hp > hp_pct {
            let ghost_alpha = (state.ghost_enemy_timer / 1.5).min(1.0);
            let ghost_color = Vec4::new(
                theme.hp_color(state.ghost_enemy_hp).x * 0.4,
                theme.hp_color(state.ghost_enemy_hp).y * 0.4,
                theme.hp_color(state.ghost_enemy_hp).z * 0.4,
                ghost_alpha * 0.6,
            );
            ui_render::bar(engine, bar_x, bar_y, bar_w, state.ghost_enemy_hp, ghost_color, theme.muted, 0.3);
        }

        // Main HP bar
        ui_render::small(engine, "HP", ex, bar_y, theme.muted);
        ui_render::bar(engine, bar_x, bar_y, bar_w, hp_pct, theme.hp_color(hp_pct), theme.muted, 0.3);
        ui_render::small(
            engine,
            &format!("{}/{}", enemy.hp, enemy.max_hp),
            ex + 5.8,
            bar_y,
            theme.hp_color(hp_pct),
        );

        // Tier info
        ui_render::small(
            engine,
            &format!("Tier: {:?}", enemy.tier),
            ex,
            ey - 1.2,
            theme.dim,
        );
    }
}

/// Get a display color for a status effect.
fn status_effect_color(effect: &StatusEffect, theme: &crate::theme::Theme) -> Vec4 {
    match effect {
        StatusEffect::Burning(_) => Vec4::new(1.0, 0.5, 0.1, 1.0),
        StatusEffect::Poisoned(_) => Vec4::new(0.3, 0.9, 0.2, 1.0),
        StatusEffect::Frozen(_) => Vec4::new(0.3, 0.7, 1.0, 1.0),
        StatusEffect::Stunned(_) => Vec4::new(1.0, 1.0, 0.3, 1.0),
        StatusEffect::Cursed(_) => Vec4::new(0.6, 0.0, 0.8, 1.0),
        StatusEffect::Blessed(_) => Vec4::new(1.0, 0.9, 0.4, 1.0),
        StatusEffect::Shielded(_) => Vec4::new(0.3, 0.6, 1.0, 1.0),
        StatusEffect::Enraged(_) => Vec4::new(1.0, 0.2, 0.1, 1.0),
        StatusEffect::Regenerating(_) => Vec4::new(0.2, 1.0, 0.4, 1.0),
        StatusEffect::Phasing(_) => Vec4::new(0.5, 0.5, 0.8, 0.7),
        StatusEffect::Empowered(_) => Vec4::new(1.0, 0.8, 0.2, 1.0),
        StatusEffect::DimensionalBleed(_) => Vec4::new(0.7, 0.1, 0.9, 1.0),
        _ => theme.primary,
    }
}

// ─── Status Effect Particles ─────────────────────────────────────────────────

/// Spawn ambient particles near player/enemy for each active status effect.
fn render_status_particles(state: &GameState, engine: &mut ProofEngine) {
    let t = state.frame as f32 * 0.016;

    // Player status particles
    if let Some(ref player) = state.player {
        for (si, effect) in player.status_effects.iter().enumerate() {
            spawn_status_particles_at(
                effect,
                PLAYER_CENTER,
                state.frame,
                si as u64,
                t,
                engine,
            );
        }
    }

    // Enemy status particles (enemies don't have status_effects on the struct,
    // but we can show visual hints based on floor_ability and special)
    // We also check if the combat_state has any enemy-relevant info
    if let Some(ref enemy) = state.enemy {
        // Visual indicator for special abilities
        if enemy.special_ability.is_some() {
            // Show a subtle ambient glow around the enemy
            let pulse = (t * 2.0).sin() * 0.3 + 0.7;
            for i in 0..2 {
                let angle = t * 1.5 + i as f32 * std::f32::consts::PI;
                let px = ENEMY_CENTER.x + angle.cos() * 1.2;
                let py = ENEMY_CENTER.y + angle.sin() * 0.8;
                engine.spawn_glyph(Glyph {
                    character: '.',
                    position: Vec3::new(px, py, 0.1),
                    scale: Vec2::splat(0.2),
                    color: Vec4::new(0.8, 0.2, 0.8, pulse * 0.5),
                    emission: 0.6,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }
    }
}

/// Spawn 2-3 particles near `center` for a given status effect.
fn spawn_status_particles_at(
    effect: &StatusEffect,
    center: Vec3,
    frame: u64,
    idx: u64,
    t: f32,
    engine: &mut ProofEngine,
) {
    let (ch, color, vy_dir) = match effect {
        StatusEffect::Burning(_) => (
            '^',
            Vec4::new(1.0, 0.5, 0.1, 0.7),
            1.0_f32, // rising
        ),
        StatusEffect::Frozen(_) => (
            '*',
            Vec4::new(0.3, 0.7, 1.0, 0.6),
            -1.0, // falling
        ),
        StatusEffect::Poisoned(_) => (
            '~',
            Vec4::new(0.3, 0.9, 0.2, 0.6),
            1.0, // rising
        ),
        StatusEffect::DimensionalBleed(_) => (
            '!',
            Vec4::new(1.0, 0.1, 0.2, 0.6),
            -0.5, // dripping
        ),
        StatusEffect::Enraged(_) => (
            '#',
            Vec4::new(1.0, 0.2, 0.1, 0.5),
            0.5,
        ),
        StatusEffect::Blessed(_) => (
            '+',
            Vec4::new(1.0, 0.9, 0.4, 0.5),
            0.8,
        ),
        StatusEffect::Shielded(_) => (
            '=',
            Vec4::new(0.3, 0.6, 1.0, 0.4),
            0.0,
        ),
        StatusEffect::Regenerating(_) => (
            '+',
            Vec4::new(0.2, 1.0, 0.4, 0.5),
            0.6,
        ),
        StatusEffect::Cursed(_) => (
            '?',
            Vec4::new(0.6, 0.0, 0.8, 0.5),
            -0.3,
        ),
        StatusEffect::Phasing(_) => (
            '.',
            Vec4::new(0.5, 0.5, 0.8, 0.3),
            0.2,
        ),
        StatusEffect::Empowered(_) => (
            '*',
            Vec4::new(1.0, 0.8, 0.2, 0.6),
            0.7,
        ),
        _ => (
            '.',
            Vec4::new(0.5, 0.5, 0.5, 0.4),
            0.3,
        ),
    };

    let particle_count = 3;
    for p in 0..particle_count {
        let seed = frame.wrapping_mul(31).wrapping_add(idx * 97).wrapping_add(p);
        let ox = hash_signed(seed * 13) * 1.2;
        let oy = hash_signed(seed * 17) * 0.8;
        let phase = hash_f32(seed * 23) * std::f32::consts::TAU;

        let px = center.x + ox + (t * 0.5 + phase).sin() * 0.2;
        let py = center.y + oy + t * vy_dir * 0.3 + (t * 0.3 + phase).cos() * 0.15;

        // Cycle alpha for twinkling
        let alpha = color.w * ((t * 2.0 + phase).sin() * 0.3 + 0.7).max(0.0);

        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(px, py, 0.05),
            velocity: Vec3::new(0.0, vy_dir * 0.5, 0.0),
            scale: Vec2::splat(0.18),
            color: Vec4::new(color.x, color.y, color.z, alpha),
            emission: 0.4,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            lifetime: 0.05, // very short — recreated each frame
            ..Default::default()
        });
    }
}

// ─── Combat Log ──────────────────────────────────────────────────────────────

/// Render scrollable combat log at bottom, last 5 entries.
fn render_combat_log(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    if state.combat_log_collapsed {
        ui_render::small(engine, "[L] Show Log", -8.0, -4.5, theme.dim);
        return;
    }

    // Log header
    ui_render::separator(engine, -8.0, -3.75, 10.0, theme.dim, 0.25);
    ui_render::text_z(engine, "Combat Log [L]", -4.5, -3.75, ui_render::Z_BORDER, theme.dim, 0.25, 0.2);

    // Separator line
    let sep_y = -4.05;
    for i in 0..30 {
        let sx = -8.0 + i as f32 * 0.5;
        if sx > 6.5 {
            break;
        }
        engine.spawn_glyph(Glyph {
            character: '-',
            position: Vec3::new(sx, sep_y, 0.0),
            scale: Vec2::splat(0.15),
            color: Vec4::new(theme.dim.x, theme.dim.y, theme.dim.z, 0.4),
            emission: 0.1,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }

    // Last 5 log entries
    let max_entries = 5;
    let start = state.combat_log.len().saturating_sub(max_entries);
    for (i, msg) in state.combat_log[start..].iter().enumerate() {
        // Truncate long messages
        let truncated: String = msg.chars().take(55).collect();
        // Newer entries are brighter
        let age_factor = (max_entries - i) as f32 / max_entries as f32;
        let dim_factor = 0.5 + age_factor * 0.5;
        let color = Vec4::new(
            theme.primary.x * dim_factor,
            theme.primary.y * dim_factor,
            theme.primary.z * dim_factor,
            0.6 + age_factor * 0.4,
        );
        let log_y = -4.2 - i as f32 * 0.38;
        ui_render::small(engine, &truncated, -8.0, log_y, color);
    }
}

// ─── Action Bar ──────────────────────────────────────────────────────────────

/// Show available actions with key hints. Highlight spells with enough mana.
fn render_action_bar(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = state.frame as f32 * 0.016;

    let action_y = -3.0;
    let mut ax = -8.0;

    // Core actions
    let actions: &[(&str, &str)] = &[
        ("[A]", "Attack"),
        ("[H]", "Heavy"),
        ("[D]", "Defend"),
        ("[F]", "Flee"),
        ("[T]", "Taunt"),
    ];

    for (key, label) in actions {
        // Key highlight: slight pulse
        let pulse = ((t * 3.0).sin() * 0.15 + 0.85).max(0.0);
        let key_color = Vec4::new(
            theme.accent.x * pulse,
            theme.accent.y * pulse,
            theme.accent.z * pulse,
            1.0,
        );
        ui_render::small(engine, key, ax, action_y, key_color);
        ax += key.len() as f32 * 0.25;
        ui_render::small(engine, label, ax, action_y, theme.primary);
        ax += label.len() as f32 * 0.25 + 0.4;
    }

    // Spell actions — show numbered if player has spells
    if let Some(ref player) = state.player {
        if !player.known_spells.is_empty() {
            let spell_y = action_y - 0.45;
            let mut sx = -8.0;
            ui_render::small(engine, "Spells:", sx, spell_y, theme.muted);
            sx += 2.2;

            for (i, spell) in player.known_spells.iter().enumerate().take(6) {
                let has_mana = state.current_mana >= spell.mana_cost;
                let on_cooldown = spell.current_cooldown > 0;

                let key_str = format!("[{}]", i + 1);
                let spell_label = if spell.name.len() > 8 {
                    format!("{}..", &spell.name[..6])
                } else {
                    spell.name.clone()
                };

                let color = if on_cooldown {
                    theme.dim
                } else if has_mana {
                    let school_c = spell_school_color(&spell.school);
                    Vec4::new(school_c.x, school_c.y, school_c.z, 1.0)
                } else {
                    Vec4::new(theme.muted.x, theme.muted.y, theme.muted.z, 0.6)
                };

                ui_render::small(engine, &key_str, sx, spell_y, color);
                sx += key_str.len() as f32 * 0.25;
                ui_render::small(engine, &spell_label, sx, spell_y, color);

                // Mana cost indicator
                let cost_str = format!("({}mp)", spell.mana_cost);
                sx += spell_label.len() as f32 * 0.25;
                ui_render::small(
                    engine,
                    &cost_str,
                    sx,
                    spell_y,
                    if has_mana { theme.mana } else { theme.dim },
                );
                sx += cost_str.len() as f32 * 0.25 + 0.3;
            }
        }
    }

    // Toggle hints at bottom
    ui_render::small(engine, "[V]iz [L]og", 5.5, action_y, theme.dim);
}

// ─── Screen Header ───────────────────────────────────────────────────────────

fn render_header(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = state.frame as f32 * 0.016;

    let header = if state.is_boss_fight {
        format!("COMBAT - Floor {} - BOSS", state.floor_num)
    } else {
        format!("COMBAT - Floor {}", state.floor_num)
    };

    // Boss header gets a pulsing danger color
    if state.is_boss_fight {
        let pulse = (t * 2.0).sin() * 0.2 + 0.8;
        let boss_heading = Vec4::new(
            theme.danger.x * pulse + theme.heading.x * (1.0 - pulse),
            theme.danger.y * pulse + theme.heading.y * (1.0 - pulse),
            theme.danger.z * pulse + theme.heading.z * (1.0 - pulse),
            1.0,
        );
        ui_render::heading_centered(engine, &header, 5.0, boss_heading);
    } else {
        ui_render::heading_centered(engine, &header, 5.0, theme.heading);
    }

    // Turn counter
    if let Some(ref combat) = state.combat_state {
        let turn_str = format!("Turn {}", combat.turn);
        ui_render::small(engine, &turn_str, 6.5, 5.0, theme.dim);
    }
}

// ─── Kill Linger Overlay ─────────────────────────────────────────────────────

/// Show victory or death overlay during kill_linger period.
fn render_kill_linger(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = state.frame as f32 * 0.016;
    let linger = state.kill_linger;

    if linger <= 0.0 {
        return;
    }

    let alpha = (linger * 2.0).min(1.0);

    match state.post_combat_screen {
        Some(AppScreen::FloorNav) => {
            // Victory
            let pulse = (t * 4.0).sin() * 0.2 + 1.0;
            ui_render::text_centered(
                engine,
                "VICTORY!",
                1.0,
                Vec4::new(
                    theme.gold.x * pulse,
                    theme.gold.y * pulse,
                    theme.gold.z * pulse,
                    alpha,
                ),
                0.7,
                1.2,
            );

            // Gold sparkle particles
            for i in 0..8 {
                let angle = t * 2.0 + i as f32 * 0.8;
                let r = 1.5 + (t * 1.5 + i as f32).sin() * 0.5;
                engine.spawn_glyph(Glyph {
                    character: '*',
                    position: Vec3::new(
                        angle.cos() * r,
                        1.0 + angle.sin() * r * 0.5,
                        0.3,
                    ),
                    scale: Vec2::splat(0.25),
                    color: Vec4::new(1.0, 0.9, 0.3, alpha * 0.7),
                    emission: 1.0,
                    glow_color: Vec3::new(1.0, 0.8, 0.0),
                    glow_radius: 0.4,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }

            // Show rewards if we have enemy info
            if let Some(ref enemy) = state.enemy {
                let xp_str = format!("+{} XP", enemy.xp_reward);
                let gold_str = format!("+{} Gold", enemy.gold_reward);
                ui_render::text_centered(
                    engine,
                    &xp_str,
                    0.2,
                    Vec4::new(theme.xp.x, theme.xp.y, theme.xp.z, alpha),
                    0.4,
                    0.6,
                );
                ui_render::text_centered(
                    engine,
                    &gold_str,
                    -0.4,
                    Vec4::new(theme.gold.x, theme.gold.y, theme.gold.z, alpha),
                    0.4,
                    0.6,
                );
            }
        }
        Some(AppScreen::GameOver) => {
            // Death
            let pulse = (t * 6.0).sin() * 0.3 + 0.7;
            ui_render::text_centered(
                engine,
                "DEFEATED",
                1.0,
                Vec4::new(
                    theme.danger.x * pulse,
                    theme.danger.y * 0.3,
                    theme.danger.z * 0.3,
                    alpha,
                ),
                0.7,
                1.5,
            );

            // Blood drip particles
            for i in 0..6 {
                let seed = (state.frame + i) as u64 * 4517;
                let dx = hash_signed(seed) * 3.0;
                let dy = 1.0 - (t * 0.8 + hash_f32(seed * 3) * 2.0) % 3.0;
                engine.spawn_glyph(Glyph {
                    character: '.',
                    position: Vec3::new(dx, dy, 0.3),
                    velocity: Vec3::new(0.0, -1.5, 0.0),
                    scale: Vec2::splat(0.2),
                    color: Vec4::new(0.8, 0.05, 0.05, alpha * 0.7),
                    emission: 0.5,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Additive,
                    lifetime: 0.08,
                    ..Default::default()
                });
            }
        }
        _ => {}
    }
}

// ─── Chaos Viz Overlay ───────────────────────────────────────────────────────

/// Optional chaos engine visualization (toggle with V).
fn render_chaos_viz(state: &GameState, engine: &mut ProofEngine) {
    if !state.chaos_viz_open {
        return;
    }
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = state.frame as f32 * 0.016;

    // ── Dark backing panel covering most of the screen ──
    ui_render::panel_bg(
        engine, -8.0, 5.0, 16.0, 10.0,
        Vec4::new(0.0, 0.0, 0.05, 0.92), 0.3,
    );
    ui_render::box_double(
        engine, -8.0, 5.0, 16.0, 10.0,
        Vec4::new(0.3, 0.2, 0.8, 0.9), 0.3, 0.4,
    );

    // ── Title with close hint ──
    let title_pulse = (t * 2.0).sin() * 0.1 + 0.9;
    let title_color = Vec4::new(
        theme.accent.x * title_pulse,
        theme.accent.y * title_pulse,
        theme.accent.z * title_pulse,
        1.0,
    );
    ui_render::text_centered(engine, "CHAOS ENGINE VISUALIZER", 4.5, title_color, 0.45, 0.7);
    ui_render::text(engine, "[V] Close", 4.5, 4.5, theme.dim, 0.25, 0.3);

    // ── Separator ──
    ui_render::separator(engine, -7.5, 4.0, 15.0, theme.border, 0.25);

    // Use state.last_roll (set after every combat action)
    if let Some(ref roll) = state.last_roll {
        // ── Final value and verdict ──
        let verdict = if roll.is_critical() {
            "CRITICAL"
        } else if roll.is_catastrophe() {
            "CATASTROPHE"
        } else if roll.final_value > 0.3 {
            "CLEAN HIT"
        } else if roll.final_value > -0.3 {
            "WEAK"
        } else {
            "MISS"
        };

        let verdict_color = if roll.is_critical() {
            Vec4::new(1.0, 0.9, 0.2, 1.0)
        } else if roll.is_catastrophe() {
            Vec4::new(1.0, 0.0, 0.3, 1.0)
        } else if roll.final_value > 0.3 {
            theme.success
        } else if roll.final_value > -0.3 {
            theme.warn
        } else {
            theme.danger
        };

        let final_str = format!("Final Value: {:+.4}   {}", roll.final_value, verdict);
        ui_render::text(engine, &final_str, -7.0, 3.5, verdict_color, 0.35, 0.6);

        // ── Progress bar: maps final_value from [-1,1] to [0,1] ──
        let bar_ratio = ((roll.final_value + 1.0) / 2.0).clamp(0.0, 1.0) as f32;
        ui_render::bar(
            engine, -7.0, 2.9, 14.0, bar_ratio,
            verdict_color, theme.muted, 0.3,
        );
        // Bar labels
        ui_render::text(engine, "-1.0", -7.0, 2.5, theme.dim, 0.2, 0.2);
        ui_render::text(engine, "0.0", -0.3, 2.5, theme.dim, 0.2, 0.2);
        ui_render::text(engine, "+1.0", 6.0, 2.5, theme.dim, 0.2, 0.2);

        // ── Separator before table ──
        ui_render::separator(engine, -7.5, 2.1, 15.0, theme.border, 0.25);

        // ── Chain steps table header ──
        ui_render::text(engine, " #  Engine Name          Input     Output    Delta", -7.0, 1.7, theme.accent, 0.25, 0.5);
        ui_render::separator(engine, -7.5, 1.4, 15.0, theme.dim, 0.2);

        // ── Chain steps rows ──
        let mut vy: f32 = 1.0;
        for (i, step) in roll.chain.iter().enumerate().take(12) {
            let delta = step.output - step.input;
            let delta_sign = if delta >= 0.0 { "+" } else { "" };

            let row_str = format!(
                "{:2}. {:<20} {:+.4}   {:+.4}   {}{:.4}",
                i + 1,
                step.engine_name,
                step.input,
                step.output,
                delta_sign,
                delta,
            );

            // Color code: green for positive delta, red for negative
            let row_color = if delta > 0.05 {
                theme.success
            } else if delta < -0.05 {
                theme.danger
            } else {
                theme.primary
            };

            ui_render::text(engine, &row_str, -7.0, vy, row_color, 0.25, 0.4);
            vy -= 0.35;
        }

        // If more steps than displayed
        if roll.chain.len() > 12 {
            let more = format!("  ... +{} more steps", roll.chain.len() - 12);
            ui_render::text(engine, &more, -7.0, vy, theme.dim, 0.22, 0.2);
            vy -= 0.35;
        }

        // ── Game value ──
        let game_str = format!("Game Value (mapped): {}", roll.game_value);
        ui_render::text(engine, &game_str, -7.0, vy - 0.2, theme.heading, 0.3, 0.5);
    } else {
        // No roll yet
        ui_render::text_centered(
            engine,
            "No chaos roll yet.",
            1.5,
            theme.dim,
            0.4,
            0.4,
        );
        ui_render::text_centered(
            engine,
            "Make a combat action to see the chaos pipeline trace.",
            0.5,
            theme.muted,
            0.3,
            0.3,
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════════

/// Process combat input and resolve actions.
pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let key_a = engine.input.just_pressed(Key::A) || engine.input.just_pressed(Key::Num1);
    let key_h = engine.input.just_pressed(Key::H) || engine.input.just_pressed(Key::Num2);
    let key_d = engine.input.just_pressed(Key::D) || engine.input.just_pressed(Key::Num3);
    let key_f = engine.input.just_pressed(Key::F);
    let key_t = engine.input.just_pressed(Key::T);
    let key_v = engine.input.just_pressed(Key::V);
    let key_l = engine.input.just_pressed(Key::L);

    // Spell keys: 4-9 map to UseSpell(0..5)
    let key_4 = engine.input.just_pressed(Key::Num4);
    let key_5 = engine.input.just_pressed(Key::Num5);
    let key_6 = engine.input.just_pressed(Key::Num6);
    let key_7 = engine.input.just_pressed(Key::Num7);
    let key_8 = engine.input.just_pressed(Key::Num8);
    let key_9 = engine.input.just_pressed(Key::Num9);

    if state.kill_linger > 0.0 {
        return;
    }

    if let (Some(ref mut player), Some(ref mut enemy), Some(ref mut combat)) =
        (&mut state.player, &mut state.enemy, &mut state.combat_state)
    {
        // Determine action from input
        let action = if key_a {
            Some(CombatAction::Attack)
        } else if key_h {
            Some(CombatAction::HeavyAttack)
        } else if key_d {
            Some(CombatAction::Defend)
        } else if key_f {
            Some(CombatAction::Flee)
        } else if key_t {
            Some(CombatAction::Taunt)
        } else if key_4 && !player.known_spells.is_empty() {
            Some(CombatAction::UseSpell(0))
        } else if key_5 && player.known_spells.len() > 1 {
            Some(CombatAction::UseSpell(1))
        } else if key_6 && player.known_spells.len() > 2 {
            Some(CombatAction::UseSpell(2))
        } else if key_7 && player.known_spells.len() > 3 {
            Some(CombatAction::UseSpell(3))
        } else if key_8 && player.known_spells.len() > 4 {
            Some(CombatAction::UseSpell(4))
        } else if key_9 && player.known_spells.len() > 5 {
            Some(CombatAction::UseSpell(5))
        } else {
            None
        };

        if let Some(ref action) = action {
            // Store previous HP for ghost bars
            let prev_player_hp = player.current_hp as f32 / player.max_hp.max(1) as f32;
            let prev_enemy_hp = enemy.hp as f32 / enemy.max_hp.max(1) as f32;

            // Set animation type
            match action {
                CombatAction::Attack => {
                    state.last_action_type = 1;
                    state.spell_beam_timer = 0.5;
                }
                CombatAction::HeavyAttack => {
                    state.last_action_type = 2;
                    state.spell_beam_timer = 0.6;
                }
                CombatAction::UseSpell(idx) => {
                    state.last_action_type = 3;
                    state.spell_beam_timer = 0.6;
                    if let Some(spell) = player.known_spells.get(*idx) {
                        state.spell_beam_color = spell_school_color(&spell.school);
                        state.last_spell_name = spell.name.clone();
                    }
                }
                CombatAction::Defend => {
                    state.last_action_type = 4;
                    state.spell_beam_timer = 0.5;
                }
                CombatAction::Taunt => {
                    state.last_action_type = 1;
                    state.spell_beam_timer = 0.3;
                }
                _ => {
                    state.last_action_type = 0;
                }
            }

            let (events, outcome) = resolve_action(player, enemy, action.clone(), combat);

            // Process events for visual feedback
            for event in &events {
                state.combat_log.push(event.to_display_string());

                match event {
                    chaos_rpg_core::combat::CombatEvent::PlayerAttack { damage, is_crit } => {
                        state.enemy_flash = 0.4;
                        state.enemy_flash_color = if *is_crit {
                            Vec4::new(1.0, 0.9, 0.2, 1.0) // gold for crits
                        } else {
                            Vec4::new(1.0, 1.0, 1.0, 1.0) // white for normal
                        };
                        // Screen shake scaled to damage
                        let shake = (*damage as f32 / 50.0).clamp(0.05, 0.4);
                        engine.add_trauma(shake);

                        // Ghost bar for enemy
                        state.ghost_enemy_hp = prev_enemy_hp;
                        state.ghost_enemy_timer = 1.5;
                    }
                    chaos_rpg_core::combat::CombatEvent::EnemyAttack { damage, is_crit } => {
                        state.player_flash = 0.4;
                        state.last_action_type = 5; // override to show enemy attack trail
                        state.spell_beam_timer = 0.4;

                        let shake = (*damage as f32 / 40.0).clamp(0.05, 0.5);
                        engine.add_trauma(shake);

                        // Ghost bar for player
                        state.ghost_player_hp = prev_player_hp;
                        state.ghost_player_timer = 1.5;
                    }
                    chaos_rpg_core::combat::CombatEvent::SpellCast { damage, backfired, .. } => {
                        let shake = (*damage as f32 / 40.0).clamp(0.05, 0.5);
                        engine.add_trauma(shake);
                        if *backfired {
                            state.player_flash = 0.5;
                            state.ghost_player_hp = prev_player_hp;
                            state.ghost_player_timer = 1.5;
                        } else {
                            state.enemy_flash = 0.5;
                            state.ghost_enemy_hp = prev_enemy_hp;
                            state.ghost_enemy_timer = 1.5;
                        }
                    }
                    chaos_rpg_core::combat::CombatEvent::PlayerDefend { .. } => {
                        state.player_flash = 0.2;
                    }
                    chaos_rpg_core::combat::CombatEvent::PlayerHealed { .. } => {
                        state.player_flash = 0.3;
                        state.ghost_player_hp = prev_player_hp;
                        state.ghost_player_timer = 1.0;
                    }
                    _ => {}
                }
            }

            // ── Chaos trace in combat log ──
            // After each action resolves, append the chaos chain to the log
            if let Some(ref roll) = state.last_roll {
                for step in &roll.chain {
                    let delta = step.output - step.input;
                    state.combat_log.push(format!(
                        "[{}] {:.2} -> {:.2} ({:+.2})",
                        step.engine_name, step.input, step.output, delta,
                    ));
                }
                let verdict = if roll.is_critical() {
                    "CRITICAL"
                } else if roll.is_catastrophe() {
                    "CATASTROPHE"
                } else if roll.final_value > 0.3 {
                    "CLEAN HIT"
                } else if roll.final_value > -0.3 {
                    "WEAK"
                } else {
                    "MISS"
                };
                state.combat_log.push(format!(
                    "Final: {:+.3} {}",
                    roll.final_value, verdict,
                ));
            }

            // ── Audio bridge: emit SFX for combat action ──
            {
                let action_type = state.last_action_type;
                let dmg = events.iter().filter_map(|e| match e {
                    chaos_rpg_core::combat::CombatEvent::PlayerAttack { damage, .. } => Some(*damage),
                    chaos_rpg_core::combat::CombatEvent::SpellCast { damage, .. } => Some(*damage),
                    _ => None,
                }).sum::<i64>();
                let is_crit = state.last_roll.as_ref().map(|r| r.is_critical()).unwrap_or(false);
                crate::audio_bridge::on_combat_action(engine, action_type, dmg, is_crit);
            }

            match outcome {
                CombatOutcome::Ongoing => {}
                CombatOutcome::PlayerWon { xp, gold } => {
                    // Audio: enemy death SFX
                    crate::audio_bridge::on_enemy_death(engine, state.is_boss_fight);

                    // Full game logic: XP, gold, loot, level up, gauntlet, nemesis
                    let level_before = state.player.as_ref().map(|p| p.level).unwrap_or(0);
                    crate::game_logic::on_combat_victory(state, xp, gold);

                    // Audio: level up check
                    let level_after = state.player.as_ref().map(|p| p.level).unwrap_or(0);
                    if level_after > level_before {
                        crate::audio_bridge::on_level_up(engine);
                    }

                    // Audio: victory vibe
                    engine.emit_audio(AudioEvent::SetMusicVibe(
                        MusicVibe::Victory,
                    ));

                    engine.add_trauma(0.6);
                }
                CombatOutcome::PlayerDied => {
                    // Audio: player death SFX + death vibe
                    crate::audio_bridge::on_player_death(engine);
                    engine.emit_audio(AudioEvent::SetMusicVibe(
                        MusicVibe::Death,
                    ));

                    // Full game logic: nemesis promotion, score saving, death screen
                    crate::game_logic::on_player_death(state);
                    engine.add_trauma(0.8);
                }
                CombatOutcome::PlayerFled => {
                    crate::game_logic::on_player_fled(state);
                }
            }
        }
    }

    if key_v {
        state.chaos_viz_open = !state.chaos_viz_open;
    }
    if key_l {
        state.combat_log_collapsed = !state.combat_log_collapsed;
    }
}

/// Render the complete combat screen.
pub fn render(state: &GameState, engine: &mut ProofEngine) {
    // Dark backing for text readability
    crate::ui_render::screen_backing(engine, 0.5);

    // 1. Arena floor (background layer)
    render_arena_floor(state, engine);

    // 2. Player entity formation
    render_player_entity(state, engine);

    // 3. Enemy entity formation
    render_enemy_entity(state, engine);

    // 4. Attack animation trails
    render_attack_animation(state, engine);

    // 5. Damage numbers
    render_damage_numbers(state, engine);

    // 6. HP / MP bars with ghost effect
    render_hp_mp_bars(state, engine);

    // 7. Status effect particles
    render_status_particles(state, engine);

    // 8. Screen shake is handled in update() via engine.add_trauma()

    // 9. Combat log
    render_combat_log(state, engine);

    // 10. Header
    render_header(state, engine);

    // 11. Action bar with spell highlights
    render_action_bar(state, engine);

    // 12. Kill linger overlay (victory / death)
    render_kill_linger(state, engine);

    // 13. Chaos viz overlay (optional)
    render_chaos_viz(state, engine);

    // 14. Boss overlay — always last
    crate::effects::boss_visuals::render_boss_overlay(state, engine);

    // 15. Auto-play indicator
    crate::auto_play::render_indicator(state, engine);

    // combat_visuals and combat_hud disabled — their content duplicates
    // what combat.rs already renders, causing overlapping unreadable text.
}
