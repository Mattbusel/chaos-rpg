//! Combat scene builder — sets up the 3D arena for combat.
//!
//! Spawns floor grid glyphs, ambient force fields based on room type,
//! and entity lights for player and enemy.

use proof_engine::prelude::*;
use crate::state::GameState;
use crate::theme::THEMES;
use crate::lighting::{SceneLighting, RoomLighting};

/// Build the combat arena: floor grid, ambient fields, entity lights.
pub fn build_arena(state: &GameState, engine: &mut ProofEngine) -> SceneLighting {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let center = Vec3::ZERO;

    // Determine room lighting type
    let room_type = if state.is_boss_fight {
        RoomLighting::Boss
    } else {
        RoomLighting::Combat
    };

    // Set up lighting
    let mut lighting = SceneLighting::new();
    lighting.setup_room(room_type, state.floor_num, center);

    // Spawn floor grid glyphs (perspective grid)
    let grid_size = 12;
    for gx in -grid_size..=grid_size {
        for gz in -grid_size..=grid_size {
            let x = gx as f32 * 1.2;
            let z = gz as f32 * 1.2;
            // Perspective: y decreases with distance (z)
            let y = -8.0 + z * 0.15;
            let dist = ((gx * gx + gz * gz) as f32).sqrt();
            let fade = (1.0 - dist / grid_size as f32).max(0.0) * 0.15;

            if fade > 0.01 {
                let ch = if (gx + gz) % 2 == 0 { '·' } else { ' ' };
                engine.spawn_glyph(Glyph {
                    character: ch,
                    position: Vec3::new(x, y, z),
                    color: Vec4::new(theme.muted.x * fade, theme.muted.y * fade, theme.muted.z * fade, fade),
                    emission: fade * 0.2,
                    layer: RenderLayer::World,
                    ..Default::default()
                });
            }
        }
    }

    // Add entity lights
    let player_pos = Vec3::new(-6.0, 0.0, 0.0);
    let enemy_pos = Vec3::new(6.0, 0.0, 0.0);
    lighting.add_player_light(player_pos);

    let enemy_tint = state.enemy.as_ref()
        .map(|e| crate::lighting::enemy_element_tint(&e.name))
        .flatten();
    lighting.add_enemy_light(enemy_pos, enemy_tint);

    // Apply boss lighting if applicable
    if let Some(boss_id) = state.boss_id {
        crate::lighting::apply_boss_lighting(&mut lighting, boss_id, state.boss_turn, center);
    }

    lighting
}
