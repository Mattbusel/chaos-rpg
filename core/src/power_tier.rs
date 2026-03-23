//! Expanded power tier system — 40+ tiers spanning from THE VOID to ΩMEGA.
//!
//! Tier is computed from the sum of all 7 stats. Tiers below zero have
//! special display effects (rainbow, glitch, pulse, etc.) that frontends
//! can opt into for animated rendering.

use serde::{Deserialize, Serialize};

// ── Display effect enum ───────────────────────────────────────────────────────

/// How the tier name should be rendered on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TierEffect {
    /// Plain colored text.
    Normal,
    /// Cycle foreground color through spectrum each frame.
    Rainbow,
    /// Faster rainbow cycle.
    RainbowFast,
    /// Alternate between bright and dim each N frames.
    Pulse,
    /// Alternate between visible and blank.
    Flash,
    /// Randomly corrupt 1-2 characters per frame.
    Glitch,
    /// Swap foreground and background.
    Inverted,
    /// Intersperse random noise characters.
    Static,
    /// Cool blue-white freeze shimmer.
    Freeze,
    /// All colors fading toward black.
    Fading,
    /// Pure black background, plain white text. Nothing else.
    PureBlack,
    /// Full screen flash on each render.
    FullFlash,
    /// Gold pulsing flash.
    GoldFlash,
    /// Bold white rapid alternation.
    BoldWhiteFlash,
    /// Dark degraded rainbow.
    DarkRainbow,
}

// ── PowerTier enum ────────────────────────────────────────────────────────────

/// Every power tier in CHAOS RPG, from the mathematically catastrophic to the
/// universe-ending. Each run computes its tier every time stats change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PowerTier {
    // ── Negative tiers (ascending severity) ──────────────────────────────────
    TheVoid,          // -1_000_000_000 and below
    HeatDeath,        // -100_000_000 to -999_999_999
    AbsoluteZero,     // -10_000_000 to -99_999_999
    GodelsGhost,      // -1_000_000 to -9_999_999
    RussellsParadox,  // -500_000 to -999_999
    NegativeAleph,    // -250_000 to -499_999
    DivisionByZero,   // -100_000 to -249_999
    Paradox,          // -50_000 to -99_999
    AntiAxiom,        // -20_000 to -49_999
    NegativeInfinity, // -10_000 to -19_999
    MathError,        // -6_000 to -9_999
    VoidTouched,      // -3_000 to -5_999
    AntiChampion,     // -1_000 to -2_999
    Abyssal,          // -600 to -999
    Forsaken,         // -400 to -599
    Damned,           // -200 to -399
    Cursed,           // -100 to -199
    Unfortunate,      // -50 to -99
    BelowAverage,     // -1 to -49
    // ── Positive tiers (ascending power) ─────────────────────────────────────
    Mortal,           // 0 to 49
    Mundane,          // 50 to 99
    Awakened,         // 100 to 199
    Proven,           // 200 to 399
    Champion,         // 400 to 599
    Legendary,        // 600 to 999
    Transcendent,     // 1_000 to 2_999
    Mythical,         // 3_000 to 5_999
    Godlike,          // 6_000 to 9_999
    BeyondMath,       // 10_000 to 19_999
    Axiom,            // 20_000 to 49_999
    Theorem,          // 50_000 to 99_999
    Cardinal,         // 100_000 to 249_999
    AlephZero,        // 250_000 to 499_999
    AlephOne,         // 500_000 to 999_999
    Continuum,        // 1_000_000 to 4_999_999
    LargeCardinal,    // 5_000_000 to 9_999_999
    Inaccessible,     // 10_000_000 to 49_999_999
    Mahlo,            // 50_000_000 to 99_999_999
    Measurable,       // 100_000_000 to 999_999_999
    Omega,            // 1_000_000_000 and above
}

