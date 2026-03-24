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

/// Z-layers for proper depth ordering.
pub const Z_BG: f32 = 0.5;       // behind everything
pub const Z_PANEL: f32 = 0.3;    // panel backgrounds
pub const Z_BORDER: f32 = 0.2;   // box borders
pub const Z_TEXT: f32 = 0.0;     // normal text (default)
pub const Z_OVERLAY: f32 = -0.3; // overlay/popup text
pub const Z_TOP: f32 = -0.5;     // topmost (tooltips, debug)

fn spacing(scale: f32) -> f32 { scale * 0.85 }

/// Core text renderer.
/// Engine coordinate system: +X = screen LEFT, +Y = screen UP (look_at_rh from +Z).
/// We negate X so callers can think in normal screen coordinates (+X = right).
pub fn text(engine: &mut ProofEngine, s: &str, x: f32, y: f32, color: Vec4, scale: f32, emission: f32) {
    text_z(engine, s, x, y, Z_TEXT, color, scale, emission);
}

/// Text at a specific Z depth. Callers use +Y=up convention.
/// We negate Y because the vertex shader negates gl_Position.y.
/// Positions are snapped to reduce sub-pixel blur.
pub fn text_z(engine: &mut ProofEngine, s: &str, x: f32, y: f32, z: f32, color: Vec4, scale: f32, emission: f32) {
    let sp = spacing(scale);
    // Snap base position to reduce sub-pixel blur
    let snap = |v: f32| -> f32 { (v * 20.0).round() / 20.0 };
    for (i, ch) in s.chars().enumerate() {
        if ch == ' ' { continue; }
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(snap(x + i as f32 * sp), snap(-y), z),
            scale: Vec2::splat(scale),
            color,
            emission: emission.min(0.3), // cap emission to prevent bloom bleed on text
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

/// Horizontal bar using block characters with smooth fill.
pub fn bar(engine: &mut ProofEngine, x: f32, y: f32, width: f32, ratio: f32, fill: Vec4, empty: Vec4, scale: f32) {
    let sp = spacing(scale);
    let n = (width / sp) as usize;
    let filled = ((ratio.clamp(0.0, 1.0) * n as f32) as usize).min(n);
    for i in 0..n {
        let (ch, c, em) = if i < filled { ('\u{2588}', fill, 0.5) } else { ('\u{2591}', empty, 0.1) };
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x + i as f32 * sp, -y, Z_TEXT),
            scale: Vec2::splat(scale), color: c, emission: em,
            layer: RenderLayer::UI, ..Default::default()
        });
    }
}

// ── Box Drawing ──────────────────────────────────────────────────────────────

/// Box-drawing character set.
const BOX_TL: char = '╔';
const BOX_TR: char = '╗';
const BOX_BL: char = '╚';
const BOX_BR: char = '╝';
const BOX_H: char  = '═';
const BOX_V: char  = '║';

/// Single-line box characters.
const BOX_TL_S: char = '┌';
const BOX_TR_S: char = '┐';
const BOX_BL_S: char = '└';
const BOX_BR_S: char = '┘';
const BOX_H_S: char  = '─';
const BOX_V_S: char  = '│';

fn emit_glyph(engine: &mut ProofEngine, ch: char, x: f32, y: f32, z: f32, color: Vec4, scale: f32, emission: f32) {
    engine.spawn_glyph(Glyph {
        character: ch,
        position: Vec3::new(x, -y, z),
        scale: Vec2::splat(scale), color, emission,
        layer: RenderLayer::UI,
        ..Default::default()
    });
}

