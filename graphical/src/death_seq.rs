// Death cinematic sequence — CHAOS RPG Visual Push
//
// When the player dies, a 3-second cinematic plays before the
// normal GameOver screen. Phases:
//   0  Inactive
//   1  Hit      (0-15)  — large damage number, earthquake
//   2  Crack    (15-40) — fracture lines radiate from center
//   3  Collapse (40-70) — tiles drift/dim, desaturation
//   4  Void     (70-100) — near-black silence
//   5  Epitaph  (100-150) — "Y O U  D I E D" typewriter + epitaph
//   6  Done     → caller transitions to GameOver screen

use bracket_lib::prelude::*;

pub const DEATH_SEQ_DONE_FRAME: u32 = 150;

pub struct DeathSeq {
    pub frame:       u32,   // frames since sequence started
    pub active:      bool,
    pub final_dmg:   i64,
    pub killer_name: String,
    pub epitaph:     String,
    // Typewriter state
    reveal_chars:    usize,
    reveal_timer:    u32,
}

impl DeathSeq {
    pub fn new() -> Self {
        Self {
            frame: 0, active: false,
            final_dmg: 0, killer_name: String::new(), epitaph: String::new(),
            reveal_chars: 0, reveal_timer: 0,
        }
    }

    pub fn start(&mut self, dmg: i64, killer: &str, epitaph: &str) {
        self.frame        = 0;
        self.active       = true;
        self.final_dmg    = dmg;
        self.killer_name  = killer.to_string();
        self.epitaph      = epitaph.to_string();
        self.reveal_chars = 0;
        self.reveal_timer = 0;
    }

    pub fn is_done(&self) -> bool { self.frame >= DEATH_SEQ_DONE_FRAME }

    pub fn update(&mut self) {
        if !self.active { return; }
        self.frame += 1;
        // Typewriter advance for epitaph phase
        if self.frame >= 100 {
            self.reveal_timer += 1;
            if self.reveal_timer >= 2 { // reveal 1 char every 2 frames
                self.reveal_timer = 0;
                self.reveal_chars += 1;
            }
        }
    }