impl PowerTier {
    /// Compute the tier from a raw stat total.
    pub fn from_total(total: i64) -> Self {
        match total {
            i64::MIN..=-1_000_000_000 => PowerTier::TheVoid,
            -999_999_999..=-100_000_000 => PowerTier::HeatDeath,
            -99_999_999..=-10_000_000  => PowerTier::AbsoluteZero,
            -9_999_999..=-1_000_000    => PowerTier::GodelsGhost,
            -999_999..=-500_000        => PowerTier::RussellsParadox,
            -499_999..=-250_000        => PowerTier::NegativeAleph,
            -249_999..=-100_000        => PowerTier::DivisionByZero,
            -99_999..=-50_000          => PowerTier::Paradox,
            -49_999..=-20_000          => PowerTier::AntiAxiom,
            -19_999..=-10_000          => PowerTier::NegativeInfinity,
            -9_999..=-6_000            => PowerTier::MathError,
            -5_999..=-3_000            => PowerTier::VoidTouched,
            -2_999..=-1_000            => PowerTier::AntiChampion,
            -999..=-600                => PowerTier::Abyssal,
            -599..=-400                => PowerTier::Forsaken,
            -399..=-200                => PowerTier::Damned,
            -199..=-100                => PowerTier::Cursed,
            -99..=-50                  => PowerTier::Unfortunate,
            -49..=-1                   => PowerTier::BelowAverage,
            0..=49                     => PowerTier::Mortal,
            50..=99                    => PowerTier::Mundane,
            100..=199                  => PowerTier::Awakened,
            200..=399                  => PowerTier::Proven,
            400..=599                  => PowerTier::Champion,
            600..=999                  => PowerTier::Legendary,
            1_000..=2_999              => PowerTier::Transcendent,
            3_000..=5_999              => PowerTier::Mythical,
            6_000..=9_999              => PowerTier::Godlike,
            10_000..=19_999            => PowerTier::BeyondMath,
            20_000..=49_999            => PowerTier::Axiom,
            50_000..=99_999            => PowerTier::Theorem,
            100_000..=249_999          => PowerTier::Cardinal,
            250_000..=499_999          => PowerTier::AlephZero,
            500_000..=999_999          => PowerTier::AlephOne,
            1_000_000..=4_999_999      => PowerTier::Continuum,
            5_000_000..=9_999_999      => PowerTier::LargeCardinal,
            10_000_000..=49_999_999    => PowerTier::Inaccessible,
            50_000_000..=99_999_999    => PowerTier::Mahlo,
            100_000_000..=999_999_999  => PowerTier::Measurable,
            _                          => PowerTier::Omega,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PowerTier::TheVoid          => "THE VOID",
            PowerTier::HeatDeath        => "HEAT DEATH",
            PowerTier::AbsoluteZero     => "ABSOLUTE ZERO",
            PowerTier::GodelsGhost      => "GODEL'S GHOST",
            PowerTier::RussellsParadox  => "RUSSELL'S PARADOX",
            PowerTier::NegativeAleph    => "NEGATIVE ALEPH",
            PowerTier::DivisionByZero   => "DIVISION BY ZERO",
            PowerTier::Paradox          => "PARADOX",
            PowerTier::AntiAxiom        => "ANTI-AXIOM",
            PowerTier::NegativeInfinity => "NEGATIVE INFINITY",
            PowerTier::MathError        => "MATHEMATICAL ERROR",
            PowerTier::VoidTouched      => "VOID-TOUCHED",
            PowerTier::AntiChampion     => "ANTI-CHAMPION",
            PowerTier::Abyssal          => "ABYSSAL",
            PowerTier::Forsaken         => "FORSAKEN",
            PowerTier::Damned           => "DAMNED",
            PowerTier::Cursed           => "CURSED",
            PowerTier::Unfortunate      => "UNFORTUNATE",
            PowerTier::BelowAverage     => "BELOW AVERAGE",
            PowerTier::Mortal           => "MORTAL",
            PowerTier::Mundane          => "MUNDANE",
            PowerTier::Awakened         => "AWAKENED",
            PowerTier::Proven           => "PROVEN",
            PowerTier::Champion         => "CHAMPION",
            PowerTier::Legendary        => "LEGENDARY",
            PowerTier::Transcendent     => "TRANSCENDENT",
            PowerTier::Mythical         => "MYTHICAL",
            PowerTier::Godlike          => "GODLIKE",
            PowerTier::BeyondMath       => "BEYOND MATH",
            PowerTier::Axiom            => "AXIOM",
            PowerTier::Theorem          => "THEOREM",
            PowerTier::Cardinal         => "CARDINAL",
            PowerTier::AlephZero        => "ALEPH-0",
            PowerTier::AlephOne         => "ALEPH-1",
            PowerTier::Continuum        => "CONTINUUM",
            PowerTier::LargeCardinal    => "LARGE CARDINAL",
            PowerTier::Inaccessible     => "INACCESSIBLE",
            PowerTier::Mahlo            => "MAHLO",
            PowerTier::Measurable       => "MEASURABLE",
            PowerTier::Omega            => "ΩMEGA",
        }
    }

