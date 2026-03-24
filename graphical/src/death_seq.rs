// Death cinematic sequence — CHAOS RPG Visual Push
//
// Phases (all at ~30fps):
//   1  KILLING BLOW (0-40)  — big red damage, killer name, held long so player reads it
//   2  CRACK        (40-65) — red-tinted fracture lines radiate from center
//   3  COLLAPSE     (65-90) — scatter debris, damage fades
//   4  VOID         (90-110)— near-black, red embers
//   5  EPITAPH      (110+)  — bold "YOU DIED" in red, killer + cause in color

use bracket_lib::prelude::*;

pub const DEATH_SEQ_DONE_FRAME: u32 = 160;

pub struct DeathSeq {
    pub frame:       u32,
    pub active:      bool,
    pub final_dmg:   i64,
    pub killer_name: String,
    pub epitaph:     String,
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
        self.killer_name  = killer.chars().take(40).collect();
        self.epitaph      = epitaph.to_string();
        self.reveal_chars = 0;
        self.reveal_timer = 0;
    }

    pub fn is_done(&self) -> bool { self.frame >= DEATH_SEQ_DONE_FRAME }

    pub fn update(&mut self) {
        if !self.active { return; }
        self.frame += 1;
        if self.frame >= 110 {
            self.reveal_timer += 1;
            if self.reveal_timer >= 2 {
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
        let cy = 38i32;

        // ── Helper: draw damage + killer persistent header (phases 1-3) ──
        let show_kill_header = f < 90;
        if show_kill_header {
            let header_fade = if f < 35 {
                1.0f32
            } else {
                1.0 - (f - 35) as f32 / 55.0
            }.max(0.0);

            // "KILLING BLOW" label in orange
            let label = "  ☠  KILLING BLOW  ☠  ";
            let lv = (header_fade * 200.0) as u8;
            ctx.print_color(cx - label.len() as i32 / 2, cy - 8,
                RGB::from_u8(lv, lv / 3, 0), bg_rgb, label);

            // Giant damage number — repeated chars for visual weight
            let dmg_str = format!("- {} -", self.final_dmg);
            let dv_r = (header_fade * 255.0) as u8;
            let dv_g = (header_fade * 40.0) as u8;
            // Draw it large by printing on two adjacent lines
            ctx.print_color(cx - dmg_str.len() as i32 / 2, cy - 6,
                RGB::from_u8(dv_r, dv_g, 0), bg_rgb, &dmg_str);
            ctx.print_color(cx - dmg_str.len() as i32 / 2, cy - 5,
                RGB::from_u8(dv_r / 2, dv_g / 4, 0), bg_rgb, &dmg_str);

            // Killer name in bright red-orange, boxed
            let kn = &self.killer_name;
            let kv = (header_fade * 240.0) as u8;
            let killed_by = format!("Killed by: {}", kn);
            ctx.print_color(cx - killed_by.len() as i32 / 2, cy - 3,
                RGB::from_u8(kv, kv / 4, kv / 8), bg_rgb, &killed_by);

            // Thin separator
            let sep_len = (killed_by.len() + 4).min(60);
            let sep: String = "─".repeat(sep_len);
            let sv = (header_fade * 100.0) as u8;
            ctx.print_color(cx - sep_len as i32 / 2, cy - 2,
                RGB::from_u8(sv, sv / 6, sv / 6), bg_rgb, &sep);
        }

        match f {
            // ── Phase 1: KILLING BLOW (0-40) ──────────────────────────────
            0..=39 => {
                // Radial flash burst — red rings expanding from center
                let burst_r = (f as f32 * 2.5) as i32;
                for y in 0..80i32 {
                    for x in 0..160i32 {
                        let dx = x - cx;
                        let dy = (y - cy) * 2;
                        let dist = ((dx*dx + dy*dy) as f32).sqrt() as i32;
                        if dist >= burst_r - 2 && dist <= burst_r + 2 {
                            let fade = 1.0 - f as f32 / 40.0;
                            let rv = (fade * 120.0) as u8;
                            if rv > 8 {
                                ctx.print_color(x, y,
                                    RGB::from_u8(rv, rv / 8, 0), bg_rgb, "░");
                            }
                        }
                    }
                }
            }

            // ── Phase 2: CRACK (40-65) ─────────────────────────────────────
            40..=64 => {
                let progress = (f - 40) as f32 / 25.0;
                let crack_len = (progress * 80.0) as i32;
                let dirs: [(i32, i32); 8] = [
                    (1,0),(-1,0),(0,1),(0,-1),(1,1),(-1,1),(1,-1),(-1,-1),
                ];
                let line_chars = ['─','─','│','│','╱','╲','╱','╲'];
                // Red-orange tint on fracture lines
                for (dir_i, &(dx, dy)) in dirs.iter().enumerate() {
                    let len = (crack_len + dir_i as i32 * 4).min(crack_len + 8);
                    for step in 0..len {
                        let x = cx + dx * step;
                        let y = cy + dy * step;
                        if x <= 0 || x >= 159 || y <= 0 || y >= 79 { break; }
                        let b = ((len - step) as f32 / len as f32 * 230.0) as u8;
                        // Alternate red-orange and bright orange for visual variety
                        let (r, g) = if dir_i % 2 == 0 { (b, b/6) } else { (b, b/3) };
                        ctx.print_color(x, y, RGB::from_u8(r, g, 0), bg_rgb,
                            &line_chars[dir_i].to_string());
                    }
                }
            }

            // ── Phase 3: COLLAPSE (65-90) ──────────────────────────────────
            65..=89 => {
                let progress = (f - 65) as f32 / 25.0;
                let fade = 1.0 - progress;
                // Fading fracture lines
                let bright = (fade * 160.0) as u8;
                let dirs: [(i32,i32); 4] = [(1,1),(-1,1),(1,-1),(-1,-1)];
                for &(dx,dy) in &dirs {
                    for step in 0..55i32 {
                        let x = cx + dx * step;
                        let y = cy + dy * step;
                        if x <= 0 || x >= 159 || y <= 0 || y >= 79 { break; }
                        let b = ((55 - step) as f32 / 55.0 * bright as f32) as u8;
                        if b < 6 { break; }
                        ctx.print_color(x, y, RGB::from_u8(b, b/5, 0), bg_rgb, "·");
                    }
                }
                // Red ember scatter
                for i in 0..30u32 {
                    let seed = i as f32 * 83.7 + f as f32 * 9.1;
                    let sx = (seed.sin() * 65.0 + cx as f32) as i32;
                    let sy = (seed.cos() * 30.0 + cy as f32 + (f - 65) as f32 * 0.6) as i32;
                    if sx > 0 && sx < 159 && sy > 0 && sy < 79 {
                        let rv = (fade * 100.0) as u8;
                        ctx.print_color(sx, sy, RGB::from_u8(rv, rv/5, 0), bg_rgb, "·");
                    }
                }
            }

            // ── Phase 4: VOID (90-110) ─────────────────────────────────────
            90..=109 => {
                let into_void = (f - 90) as f32 / 20.0;
                // Pulsing deep red embers at screen edges
                let ember_v = ((into_void * 0.5 + 0.5) * 40.0) as u8;
                for i in 0..8u32 {
                    let seed = i as f32 * 37.3 + f as f32 * 3.1;
                    let ex = (seed.sin().abs() * 155.0) as i32;
                    let ey = (seed.cos().abs() * 75.0) as i32;
                    ctx.print_color(ex.clamp(1,158), ey.clamp(1,78),
                        RGB::from_u8(ember_v, ember_v/8, 0), bg_rgb, "·");
                }
                // Faint "..." pulsing in center
                let pulse = ((f as f32 * 0.15).sin() * 0.5 + 0.5);
                let pv = (pulse * 35.0) as u8;
                ctx.print_color(cx - 1, cy, RGB::from_u8(pv, pv/6, 0), bg_rgb, "...");
            }

            // ── Phase 5: EPITAPH (110+) ────────────────────────────────────
            _ => {
                let title = "Y  O  U    D  I  E  D";
                let title_len = title.len().min(self.reveal_chars);
                let title_shown: String = title.chars().take(title_len).collect();

                // Bright pulsing red title
                let pulse = ((self.frame as f32 * 0.12).sin() * 0.2 + 0.8);
                let tr = (pulse * 255.0) as u8;
                let tg = (pulse * 30.0) as u8;
                ctx.print_color(cx - title.len() as i32 / 2, cy - 5,
                    RGB::from_u8(tr, tg, 0), bg_rgb, &title_shown);

                // Killer line — gold, visible immediately
                if self.reveal_chars > title.len() + 2 {
                    let by_line = format!("☠  Killed by: {}", self.killer_name);
                    let by_fade = ((self.reveal_chars as f32 - title.len() as f32 - 2.0) / 10.0).min(1.0);
                    let bv = (by_fade * 220.0) as u8;
                    ctx.print_color(cx - by_line.len() as i32 / 2, cy - 3,
                        RGB::from_u8(bv, bv * 3 / 4, 0), bg_rgb, &by_line);
                }

                // Damage line — orange
                if self.reveal_chars > title.len() + 8 {
                    let dmg_line = format!("Final blow: {} damage", self.final_dmg);
                    let df = ((self.reveal_chars as f32 - title.len() as f32 - 8.0) / 10.0).min(1.0);
                    let dv = (df * 200.0) as u8;
                    ctx.print_color(cx - dmg_line.len() as i32 / 2, cy - 1,
                        RGB::from_u8(dv, dv / 3, 0), bg_rgb, &dmg_line);
                }

                // Separator
                if self.reveal_chars > title.len() + 14 {
                    let sep = "─".repeat(50);
                    let sv = 60u8;
                    ctx.print_color(cx - 25, cy + 1,
                        RGB::from_u8(sv, sv/6, sv/6), bg_rgb, &sep);
                }

                // Epitaph text — warm dim red, readable
                let epi_start = title.len() + 20;
                if self.reveal_chars > epi_start {
                    let epi_len = (self.reveal_chars - epi_start).min(self.epitaph.len());
                    let epi_shown: String = self.epitaph.chars().take(epi_len).collect();
                    let mut ey = cy + 3;
                    let mut line = String::new();
                    let epi_fade = ((self.reveal_chars as f32 - epi_start as f32) / 25.0).min(1.0);
                    let ev = (epi_fade * 180.0) as u8;
                    for word in epi_shown.split_whitespace() {
                        if line.len() + word.len() + 1 > 56 {
                            ctx.print_color(cx - line.len() as i32 / 2, ey,
                                RGB::from_u8(ev, ev / 3, ev / 5), bg_rgb, &line);
                            line = word.to_string();
                            ey += 1;
                            if ey > cy + 7 { break; }
                        } else {
                            if !line.is_empty() { line.push(' '); }
                            line.push_str(word);
                        }
                    }
                    if !line.is_empty() && ey <= cy + 7 {
                        ctx.print_color(cx - line.len() as i32 / 2, ey,
                            RGB::from_u8(ev, ev / 3, ev / 5), bg_rgb, &line);
                    }
                }

                // Continue hint — fades in near the end
                if self.frame > 148 {
                    let hint_fade = ((self.frame - 148) as f32 / 12.0).min(1.0);
                    let hv = (hint_fade * 100.0) as u8;
                    ctx.print_color(cx - 14, cy + 9,
                        RGB::from_u8(hv, hv, hv), bg_rgb, "[ Enter ] View run summary");
                }
            }
        }
    }
}
