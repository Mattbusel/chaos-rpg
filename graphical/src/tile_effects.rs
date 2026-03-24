// Per-tile animation and overlay effects — CHAOS RPG Visual Push
//
// Maintains a 160×80 influence grid updated every frame.
// Effects are drawn as an overlay pass AFTER the main UI:
//   - Pulse rings    expanding circles of bright chars
//   - Impact ripples brightness wave from hit point
//   - Earthquake     per-tile jitter (random brightness overlay)
//   - Screen lighting point lights with radial falloff
//   - Vignette       dark overlay at screen edges
//   - Low-HP edge    red pulsing edge glow when player is near death
//   - Bloom          color bleed from registered bright positions

use bracket_lib::prelude::*;

pub const W: usize = 160;
pub const H: usize = 80;

// ── Influence cell ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Default)]
struct Cell {
    brightness: f32,           // additive brightness: 0.0 = no boost
    color: (f32, f32, f32),    // additive color tint
}

// ── Effects ───────────────────────────────────────────────────────────────────

pub struct PulseRing {
    pub cx:     f32,
    pub cy:     f32,
    pub radius: f32,
    max_r:      f32,
    pub intensity: f32,
    pub color:  (f32, f32, f32),
    speed:      f32,
    dead:       bool,
}

pub struct Ripple {
    cx:        f32,
    cy:        f32,
    radius:    f32,
    max_r:     f32,
    amplitude: f32,
    color:     (f32, f32, f32),
    dead:      bool,
}

pub struct LightSource {
    pub x:         f32,
    pub y:         f32,
    pub radius:    f32,
    pub intensity: f32,
    pub color:     (f32, f32, f32),
    pub pulse:     bool,
    phase:         f32,
}

pub struct BloomSpot {
    x:     i32,
    y:     i32,
    color: (f32, f32, f32),
    strength: f32,
}

// ── TileEffects ───────────────────────────────────────────────────────────────

pub struct TileEffects {
    grid:        Vec<Cell>,        // 160 × 80 influence
    border_phase: f32,             // breathing border sine

    pub pulse_rings: Vec<PulseRing>,
    pub ripples:     Vec<Ripple>,
    bloom_spots:     Vec<BloomSpot>,

    pub earthquake_frames: u32,
    pub earthquake_max:    u32,
    pub earthquake_intensity: f32,

    pub lights: Vec<LightSource>,

    pub vignette:        f32,
    pub vignette_target: f32,
    vignette_color:      (f32, f32, f32),

    pub low_hp:        f32,
    pub low_hp_target: f32,
    pub low_hp_pulse:  f32,     // oscillating phase for pulse
}

impl TileEffects {
    pub fn new() -> Self {
        Self {
            grid:          vec![Cell::default(); W * H],
            border_phase:  1.0,
            pulse_rings:   Vec::new(),
            ripples:       Vec::new(),
            bloom_spots:   Vec::new(),
            earthquake_frames: 0,
            earthquake_max:    1,
            earthquake_intensity: 0.0,
            lights:        Vec::new(),
            vignette:        0.0,
            vignette_target: 0.0,
            vignette_color:  (0.0, 0.0, 0.0),
            low_hp:        0.0,
            low_hp_target: 0.0,
            low_hp_pulse:  0.0,
        }
    }

    // ── Emitters ─────────────────────────────────────────────────────────────

    pub fn emit_pulse_ring(&mut self, cx: f32, cy: f32, color: (f32,f32,f32), intensity: f32, max_r: f32) {
        self.pulse_rings.push(PulseRing { cx, cy, radius: 0.0, max_r, intensity, color, speed: 0.45, dead: false });
    }

    pub fn emit_impact_ripple(&mut self, cx: f32, cy: f32, color: (f32,f32,f32)) {
        self.ripples.push(Ripple { cx, cy, radius: 0.0, max_r: 10.0, amplitude: 0.7, color, dead: false });
    }

    pub fn emit_earthquake(&mut self, intensity: f32, frames: u32) {
        self.earthquake_frames = frames;
        self.earthquake_max    = frames;
        self.earthquake_intensity = intensity;
    }

    pub fn register_bloom(&mut self, x: i32, y: i32, color: (f32,f32,f32), strength: f32) {
        self.bloom_spots.push(BloomSpot { x, y, color, strength });
    }

    pub fn set_vignette(&mut self, target: f32, color: (f32,f32,f32)) {
        self.vignette_target = target.clamp(0.0, 1.0);
        self.vignette_color  = color;
    }

    pub fn set_low_hp(&mut self, target: f32) {
        self.low_hp_target = target.clamp(0.0, 1.0);
    }

    pub fn clear_low_hp(&mut self) {
        self.low_hp        = 0.0;
        self.low_hp_target = 0.0;
    }

    pub fn add_light(&mut self, x: f32, y: f32, radius: f32, intensity: f32, color: (f32,f32,f32), pulse: bool) {
        self.lights.push(LightSource { x, y, radius, intensity, color, pulse, phase: 0.0 });
    }

    pub fn clear_lights(&mut self) { self.lights.clear(); }

    // ── Per-frame update ─────────────────────────────────────────────────────

