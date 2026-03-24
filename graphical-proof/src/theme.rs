//! Visual themes mapped to proof-engine color and shader presets.
//!
//! Each theme defines colors as `glam::Vec4` for direct use with the engine,
//! plus engine-specific properties (bloom intensity, post-processing hints).

use proof_engine::prelude::{Vec3, Vec4};

/// Convert (u8, u8, u8) to Vec4 with full alpha.
const fn rgb(r: u8, g: u8, b: u8) -> Vec4 {
    Vec4::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}

/// Convert (u8, u8, u8) to Vec3 for glow colors.
const fn rgb3(r: u8, g: u8, b: u8) -> Vec3 {
    Vec3::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub name: &'static str,
    pub tagline: &'static str,

    // Structural
    pub bg: Vec4,
    pub border: Vec4,
    pub panel: Vec4,

    // Text hierarchy
    pub heading: Vec4,
    pub primary: Vec4,
    pub selected: Vec4,
    pub dim: Vec4,
    pub muted: Vec4,

    // Semantic
    pub accent: Vec4,
    pub danger: Vec4,
    pub warn: Vec4,
    pub success: Vec4,

    // Stats
    pub hp_high: Vec4,
    pub hp_mid: Vec4,
    pub hp_low: Vec4,
    pub mana: Vec4,
    pub gold: Vec4,
    pub xp: Vec4,

    // Engine-specific
    pub bloom_intensity: f32,
    pub chromatic_aberration: f32,
    pub vignette_strength: f32,
    pub chaos_field_brightness: f32,
}

impl Theme {
    pub fn hp_color(&self, pct: f32) -> Vec4 {
        if pct > 0.6 {
            self.hp_high
        } else if pct > 0.3 {
            self.hp_mid
        } else {
            self.hp_low
        }
    }

    pub fn glow_from(&self, color: Vec4) -> Vec3 {
        Vec3::new(color.x, color.y, color.z)
    }
}

// ── 5 Themes ─────────────────────────────────────────────────────────────────

pub const THEME_VOID: Theme = Theme {
    name: "VOID PROTOCOL",
    tagline: "Stare into the Mandelbrot set. It stares back.",
    bg:       rgb(4, 2, 12),
    border:   rgb(140, 50, 255),
    panel:    rgb(12, 6, 30),
    heading:  rgb(220, 160, 255),
    primary:  rgb(160, 110, 240),
    selected: rgb(255, 255, 255),
    dim:      rgb(90, 60, 140),
    muted:    rgb(40, 26, 70),
    accent:   rgb(0, 255, 255),
    danger:   rgb(255, 0, 80),
    warn:     rgb(255, 160, 0),
    success:  rgb(0, 255, 120),
    hp_high:  rgb(0, 255, 120),
    hp_mid:   rgb(255, 220, 0),
    hp_low:   rgb(255, 0, 80),
    mana:     rgb(60, 140, 255),
    gold:     rgb(255, 220, 0),
    xp:       rgb(210, 80, 255),
    bloom_intensity: 1.2,
    chromatic_aberration: 0.003,
    vignette_strength: 0.4,
    chaos_field_brightness: 0.08,
};

pub const THEME_BLOOD: Theme = Theme {
    name: "BLOOD PACT",
    tagline: "Every step costs you. Some steps cost everything.",
    bg:       rgb(6, 0, 0),
    border:   rgb(230, 20, 20),
    panel:    rgb(18, 3, 3),
    heading:  rgb(255, 130, 60),
    primary:  rgb(215, 55, 55),
    selected: rgb(255, 235, 215),
    dim:      rgb(130, 35, 35),
    muted:    rgb(55, 14, 14),
    accent:   rgb(255, 100, 0),
    danger:   rgb(255, 0, 0),
    warn:     rgb(255, 160, 20),
    success:  rgb(215, 170, 90),
    hp_high:  rgb(210, 75, 50),
    hp_mid:   rgb(255, 130, 30),
    hp_low:   rgb(255, 0, 0),
    mana:     rgb(170, 50, 210),
    gold:     rgb(255, 200, 40),
    xp:       rgb(220, 70, 120),
    bloom_intensity: 0.8,
    chromatic_aberration: 0.001,
    vignette_strength: 0.6,
    chaos_field_brightness: 0.06,
};

