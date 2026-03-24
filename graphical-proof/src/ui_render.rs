//! UI rendering with correct screen-space positioning.
//!
//! Camera: z=10, look_at_rh toward origin, FOV=60°.
//! With look_at_rh, +X world = LEFT on screen. So we negate X to render
//! text left-to-right. Visible area at z=0: roughly ±8.7 x, ±5.4 y.

use proof_engine::prelude::*;

fn spacing(scale: f32) -> f32 { scale * 0.85 }

/// Core text renderer. Negates X so text reads left-to-right on screen.
pub fn text(engine: &mut ProofEngine, s: &str, x: f32, y: f32, color: Vec4, scale: f32, emission: f32) {
    let sp = spacing(scale);
    // Negate X: in RH look_at from +Z, world +X = screen LEFT.
    // We want character 0 on the left (screen), so it needs the most positive world X,
    // and character N on the right (screen) needs the most negative world X.
    for (i, ch) in s.chars().enumerate() {
        if ch == ' ' { continue; }
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(-(x + i as f32 * sp), y, 0.0),
            scale: Vec2::splat(scale),
            color, emission,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}

/// Render text centered horizontally.
pub fn text_centered(engine: &mut ProofEngine, s: &str, y: f32, color: Vec4, scale: f32, emission: f32) {
    let w = s.len() as f32 * spacing(scale);
    // Center: start at +w/2 in world (screen-left), end at -w/2 (screen-right)
    text(engine, s, -w * 0.5, y, color, scale, emission);
}

pub fn heading_centered(engine: &mut ProofEngine, s: &str, y: f32, color: Vec4) {
    text_centered(engine, s, y, color, 0.7, 0.7);
}

pub fn body(engine: &mut ProofEngine, s: &str, x: f32, y: f32, color: Vec4) {
    text(engine, s, x, y, color, 0.4, 0.5);
}

pub fn small(engine: &mut ProofEngine, s: &str, x: f32, y: f32, color: Vec4) {
    text(engine, s, x, y, color, 0.3, 0.3);
}

pub fn title(engine: &mut ProofEngine, s: &str, y: f32, color: Vec4) {
    text_centered(engine, s, y, color, 1.3, 1.2);
}

pub fn bar(engine: &mut ProofEngine, x: f32, y: f32, width: f32, ratio: f32, fill: Vec4, empty: Vec4, scale: f32) {
    let sp = spacing(scale);
    let n = (width / sp) as usize;
    let filled = ((ratio.clamp(0.0, 1.0) * n as f32) as usize).min(n);
    for i in 0..n {
        let (ch, c, em) = if i < filled { ('\u{2588}', fill, 0.5) } else { ('\u{2591}', empty, 0.1) };
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(-(x + i as f32 * sp), y, 0.0),
            scale: Vec2::splat(scale), color: c, emission: em,
            layer: RenderLayer::UI, ..Default::default()
        });
    }
}
