// Ambient weather system — CHAOS RPG Visual Push
//
// Floor-range-appropriate atmosphere drawn as sparse particle overlays
// AFTER the chaos field and BEFORE the main UI panels.
// Draws faint characters in open areas — intentionally sparse so it
// doesn't interfere with UI content.

use bracket_lib::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum WeatherType {
    Clear,       // no weather
    DigitalRain, // floors 1-10: faint green/blue char columns
    Static,      // floors 26-50: random white flashes
    Ash,         // floors 51-75: gray dots drifting diagonally
    Sparks,      // floors 76-99: brief bright arcs
    VoidSnow,    // floors 100+: ultra-slow white dots
    Storm,       // boss fights: occasional full-screen brightness spike
}

impl WeatherType {
    pub fn for_floor(floor: u32, is_boss: bool) -> Self {
        if is_boss { return WeatherType::Storm; }
        match floor {
            0..=10  => WeatherType::Clear,
            11..=25 => WeatherType::Clear,
            26..=50 => WeatherType::Static,
            51..=75 => WeatherType::Ash,
            76..=99 => WeatherType::Sparks,
            _       => WeatherType::VoidSnow,
        }
    }
}

struct WParticle {
    x:     f32,
    y:     f32,
    vx:    f32,
    vy:    f32,
    life:  u32,
    max_life: u32,
    ch:    char,
    col:   (u8, u8, u8),
}

pub struct Weather {
    pub weather_type: WeatherType,
    particles: Vec<WParticle>,
    spawn_acc: f32,  // fractional spawn accumulator
    // For Storm: inter-flash timer
    storm_timer: u32,
    pub storm_flash: u32,  // frames of brightness spike remaining
}

impl Weather {
    pub fn new() -> Self {
        Self {
            weather_type: WeatherType::Clear,
            particles:    Vec::with_capacity(200),
            spawn_acc:    0.0,
            storm_timer:  120,
            storm_flash:  0,
        }
    }

    pub fn set_type(&mut self, wt: WeatherType) {
        if wt != self.weather_type {
            self.weather_type = wt;
            self.particles.clear();
        }
    }

