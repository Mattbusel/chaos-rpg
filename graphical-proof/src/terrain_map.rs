//! Terrain-based floor map — isometric mathematical surface.
//!
//! The floor map is rendered as a 3D terrain surface generated from noise.
//! Each room sits at a noise-evaluated height. Room type affects local terrain.

use proof_engine::prelude::*;
use crate::state::GameState;
use crate::theme::THEMES;

// ═══════════════════════════════════════════════════════════════════════════════
// TERRAIN GENERATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Simple value noise for terrain height.
fn value_noise(x: f32, y: f32, seed: u64) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let fx = x - ix as f32;
    let fy = y - iy as f32;
    let u = fx * fx * (3.0 - 2.0 * fx);
    let v = fy * fy * (3.0 - 2.0 * fy);

    let hash = |px: i32, py: i32| -> f32 {
        let h = ((px as u64).wrapping_mul(73856093) ^ (py as u64).wrapping_mul(19349663) ^ seed)
            .wrapping_mul(2654435761);
        (h & 0xFFFF) as f32 / 65535.0
    };

    let c00 = hash(ix, iy);
    let c10 = hash(ix + 1, iy);
    let c01 = hash(ix, iy + 1);
    let c11 = hash(ix + 1, iy + 1);

    let a = c00 + (c10 - c00) * u;
    let b = c01 + (c11 - c01) * u;
    a + (b - a) * v
}

