//! Combat visual effects — damage numbers, hit sparks, death explosions.
//!
//! Now integrated with the physics bridge for debris, fluids, and weapon trails.

use proof_engine::prelude::*;
use proof_engine::integration::GameEvent;
use crate::physics_bridge::PhysicsBridge;

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

/// Trigger enemy death explosion with physics debris and elemental fluids.
pub fn death_explosion(engine: &mut ProofEngine, position: Vec3) {
    engine.dispatch(GameEvent::EntityDeath { position });
}

/// Trigger a physics-driven enemy death: debris scatter + element-specific
/// fluid effects via the physics bridge.
pub fn death_explosion_physics(
    physics: &mut PhysicsBridge,
    engine: &mut ProofEngine,
    position: Vec3,
    glyphs: &[char],
    colors: &[[f32; 4]],
    element: &str,
) {
    // Dispatch the engine-level event for screen effects
    engine.dispatch(GameEvent::EntityDeath { position });

    // Spawn debris and fluids via the physics bridge
    physics.on_enemy_death(position, glyphs, colors, element);
}

/// Trigger a weapon impact with physics-driven debris and damage numbers.
pub fn weapon_impact_physics(
    physics: &mut PhysicsBridge,
    engine: &mut ProofEngine,
    contact_point: Vec3,
    weapon_type: &str,
    damage: i64,
    is_crit: bool,
) {
    // Screen shake
    hit_shake(engine, damage, is_crit);

    // Damage number via engine
    damage_number(engine, damage, contact_point + Vec3::new(0.0, 1.0, 0.0), is_crit);

    // Physics-driven impact (debris, trail compression, etc.)
    physics.on_weapon_impact(contact_point, weapon_type, damage, is_crit);
}

/// Trigger a bleed tick effect — dripping blood fluid.
pub fn bleed_tick_physics(physics: &mut PhysicsBridge, entity_pos: Vec3) {
    physics.on_bleed_tick(entity_pos);
}

/// Trigger a spell cast effect — element-specific fluid spawning.
pub fn spell_cast_physics(physics: &mut PhysicsBridge, spell_element: &str, target_pos: Vec3) {
    physics.on_spell_cast(spell_element, target_pos);
}

/// Begin a weapon swing trail via the physics bridge.
pub fn weapon_swing_physics(
    physics: &mut PhysicsBridge,
    weapon_type: &str,
    arc_start: f32,
    arc_end: f32,
    origin: Vec3,
) {
    physics.on_weapon_swing(weapon_type, arc_start, arc_end, origin);
}
