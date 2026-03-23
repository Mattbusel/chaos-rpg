// CP437 tile renderer helpers for the graphical frontend.
// Wraps bracket-lib drawing primitives with game-specific helpers.

use bracket_lib::prelude::*;

/// Draw a bordered box with a title.
pub fn draw_titled_box(ctx: &mut BTerm, x: i32, y: i32, w: i32, h: i32, title: &str, fg: (u8, u8, u8), bg: (u8, u8, u8)) {
    ctx.draw_box(x, y, w, h, RGB::named(fg), RGB::named(bg));
    if !title.is_empty() {
        ctx.print_color(x + 2, y, RGB::named(YELLOW), RGB::named(bg), format!(" {} ", title));
    }
}

/// Draw a horizontal progress bar using block characters.
pub fn draw_bar(ctx: &mut BTerm, x: i32, y: i32, width: i32, current: i32, max: i32, fg: (u8, u8, u8), bg: (u8, u8, u8)) {
    let filled = if max > 0 { (current * width / max).min(width) } else { 0 };
    for i in 0..width {
        let ch = if i < filled { 219u16 } else { 176u16 }; // █ vs ░ in CP437
        let color = if i < filled { RGB::named(fg) } else { RGB::named(DARK_GRAY) };
        ctx.set(x + i, y, color, RGB::named(bg), ch);
    }
}

/// Render centered text within a given column range.
pub fn print_centered(ctx: &mut BTerm, cx: i32, y: i32, half_w: i32, text: &str, fg: (u8, u8, u8), bg: (u8, u8, u8)) {
    let offset = (half_w - text.len() as i32 / 2).max(0);
    ctx.print_color(cx - half_w + offset, y, RGB::named(fg), RGB::named(bg), text);
}
