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

/// 1. VOID PROTOCOL — Deep space, violet/indigo with electric blue highlights.
///    Feels like staring into the Mandelbrot set at 3 AM.
pub const THEME_VOID: Theme = Theme {
    name:    "VOID PROTOCOL",
    tagline: "Stare into the Mandelbrot set. It stares back.",
    bg:      (8,  6,  18),
    border:  (80, 50, 160),
    panel:   (14, 10, 28),
    heading: (200, 160, 255),
    primary: (140, 110, 220),
    selected:(255, 255, 255),
    dim:     (80,  60, 120),
    muted:   (45,  35,  70),
    accent:  (0,  210, 255),
    danger:  (220,  40,  80),
    warn:    (240, 140,  20),
    success: (80,  220, 140),
    hp_high: (80,  220, 140),
    hp_mid:  (240, 200,  40),
    hp_low:  (220,  40,  80),
    mana:    (80, 140, 255),
    gold:    (240, 200,  40),
    xp:      (160, 100, 255),
};

/// 2. BLOOD PACT — Crimson gothic. Every room costs you something.
///    Deep blacks with blood-red accents and ember highlights.
pub const THEME_BLOOD: Theme = Theme {
    name:    "BLOOD PACT",
    tagline: "Every step costs you. Some steps cost everything.",
    bg:      (10,  2,  2),
    border:  (160, 20, 20),
    panel:   (18,  4,  4),
    heading: (255, 120,  80),
    primary: (200,  60,  60),
    selected:(255, 220, 200),
    dim:     (120,  40,  40),
    muted:   (60,  20,  20),
    accent:  (255, 100,  20),
    danger:  (255,  20,  20),
    warn:    (255, 150,  30),
    success: (220, 160, 100),
    hp_high: (200,  80,  60),
    hp_mid:  (255, 130,  40),
    hp_low:  (255,  20,  20),
    mana:    (140,  60, 180),
    gold:    (255, 180,  60),
    xp:      (200,  80, 120),
};

/// 3. EMERALD ENGINE — Matrix green, circuit-board geometry.
///    Data flows. Equations cascade. The dungeon is a compiler.
pub const THEME_EMERALD: Theme = Theme {
    name:    "EMERALD ENGINE",
    tagline: "The dungeon is a compiler. You are undefined behavior.",
    bg:      (0,  10,  4),
    border:  (0, 160,  60),
    panel:   (0,  16,  6),
    heading: (120, 255, 140),
    primary: (0,  200,  80),
    selected:(220, 255, 220),
    dim:     (0,  100,  40),
    muted:   (0,   50,  20),
    accent:  (0,  255, 160),
    danger:  (255,  80,  40),
    warn:    (200, 220,  0),
    success: (0,  255, 100),
    hp_high: (0,  200,  80),
    hp_mid:  (160, 220,  40),
    hp_low:  (255,  80,  40),
    mana:    (0,  180, 240),
    gold:    (200, 220,  40),
    xp:      (80, 240, 160),
};

/// 4. SOLAR FORGE — Amber/gold desert heat. Alchemical fire.
///    Warm, dusty, dangerous. The math burns here.
pub const THEME_SOLAR: Theme = Theme {
    name:    "SOLAR FORGE",
    tagline: "The equations combust at this temperature. Good.",
    bg:      (14,  8,  0),
    border:  (200, 100,  0),
    panel:   (20, 12,  0),
    heading: (255, 200,  60),
    primary: (220, 140,  20),
    selected:(255, 240, 180),
    dim:     (140,  80,  20),
    muted:   (70,   40,  10),
    accent:  (255, 180,  0),
    danger:  (255,  60,  0),
    warn:    (255, 140,  0),
    success: (180, 220,  80),
    hp_high: (180, 200,  60),
    hp_mid:  (255, 160,  20),
    hp_low:  (255,  60,  0),
    mana:    (80, 160, 220),
    gold:    (255, 210,  40),
    xp:      (200, 140,  40),
};

/// 5. GLACIAL ABYSS — Icy blue, crystalline cold. Absolute zero math.
///    Clean, precise, merciless. The coldest algorithms live here.
pub const THEME_GLACIAL: Theme = Theme {
    name:    "GLACIAL ABYSS",
    tagline: "Absolute zero. The equations freeze mid-cascade.",
    bg:      (0,   8,  16),
    border:  (0,  140, 200),
    panel:   (0,  12,  22),
    heading: (140, 220, 255),
    primary: (0,  180, 240),
    selected:(220, 240, 255),
    dim:     (0,   90, 140),
    muted:   (0,   45,  70),
    accent:  (80, 240, 255),
    danger:  (220,  60, 100),
    warn:    (220, 180,  40),
    success: (80, 220, 200),
    hp_high: (80, 200, 220),
    hp_mid:  (140, 200,  60),
    hp_low:  (220,  60, 100),
    mana:    (100, 160, 255),
    gold:    (200, 220,  80),
    xp:      (80, 200, 255),
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