/// FBM (fractional Brownian motion) for terrain.
fn fbm(x: f32, y: f32, octaves: u32, seed: u64) -> f32 {
    let mut sum = 0.0_f32;
    let mut amp = 1.0_f32;
    let mut freq = 1.0_f32;
    let mut max_amp = 0.0_f32;
    for i in 0..octaves {
        sum += value_noise(x * freq, y * freq, seed.wrapping_add(i as u64 * 1000)) * amp;
        max_amp += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / max_amp
}

/// Get terrain height at world position, with room-type modifiers.
fn terrain_height(x: f32, z: f32, floor: u32, seed: u64) -> f32 {
    let base = fbm(x * 0.15, z * 0.15, 4, seed) * 2.0 - 1.0;
    // Floor depth affects terrain amplitude
    let amplitude = match floor {
        0..=10  => 0.5,   // gentle rolling hills
        11..=25 => 1.0,   // steeper
        26..=50 => 1.5,   // jagged
        51..=75 => 0.3,   // mostly flat void with pillars
        76..=99 => 0.8,   // industrial
        _       => 0.1,   // pure void
    };
    base * amplitude
}

/// Get the glyph character for terrain at a given floor depth.
fn terrain_glyph(height: f32, floor: u32, x: f32, z: f32, seed: u64) -> char {
    let noise_val = value_noise(x * 0.5, z * 0.5, seed.wrapping_add(999));
    match floor {
        0..=10 => {
            // Axiom Age: orderly grid, grass
            let chars = ['.', ',', '\'', '.', '·', '.'];
            chars[(noise_val * chars.len() as f32) as usize % chars.len()]
        }
        11..=25 => {
            // Expansion: mathematical symbols
            let chars = ['+', '-', '=', '·', '*', '/', '~'];
            chars[(noise_val * chars.len() as f32) as usize % chars.len()]
        }
        26..=50 => {
            // Recursion: crystalline
            let chars = ['♦', '◊', '✦', '·', '*', '#', '^'];
            chars[(noise_val * chars.len() as f32) as usize % chars.len()]
        }
        51..=75 => {
            // Collapse: flat void with pillars
            if height.abs() > 0.2 { '█' } else { ' ' }
        }
        76..=99 => {
            // Industrial
            let chars = ['═', '║', '╬', '╦', '╩', '─', '│'];
            chars[(noise_val * chars.len() as f32) as usize % chars.len()]
        }
        _ => {
            // Pure void
            if noise_val > 0.85 { '·' } else { ' ' }
        }
    }
}

/// Room-type elevation modifier.
fn room_elevation(room_type: &chaos_rpg_core::world::RoomType) -> f32 {
    use chaos_rpg_core::world::RoomType;
    match room_type {
        RoomType::Boss => 1.5,           // elevated plateau
        RoomType::Trap => -1.0,          // sunken valley
        RoomType::Shrine => 0.8,         // smooth hilltop
        RoomType::ChaosRift => 0.0,      // fractured (handled by noise)
        RoomType::Portal => 2.0,         // floating island
        RoomType::Treasure => 0.3,       // slight rise
        RoomType::Shop => 0.2,           // flat platform
        RoomType::CraftingBench => 0.1,  // workshop level
        _ => 0.0,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TERRAIN RENDERER
// ═══════════════════════════════════════════════════════════════════════════════

/// Render the floor map as an isometric terrain surface.
pub fn render_terrain_map(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let floor = state.floor_num;
    let seed = state.floor_seed;

    let floor_data = match &state.floor {
        Some(f) => f,
        None => return,
    };

    let room_count = floor_data.rooms.len();
    let current = floor_data.current_room;

    // Render terrain grid (isometric projection)
    let grid_w = 20;
    let grid_h = 12;
    let iso_scale = 0.6;

    for gz in 0..grid_h {
        for gx in 0..grid_w {
            let world_x = (gx as f32 - grid_w as f32 / 2.0) * 1.5;
            let world_z = (gz as f32 - grid_h as f32 / 2.0) * 1.5;

            let height = terrain_height(world_x + seed as f32 * 0.001, world_z, floor, seed);

            // Isometric projection
            let iso_x = (world_x - world_z) * iso_scale;
            let iso_y = (world_x + world_z) * iso_scale * 0.5 + height * 0.8;

            // Check if a room is near this grid position
            let room_idx = ((gx as f32 / grid_w as f32) * room_count as f32) as usize;
            let is_room_pos = room_idx < room_count && (gz == grid_h / 2);
            let room_height = if is_room_pos {
                room_elevation(&floor_data.rooms[room_idx].room_type)
            } else {
                0.0
            };

            let total_height = height + room_height;
            let ch = if is_room_pos {
                room_glyph(&floor_data.rooms[room_idx].room_type)
            } else {
                terrain_glyph(height, floor, world_x, world_z, seed)
            };

            // Color based on height and room
            let base_brightness = if is_room_pos {
                if room_idx == current { 0.8 } else { 0.4 }
            } else {
                (0.1 + (total_height + 1.0) * 0.1).clamp(0.05, 0.3)
            };

            let color = if is_room_pos && room_idx == current {
                theme.selected
            } else if is_room_pos {
                theme.primary
            } else {
                Vec4::new(
                    theme.muted.x * base_brightness,
                    theme.muted.y * base_brightness,
                    theme.muted.z * base_brightness,
                    base_brightness,
                )
            };

            // Position in screen space
            let screen_x = iso_x - 2.0;
            let screen_y = iso_y - 6.0;

            if ch != ' ' {
                engine.spawn_glyph(Glyph {
                    character: ch,
                    position: Vec3::new(screen_x, screen_y, -1.0),
                    color,
                    emission: base_brightness * 0.3,
                    layer: RenderLayer::World,
                    ..Default::default()
                });
            }
        }
    }

    // Room labels (above terrain)
    for (i, room) in floor_data.rooms.iter().enumerate() {
        let frac = i as f32 / room_count.max(1) as f32;
        let world_x = (frac - 0.5) * grid_w as f32 * 1.5;
        let world_z = 0.0;
        let height = terrain_height(world_x + seed as f32 * 0.001, world_z, floor, seed);
        let room_h = room_elevation(&room.room_type);

        let iso_x = (world_x - world_z) * iso_scale - 2.0;
        let iso_y = (world_x + world_z) * iso_scale * 0.5 + (height + room_h) * 0.8 - 5.0;

        let is_current = i == current;
        let label_color = if is_current { theme.selected } else { theme.dim };
        let label = format!("{}", i + 1);
        render_text(engine, &label, iso_x, iso_y, label_color, if is_current { 0.6 } else { 0.2 });
    }
}

fn room_glyph(room_type: &chaos_rpg_core::world::RoomType) -> char {
    use chaos_rpg_core::world::RoomType;
    match room_type {
        RoomType::Combat => '×',
        RoomType::Boss => '★',
        RoomType::Treasure => '♦',
        RoomType::Shop => '$',
        RoomType::Shrine => '~',
        RoomType::Trap => '!',
        RoomType::Portal => '^',
        RoomType::Empty => '·',
        RoomType::ChaosRift => '⚡',
        RoomType::CraftingBench => '⚒',
    }
}

fn render_text(engine: &mut ProofEngine, text: &str, x: f32, y: f32, color: Vec4, emission: f32) {
    for (i, ch) in text.chars().enumerate() {
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x + i as f32 * 0.45, y, 0.0),
            color, emission,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}
