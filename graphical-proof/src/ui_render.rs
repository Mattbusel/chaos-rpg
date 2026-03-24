//! UI rendering with correct screen-space positioning.
//!
//! Camera is at (0, 0, -10) looking at origin with look_at_rh.
//! This gives: +X = screen right, +Y = screen up.
//! Visible area at z=0: roughly ±8.7 x, ±5.4 y (at 16:10 aspect).
//!
//! Scale guide:
//!   0.25 = tiny (debug text, ~70 chars per line)
//!   0.3  = small (hints, labels, ~58 chars)
//!   0.4  = body text (~43 chars)
//!   0.45 = large body (~38 chars)
//!   0.7  = heading (~25 chars)
//!   1.3  = title (~13 chars)

use proof_engine::prelude::*;

fn spacing(scale: f32) -> f32 { scale * 0.85 }

/// Core text renderer. +X = screen right (camera at -Z fixes the mirror).
pub fn text(engine: &mut ProofEngine, s: &str, x: f32, y: f32, color: Vec4, scale: f32, emission: f32) {
    let sp = spacing(scale);
    for (i, ch) in s.chars().enumerate() {
        if ch == ' ' { continue; }
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x + i as f32 * sp, y, 0.0),
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
            position: Vec3::new(x + i as f32 * sp, y, 0.0),
            scale: Vec2::splat(scale), color: c, emission: em,
            layer: RenderLayer::UI, ..Default::default()
        });
    }
}