    pub fn update(&mut self, frame: u64) {
        let ft = frame as f32;

        // Breathing border
        self.border_phase = (ft * 0.018).sin() * 0.07 + 1.0;

        // Lerp vignette
        self.vignette += (self.vignette_target - self.vignette) * 0.04;

        // Lerp low HP
        self.low_hp += (self.low_hp_target - self.low_hp) * 0.05;
        self.low_hp_pulse = (ft * 0.08).sin() * 0.5 + 0.5;

        // Advance pulse rings
        for r in &mut self.pulse_rings {
            r.radius += r.speed;
            if r.radius >= r.max_r { r.dead = true; }
        }
        self.pulse_rings.retain(|r| !r.dead);

        // Advance ripples
        for r in &mut self.ripples {
            r.radius += 0.35;
            r.amplitude *= 0.88;
            if r.amplitude < 0.02 || r.radius >= r.max_r { r.dead = true; }
        }
        self.ripples.retain(|r| !r.dead);

        // Advance earthquake
        if self.earthquake_frames > 0 { self.earthquake_frames -= 1; }

        // Advance lights
        for l in &mut self.lights { l.phase += 0.05; }

        // Reset grid
        for c in &mut self.grid { *c = Cell::default(); }

        // ── Accumulate into grid ──────────────────────────────────────────────

        // Pulse rings
        for ring in &self.pulse_rings {
            let fade = 1.0 - ring.radius / ring.max_r;
            let r_sq = ring.radius * ring.radius;
            let inner = (ring.radius - 1.5).max(0.0);
            let inner_sq = inner * inner;
            for y in 0..H {
                for x in 0..W {
                    let dx = x as f32 - ring.cx;
                    let dy = y as f32 - ring.cy;
                    let dsq = dx*dx + dy*dy;
                    if dsq <= r_sq && dsq >= inner_sq {
                        let dist_from_edge = (r_sq.sqrt() - dsq.sqrt()).min((dsq.sqrt() - inner_sq.sqrt()).max(0.0));
                        let strength = (1.0 - (dist_from_edge / 1.5).min(1.0)) * ring.intensity * fade;
                        let c = &mut self.grid[y * W + x];
                        c.brightness += strength * 0.5;
                        c.color.0 += ring.color.0 * strength * 0.3;
                        c.color.1 += ring.color.1 * strength * 0.3;
                        c.color.2 += ring.color.2 * strength * 0.3;
                    }
                }
            }
        }

        // Ripples
        for rip in &self.ripples {
            for y in 0..H {
                for x in 0..W {
                    let dx = x as f32 - rip.cx;
                    let dy = y as f32 - rip.cy;
                    let dist = (dx*dx + dy*dy).sqrt();
                    let ring_d = (dist - rip.radius).abs();
                    if ring_d < 2.0 {
                        let s = (1.0 - ring_d / 2.0) * rip.amplitude;
                        let c = &mut self.grid[y * W + x];
                        c.brightness += s * 0.4;
                        c.color.0 += rip.color.0 * s * 0.2;
                        c.color.1 += rip.color.1 * s * 0.2;
                        c.color.2 += rip.color.2 * s * 0.2;
                    }
                }
            }
        }

        // Screen-space lighting
        for light in &self.lights {
            let pulse_mod = if light.pulse { (light.phase.sin() * 0.2 + 1.0) } else { 1.0 };
            let eff = light.intensity * pulse_mod;
            let rx = light.x as i32;
            let ry = light.y as i32;
            let ir = light.radius as i32 + 1;
            for dy in -ir..=ir {
                for dx in -ir..=ir {
                    let tx = rx + dx;
                    let ty = ry + dy;
                    if tx < 0 || tx >= W as i32 || ty < 0 || ty >= H as i32 { continue; }
                    let dist = ((dx*dx + dy*dy) as f32).sqrt();
                    if dist < light.radius {
                        let falloff = (1.0 - dist / light.radius).powi(2);
                        let c = &mut self.grid[ty as usize * W + tx as usize];
                        c.brightness += falloff * eff * 0.35;
                        c.color.0 += light.color.0 * falloff * eff * 0.12;
                        c.color.1 += light.color.1 * falloff * eff * 0.12;
                        c.color.2 += light.color.2 * falloff * eff * 0.12;
                    }
                }
            }
        }

        // Bloom — bleed registered bright spots into neighbors
        for spot in &self.bloom_spots {
            for dy in -2i32..=2 {
                for dx in -2i32..=2 {
                    if dx == 0 && dy == 0 { continue; }
                    let nx = spot.x + dx;
                    let ny = spot.y + dy;
                    if nx < 0 || nx >= W as i32 || ny < 0 || ny >= H as i32 { continue; }
                    let dist = ((dx*dx + dy*dy) as f32).sqrt();
                    let bleed = spot.strength * (1.0 - dist / 3.0).max(0.0) * 0.10;
                    let c = &mut self.grid[ny as usize * W + nx as usize];
                    c.brightness += bleed;
                    c.color.0 += spot.color.0 * bleed;
                    c.color.1 += spot.color.1 * bleed;
                    c.color.2 += spot.color.2 * bleed;
                }
            }
        }
        self.bloom_spots.clear();

        // Earthquake — random per-tile brightness noise decaying over time
        if self.earthquake_frames > 0 {
            let decay = self.earthquake_frames as f32 / self.earthquake_max as f32;
            let intensity = self.earthquake_intensity * decay;
            for y in 0..H {
                for x in 0..W {
                    let n = ((x as f32 * 17.3 + y as f32 * 11.7 + ft * 4.1).sin()
                             * (x as f32 * 7.9  + y as f32 * 13.1 + ft * 2.3).cos()) * 0.5 + 0.5;
                    self.grid[y * W + x].brightness += (n - 0.5) * intensity * 0.6;
                }
            }
        }
    }

