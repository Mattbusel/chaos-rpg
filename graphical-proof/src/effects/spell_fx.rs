//! Per-element spell visual effects.

use proof_engine::prelude::*;
use proof_engine::integration::GameEvent;

/// Spell impact based on element name.
pub fn spell_impact(engine: &mut ProofEngine, spell_name: &str, position: Vec3) {
    let (color, radius) = element_visual(spell_name);
    engine.dispatch(GameEvent::SpellImpact {
        position,
        color,
        radius,
    });
}

fn element_visual(spell_name: &str) -> (Vec4, f32) {
    let name = spell_name.to_lowercase();
    if name.contains("fire") || name.contains("burn") || name.contains("blaze") {
        (Vec4::new(1.0, 0.4, 0.1, 1.0), 2.0) // orange-red
    } else if name.contains("ice") || name.contains("frost") || name.contains("freeze") {
        (Vec4::new(0.3, 0.7, 1.0, 1.0), 1.8) // ice blue
    } else if name.contains("lightning") || name.contains("shock") || name.contains("thunder") {
        (Vec4::new(1.0, 1.0, 0.5, 1.0), 2.5) // bright yellow-white
    } else if name.contains("necrotic") || name.contains("death") || name.contains("drain") {
        (Vec4::new(0.3, 0.8, 0.2, 1.0), 1.5) // sickly green
    } else if name.contains("arcane") || name.contains("chaos") || name.contains("void") {
        (Vec4::new(0.6, 0.2, 1.0, 1.0), 2.2) // deep purple
    } else if name.contains("heal") || name.contains("restore") || name.contains("divine") {
        (Vec4::new(0.3, 1.0, 0.5, 1.0), 1.5) // golden green
    } else {
        (Vec4::new(0.8, 0.8, 1.0, 1.0), 1.5) // default white-blue
    }
}
