// Visual themes for CHAOS RPG — Graphical Edition.
// Each theme defines a complete color palette that every draw function consumes.
// Cycle themes with [T] on the title screen.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub name:     &'static str,
    pub tagline:  &'static str,

    // Structural
    pub bg:       (u8, u8, u8),   // canvas background
    pub border:   (u8, u8, u8),   // box borders
    pub panel:    (u8, u8, u8),   // inner panel bg tint (used as bg for sub-boxes)

    // Text hierarchy
    pub heading:  (u8, u8, u8),   // section headings / titles
    pub primary:  (u8, u8, u8),   // normal interactive text
    pub selected: (u8, u8, u8),   // highlighted / selected item
    pub dim:      (u8, u8, u8),   // secondary / description text
    pub muted:    (u8, u8, u8),   // very faint labels

    // Semantic
    pub accent:   (u8, u8, u8),   // eye-catch accents (room icons, separators)
    pub danger:   (u8, u8, u8),   // damage, death, boss, enemy
    pub warn:     (u8, u8, u8),   // warnings, traps, cursed
    pub success:  (u8, u8, u8),   // victory, heal, positive

    // Stats
    pub hp_high:  (u8, u8, u8),
    pub hp_mid:   (u8, u8, u8),
    pub hp_low:   (u8, u8, u8),
    pub mana:     (u8, u8, u8),
    pub gold:     (u8, u8, u8),
    pub xp:       (u8, u8, u8),
}

// ── 5 Unique Themes ───────────────────────────────────────────────────────────

/// 1. VOID PROTOCOL — Deep space, violet/indigo with electric cyan highlights.
///    Feels like staring into the Mandelbrot set at 3 AM.
pub const THEME_VOID: Theme = Theme {
    name:    "VOID PROTOCOL",
    tagline: "Stare into the Mandelbrot set. It stares back.",
    bg:      (6,   4,  16),
    border:  (120,  70, 220),
    panel:   (18,  12,  36),
    heading: (230, 180, 255),
    primary: (170, 130, 245),
    selected:(255, 255, 255),
    dim:     (100,  75, 155),
    muted:   (50,   38,  85),
    accent:  (0,   235, 255),
    danger:  (255,  45,  90),
    warn:    (255, 155,  25),
    success: (80,  235, 150),
    hp_high: (80,  235, 150),
    hp_mid:  (255, 215,  45),
    hp_low:  (255,  45,  90),
    mana:    (90,  155, 255),
    gold:    (255, 215,  45),
    xp:      (200, 110, 255),
};

/// 2. BLOOD PACT — Crimson gothic. Every room costs you something.
///    Deep blacks with blood-red accents and hellfire highlights.
pub const THEME_BLOOD: Theme = Theme {
    name:    "BLOOD PACT",
    tagline: "Every step costs you. Some steps cost everything.",
    bg:      (8,   2,   2),
    border:  (210,  30,  30),
    panel:   (22,   6,   6),
    heading: (255, 145,  85),
    primary: (220,  70,  70),
    selected:(255, 230, 210),
    dim:     (140,  48,  48),
    muted:   (65,   22,  22),
    accent:  (255, 120,  15),
    danger:  (255,  20,  20),
    warn:    (255, 165,  30),
    success: (220, 175, 100),
    hp_high: (220,  90,  60),
    hp_mid:  (255, 140,  40),
    hp_low:  (255,  20,  20),
    mana:    (160,  65, 200),
    gold:    (255, 195,  60),
    xp:      (210,  85, 130),
};

/// 3. EMERALD ENGINE — Matrix green, circuit-board geometry.
///    Data flows. Equations cascade. The dungeon is a compiler.
pub const THEME_EMERALD: Theme = Theme {
    name:    "EMERALD ENGINE",
    tagline: "The dungeon is a compiler. You are undefined behavior.",
    bg:      (0,   8,   2),
    border:  (0,  210,  70),
    panel:   (0,   20,   8),
    heading: (130, 255, 150),
    primary: (0,  215,  90),
    selected:(225, 255, 225),
    dim:     (0,  115,  50),
    muted:   (0,   55,  22),
    accent:  (0,  255, 180),
    danger:  (255,  85,  40),
    warn:    (215, 235,   0),
    success: (0,  255, 110),
    hp_high: (0,  215,  90),
    hp_mid:  (170, 230,  40),
    hp_low:  (255,  85,  40),
    mana:    (0,  190, 255),
    gold:    (215, 235,  40),
    xp:      (90,  255, 170),
};

/// 4. SOLAR FORGE — Amber/gold desert heat. Alchemical fire.
///    Warm, dusty, dangerous. The math burns here.
pub const THEME_SOLAR: Theme = Theme {
    name:    "SOLAR FORGE",
    tagline: "The equations combust at this temperature. Good.",
    bg:      (12,   7,   0),
    border:  (245, 130,   0),
    panel:   (24,  14,   0),
    heading: (255, 215,  65),
    primary: (235, 155,  25),
    selected:(255, 245, 185),
    dim:     (160,  90,  22),
    muted:   (80,   45,  10),
    accent:  (255, 195,   0),
    danger:  (255,  60,   0),
    warn:    (255, 150,   0),
    success: (185, 230,  80),
    hp_high: (185, 215,  60),
    hp_mid:  (255, 170,  20),
    hp_low:  (255,  60,   0),
    mana:    (85,  170, 235),
    gold:    (255, 220,  40),
    xp:      (215, 150,  45),
};

/// 5. GLACIAL ABYSS — Icy blue, crystalline cold. Absolute zero math.
///    Clean, precise, merciless. The coldest algorithms live here.
pub const THEME_GLACIAL: Theme = Theme {
    name:    "GLACIAL ABYSS",
    tagline: "Absolute zero. The equations freeze mid-cascade.",
    bg:      (0,   7,  15),
    border:  (0,  175, 245),
    panel:   (0,  14,  28),
    heading: (155, 230, 255),
    primary: (0,  195, 255),
    selected:(225, 245, 255),
    dim:     (0,  105, 165),
    muted:   (0,   52,  85),
    accent:  (90,  255, 255),
    danger:  (235,  65, 110),
    warn:    (230, 195,  40),
    success: (85,  235, 210),
    hp_high: (85,  215, 235),
    hp_mid:  (150, 210,  60),
    hp_low:  (235,  65, 110),
    mana:    (110, 170, 255),
    gold:    (210, 235,  85),
    xp:      (90,  215, 255),
};

pub const THEMES: [Theme; 5] = [
    THEME_VOID,
    THEME_BLOOD,
    THEME_EMERALD,
    THEME_SOLAR,
    THEME_GLACIAL,
];

impl Theme {
    pub fn hp_color(&self, pct: f32) -> (u8, u8, u8) {
        if pct > 0.6 { self.hp_high } else if pct > 0.3 { self.hp_mid } else { self.hp_low }
    }

    /// Lerp two colors for gradient effects.
    pub fn lerp(a: (u8,u8,u8), b: (u8,u8,u8), t: f32) -> (u8,u8,u8) {
        let t = t.clamp(0.0, 1.0);
        (
            (a.0 as f32 + (b.0 as f32 - a.0 as f32) * t) as u8,
            (a.1 as f32 + (b.1 as f32 - a.1 as f32) * t) as u8,
            (a.2 as f32 + (b.2 as f32 - a.2 as f32) * t) as u8,
        )
    }
}
