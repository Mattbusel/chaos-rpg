// Dynamic color grading system — CHAOS RPG Visual Push
//
// A global ColorGrade lerps between named presets each frame.
// Applied via State::theme_graded() which bakes the grade into a Theme copy
// before any draw call — zero changes at draw-call sites.

use crate::theme::Theme;

// ── HSL helpers ───────────────────────────────────────────────────────────────

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;
    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;
    let l = (max + min) * 0.5;
    let s = if delta < 0.001 { 0.0 } else { delta / (1.0 - (2.0 * l - 1.0).abs()).max(0.001) };
    let h = if delta < 0.001 {
        0.0f32
    } else if max == rf {
        60.0 * (((gf - bf) / delta).rem_euclid(6.0))
    } else if max == gf {
        60.0 * ((bf - rf) / delta + 2.0)
    } else {
        60.0 * ((rf - gf) / delta + 4.0)
    };
    (h.rem_euclid(360.0), s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
    let m = l - c * 0.5;
    let (rf, gf, bf) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((rf + m).clamp(0.0, 1.0) * 255.0) as u8,
        ((gf + m).clamp(0.0, 1.0) * 255.0) as u8,
        ((bf + m).clamp(0.0, 1.0) * 255.0) as u8,
    )
}

// ── Grade ────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct Grade {
    pub saturation: f32,           // 0.0=grayscale 1.0=normal 1.5=oversaturated
    pub brightness: f32,           // global multiplier
    pub contrast:   f32,           // 1.0=normal
    pub tint:       (f32, f32, f32), // additive RGB tint (-0.3..0.3)
    pub hue_shift:  f32,           // degrees to rotate all hues
}

impl Default for Grade {
    fn default() -> Self {
        Self { saturation: 1.0, brightness: 1.0, contrast: 1.0, tint: (0.0, 0.0, 0.0), hue_shift: 0.0 }
    }
}

impl Grade {
    pub const NORMAL: Grade = Grade {
        saturation: 1.0, brightness: 1.0, contrast: 1.0,
        tint: (0.0, 0.0, 0.0), hue_shift: 0.0,
    };
    pub const LOW_HP: Grade = Grade {
        saturation: 0.85, brightness: 0.9, contrast: 1.1,
        tint: (0.06, -0.01, -0.01), hue_shift: 0.0,
    };
    pub const NULL_FIGHT: Grade = Grade {
        saturation: 0.0, brightness: 0.85, contrast: 1.15,
        tint: (0.0, 0.0, 0.0), hue_shift: 0.0,
    };
    pub const PARADOX: Grade = Grade {
        saturation: 1.2, brightness: 0.95, contrast: 1.05,
        tint: (0.0, 0.0, 0.0), hue_shift: 180.0,
    };
    pub const DEATH: Grade = Grade {
        saturation: 0.0, brightness: 0.55, contrast: 1.2,
        tint: (0.0, 0.0, 0.0), hue_shift: 0.0,
    };
    pub const SHRINE: Grade = Grade {
        saturation: 1.1, brightness: 1.08, contrast: 0.95,
        tint: (0.0, 0.02, 0.04), hue_shift: 0.0,
    };
    pub const CHAOS_RIFT: Grade = Grade {
        saturation: 1.5, brightness: 1.1, contrast: 1.2,
        tint: (0.0, 0.0, 0.0), hue_shift: 0.0, // hue oscillates externally
    };
    pub const DEEP_FLOOR: Grade = Grade {
        saturation: 0.95, brightness: 0.93, contrast: 1.1,
        tint: (0.01, 0.0, 0.02), hue_shift: 0.0,
    };
    pub const HIGH_CORRUPTION: Grade = Grade {
        saturation: 1.1, brightness: 0.92, contrast: 1.15,
        tint: (0.02, 0.0, 0.04), hue_shift: 5.0,
    };
    pub const BOSS_PHASE2: Grade = Grade {
        saturation: 1.15, brightness: 0.97, contrast: 1.1,
        tint: (0.03, 0.0, 0.0), hue_shift: 0.0,
    };
    pub const BOSS_PHASE3: Grade = Grade {
        saturation: 1.3, brightness: 0.94, contrast: 1.2,
        tint: (0.04, 0.0, 0.01), hue_shift: 10.0,
    };
    pub const VICTORY: Grade = Grade {
        saturation: 1.2, brightness: 1.1, contrast: 0.95,
        tint: (0.03, 0.03, 0.0), hue_shift: 0.0,
    };

    pub fn lerp(a: &Grade, b: &Grade, t: f32) -> Grade {
        let t = t.clamp(0.0, 1.0);
        Grade {
            saturation: a.saturation + (b.saturation - a.saturation) * t,
            brightness: a.brightness + (b.brightness - a.brightness) * t,
            contrast:   a.contrast   + (b.contrast   - a.contrast)   * t,
            tint: (
                a.tint.0 + (b.tint.0 - a.tint.0) * t,
                a.tint.1 + (b.tint.1 - a.tint.1) * t,
                a.tint.2 + (b.tint.2 - a.tint.2) * t,
            ),
            hue_shift: a.hue_shift + (b.hue_shift - a.hue_shift) * t,
        }
    }
}

// ── ColorGrade ────────────────────────────────────────────────────────────────

pub struct ColorGrade {
    pub current: Grade,
    pub target:  Grade,
    pub speed:   f32,   // lerp speed per frame (0.02 = slow, 0.12 = fast)