    // ── Color application (call when computing fg/bg for any drawn element) ──

    /// Apply tile influence at position (x, y) to a color tuple.
    pub fn apply(&self, x: i32, y: i32, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        if x < 0 || x >= W as i32 || y < 0 || y >= H as i32 {
            return (r, g, b);
        }
        let c = &self.grid[y as usize * W + x as usize];
        if c.brightness.abs() < 0.01 && c.color.0.abs() < 0.01
                && c.color.1.abs() < 0.01 && c.color.2.abs() < 0.01 {
            return (r, g, b);
        }
        let rf = (r as f32 / 255.0 + c.color.0) * (1.0 + c.brightness);
        let gf = (g as f32 / 255.0 + c.color.1) * (1.0 + c.brightness);
        let bf = (b as f32 / 255.0 + c.color.2) * (1.0 + c.brightness);
        ((rf.clamp(0.0, 1.0) * 255.0) as u8,
         (gf.clamp(0.0, 1.0) * 255.0) as u8,
         (bf.clamp(0.0, 1.0) * 255.0) as u8)
    }

    /// Border brightness multiplier for breathing panels.
    pub fn border_brightness(&self) -> f32 { self.border_phase }

    // ── Overlay draw pass (call AFTER main UI drawing) ─────────────────────

    pub fn draw_overlay(&self, ctx: &mut BTerm, bg: (u8, u8, u8)) {
        let bg_rgb = RGB::from_u8(bg.0, bg.1, bg.2);

        // Pulse rings — drawn as bright ring of `·` characters at ring perimeter
        for ring in &self.pulse_rings {
            let fade = 1.0 - ring.radius / ring.max_r;
            let r_lo = (ring.radius - 1.0).max(0.0) as i32;
            let r_hi = (ring.radius + 1.0) as i32;
            for y in 0..H as i32 {
                for x in 0..W as i32 {
                    let dx = x as f32 - ring.cx;
                    let dy = y as f32 - ring.cy;
                    let dist = (dx*dx + dy*dy).sqrt() as i32;
                    if dist >= r_lo && dist <= r_hi {
                        let edge_d = ((dx*dx + dy*dy).sqrt() - ring.radius).abs();
                        let s = (1.0 - edge_d / 1.2).max(0.0) * ring.intensity * fade;
                        if s > 0.04 {
                            let r = (ring.color.0 * s * 255.0) as u8;
                            let g = (ring.color.1 * s * 255.0) as u8;
                            let b = (ring.color.2 * s * 255.0) as u8;
                            ctx.print_color(x, y, RGB::from_u8(r.max(4), g.max(4), b.max(4)), bg_rgb, "·");
                        }
                    }
                }
            }
        }

        // Vignette — dark overlay at screen edges
        if self.vignette > 0.01 {
            for y in 0..H as i32 {
                for x in 0..W as i32 {
                    let ex = (x.min(W as i32 - 1 - x) as f32 / (W as f32 * 0.14)).min(1.0);
                    let ey = (y.min(H as i32 - 1 - y) as f32 / (H as f32 * 0.14)).min(1.0);
                    let edge = (1.0 - ex.min(ey)).powi(2) * self.vignette;
                    if edge > 0.12 {
                        let vc = self.vignette_color;
                        let r = (vc.0 * edge * 180.0) as u8;
                        let g = (vc.1 * edge * 180.0) as u8;
                        let b = (vc.2 * edge * 180.0) as u8;
                        let dark = RGB::from_u8(r, g, b);
                        ctx.print_color(x, y, dark, dark, " ");
                    }
                }
            }
        }

        // Low-HP pulsing red edge
        if self.low_hp > 0.02 {
            let pulse_mult = 0.7 + self.low_hp_pulse * 0.3;
            let strength = self.low_hp * pulse_mult;
            for y in 0..H as i32 {
                for x in 0..W as i32 {
                    let ex = (x.min(W as i32 - 1 - x) as f32 / (W as f32 * 0.10)).min(1.0);
                    let ey = (y.min(H as i32 - 1 - y) as f32 / (H as f32 * 0.10)).min(1.0);
                    let edge = (1.0 - ex.min(ey)).powi(2) * strength;
                    if edge > 0.10 {
                        let r = (edge * 140.0) as u8;
                        let red = RGB::from_u8(r.max(5), 0, 0);
                        ctx.print_color(x, y, red, red, " ");
                    }
                }
            }
        }
    }
}
