// Enhanced renderer helpers for CHAOS RPG — Graphical Edition.
// All drawing uses theme colors. No hardcoded RGB::named() outside this file.

use bracket_lib::prelude::*;
use crate::theme::Theme;

// ── Box drawing ───────────────────────────────────────────────────────────────

/// Single-line box (uses bracket-lib built-in).
pub fn draw_box_single(ctx: &mut BTerm, x: i32, y: i32, w: i32, h: i32, t: &Theme) {
    ctx.draw_box(x, y, w, h, RGB::from_u8(t.border.0, t.border.1, t.border.2),
                             RGB::from_u8(t.bg.0, t.bg.1, t.bg.2));
}

/// Double-line box using CP437 characters (╔ ═ ╗ ║ ╚ ╝).
pub fn draw_box_double(ctx: &mut BTerm, x: i32, y: i32, w: i32, h: i32, t: &Theme) {
    let fg = RGB::from_u8(t.border.0, t.border.1, t.border.2);
    let bg = RGB::from_u8(t.bg.0, t.bg.1, t.bg.2);
    // Corners: 201=╔ 187=╗ 200=╚ 188=╝
    ctx.set(x,     y,     fg, bg, 201u16);
    ctx.set(x + w, y,     fg, bg, 187u16);
    ctx.set(x,     y + h, fg, bg, 200u16);
    ctx.set(x + w, y + h, fg, bg, 188u16);
    // Horizontal: 205=═
    for i in 1..w { ctx.set(x + i, y,     fg, bg, 205u16); }
    for i in 1..w { ctx.set(x + i, y + h, fg, bg, 205u16); }
    // Vertical: 186=║
    for j in 1..h { ctx.set(x,     y + j, fg, bg, 186u16); }
    for j in 1..h { ctx.set(x + w, y + j, fg, bg, 186u16); }
}

/// Double box with a title label inset into the top border.
pub fn draw_panel(ctx: &mut BTerm, x: i32, y: i32, w: i32, h: i32, title: &str, t: &Theme) {
    draw_box_double(ctx, x, y, w, h, t);
    if !title.is_empty() {
        let label = format!(" {} ", title);
        let lx = x + 2;
        ctx.print_color(lx, y,
            RGB::from_u8(t.heading.0, t.heading.1, t.heading.2),
            RGB::from_u8(t.bg.0, t.bg.1, t.bg.2),
            &label);
    }
}

/// Inner panel using single box (sub-panel inside a double-box).
pub fn draw_subpanel(ctx: &mut BTerm, x: i32, y: i32, w: i32, h: i32, title: &str, t: &Theme) {
    // Dim the border slightly from the outer box to create visual depth
    let r = (t.border.0 as u16 * 55 / 100) as u8;
    let g = (t.border.1 as u16 * 55 / 100) as u8;
    let b = (t.border.2 as u16 * 55 / 100) as u8;
    let fg = RGB::from_u8(r, g, b);
    let bg = RGB::from_u8(t.bg.0, t.bg.1, t.bg.2);
    ctx.draw_box(x, y, w, h, fg, bg);
    if !title.is_empty() {
        ctx.print_color(x + 1, y,
            RGB::from_u8(t.accent.0, t.accent.1, t.accent.2),
            bg,
            &format!(" {} ", title));
    }
}

// ── Progress bars ─────────────────────────────────────────────────────────────

/// Gradient health/resource bar. Filled portion uses color lerp.
pub fn draw_bar_gradient(ctx: &mut BTerm, x: i32, y: i32, w: i32,
                         cur: i64, max: i64,
                         col_full: (u8,u8,u8), col_empty: (u8,u8,u8), t: &Theme) {
    let filled = if max > 0 { ((cur * w as i64) / max.max(1)).clamp(0, w as i64) as i32 } else { 0 };
    let bg = RGB::from_u8(t.bg.0, t.bg.1, t.bg.2);
    for i in 0..w {
        let (ch, color) = if i < filled {
            let pct = i as f32 / w.max(1) as f32;
            // Gradient: brighter near the filled tip
            let c = Theme::lerp(col_empty, col_full, pct * 1.5);
            (219u16, RGB::from_u8(c.0, c.1, c.2)) // █
        } else {
            (176u16, RGB::from_u8(t.muted.0, t.muted.1, t.muted.2)) // ░
        };
        ctx.set(x + i, y, color, bg, ch);
    }
}

/// Simple solid bar (no gradient, for mana, xp, etc.)
pub fn draw_bar_solid(ctx: &mut BTerm, x: i32, y: i32, w: i32,
                      cur: i64, max: i64, col: (u8,u8,u8), t: &Theme) {
    let filled = if max > 0 { ((cur * w as i64) / max.max(1)).clamp(0, w as i64) as i32 } else { 0 };
    let bg = RGB::from_u8(t.bg.0, t.bg.1, t.bg.2);
    let fc = RGB::from_u8(col.0, col.1, col.2);
    let dc = RGB::from_u8(t.muted.0, t.muted.1, t.muted.2);
    for i in 0..w {
        let (ch, c) = if i < filled { (219u16, fc) } else { (176u16, dc) };
        ctx.set(x + i, y, c, bg, ch);
    }
}

