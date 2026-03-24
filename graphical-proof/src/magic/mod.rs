//! Magic visual system — spell casting, status effects, and boss-specific magic.
//!
//! This module provides spectacular glyph-based visuals for every element,
//! spell type, status effect, and boss encounter in CHAOS RPG.
//!
//! All rendering is immediate-mode: glyphs are spawned fresh each frame via
//! `crate::ui_render` conventions and `engine.spawn_glyph()`.

pub mod spell_visuals;
pub mod status_visuals;
pub mod boss_magic;

pub use spell_visuals::{
    Element, SpellVisualStage, SpellVisual, SpellVisualManager,
};
pub use status_visuals::{
    StatusEffect, StatusVisualManager,
};
pub use boss_magic::{
    BossMagicProfile, BossMagicRenderer,
};

/// Convenience: render a short text string as glyphs at a world position.
/// Used internally by all magic visual subsystems.
pub(crate) fn render_magic_text(
    engine: &mut proof_engine::ProofEngine,
    text: &str,
    x: f32,
    y: f32,
    color: glam::Vec4,
    emission: f32,
    layer: proof_engine::glyph::RenderLayer,
) {
    use proof_engine::prelude::*;
    let sp = 0.35;
    for (i, ch) in text.chars().enumerate() {
        if ch == ' ' { continue; }
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x + i as f32 * sp, y, 0.0),
            color,
            emission,
            layer,
            ..Default::default()
        });
    }
}