    // For chaos rift: oscillating hue
    pub hue_oscillate: bool,
    pub hue_osc_phase: f32,
}

impl Default for ColorGrade {
    fn default() -> Self {
        Self {
            current: Grade::NORMAL,
            target:  Grade::NORMAL,
            speed:   0.04,
            hue_oscillate: false,
            hue_osc_phase: 0.0,
        }
    }
}

impl ColorGrade {
    pub fn set_target(&mut self, target: Grade, speed: f32) {
        self.target = target;
        self.speed  = speed;
        self.hue_oscillate = false;
    }

    pub fn set_normal(&mut self)        { self.set_target(Grade::NORMAL,       0.04); }
    pub fn set_low_hp(&mut self)        { self.set_target(Grade::LOW_HP,       0.03); }
    pub fn set_null_fight(&mut self)    { self.set_target(Grade::NULL_FIGHT,   0.015); }
    pub fn set_paradox(&mut self)       { self.set_target(Grade::PARADOX,      0.06); }
    pub fn set_death(&mut self)         { self.set_target(Grade::DEATH,        0.12); }
    pub fn set_shrine(&mut self)        { self.set_target(Grade::SHRINE,       0.08); }
    pub fn set_high_corruption(&mut self) { self.set_target(Grade::HIGH_CORRUPTION, 0.02); }
    pub fn set_boss_phase2(&mut self)   { self.set_target(Grade::BOSS_PHASE2,  0.08); }
    pub fn set_boss_phase3(&mut self)   { self.set_target(Grade::BOSS_PHASE3,  0.08); }
    pub fn set_victory(&mut self)       { self.set_target(Grade::VICTORY,      0.05); }
    pub fn set_deep_floor(&mut self)    { self.set_target(Grade::DEEP_FLOOR,   0.01); }

    pub fn set_chaos_rift(&mut self) {
        self.set_target(Grade::CHAOS_RIFT, 0.10);
        self.hue_oscillate = true;
    }

    pub fn update(&mut self) {
        let t = self.speed;
        self.current.saturation += (self.target.saturation - self.current.saturation) * t;
        self.current.brightness += (self.target.brightness - self.current.brightness) * t;
        self.current.contrast   += (self.target.contrast   - self.current.contrast)   * t;
        self.current.tint.0 += (self.target.tint.0 - self.current.tint.0) * t;
        self.current.tint.1 += (self.target.tint.1 - self.current.tint.1) * t;
        self.current.tint.2 += (self.target.tint.2 - self.current.tint.2) * t;
        if self.hue_oscillate {
            self.hue_osc_phase += 0.03;
            self.current.hue_shift = self.hue_osc_phase.sin() * 60.0;
        } else {
            self.current.hue_shift += (self.target.hue_shift - self.current.hue_shift) * t;
        }
    }

    /// Apply the current grade to a single (r,g,b) color tuple.
    pub fn apply(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        let g_ = &self.current;
        let mut rf = r as f32 / 255.0;
        let mut gf = g as f32 / 255.0;
        let mut bf = b as f32 / 255.0;

        // Contrast (pivot at 0.5)
        rf = ((rf - 0.5) * g_.contrast + 0.5).clamp(0.0, 1.0);
        gf = ((gf - 0.5) * g_.contrast + 0.5).clamp(0.0, 1.0);
        bf = ((bf - 0.5) * g_.contrast + 0.5).clamp(0.0, 1.0);

        // Saturation
        let luma = 0.299 * rf + 0.587 * gf + 0.114 * bf;
        rf = (luma + (rf - luma) * g_.saturation).clamp(0.0, 1.0);
        gf = (luma + (gf - luma) * g_.saturation).clamp(0.0, 1.0);
        bf = (luma + (bf - luma) * g_.saturation).clamp(0.0, 1.0);

        // Hue shift
        if g_.hue_shift.abs() > 0.5 {
            let (h, s, l) = rgb_to_hsl(
                (rf * 255.0) as u8,
                (gf * 255.0) as u8,
                (bf * 255.0) as u8,
            );
            let (nr, ng, nb) = hsl_to_rgb((h + g_.hue_shift).rem_euclid(360.0), s, l);
            rf = nr as f32 / 255.0;
            gf = ng as f32 / 255.0;
            bf = nb as f32 / 255.0;
        }

        // Brightness
        rf = (rf * g_.brightness).clamp(0.0, 1.0);
        gf = (gf * g_.brightness).clamp(0.0, 1.0);
        bf = (bf * g_.brightness).clamp(0.0, 1.0);

        // Tint
        rf = (rf + g_.tint.0).clamp(0.0, 1.0);
        gf = (gf + g_.tint.1).clamp(0.0, 1.0);
        bf = (bf + g_.tint.2).clamp(0.0, 1.0);

        ((rf * 255.0) as u8, (gf * 255.0) as u8, (bf * 255.0) as u8)
    }

    /// Apply grade to every color field of a Theme clone.
    pub fn apply_to_theme(&self, t: &mut Theme) {
        macro_rules! g {
            ($f:ident) => { t.$f = self.apply(t.$f.0, t.$f.1, t.$f.2); };
        }
        g!(bg); g!(border); g!(panel); g!(heading); g!(primary);
        g!(selected); g!(dim); g!(muted); g!(accent); g!(danger);
        g!(warn); g!(success); g!(hp_high); g!(hp_mid); g!(hp_low);
        g!(mana); g!(gold); g!(xp);
    }
}
