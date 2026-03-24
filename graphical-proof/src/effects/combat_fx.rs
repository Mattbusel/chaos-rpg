//! Combat visual effects — damage numbers, hit sparks, death explosions.

use proof_engine::prelude::*;
use proof_engine::integration::GameEvent;

/// Spawn a floating damage number.
pub fn damage_number(engine: &mut ProofEngine, amount: i64, position: Vec3, is_crit: bool) {
    engine.dispatch(GameEvent::DamageNumber {
        amount: amount as f32,
        position,
        critical: is_crit,
    });
}

/// Trigger screen shake scaled to damage.
pub fn hit_shake(engine: &mut ProofEngine, damage: i64, is_crit: bool) {
    let intensity = if is_crit {
        (damage as f32 / 200.0).clamp(0.3, 0.8)
    } else {
        (damage as f32 / 500.0).clamp(0.05, 0.3)
    };
    engine.dispatch(GameEvent::ScreenShake { intensity });
}

/// Trigger enemy death explosion.
pub fn death_explosion(engine: &mut ProofEngine, position: Vec3) {
    engine.dispatch(GameEvent::EntityDeath { position });
}