/// Draw a double-line box border. (x, y) = top-left corner, w/h in world units.
pub fn box_double(engine: &mut ProofEngine, x: f32, y: f32, w: f32, h: f32, color: Vec4, scale: f32, emission: f32) {
    let sp = spacing(scale);
    let cols = ((w / sp) as usize).max(2);
    let rows = ((h / (scale * 1.1)) as usize).max(2);
    let row_sp = scale * 1.1;

    // Top edge
    emit_glyph(engine, BOX_TL, x, y, Z_BORDER, color, scale, emission);
    for c in 1..cols - 1 {
        emit_glyph(engine, BOX_H, x + c as f32 * sp, y, Z_BORDER, color, scale, emission);
    }
    emit_glyph(engine, BOX_TR, x + (cols - 1) as f32 * sp, y, Z_BORDER, color, scale, emission);

    // Sides
    for r in 1..rows - 1 {
        let ry = y - r as f32 * row_sp;
        emit_glyph(engine, BOX_V, x, ry, Z_BORDER, color, scale, emission);
        emit_glyph(engine, BOX_V, x + (cols - 1) as f32 * sp, ry, Z_BORDER, color, scale, emission);
    }

    // Bottom edge
    let by = y - (rows - 1) as f32 * row_sp;
    emit_glyph(engine, BOX_BL, x, by, Z_BORDER, color, scale, emission);
    for c in 1..cols - 1 {
        emit_glyph(engine, BOX_H, x + c as f32 * sp, by, Z_BORDER, color, scale, emission);
    }
    emit_glyph(engine, BOX_BR, x + (cols - 1) as f32 * sp, by, Z_BORDER, color, scale, emission);
}

/// Draw a single-line box border.
pub fn box_single(engine: &mut ProofEngine, x: f32, y: f32, w: f32, h: f32, color: Vec4, scale: f32, emission: f32) {
    let sp = spacing(scale);
    let cols = ((w / sp) as usize).max(2);
    let rows = ((h / (scale * 1.1)) as usize).max(2);
    let row_sp = scale * 1.1;

    emit_glyph(engine, BOX_TL_S, x, y, Z_BORDER, color, scale, emission);
    for c in 1..cols - 1 {
        emit_glyph(engine, BOX_H_S, x + c as f32 * sp, y, Z_BORDER, color, scale, emission);
    }
    emit_glyph(engine, BOX_TR_S, x + (cols - 1) as f32 * sp, y, Z_BORDER, color, scale, emission);

    for r in 1..rows - 1 {
        let ry = y - r as f32 * row_sp;
        emit_glyph(engine, BOX_V_S, x, ry, Z_BORDER, color, scale, emission);
        emit_glyph(engine, BOX_V_S, x + (cols - 1) as f32 * sp, ry, Z_BORDER, color, scale, emission);
    }

    let by = y - (rows - 1) as f32 * row_sp;
    emit_glyph(engine, BOX_BL_S, x, by, Z_BORDER, color, scale, emission);
    for c in 1..cols - 1 {
        emit_glyph(engine, BOX_H_S, x + c as f32 * sp, by, Z_BORDER, color, scale, emission);
    }
    emit_glyph(engine, BOX_BR_S, x + (cols - 1) as f32 * sp, by, Z_BORDER, color, scale, emission);
}