    pub fn flavor(&self) -> &'static str {
        match self {
            PowerTier::TheVoid          => "There is nothing here. Not even nothing. Less than that.",
            PowerTier::HeatDeath        => "The universe has run out of entropy to give you.",
            PowerTier::AbsoluteZero     => "Not thermal. Mathematical. The coldest possible number.",
            PowerTier::GodelsGhost      => "Provably unprovable. Your stat sheet is an incomplete system.",
            PowerTier::RussellsParadox  => "The set of all stats that don't contain themselves. You are that set.",
            PowerTier::NegativeAleph    => "Countably anti-infinite. You have less than nothing, infinitely.",
            PowerTier::DivisionByZero   => "Somewhere, a denominator weeps.",
            PowerTier::Paradox          => "You shouldn't exist. You do. Both statements are proven true.",
            PowerTier::AntiAxiom        => "Your existence disproves something fundamental. Logicians are upset.",
            PowerTier::NegativeInfinity => "The limit of your potential approaches negative infinity. It has arrived.",
            PowerTier::MathError        => "NaN. Not a Number. Not a person. A rounding error given legs.",
            PowerTier::VoidTouched      => "The Mandelbrot set doesn't render where you stand.",
            PowerTier::AntiChampion     => "You are the equal and opposite of a hero. Bards sing warnings.",
            PowerTier::Abyssal          => "The math has forsaken you. You exist only through spite.",
            PowerTier::Forsaken         => "The chaos pipeline has filed a restraining order against you.",
            PowerTier::Damned           => "The algorithms hate you specifically. Keep going.",
            PowerTier::Cursed           => "Even rats pity you. Negative stats are technically valid.",
            PowerTier::Unfortunate      => "The dice see you and feel guilty.",
            PowerTier::BelowAverage     => "The NPCs pity you, and they're made of math.",
            PowerTier::Mortal           => "Statistically average. The Logistic Map is neutral on you.",
            PowerTier::Mundane          => "You exist. The algorithms acknowledge this and nothing more.",
            PowerTier::Awakened         => "The prime numbers notice you. That is an improvement.",
            PowerTier::Proven           => "You've survived long enough to become a data point.",
            PowerTier::Champion         => "The Lorenz attractor bends in your favor.",
            PowerTier::Legendary        => "The Riemann zeros align. You are an anomaly.",
            PowerTier::Transcendent     => "The Mandelbrot boundary recognizes your face.",
            PowerTier::Mythical         => "Your stat sheet is a published paper.",
            PowerTier::Godlike          => "You ARE the chaos engine. The math screams.",
            PowerTier::BeyondMath       => "ERROR: STAT OVERFLOW. YOU HAVE BROKEN THE ALGORITHM.",
            PowerTier::Axiom            => "You are no longer computed. You are assumed.",
            PowerTier::Theorem          => "Your existence is a proof. The universe is the paper.",
            PowerTier::Cardinal         => "A number so large it required a new kind of counting.",
            PowerTier::AlephZero        => "Countably infinite. Every stat is a natural number that forgot to stop.",
            PowerTier::AlephOne         => "Uncountably infinite. There are more of you than there are numbers for you.",
            PowerTier::Continuum        => "The continuum hypothesis cannot determine what you are.",
            PowerTier::LargeCardinal    => "Set theorists are arguing about whether you can exist.",
            PowerTier::Inaccessible     => "No operation applied to smaller things can produce you.",
            PowerTier::Mahlo            => "You are inaccessible AND the limit of inaccessible things.",
            PowerTier::Measurable       => "Your existence contradicts the axiom of constructibility.",
            PowerTier::Omega            => "The first letter of an alphabet that doesn't exist yet. The game concedes.",
        }
    }

    /// ANSI terminal color code for the tier name.
    pub fn ansi_color(&self) -> &'static str {
        match self {
            PowerTier::TheVoid          => "\x1b[30;47m",  // black on white
            PowerTier::HeatDeath        => "\x1b[2;35m",   // dim magenta
            PowerTier::AbsoluteZero     => "\x1b[96m",     // bright cyan
            PowerTier::GodelsGhost      => "\x1b[2;37m",   // dim white/static
            PowerTier::RussellsParadox  => "\x1b[1;35m",   // bold magenta
            PowerTier::NegativeAleph    => "\x1b[2;31m",   // dim red
            PowerTier::DivisionByZero   => "\x1b[31m",     // red
            PowerTier::Paradox          => "\x1b[7m",      // inverted
            PowerTier::AntiAxiom        => "\x1b[2;35m",   // dark magenta
            PowerTier::NegativeInfinity => "\x1b[37;41m",  // white on dark red
            PowerTier::MathError        => "\x1b[1;35m",   // bold magenta
            PowerTier::VoidTouched      => "\x1b[35m",     // magenta
            PowerTier::AntiChampion     => "\x1b[1;31m",   // bold dark red
            PowerTier::Abyssal          => "\x1b[31m",     // dark red
            PowerTier::Forsaken         => "\x1b[31m",     // red
            PowerTier::Damned           => "\x1b[33m",     // orange
            PowerTier::Cursed           => "\x1b[33m",     // dark yellow
            PowerTier::Unfortunate      => "\x1b[2;37m",   // dim gray
            PowerTier::BelowAverage     => "\x1b[37m",     // gray
            PowerTier::Mortal           => "\x1b[97m",     // white
            PowerTier::Mundane          => "\x1b[97m",     // white
            PowerTier::Awakened         => "\x1b[36m",     // cyan
            PowerTier::Proven           => "\x1b[36m",     // cyan
            PowerTier::Champion         => "\x1b[32m",     // green
            PowerTier::Legendary        => "\x1b[32m",     // green
            PowerTier::Transcendent     => "\x1b[33m",     // yellow
            PowerTier::Mythical         => "\x1b[33m",     // yellow
            PowerTier::Godlike          => "\x1b[1;33m",   // bold gold
            PowerTier::BeyondMath       => "\x1b[35m",     // magenta
            PowerTier::Axiom            => "\x1b[1;35m",   // bold magenta
            PowerTier::Theorem          => "\x1b[1;31m",   // bold red
            PowerTier::Cardinal         => "\x1b[1;31m",   // bold red
            PowerTier::AlephZero        => "\x1b[96m",     // rainbow (front-end animates)
            PowerTier::AlephOne         => "\x1b[96m",     // rainbow fast
            PowerTier::Continuum        => "\x1b[37;45m",  // white on magenta
            PowerTier::LargeCardinal    => "\x1b[37;41m",  // white on red
            PowerTier::Inaccessible     => "\x1b[1;97m",   // bold white
            PowerTier::Mahlo            => "\x1b[1;33m",   // gold flash
            PowerTier::Measurable       => "\x1b[5;97m",   // full screen flash
            PowerTier::Omega            => "\x1b[1;5;96m", // seizure rainbow
        }
    }

    /// Visual effect the frontend should apply when rendering this tier name.
    pub fn effect(&self) -> TierEffect {
        match self {
            PowerTier::TheVoid          => TierEffect::PureBlack,
            PowerTier::HeatDeath        => TierEffect::Fading,
            PowerTier::AbsoluteZero     => TierEffect::Freeze,
            PowerTier::GodelsGhost      => TierEffect::Static,
            PowerTier::RussellsParadox  => TierEffect::Glitch,
            PowerTier::NegativeAleph    => TierEffect::DarkRainbow,
            PowerTier::DivisionByZero   => TierEffect::Flash,
            PowerTier::Paradox          => TierEffect::Inverted,
            PowerTier::AntiAxiom        => TierEffect::Pulse,
            PowerTier::NegativeInfinity => TierEffect::Normal,
            PowerTier::MathError        => TierEffect::Normal,
            PowerTier::VoidTouched      => TierEffect::Normal,
            PowerTier::AntiChampion     => TierEffect::Normal,
            PowerTier::Abyssal          => TierEffect::Normal,
            PowerTier::Forsaken         => TierEffect::Normal,
            PowerTier::Damned           => TierEffect::Normal,
            PowerTier::Cursed           => TierEffect::Normal,
            PowerTier::Unfortunate      => TierEffect::Normal,
            PowerTier::BelowAverage     => TierEffect::Normal,
            PowerTier::Mortal           => TierEffect::Normal,
            PowerTier::Mundane          => TierEffect::Normal,
            PowerTier::Awakened         => TierEffect::Normal,
            PowerTier::Proven           => TierEffect::Normal,
            PowerTier::Champion         => TierEffect::Normal,
            PowerTier::Legendary        => TierEffect::Normal,
            PowerTier::Transcendent     => TierEffect::Normal,
            PowerTier::Mythical         => TierEffect::Normal,
            PowerTier::Godlike          => TierEffect::Normal,
            PowerTier::BeyondMath       => TierEffect::Normal,
            PowerTier::Axiom            => TierEffect::Pulse,
            PowerTier::Theorem          => TierEffect::Pulse,
            PowerTier::Cardinal         => TierEffect::Flash,
            PowerTier::AlephZero        => TierEffect::Rainbow,
            PowerTier::AlephOne         => TierEffect::RainbowFast,
            PowerTier::Continuum        => TierEffect::Normal,
            PowerTier::LargeCardinal    => TierEffect::Normal,
            PowerTier::Inaccessible     => TierEffect::BoldWhiteFlash,
            PowerTier::Mahlo            => TierEffect::GoldFlash,
            PowerTier::Measurable       => TierEffect::FullFlash,
            PowerTier::Omega            => TierEffect::Rainbow,
        }
    }

    /// True for tiers that should show misery as primary metric instead of power.
    pub fn is_negative(&self) -> bool {
        matches!(self,
            PowerTier::TheVoid | PowerTier::HeatDeath | PowerTier::AbsoluteZero |
            PowerTier::GodelsGhost | PowerTier::RussellsParadox | PowerTier::NegativeAleph |
            PowerTier::DivisionByZero | PowerTier::Paradox | PowerTier::AntiAxiom |
            PowerTier::NegativeInfinity | PowerTier::MathError | PowerTier::VoidTouched |
            PowerTier::AntiChampion | PowerTier::Abyssal | PowerTier::Forsaken |
            PowerTier::Damned | PowerTier::Cursed | PowerTier::Unfortunate |
            PowerTier::BelowAverage
        )
    }

    /// True for visually extreme tiers that get animated effects in the graphical frontend.
    pub fn has_effect(&self) -> bool {
        !matches!(self.effect(), TierEffect::Normal)
    }

    /// Animate the tier name for terminal output by applying simple frame-based effects.
    /// Returns the decorated string. `frame` is the render frame counter.
    pub fn render_terminal(&self, frame: u64) -> String {
        let name = self.name();
        match self.effect() {
            TierEffect::Normal => format!("{}{}\x1b[0m", self.ansi_color(), name),
            TierEffect::Rainbow | TierEffect::RainbowFast => {
                let speed = if matches!(self.effect(), TierEffect::RainbowFast) { 2 } else { 4 };
                let colors = ["\x1b[31m","\x1b[33m","\x1b[32m","\x1b[36m","\x1b[34m","\x1b[35m"];
                let col = colors[((frame / speed) as usize) % colors.len()];
                format!("{}{}\x1b[0m", col, name)
            }
            TierEffect::Pulse => {
                let bright = (frame / 15) % 2 == 0;
                let prefix = if bright { "\x1b[1m" } else { "\x1b[2m" };
                format!("{}{}{}\x1b[0m", self.ansi_color(), prefix, name)
            }
            TierEffect::Flash => {
                if (frame / 12) % 2 == 0 {
                    format!("{}{}\x1b[0m", self.ansi_color(), name)
                } else {
                    " ".repeat(name.len())
                }
            }
            TierEffect::Glitch => {
                const GLITCH: &[char] = &['█','▓','▒','░','±','∑','∞','∂','Δ','Ω','#','@','%'];
                let mut chars: Vec<char> = name.chars().collect();
                let idx1 = (frame.wrapping_mul(7919)) as usize % chars.len();
                let idx2 = (frame.wrapping_mul(6271)) as usize % chars.len();
                if (frame / 3) % 3 != 0 {
                    chars[idx1] = GLITCH[(frame as usize / 5) % GLITCH.len()];
                    if idx2 != idx1 { chars[idx2] = GLITCH[(frame as usize / 3) % GLITCH.len()]; }
                }
                format!("{}{}\x1b[0m", self.ansi_color(), chars.iter().collect::<String>())
            }
            TierEffect::Inverted => format!("\x1b[7m{}\x1b[0m", name),
            TierEffect::Static => {
                const NOISE: &[char] = &['.', ':', '·', '•', '░', '▒'];
                let out: String = name.chars().enumerate().map(|(i, c)| {
                    if (frame.wrapping_add(i as u64 * 3173)) % 7 == 0 {
                        NOISE[(frame as usize + i * 17) % NOISE.len()]
                    } else { c }
                }).collect();
                format!("{}{}\x1b[0m", self.ansi_color(), out)
            }
            TierEffect::Freeze => {
                let t = ((frame as f64 * 0.08).sin() * 0.5 + 0.5) * 255.0;
                let b = t as u8;
                format!("\x1b[38;2;{};{};255m{}\x1b[0m", b, b, name)
            }
            TierEffect::Fading => {
                let v = ((frame as f64 * 0.05).sin().abs() * 180.0) as u8;
                format!("\x1b[38;2;{v};0;0m{name}\x1b[0m")
            }
            TierEffect::PureBlack => format!("\x1b[30;47m {name} \x1b[0m"),
            TierEffect::DarkRainbow => {
                let colors = ["\x1b[2;31m","\x1b[2;33m","\x1b[2;32m","\x1b[2;36m","\x1b[2;35m"];
                let col = colors[((frame / 6) as usize) % colors.len()];
                format!("{}{}\x1b[0m", col, name)
            }
            TierEffect::BoldWhiteFlash => {
                if (frame / 8) % 2 == 0 { format!("\x1b[1;97m{}\x1b[0m", name) }
                else { format!("\x1b[2;37m{}\x1b[0m", name) }
            }
            TierEffect::GoldFlash => {
                if (frame / 10) % 2 == 0 { format!("\x1b[1;33m{}\x1b[0m", name) }
                else { format!("\x1b[33m{}\x1b[0m", name) }
            }
            TierEffect::FullFlash => {
                let bright = (frame / 6) % 2 == 0;
                if bright { format!("\x1b[1;97m{}\x1b[0m", name) }
                else { format!("\x1b[2;31m{}\x1b[0m", name) }
            }
        }
    }

    /// Theme-compatible RGB color for graphical frontend (r, g, b).
    pub fn rgb(&self) -> (u8, u8, u8) {
        match self {
            PowerTier::TheVoid          => (220, 220, 220),
            PowerTier::HeatDeath        => (80, 20, 60),
            PowerTier::AbsoluteZero     => (100, 200, 255),
            PowerTier::GodelsGhost      => (120, 120, 120),
            PowerTier::RussellsParadox  => (200, 50, 200),
            PowerTier::NegativeAleph    => (100, 0, 60),
            PowerTier::DivisionByZero   => (200, 40, 40),
            PowerTier::Paradox          => (180, 180, 40),
            PowerTier::AntiAxiom        => (140, 0, 140),
            PowerTier::NegativeInfinity => (220, 60, 60),
            PowerTier::MathError        => (200, 80, 200),
            PowerTier::VoidTouched      => (160, 40, 180),
            PowerTier::AntiChampion     => (180, 30, 30),
            PowerTier::Abyssal          => (160, 20, 20),
            PowerTier::Forsaken         => (200, 50, 20),
            PowerTier::Damned           => (220, 100, 20),
            PowerTier::Cursed           => (180, 140, 20),
            PowerTier::Unfortunate      => (140, 140, 140),
            PowerTier::BelowAverage     => (160, 160, 160),
            PowerTier::Mortal           => (200, 200, 200),
            PowerTier::Mundane          => (200, 200, 200),
            PowerTier::Awakened         => (80, 200, 220),
            PowerTier::Proven           => (80, 200, 220),
            PowerTier::Champion         => (80, 200, 80),
            PowerTier::Legendary        => (100, 220, 100),
            PowerTier::Transcendent     => (220, 200, 40),
            PowerTier::Mythical         => (240, 220, 60),
            PowerTier::Godlike          => (255, 200, 0),
            PowerTier::BeyondMath       => (220, 80, 240),
            PowerTier::Axiom            => (240, 100, 255),
            PowerTier::Theorem          => (255, 80, 80),
            PowerTier::Cardinal         => (255, 60, 60),
            PowerTier::AlephZero        => (0, 230, 255),
            PowerTier::AlephOne         => (0, 255, 200),
            PowerTier::Continuum        => (240, 200, 255),
            PowerTier::LargeCardinal    => (255, 200, 200),
            PowerTier::Inaccessible     => (255, 255, 255),
            PowerTier::Mahlo            => (255, 220, 40),
            PowerTier::Measurable       => (255, 255, 200),
            PowerTier::Omega            => (255, 100, 255),
        }
    }

    /// True if the tier is at or above ALEPH-0 / at or below PARADOX — extreme enough
    /// to warrant animated rendering.
    pub fn is_extreme_positive(&self) -> bool {
        matches!(self,
            PowerTier::AlephZero | PowerTier::AlephOne | PowerTier::Continuum |
            PowerTier::LargeCardinal | PowerTier::Inaccessible | PowerTier::Mahlo |
            PowerTier::Measurable | PowerTier::Omega | PowerTier::Cardinal
        )
    }

    pub fn is_extreme_negative(&self) -> bool {
        matches!(self,
            PowerTier::Paradox | PowerTier::AntiAxiom | PowerTier::NegativeInfinity |
            PowerTier::MathError | PowerTier::VoidTouched | PowerTier::GodelsGhost |
            PowerTier::RussellsParadox | PowerTier::NegativeAleph | PowerTier::DivisionByZero |
            PowerTier::AbsoluteZero | PowerTier::HeatDeath | PowerTier::TheVoid
        )
    }
}