// ── Text helpers ──────────────────────────────────────────────────────────────

/// Print left-aligned with theme color.
pub fn print_t(ctx: &mut BTerm, x: i32, y: i32, col: (u8,u8,u8), t: &Theme, text: &str) {
    ctx.print_color(x, y,
        RGB::from_u8(col.0, col.1, col.2),
        RGB::from_u8(t.bg.0, t.bg.1, t.bg.2),
        text);
}

/// Print centered within [x_start..x_start+width].
pub fn print_center(ctx: &mut BTerm, x_start: i32, y: i32, width: i32,
                    col: (u8,u8,u8), t: &Theme, text: &str) {
    let pad = ((width - text.len() as i32) / 2).max(0);
    print_t(ctx, x_start + pad, y, col, t, text);
}

/// Print a labelled key hint like "[E] Enter".
pub fn print_hint(ctx: &mut BTerm, x: i32, y: i32, key: &str, desc: &str, t: &Theme) {
    let bg = RGB::from_u8(t.bg.0, t.bg.1, t.bg.2);
    ctx.print_color(x, y,
        RGB::from_u8(t.accent.0, t.accent.1, t.accent.2), bg, key);
    ctx.print_color(x + key.len() as i32, y,
        RGB::from_u8(t.dim.0, t.dim.1, t.dim.2), bg, desc);
}

/// Horizontal separator line using a themed character.
pub fn draw_separator(ctx: &mut BTerm, x: i32, y: i32, w: i32, t: &Theme) {
    let col = RGB::from_u8(t.muted.0, t.muted.1, t.muted.2);
    let bg  = RGB::from_u8(t.bg.0, t.bg.1, t.bg.2);
    for i in 0..w { ctx.set(x + i, y, col, bg, 196u16); } // ─
}

/// Animated cursor indicator (pulses between ► and a dim version based on frame).
pub fn cursor_char(frame: u64) -> char {
    if (frame / 15) % 2 == 0 { '►' } else { '▶' }
}

/// Selection row: draws prefix + text with highlighted or dim styling.
pub fn print_selectable(ctx: &mut BTerm, x: i32, y: i32, selected: bool,
                        text: &str, frame: u64, t: &Theme) {
    let bg = RGB::from_u8(t.bg.0, t.bg.1, t.bg.2);
    if selected {
        // Highlight bar — use a faint version of accent as background tint
        let bar_w = (text.len() as i32 + 4).min(80);
        let bg_r = (t.accent.0 as u16 * 12 / 100) as u8;
        let bg_g = (t.accent.1 as u16 * 12 / 100) as u8;
        let bg_b = (t.accent.2 as u16 * 12 / 100) as u8;
        for i in 0..bar_w {
            ctx.set(x + i - 2, y, RGB::from_u8(t.bg.0, t.bg.1, t.bg.2),
                    RGB::from_u8(bg_r, bg_g, bg_b), 32u16);
        }
        let pfx = format!("{} ", cursor_char(frame));
        ctx.print_color(x, y, RGB::from_u8(t.accent.0, t.accent.1, t.accent.2), bg, &pfx);
        ctx.print_color(x + pfx.len() as i32, y,
            RGB::from_u8(t.selected.0, t.selected.1, t.selected.2), bg, text);
    } else {
        ctx.print_color(x, y, RGB::from_u8(t.dim.0, t.dim.1, t.dim.2), bg,
            &format!("  {}", text));
    }
}

// ── Room minimap cell ─────────────────────────────────────────────────────────

/// Draw one minimap cell — current room, visited, or future.
pub fn draw_minimap_cell(ctx: &mut BTerm, rx: i32, ry: i32,
                         state: MinimapState, room_col: (u8,u8,u8),
                         sym: &str, t: &Theme) {
    let bg = RGB::from_u8(t.bg.0, t.bg.1, t.bg.2);
    match state {
        MinimapState::Current => {
            ctx.print_color(rx, ry,
                RGB::from_u8(t.selected.0, t.selected.1, t.selected.2), bg,
                &format!("[{}]", sym.trim_matches(|c| c == '[' || c == ']')));
        }
        MinimapState::Visited => {
            ctx.print_color(rx, ry, RGB::from_u8(t.muted.0, t.muted.1, t.muted.2), bg, "···");
        }
        MinimapState::Ahead => {
            ctx.print_color(rx, ry, RGB::from_u8(room_col.0, room_col.1, room_col.2), bg, sym);
        }
    }
}

#[derive(Clone, Copy)]
pub enum MinimapState { Current, Visited, Ahead }

// ── Stat badge ────────────────────────────────────────────────────────────────

/// Draw "Label: value" with coloured value.
pub fn stat_line(ctx: &mut BTerm, x: i32, y: i32, label: &str, value: &str,
                 val_col: (u8,u8,u8), t: &Theme) {
    let bg = RGB::from_u8(t.bg.0, t.bg.1, t.bg.2);
    ctx.print_color(x, y, RGB::from_u8(t.dim.0, t.dim.1, t.dim.2), bg, label);
    ctx.print_color(x + label.len() as i32, y,
        RGB::from_u8(val_col.0, val_col.1, val_col.2), bg, value);
}
