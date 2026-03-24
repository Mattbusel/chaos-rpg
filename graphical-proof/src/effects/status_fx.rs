//! Status effect ambient particle visuals.

use proof_engine::prelude::*;

/// Status effect bitmask constants.
pub const STATUS_BURN: u32 = 1;
pub const STATUS_FREEZE: u32 = 2;
pub const STATUS_POISON: u32 = 4;
pub const STATUS_BLEED: u32 = 8;
pub const STATUS_STUN: u32 = 16;
pub const STATUS_REGEN: u32 = 32;

/// Spawn ambient particles for active status effects on an entity.
pub fn emit_status_particles(
    engine: &mut ProofEngine,
    position: Vec3,
    flags: u32,
    frame: u64,
) {
    // Rate-limit: only emit every few frames
    if frame % 6 != 0 { return; }

    if flags & STATUS_BURN != 0 {
        // Orange sparks floating upward
        engine.spawn_glyph(Glyph {
            character: '·',
            position: position + Vec3::new(
                ((frame as f32 * 0.7).sin()) * 0.8,
                1.5,
                0.0,
            ),
            color: Vec4::new(1.0, 0.43, 0.08, 0.8),
            emission: 1.0,
            life_function: Some(MathFunction::Breathing { rate: 3.0, depth: 0.5 }),
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    if flags & STATUS_FREEZE != 0 {
        engine.spawn_glyph(Glyph {
            character: '❄',
            position: position + Vec3::new(
                ((frame as f32 * 0.5).cos()) * 1.0,
                -0.5,
                0.0,
            ),
            color: Vec4::new(0.3, 0.63, 1.0, 0.7),
            emission: 0.6,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    if flags & STATUS_POISON != 0 {
        engine.spawn_glyph(Glyph {
            character: 'o',
            position: position + Vec3::new(
                ((frame as f32 * 0.3).sin()) * 0.6,
                1.0,
                0.0,
            ),
            color: Vec4::new(0.16, 0.82, 0.27, 0.7),
            emission: 0.5,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    if flags & STATUS_BLEED != 0 {
        engine.spawn_glyph(Glyph {
            character: '▪',
            position: position + Vec3::new(
                ((frame as f32 * 0.9).cos()) * 0.5,
                -0.3,
                0.0,
            ),
            color: Vec4::new(0.78, 0.08, 0.08, 0.8),
            emission: 0.4,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}
