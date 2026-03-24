//! Enemy entities — AmorphousEntity scaled by tier.

use proof_engine::prelude::*;

/// Build an enemy entity with glyph count scaled by tier.
pub fn build_enemy_entity(name: &str, tier: u32, position: Vec3) -> AmorphousEntity {
    let glyph_count = match tier {
        0..=1 => 15,
        2..=3 => 25,
        4..=5 => 35,
        _ => 50,
    };

    let color = Vec4::new(0.9, 0.25, 0.2, 1.0);
    let chars: Vec<char> = name.chars().chain("░▒▓█●◆".chars()).collect();

    let mut entity = AmorphousEntity::default();
    entity.position = position;
    entity.entity_mass = 30.0 + tier as f32 * 10.0;

    let mut positions = Vec::new();
    let mut formation_chars = Vec::new();
    let mut colors = Vec::new();

    // Ring formation for enemies
    let rings = (glyph_count as f32).sqrt() as usize;
    let mut placed = 0;
    for ring in 0..rings {
        let r = (ring + 1) as f32 * 0.5;
        let count = (ring + 1) * 6;
        for i in 0..count {
            if placed >= glyph_count { break; }
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
            positions.push(Vec3::new(angle.cos() * r, angle.sin() * r, 0.0));
            formation_chars.push(chars[placed % chars.len()]);
            colors.push(color);
            placed += 1;
        }
    }

    entity.formation = positions;
    entity.formation_chars = formation_chars;
    entity.formation_colors = colors;
    entity
}