/// Fill a rectangular region with a dim background character for contrast.
/// Renders behind text at Z_PANEL.
pub fn panel_bg(engine: &mut ProofEngine, x: f32, y: f32, w: f32, h: f32, color: Vec4, scale: f32) {
    let sp = spacing(scale);
    let cols = ((w / sp) as usize).max(1);
    let rows = ((h / (scale * 1.1)) as usize).max(1);
    let row_sp = scale * 1.1;
    let dim_color = Vec4::new(color.x, color.y, color.z, color.w * 0.3);

    for r in 0..rows {
        for c in 0..cols {
            engine.spawn_glyph(Glyph {
                character: '░',
                position: Vec3::new(x + c as f32 * sp, -(y - r as f32 * row_sp), Z_PANEL),
                scale: Vec2::splat(scale),
                color: dim_color,
                emission: 0.05,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }
    }
}

/// Dark screen-wide backing to improve text readability over chaos field.
/// Call this at the start of any screen's render function.
pub fn screen_backing(engine: &mut ProofEngine, opacity: f32) {
    // Single large dark glyph behind everything
    engine.spawn_glyph(Glyph {
        character: '█',
        position: Vec3::new(0.0, 0.0, 1.0), // behind UI text (z=0)
        scale: Vec2::new(25.0, 15.0),
        color: Vec4::new(0.0, 0.0, 0.0, opacity),
        emission: 0.0,
        layer: RenderLayer::World, // below UI layer
        ..Default::default()
    });
}

/// Draw a complete panel: background fill + double-line border.
pub fn panel(engine: &mut ProofEngine, x: f32, y: f32, w: f32, h: f32, border_color: Vec4, bg_color: Vec4, scale: f32) {
    panel_bg(engine, x, y, w, h, bg_color, scale);
    box_double(engine, x, y, w, h, border_color, scale, 0.4);
}

/// Draw a panel with a title in the top border.
pub fn panel_titled(engine: &mut ProofEngine, title_str: &str, x: f32, y: f32, w: f32, h: f32, border_color: Vec4, bg_color: Vec4, title_color: Vec4, scale: f32) {
    panel(engine, x, y, w, h, border_color, bg_color, scale);
    // Title centered in top border
    let sp = spacing(scale);
    let title_w = title_str.len() as f32 * sp;
    let title_x = x + (w - title_w) * 0.5;
    text_z(engine, title_str, title_x, y, Z_BORDER, title_color, scale, 0.6);
}

/// Horizontal separator line using box-drawing characters.
pub fn separator(engine: &mut ProofEngine, x: f32, y: f32, width: f32, color: Vec4, scale: f32) {
    let sp = spacing(scale);
    let n = (width / sp) as usize;
    for i in 0..n {
        emit_glyph(engine, BOX_H_S, x + i as f32 * sp, y, Z_BORDER, color, scale, 0.2);
    }
}

/// Horizontal separator with endpoints.
pub fn separator_capped(engine: &mut ProofEngine, x: f32, y: f32, width: f32, color: Vec4, scale: f32) {
    let sp = spacing(scale);
    let n = (width / sp) as usize;
    if n < 2 { return; }
    emit_glyph(engine, '╟', x, y, Z_BORDER, color, scale, 0.2);
    for i in 1..n - 1 {
        emit_glyph(engine, BOX_H_S, x + i as f32 * sp, y, Z_BORDER, color, scale, 0.2);
    }
    emit_glyph(engine, '╢', x + (n - 1) as f32 * sp, y, Z_BORDER, color, scale, 0.2);
}

/// Render a tooltip box at position with text lines.
pub fn tooltip(engine: &mut ProofEngine, lines: &[&str], x: f32, y: f32, border_color: Vec4, bg_color: Vec4, text_color: Vec4) {
    let scale = 0.25;
    let sp = spacing(scale);
    let row_h = scale * 1.1;
    let max_len = lines.iter().map(|l| l.len()).max().unwrap_or(0);
    let w = (max_len + 2) as f32 * sp;
    let h = (lines.len() + 1) as f32 * row_h;

    panel(engine, x, y, w, h, border_color, bg_color, scale);
    for (i, line) in lines.iter().enumerate() {
        text_z(engine, line, x + sp, y - (i + 1) as f32 * row_h + row_h * 0.3, Z_OVERLAY, text_color, scale, 0.4);
    }
}

/// Render a selection cursor arrow.
pub fn cursor_arrow(engine: &mut ProofEngine, x: f32, y: f32, color: Vec4, scale: f32, frame: u64) {
    let pulse = ((frame as f32 * 0.12).sin() * 0.3 + 0.7).max(0.0);
    emit_glyph(engine, '▶', x, y, Z_TEXT,
        Vec4::new(color.x * pulse, color.y * pulse, color.z * pulse, color.w),
        scale, 0.6);
}

/// Render text that is clipped to a max character width.
pub fn text_clipped(engine: &mut ProofEngine, s: &str, x: f32, y: f32, max_chars: usize, color: Vec4, scale: f32, emission: f32) {
    let truncated: String = s.chars().take(max_chars).collect();
    text(engine, &truncated, x, y, color, scale, emission);
}

/// Word-wrap text into lines of max_chars width, render starting at (x, y) going down.
/// Returns the y position after the last line.
pub fn text_wrapped(engine: &mut ProofEngine, s: &str, x: f32, y: f32, max_chars: usize, color: Vec4, scale: f32, emission: f32) -> f32 {
    let row_h = scale * 1.2;
    let mut cy = y;
    let words: Vec<&str> = s.split_whitespace().collect();
    let mut line = String::new();

    for word in words {
        if line.len() + word.len() + 1 > max_chars {
            if !line.is_empty() {
                text(engine, &line, x, cy, color, scale, emission);
                cy -= row_h;
                line.clear();
            }
        }
        if !line.is_empty() { line.push(' '); }
        line.push_str(word);
    }
    if !line.is_empty() {
        text(engine, &line, x, cy, color, scale, emission);
        cy -= row_h;
    }
    cy
}