    pub fn update(&mut self, frame: u64) {
        let ft = frame as f32;

        // Age and remove dead particles
        for p in &mut self.particles { if p.life > 0 { p.life -= 1; } }
        self.particles.retain(|p| p.life > 0);

        // Storm flash countdown
        if self.storm_flash > 0 { self.storm_flash -= 1; }

        match self.weather_type {
            WeatherType::Clear => {}

            WeatherType::DigitalRain => {
                // Sparse character columns; spawn ~3/frame
                self.spawn_acc += 3.0;
                while self.spawn_acc >= 1.0 && self.particles.len() < 120 {
                    self.spawn_acc -= 1.0;
                    let col_seed = (frame.wrapping_mul(13) + self.particles.len() as u64 * 7) % 160;
                    let x = col_seed as f32;
                    let chars = ['0','1','2','3','4','5','6','7','8','9','+','-','=','π','∞','λ'];
                    let ci = (col_seed as usize + frame as usize / 4) % chars.len();
                    let g = (30 + (col_seed % 60)) as u8;
                    self.particles.push(WParticle {
                        x, y: 0.0, vx: 0.0, vy: 0.22,
                        life: 340 + (col_seed % 80) as u32, max_life: 420,
                        ch: chars[ci],
                        col: (0, g, g / 2),
                    });
                }
                for p in &mut self.particles { p.y += p.vy; }
                self.particles.retain(|p| p.y < 79.0);
            }

            WeatherType::Static => {
                // Random brief flashes — ~6/frame, life 2 frames
                self.spawn_acc += 6.0;
                while self.spawn_acc >= 1.0 && self.particles.len() < 40 {
                    self.spawn_acc -= 1.0;
                    let seed = frame.wrapping_mul(7919) + self.particles.len() as u64 * 1234567;
                    let x = (seed % 158 + 1) as f32;
                    let y = (seed / 158 % 78 + 1) as f32;
                    let bright = 80 + (seed % 100) as u8;
                    self.particles.push(WParticle {
                        x, y, vx: 0.0, vy: 0.0,
                        life: 2, max_life: 2,
                        ch: '·',
                        col: (bright, bright, bright),
                    });
                }
            }

            WeatherType::Ash => {
                // Slow diagonal drift, gray
                self.spawn_acc += 1.5;
                while self.spawn_acc >= 1.0 && self.particles.len() < 80 {
                    self.spawn_acc -= 1.0;
                    let seed = frame.wrapping_mul(3517) + self.particles.len() as u64 * 9991;
                    let x = (seed % 158 + 1) as f32;
                    let g = 30 + (seed % 40) as u8;
                    self.particles.push(WParticle {
                        x, y: 0.0,
                        vx: ((seed % 5) as f32 - 2.0) * 0.04,
                        vy: 0.06 + (seed % 6) as f32 * 0.01,
                        life: 500 + (seed % 200) as u32, max_life: 700,
                        ch: '·',
                        col: (g, g, g),
                    });
                }
                for p in &mut self.particles { p.x += p.vx; p.y += p.vy; }
                self.particles.retain(|p| p.y < 79.0 && p.x > 0.0 && p.x < 159.0);
            }

            WeatherType::Sparks => {
                // Brief bright particles that arc outward
                self.spawn_acc += 0.8;
                while self.spawn_acc >= 1.0 && self.particles.len() < 60 {
                    self.spawn_acc -= 1.0;
                    let seed = frame.wrapping_mul(6271) + self.particles.len() as u64 * 8191;
                    let x = (seed % 158 + 1) as f32;
                    let y = (seed / 158 % 78 + 1) as f32;
                    let angle = (seed % 628) as f32 * 0.01;
                    let speed = 0.15 + (seed % 10) as f32 * 0.02;
                    let orange = 150 + (seed % 80) as u8;
                    self.particles.push(WParticle {
                        x, y,
                        vx: angle.cos() * speed,
                        vy: angle.sin() * speed,
                        life: 12 + (seed % 15) as u32, max_life: 27,
                        ch: '*',
                        col: (orange, orange / 2, 0),
                    });
                }
                for p in &mut self.particles { p.x += p.vx; p.y += p.vy; }
                self.particles.retain(|p| p.y >= 0.0 && p.y < 79.0 && p.x > 0.0 && p.x < 159.0);
            }

            WeatherType::VoidSnow => {
                // Ultra-slow white dots
                self.spawn_acc += 0.4;
                while self.spawn_acc >= 1.0 && self.particles.len() < 50 {
                    self.spawn_acc -= 1.0;
                    let seed = frame.wrapping_mul(9001) + self.particles.len() as u64 * 6007;
                    let x = (seed % 158 + 1) as f32;
                    let b = 60 + (seed % 60) as u8;
                    self.particles.push(WParticle {
                        x, y: 0.0,
                        vx: ((seed % 3) as f32 - 1.0) * 0.012,
                        vy: 0.015 + (seed % 5) as f32 * 0.004,
                        life: 2000 + (seed % 500) as u32, max_life: 2500,
                        ch: '·',
                        col: (b, b, b),
                    });
                }
                for p in &mut self.particles { p.x += p.vx; p.y += p.vy; }
                self.particles.retain(|p| p.y < 79.0);
            }

            WeatherType::Storm => {
                // Random lightning flash every 5-15 seconds
                if self.storm_timer > 0 {
                    self.storm_timer -= 1;
                } else {
                    self.storm_flash = 3; // bright for 3 frames
                    let delay_seed = frame.wrapping_mul(2017);
                    self.storm_timer = 300 + (delay_seed % 600) as u32; // 5-15 seconds
                }
            }
        }

        // Advance spawn accumulator decay
        self.spawn_acc = self.spawn_acc.fract();
        let _ = ft; // suppress unused warning
    }

    pub fn draw(&self, ctx: &mut BTerm, bg: (u8, u8, u8)) {
        let bg_rgb = RGB::from_u8(bg.0, bg.1, bg.2);

        for p in &self.particles {
            let x = p.x as i32;
            let y = p.y as i32;
            if x <= 0 || x >= 159 || y <= 0 || y >= 79 { continue; }

            // Fade based on life
            let life_frac = p.life as f32 / p.max_life as f32;
            let alpha = life_frac.min(1.0 - life_frac * 0.0).clamp(0.0, 1.0);
            let r = (p.col.0 as f32 * alpha) as u8;
            let g = (p.col.1 as f32 * alpha) as u8;
            let b = (p.col.2 as f32 * alpha) as u8;
            if r < 3 && g < 3 && b < 3 { continue; }
            ctx.print_color(x, y, RGB::from_u8(r, g, b), bg_rgb, &p.ch.to_string());
        }

        // Storm flash: drawn by caller via storm_flash field
    }
}