    pub fn draw(&self, ctx: &mut BTerm, bg: (u8, u8, u8)) {
        if !self.active { return; }
        let f = self.frame;
        let bg_rgb = RGB::from_u8(bg.0, bg.1, bg.2);
        let cx = 80i32;
        let cy = 40i32;

        match f {
            // ── Phase 1: HIT (0-15) ────────────────────────────────────────
            0..=15 => {
                // Large damage number at center
                let scale_str = format!("-{}", self.final_dmg);
                let fade = (15 - f) as f32 / 15.0;
                let r = (255.0 * fade) as u8;
                let g = (30.0 * fade) as u8;
                ctx.print_color(cx - scale_str.len() as i32 / 2, cy,
                    RGB::from_u8(r, g, 0), bg_rgb, &scale_str);
                // Killer name below
                let kn: String = self.killer_name.chars().take(40).collect();
                ctx.print_color(cx - kn.len() as i32 / 2, cy + 2,
                    RGB::from_u8((200.0 * fade) as u8, 0, 0), bg_rgb, &kn);
            }

            // ── Phase 2: CRACK (15-40) ─────────────────────────────────────
            15..=40 => {
                let progress = (f - 15) as f32 / 25.0;
                // Fracture lines radiating from center (8 directions)
                let crack_len = (progress * 78.0) as i32;
                let dirs: [(i32, i32); 8] = [
                    (1, 0), (-1, 0), (0, 1), (0, -1),
                    (1, 1), (-1, 1), (1, -1), (-1, -1),
                ];
                let line_chars = ['/', '\\', '|', '-', '/', '\\', '/', '\\'];
                for (dir_i, &(dx, dy)) in dirs.iter().enumerate() {
                    let len = (crack_len + dir_i as i32 * 3).min(crack_len + 6);
                    for step in 0..len {
                        let x = cx + dx * step;
                        let y = cy + dy * step;
                        if x <= 0 || x >= 159 || y <= 0 || y >= 79 { break; }
                        let bright = ((len - step) as f32 / len as f32 * 220.0) as u8;
                        ctx.print_color(x, y,
                            RGB::from_u8(bright, bright, bright), bg_rgb,
                            &line_chars[dir_i].to_string());
                    }
                }
            }

            // ── Phase 3: COLLAPSE (40-70) ──────────────────────────────────
            40..=70 => {
                let progress = (f - 40) as f32 / 30.0;
                // Fractures from phase 2 still visible, fading
                let fade = 1.0 - progress;
                let bright = (fade * 180.0) as u8;
                let dirs: [(i32, i32); 8] = [
                    (1,0),(-1,0),(0,1),(0,-1),(1,1),(-1,1),(1,-1),(-1,-1)
                ];
                let line_chars = ['/','\\',' ',' ','/','\\',' ',' '];
                for (dir_i, &(dx, dy)) in dirs.iter().enumerate() {
                    for step in 0..60i32 {
                        let x = cx + dx * step;
                        let y = cy + dy * step;
                        if x <= 0 || x >= 159 || y <= 0 || y >= 79 { break; }
                        let b = ((60 - step) as f32 / 60.0 * bright as f32) as u8;
                        if b < 5 { break; }
                        ctx.print_color(x, y, RGB::from_u8(b, b, b), bg_rgb,
                            &line_chars[dir_i].to_string());
                    }
                }
                // Scatter dots (simulating falling tiles)
                for i in 0..40u32 {
                    let seed = i as f32 * 73.1 + f as f32 * 11.3;
                    let sx = (seed.sin() * 70.0 + cx as f32) as i32;
                    let sy = (seed.cos() * 35.0 + cy as f32 + (f - 40) as f32 * 0.5) as i32;
                    if sx > 0 && sx < 159 && sy > 0 && sy < 79 {
                        let b = (fade * 80.0) as u8;
                        ctx.print_color(sx, sy, RGB::from_u8(b, b, b), bg_rgb, "·");
                    }
                }
            }

            // ── Phase 4: VOID (70-100) ─────────────────────────────────────
            70..=99 => {
                // Near-black — only residual chaos field visible (caller handles bg)
                // Draw a very faint "..." to hint something is coming
                let fade_in = ((f - 70) as f32 / 30.0 * 20.0) as u8;
                ctx.print_color(cx - 1, cy, RGB::from_u8(fade_in, fade_in, fade_in), bg_rgb, "...");
            }

            // ── Phase 5: EPITAPH (100+) ────────────────────────────────────
            _ => {
                // "Y O U  D I E D" typewriter
                let title = "Y O U   D I E D";
                let sub   = "The mathematics have consumed you.";
                let title_len = title.len().min(self.reveal_chars);
                let title_shown: String = title.chars().take(title_len).collect();
                ctx.print_color(cx - title.len() as i32 / 2, cy - 4,
                    RGB::from_u8(220, 20, 20), bg_rgb, &title_shown);

                if self.reveal_chars > title.len() + 10 {
                    let sub_len = (self.reveal_chars - title.len() - 10).min(sub.len());
                    let sub_shown: String = sub.chars().take(sub_len).collect();
                    let fade = ((self.reveal_chars as f32 - title.len() as f32 - 10.0) / 20.0).min(1.0);
                    let v = (fade * 160.0) as u8;
                    ctx.print_color(cx - sub.len() as i32 / 2, cy - 2,
                        RGB::from_u8(v, v / 3, v / 3), bg_rgb, &sub_shown);
                }

                // Epitaph (fades in after sub)
                let epi_start = title.len() + sub.len() + 20;
                if self.reveal_chars > epi_start {
                    let epi_len = (self.reveal_chars - epi_start).min(self.epitaph.len());
                    let epi_shown: String = self.epitaph.chars().take(epi_len).collect();
                    // Word-wrap at 60 chars
                    let mut ex = cx - 30;
                    let mut ey = cy + 1;
                    let mut line = String::new();
                    for word in epi_shown.split_whitespace() {
                        if line.len() + word.len() + 1 > 58 {
                            let fade = ((self.reveal_chars as f32 - epi_start as f32) / 30.0).min(1.0);
                            let v = (fade * 140.0) as u8;
                            ctx.print_color(ex, ey, RGB::from_u8(v, v/2, v/2), bg_rgb, &line);
                            line = word.to_string();
                            ey += 1;
                            if ey > cy + 5 { break; }
                        } else {
                            if !line.is_empty() { line.push(' '); }
                            line.push_str(word);
                        }
                    }
                    if !line.is_empty() && ey <= cy + 5 {
                        let fade = ((self.reveal_chars as f32 - epi_start as f32) / 30.0).min(1.0);
                        let v = (fade * 140.0) as u8;
                        ctx.print_color(ex, ey, RGB::from_u8(v, v/2, v/2), bg_rgb, &line);
                    }
                    let _ = ex; // suppress warning
                }

                // Hint to continue
                if self.frame > 140 {
                    let hint_fade = ((self.frame - 140) as f32 / 10.0).min(1.0);
                    let v = (hint_fade * 80.0) as u8;
                    ctx.print_color(cx - 12, cy + 8,
                        RGB::from_u8(v, v, v), bg_rgb, "[ Enter ] View run summary");
                }
            }
        }
    }
}
