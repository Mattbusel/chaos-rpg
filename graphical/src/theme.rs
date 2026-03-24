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

/// 1. VOID PROTOCOL — Deep space neon. Pitch black bg, electric violet borders,
///    searing cyan accents. Feels like staring into the abyss at 3 AM.
pub const THEME_VOID: Theme = Theme {
    name:    "VOID PROTOCOL",
    tagline: "Stare into the Mandelbrot set. It stares back.",
    bg:      (4,   2,  12),
    border:  (140,  50, 255),
    panel:   (12,   6,  30),
    heading: (220, 160, 255),
    primary: (160, 110, 240),
    selected:(255, 255, 255),
    dim:     (90,   60, 140),
    muted:   (40,   26,  70),
    accent:  (0,   255, 255),
    danger:  (255,   0,  80),
    warn:    (255, 160,   0),
    success: (0,   255, 120),
    hp_high: (0,   255, 120),
    hp_mid:  (255, 220,   0),
    hp_low:  (255,   0,  80),
    mana:    (60,  140, 255),
    gold:    (255, 220,   0),
    xp:      (210,  80, 255),
};

/// 2. BLOOD PACT — Hellfire gothic. Pure black, screaming red borders,
///    molten orange accents. Every room is a sacrifice.
pub const THEME_BLOOD: Theme = Theme {
    name:    "BLOOD PACT",
    tagline: "Every step costs you. Some steps cost everything.",
    bg:      (6,   0,   0),
    border:  (230,  20,  20),
    panel:   (18,   3,   3),
    heading: (255, 130,  60),
    primary: (215,  55,  55),
    selected:(255, 235, 215),
    dim:     (130,  35,  35),
    muted:   (55,   14,  14),
    accent:  (255, 100,   0),
    danger:  (255,   0,   0),
    warn:    (255, 160,  20),
    success: (215, 170,  90),
    hp_high: (210,  75,  50),
    hp_mid:  (255, 130,  30),
    hp_low:  (255,   0,   0),
    mana:    (170,  50, 210),
    gold:    (255, 200,  40),
    xp:      (220,  70, 120),
};

/// 3. EMERALD ENGINE — Neon matrix. True black, electric green borders,
///    acid cyan accents. The dungeon is a compiler. You are a bug.
pub const THEME_EMERALD: Theme = Theme {
    name:    "EMERALD ENGINE",
    tagline: "The dungeon is a compiler. You are undefined behavior.",
    bg:      (0,   6,   1),
    border:  (0,  230,  60),
    panel:   (0,  14,   4),
    heading: (120, 255, 140),
    primary: (0,  220,  80),
    selected:(230, 255, 230),
    dim:     (0,  110,  45),
    muted:   (0,   48,  18),
    accent:  (0,  255, 200),
    danger:  (255,  70,  30),
    warn:    (220, 255,   0),
    success: (0,  255,  90),
    hp_high: (0,  220,  80),
    hp_mid:  (180, 240,  30),
    hp_low:  (255,  70,  30),
    mana:    (0,  200, 255),
    gold:    (220, 250,  30),
    xp:      (80,  255, 180),
};

/// 4. SOLAR FORGE — Molten gold. Near-black warm bg, blazing amber borders,
///    white-hot accent. The equations combust here.
pub const THEME_SOLAR: Theme = Theme {
    name:    "SOLAR FORGE",
    tagline: "The equations combust at this temperature. Good.",
    bg:      (10,   5,   0),
    border:  (255, 140,   0),
    panel:   (20,  10,   0),
    heading: (255, 225,  60),
    primary: (240, 160,  20),
    selected:(255, 250, 200),
    dim:     (155,  85,  18),
    muted:   (70,   38,   6),
    accent:  (255, 210,   0),
    danger:  (255,  50,   0),
    warn:    (255, 155,   0),
    success: (190, 240,  70),
    hp_high: (190, 225,  50),
    hp_mid:  (255, 175,  15),
    hp_low:  (255,  50,   0),
    mana:    (80,  175, 245),
    gold:    (255, 230,  30),
    xp:      (220, 155,  40),
};

/// 5. GLACIAL ABYSS — Arctic neon. Black-blue bg, electric ice borders,
///    screaming cyan accent. Absolute zero math.
pub const THEME_GLACIAL: Theme = Theme {
    name:    "GLACIAL ABYSS",
    tagline: "Absolute zero. The equations freeze mid-cascade.",
    bg:      (0,   5,  14),
    border:  (0,  200, 255),
    panel:   (0,  10,  24),
    heading: (140, 240, 255),
    primary: (0,  210, 255),
    selected:(220, 250, 255),
    dim:     (0,  100, 160),
    muted:   (0,   46,  80),
    accent:  (0,  255, 255),
    danger:  (255,  50, 100),
    warn:    (240, 210,  30),
    success: (70,  245, 215),
    hp_high: (70,  220, 240),
    hp_mid:  (140, 215,  50),
    hp_low:  (255,  50, 100),
    mana:    (100, 165, 255),
    gold:    (215, 240,  80),
    xp:      (80,  220, 255),
};

pub const THEMES: [Theme; 5] = [
    THEME_VOID,
    THEME_BLOOD,
    THEME_EMERALD,
    THEME_SOLAR,
    THEME_GLACIAL,
];

// ── 5-Tier Brightness System ──────────────────────────────────────────────────
// Used to enforce visual hierarchy across all screens.
// T1 = player-critical info, T5 = near-invisible background noise.
//
//   T1 CRITICAL   — selected, boss names, crit numbers, death, level-up
//   T2 ACTIVE     — HP/MP values, current item, action labels, headings
//   T3 SECONDARY  — room descriptions, modifiers list, inactive spells
//   T4 MUTED      — labels ("HP", "Floor", "Kills"), borders, separators
//   T5 GHOST      — chaos field, visited rooms, empty-slot placeholders
//
// In practice: use Theme fields directly but classify them by tier intent:
//   T1 → selected, heading (when emphasis), accent on confirm
//   T2 → heading, hp_high, hp_mid, mana, gold, success
//   T3 → primary, dim
//   T4 → muted, border
//   T5 → chaos field brightness values (0.055–0.16)

impl Theme {
    pub fn hp_color(&self, pct: f32) -> (u8, u8, u8) {
        if pct > 0.6 { self.hp_high } else if pct > 0.3 { self.hp_mid } else { self.hp_low }
    }

    /// Dim a color toward theme bg for T4/T5 usage.
    pub fn dim_color(&self, col: (u8,u8,u8), factor: f32) -> (u8,u8,u8) {
        let f = factor.clamp(0.0, 1.0);
        (
            ((col.0 as f32 * f + self.bg.0 as f32 * (1.0 - f)) as u8),
            ((col.1 as f32 * f + self.bg.1 as f32 * (1.0 - f)) as u8),
            ((col.2 as f32 * f + self.bg.2 as f32 * (1.0 - f)) as u8),
        )
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