pub const THEME_EMERALD: Theme = Theme {
    name: "EMERALD ENGINE",
    tagline: "The dungeon is a compiler. You are undefined behavior.",
    bg:       rgb(0, 6, 1),
    border:   rgb(0, 230, 60),
    panel:    rgb(0, 14, 4),
    heading:  rgb(120, 255, 140),
    primary:  rgb(0, 220, 80),
    selected: rgb(230, 255, 230),
    dim:      rgb(0, 110, 45),
    muted:    rgb(0, 48, 18),
    accent:   rgb(0, 255, 200),
    danger:   rgb(255, 70, 30),
    warn:     rgb(220, 255, 0),
    success:  rgb(0, 255, 90),
    hp_high:  rgb(0, 220, 80),
    hp_mid:   rgb(180, 240, 30),
    hp_low:   rgb(255, 70, 30),
    mana:     rgb(0, 200, 255),
    gold:     rgb(220, 250, 30),
    xp:       rgb(80, 255, 180),
    bloom_intensity: 1.0,
    chromatic_aberration: 0.002,
    vignette_strength: 0.3,
    chaos_field_brightness: 0.07,
};

pub const THEME_SOLAR: Theme = Theme {
    name: "SOLAR FORGE",
    tagline: "The equations combust at this temperature. Good.",
    bg:       rgb(10, 5, 0),
    border:   rgb(255, 140, 0),
    panel:    rgb(20, 10, 0),
    heading:  rgb(255, 225, 60),
    primary:  rgb(240, 160, 20),
    selected: rgb(255, 250, 200),
    dim:      rgb(155, 85, 18),
    muted:    rgb(70, 38, 6),
    accent:   rgb(255, 210, 0),
    danger:   rgb(255, 50, 0),
    warn:     rgb(255, 155, 0),
    success:  rgb(190, 240, 70),
    hp_high:  rgb(190, 225, 50),
    hp_mid:   rgb(255, 175, 15),
    hp_low:   rgb(255, 50, 0),
    mana:     rgb(80, 175, 245),
    gold:     rgb(255, 230, 30),
    xp:       rgb(220, 155, 40),
    bloom_intensity: 1.4,
    chromatic_aberration: 0.0,
    vignette_strength: 0.5,
    chaos_field_brightness: 0.09,
};

pub const THEME_GLACIAL: Theme = Theme {
    name: "GLACIAL ABYSS",
    tagline: "Absolute zero. The equations freeze mid-cascade.",
    bg:       rgb(0, 5, 14),
    border:   rgb(0, 200, 255),
    panel:    rgb(0, 10, 24),
    heading:  rgb(140, 240, 255),
    primary:  rgb(0, 210, 255),
    selected: rgb(220, 250, 255),
    dim:      rgb(0, 100, 160),
    muted:    rgb(0, 46, 80),
    accent:   rgb(0, 255, 255),
    danger:   rgb(255, 50, 100),
    warn:     rgb(240, 210, 30),
    success:  rgb(70, 245, 215),
    hp_high:  rgb(70, 220, 240),
    hp_mid:   rgb(140, 215, 50),
    hp_low:   rgb(255, 50, 100),
    mana:     rgb(100, 165, 255),
    gold:     rgb(215, 240, 80),
    xp:       rgb(80, 220, 255),
    bloom_intensity: 0.9,
    chromatic_aberration: 0.001,
    vignette_strength: 0.35,
    chaos_field_brightness: 0.065,
};

pub const THEMES: [Theme; 5] = [
    THEME_VOID,
    THEME_BLOOD,
    THEME_EMERALD,
    THEME_SOLAR,
    THEME_GLACIAL,
];
