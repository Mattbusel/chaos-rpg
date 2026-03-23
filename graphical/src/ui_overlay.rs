// HUD overlays drawn on top of the main bracket-lib console.
// Handles floating damage numbers, status popups, and tooltips.

use bracket_lib::prelude::*;

/// A floating damage number that fades upward over time.
pub struct DamageFloat {
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub text: String,
    pub color: RGB,
    pub ttl: f32, // seconds remaining
}

impl DamageFloat {
    pub fn new(x: i32, y: i32, amount: i32, is_heal: bool) -> Self {
        let color = if is_heal { RGB::named(GREEN) } else { RGB::named(RED) };
        let text = if is_heal {
            format!("+{}", amount)
        } else {
            format!("-{}", amount)
        };
        Self { x: x as f32, y: y as f32, vy: -0.05, text, color, ttl: 1.5 }
    }

    pub fn tick(&mut self, dt: f32) {
        self.y += self.vy;
        self.ttl -= dt;
    }

    pub fn alive(&self) -> bool {
        self.ttl > 0.0
    }

    pub fn draw(&self, ctx: &mut BTerm) {
        let alpha = (self.ttl / 1.5).min(1.0);
        let r = (self.color.r * alpha) as u8;
        let g = (self.color.g * alpha) as u8;
        let b = (self.color.b * alpha) as u8;
        ctx.print_color(self.x as i32, self.y as i32, RGB::from_u8(r, g, b), RGB::named(BLACK), &self.text);
    }
}

/// Manager for all active floating numbers.
pub struct FloatManager {
    pub floats: Vec<DamageFloat>,
}

impl FloatManager {
    pub fn new() -> Self {
        Self { floats: Vec::new() }
    }

    pub fn push(&mut self, x: i32, y: i32, amount: i32, is_heal: bool) {
        self.floats.push(DamageFloat::new(x, y, amount, is_heal));
    }

    pub fn tick_and_draw(&mut self, ctx: &mut BTerm, dt: f32) {
        for f in &mut self.floats {
            f.tick(dt);
        }
        self.floats.retain(|f| f.alive());
        for f in &self.floats {
            f.draw(ctx);
        }
    }
}

/// Draw a small tooltip box at (x, y) with given lines.
pub fn draw_tooltip(ctx: &mut BTerm, x: i32, y: i32, lines: &[&str]) {
    if lines.is_empty() { return; }
    let w = lines.iter().map(|l| l.len()).max().unwrap_or(0) as i32 + 3;
    let h = lines.len() as i32 + 1;
    ctx.draw_box(x, y, w, h, RGB::named(YELLOW), RGB::named(BLACK));
    for (i, line) in lines.iter().enumerate() {
        ctx.print_color(x + 1, y + 1 + i as i32, RGB::named(WHITE), RGB::named(BLACK), *line);
    }
}

/// Draw a centered status message (e.g. "LEVEL UP!") flashing on screen.
pub fn draw_status_banner(ctx: &mut BTerm, screen_w: i32, y: i32, text: &str, color: (u8, u8, u8)) {
    let x = (screen_w - text.len() as i32) / 2;
    ctx.print_color(x, y, RGB::named(color), RGB::named(BLACK), text);
}
