//! Timeline-driven cinematics for CHAOS RPG.
//!
//! Scripted sequences for boss entrances, phase transitions, floor transitions,
//! level ups, achievement unlocks, misery milestones, and corruption milestones.
//! Each cinematic is a timed sequence of visual events rendered as glyphs.

use proof_engine::prelude::*;
use crate::state::GameState;
use crate::theme::THEMES;

// ═══════════════════════════════════════════════════════════════════════════════
// CINEMATIC STATE
// ═══════════════════════════════════════════════════════════════════════════════

/// Active cinematic sequence.
pub struct CinematicState {
    pub kind: CinematicKind,
    pub elapsed: f32,
    pub duration: f32,
    pub data: CinematicData,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CinematicKind {
    BossEntrance,
    BossPhaseTransition,
    FloorTransition,
    LevelUp,
    AchievementUnlock,
    MiseryMilestone,
    CorruptionMilestone,
    NemesisReveal,
}

/// Extra data needed for specific cinematics.
pub struct CinematicData {
    pub text_primary: String,
    pub text_secondary: String,
    pub boss_id: Option<u8>,
    pub phase: u8,
    pub floor_num: u32,
    pub level: u32,
    pub rarity: u8,     // achievement rarity (0=Common..5=Omega)
    pub milestone: u32, // misery milestone value
}

impl Default for CinematicData {
    fn default() -> Self {
        Self {
            text_primary: String::new(), text_secondary: String::new(),
            boss_id: None, phase: 0, floor_num: 0, level: 0, rarity: 0, milestone: 0,
        }
    }
}

impl CinematicState {
    pub fn new(kind: CinematicKind, duration: f32, data: CinematicData) -> Self {
        Self { kind, elapsed: 0.0, duration, data }
    }

    pub fn is_done(&self) -> bool { self.elapsed >= self.duration }
    pub fn progress(&self) -> f32 { (self.elapsed / self.duration).clamp(0.0, 1.0) }

    pub fn tick(&mut self, dt: f32) { self.elapsed += dt; }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BOSS ENTRANCE SEQUENCES (3 seconds each)
// ═══════════════════════════════════════════════════════════════════════════════

const BOSS_NAMES: &[&str] = &[
    "", "THE MIRROR", "THE ACCOUNTANT", "THE FIBONACCI HYDRA",
    "THE EIGENSTATE", "THE TAXMAN", "THE NULL",
    "THE OUROBOROS", "THE COLLATZ TITAN", "THE COMMITTEE",
    "THE RECURSION", "THE PARADOX", "THE ALGORITHM REBORN",
];

const BOSS_TAGLINES: &[&str] = &[
    "",
    "It built a function that returned its own input.",
    "Your suffering has been itemized.",
    "Each head grows two more. The math is exponential.",
    "Alive and dead. One HP and ten thousand. Observe to collapse.",
    "Everything you own is taxable. Including your HP.",
    "Where mathematics cannot exist.",
    "The serpent eats its tail. The cycle begins again.",
    "3n+1. Always 3n+1. Unless it's even.",
    "Five judges. Majority rules. Democracy in the dungeon.",
    "Every action you take is added to the stack. It will all come back.",
    "Green means danger. Red means safety. Everything you know is wrong.",
    "The Proof itself, given will. It has waited since the Mathematician vanished.",
];

pub fn boss_entrance(boss_id: u8) -> CinematicState {
    let name = BOSS_NAMES.get(boss_id as usize).copied().unwrap_or("UNKNOWN BOSS");
    let tagline = BOSS_TAGLINES.get(boss_id as usize).copied().unwrap_or("");
    CinematicState::new(
        CinematicKind::BossEntrance,
        3.0,
        CinematicData {
            text_primary: name.to_string(),
            text_secondary: tagline.to_string(),
            boss_id: Some(boss_id),
            ..Default::default()
        },
    )
}

/// Render boss entrance cinematic.
pub fn render_boss_entrance(cin: &CinematicState, engine: &mut ProofEngine, state: &GameState) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = cin.elapsed;
    let frame = state.frame;

    // Phase 1 (0.0 - 1.0s): Vignette darkens, camera dolly
    if t < 1.0 {
        let darken = t;
        // Dark overlay
        for i in 0..20 {
            let angle = (i as f32 / 20.0) * std::f32::consts::TAU;
            let r = 18.0 - t * 3.0;
            engine.spawn_glyph(Glyph {
                character: '░',
                position: Vec3::new(angle.cos() * r, angle.sin() * r * 0.5, 1.0),
                color: Vec4::new(0.0, 0.0, 0.0, darken * 0.5),
                emission: 0.0,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }
        // "BOSS ENCOUNTER" text
        if t > 0.5 {
            let fade = ((t - 0.5) * 4.0).min(1.0);
            render_centered(engine, "BOSS ENCOUNTER", 4.0,
                Vec4::new(theme.danger.x * fade, theme.danger.y * fade, theme.danger.z * fade, fade), fade);
        }
    }

    // Phase 2 (1.0 - 2.0s): Boss name typewriter
    if t >= 1.0 && t < 2.0 {
        let phase_t = t - 1.0;
        let name = &cin.data.text_primary;
        let chars_revealed = ((phase_t * 20.0) as usize).min(name.len());
        let shown: String = name.chars().take(chars_revealed).collect();
        let pulse = ((frame as f32 * 0.15).sin() * 0.2 + 0.8).max(0.0);
        render_centered(engine, &shown, 2.0,
            Vec4::new(theme.heading.x * pulse, theme.heading.y * pulse, theme.heading.z * pulse, 1.0), 1.2);

        // Boss-specific entrance particles
        if let Some(boss_id) = cin.data.boss_id {
            render_boss_entrance_particles(engine, boss_id, phase_t, frame);
        }
    }

    // Phase 3 (2.0 - 3.0s): Tagline fade + vignette out
    if t >= 2.0 {
        let phase_t = t - 2.0;
        // Full name
        render_centered(engine, &cin.data.text_primary, 2.0, theme.heading, 1.0);
        // Tagline fade in
        let fade = (phase_t * 2.0).min(1.0);
        render_centered(engine, &cin.data.text_secondary, 0.0,
            Vec4::new(theme.dim.x * fade, theme.dim.y * fade, theme.dim.z * fade, fade), 0.4);
    }
}

fn render_boss_entrance_particles(engine: &mut ProofEngine, boss_id: u8, t: f32, frame: u64) {
    let count = (t * 15.0) as usize;
    match boss_id {
        1 => {
            // Mirror: symmetric particle split from center
            for i in 0..count {
                let spread = (i as f32 + 1.0) * t * 2.0;
                engine.spawn_glyph(Glyph {
                    character: '│',
                    position: Vec3::new(0.0, -8.0 + i as f32 * 0.8, 0.5),
                    color: Vec4::new(0.5, 0.8, 1.0, 0.6),
                    emission: 0.5,
                    layer: RenderLayer::Overlay, ..Default::default()
                });
            }
        }
        6 => {
            // The Null: particles disappearing
            for i in 0..count.min(10) {
                let seed_f = i as f32 * 47.3;
                let x = seed_f.sin() * 10.0;
                let y = seed_f.cos() * 6.0;
                let fade = (1.0 - t).max(0.0);
                engine.spawn_glyph(Glyph {
                    character: ' ',
                    position: Vec3::new(x, y, 0.5),
                    color: Vec4::new(0.1, 0.1, 0.1, fade),
                    emission: 0.0,
                    layer: RenderLayer::Overlay, ..Default::default()
                });
            }
        }
        12 => {
            // Algorithm Reborn: chaos field freezes, symbols converge
            let symbols = ['∑', '∫', '∏', 'Ω', '∂', 'λ', 'π', 'φ'];
            for (i, &sym) in symbols.iter().enumerate() {
                let target_x = -3.0 + (i as f32) * 0.9;
                let start_x = (i as f32 * 73.1 + frame as f32 * 0.01).sin() * 15.0;
                let start_y = (i as f32 * 31.7 + frame as f32 * 0.01).cos() * 8.0;
                let lerp_t = (t * 1.5).min(1.0);
                let x = start_x + (target_x - start_x) * lerp_t;
                let y = start_y + (-2.0 - start_y) * lerp_t;
                engine.spawn_glyph(Glyph {
                    character: sym,
                    position: Vec3::new(x, y, 0.5),
                    color: Vec4::new(0.7, 0.3, 1.0, lerp_t),
                    emission: lerp_t * 0.8,
                    glow_color: Vec3::new(0.5, 0.2, 0.8),
                    glow_radius: lerp_t * 2.0,
                    layer: RenderLayer::Overlay, ..Default::default()
                });
            }
        }
        _ => {
            // Generic entrance burst
            for i in 0..count.min(20) {
                let angle = (i as f32 / 20.0) * std::f32::consts::TAU;
                let r = t * 8.0;
                engine.spawn_glyph(Glyph {
                    character: '✦',
                    position: Vec3::new(angle.cos() * r, angle.sin() * r * 0.5 + 2.0, 0.5),
                    color: Vec4::new(1.0, 0.5, 0.2, (1.0 - t).max(0.0)),
                    emission: (1.0 - t) * 0.8,
                    layer: RenderLayer::Overlay, ..Default::default()
                });
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FLOOR TRANSITION (2.5 seconds)
// ═══════════════════════════════════════════════════════════════════════════════

pub fn floor_transition(floor_num: u32) -> CinematicState {
    let flavor = match floor_num {
        1..=10 => "The equations welcome you.",
        11..=25 => "The mathematics grow restless.",
        26..=50 => "The Proof tests your resolve.",
        51..=75 => "Reality thins. The numbers bleed.",
        76..=99 => "The void between theorems.",
        _ => "Beyond axioms. Beyond proof.",
    };
    CinematicState::new(
        CinematicKind::FloorTransition,
        2.5,
        CinematicData {
            text_primary: format!("FLOOR {}", floor_num),
            text_secondary: flavor.to_string(),
            floor_num,
            ..Default::default()
        },
    )
}

pub fn render_floor_transition(cin: &CinematicState, engine: &mut ProofEngine, state: &GameState) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = cin.elapsed;

    // Fade in (0-0.5), hold (0.5-2.0), fade out (2.0-2.5)
    let alpha = if t < 0.5 { t * 2.0 }
        else if t < 2.0 { 1.0 }
        else { (2.5 - t) * 2.0 };
    let alpha = alpha.clamp(0.0, 1.0);

    // Dark overlay
    for y_i in 0..30 {
        let y = -15.0 + y_i as f32;
        for x_i in 0..4 {
            let x = -20.0 + x_i as f32 * 13.0;
            engine.spawn_glyph(Glyph {
                character: '█',
                position: Vec3::new(x, y, 2.0),
                color: Vec4::new(0.0, 0.0, 0.0, alpha * 0.7),
                emission: 0.0,
                layer: RenderLayer::Overlay, ..Default::default()
            });
        }
    }

    // Floor number (large)
    if t > 0.3 {
        let text_alpha = if t < 0.8 { (t - 0.3) * 2.0 } else { alpha };
        let text_alpha = text_alpha.clamp(0.0, 1.0);
        render_centered(engine, &cin.data.text_primary, 2.0,
            Vec4::new(theme.heading.x * text_alpha, theme.heading.y * text_alpha,
                      theme.heading.z * text_alpha, text_alpha), 1.0);
    }

    // Flavor text
    if t > 0.8 {
        let sub_alpha = if t < 1.3 { (t - 0.8) * 2.0 } else { alpha };
        let sub_alpha = sub_alpha.clamp(0.0, 1.0);
        render_centered(engine, &cin.data.text_secondary, 0.0,
            Vec4::new(theme.dim.x * sub_alpha, theme.dim.y * sub_alpha,
                      theme.dim.z * sub_alpha, sub_alpha), 0.4);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// LEVEL UP (2 seconds)
// ═══════════════════════════════════════════════════════════════════════════════

pub fn level_up(level: u32) -> CinematicState {
    CinematicState::new(
        CinematicKind::LevelUp,
        2.0,
        CinematicData {
            text_primary: format!("LEVEL {}", level),
            level,
            ..Default::default()
        },
    )
}

pub fn render_level_up(cin: &CinematicState, engine: &mut ProofEngine, state: &GameState) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = cin.elapsed;
    let frame = state.frame;

    // Gold light pillar from player position
    let pillar_height = (t * 20.0).min(15.0);
    for i in 0..(pillar_height as usize) {
        let y = -8.0 + i as f32;
        let shimmer = ((frame as f32 * 0.1 + i as f32 * 0.3).sin() * 0.2 + 0.8).max(0.0);
        engine.spawn_glyph(Glyph {
            character: '│',
            position: Vec3::new(-6.0, y, 0.5),
            color: Vec4::new(1.0, 0.88 * shimmer, 0.2, 0.6),
            emission: shimmer,
            glow_color: Vec3::new(1.0, 0.8, 0.2),
            glow_radius: 1.5,
            layer: RenderLayer::Overlay, ..Default::default()
        });
    }

    // "LEVEL UP" banner
    if t > 0.3 {
        let fade = ((t - 0.3) * 3.0).min(1.0);
        let gold = Vec4::new(1.0, 0.88 * fade, 0.2, fade);
        render_centered(engine, "LEVEL UP", 6.0, gold, 1.2);
    }

    // Level number
    if t > 0.6 {
        let fade = ((t - 0.6) * 2.0).min(1.0);
        render_centered(engine, &cin.data.text_primary, 4.0,
            Vec4::new(theme.heading.x * fade, theme.heading.y * fade, theme.heading.z * fade, fade), 0.8);
    }

    // Gold particles
    if t > 0.5 {
        let count = ((t - 0.5) * 12.0).min(15.0) as usize;
        for i in 0..count {
            let seed_f = i as f32 * 73.1 + frame as f32 * 0.05;
            let x = -6.0 + seed_f.sin() * 4.0;
            let y = (seed_f.cos() * 3.0).abs() + 2.0;
            engine.spawn_glyph(Glyph {
                character: ['★', '✦', '+', '·'][i % 4],
                position: Vec3::new(x, y, 0.5),
                color: Vec4::new(1.0, 0.85, 0.1, 0.7),
                emission: 0.6,
                layer: RenderLayer::Particle, ..Default::default()
            });
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACHIEVEMENT UNLOCK (scales with rarity: 1.5-4 seconds)
// ═══════════════════════════════════════════════════════════════════════════════

pub fn achievement_unlock(name: &str, rarity: u8) -> CinematicState {
    let duration = match rarity {
        0 => 1.5,  // Common
        1 => 2.0,  // Rare
        2 => 2.5,  // Epic
        3 => 3.0,  // Legendary
        4 => 3.5,  // Mythic
        _ => 4.0,  // Omega
    };
    CinematicState::new(
        CinematicKind::AchievementUnlock,
        duration,
        CinematicData {
            text_primary: name.to_string(),
            rarity,
            ..Default::default()
        },
    )
}

pub fn render_achievement(cin: &CinematicState, engine: &mut ProofEngine, state: &GameState) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = cin.elapsed;
    let frame = state.frame;
    let rarity = cin.data.rarity;

    let (banner_color, particle_count, glow) = match rarity {
        0 => (theme.primary, 8, 0.3),
        1 => (Vec4::new(0.3, 0.5, 1.0, 1.0), 15, 0.5),
        2 => (Vec4::new(0.6, 0.2, 1.0, 1.0), 20, 0.8),
        3 => (Vec4::new(1.0, 0.85, 0.1, 1.0), 30, 1.0),
        4 => (Vec4::new(1.0, 0.3, 0.2, 1.0), 40, 1.2),
        _ => (Vec4::new(1.0, 1.0, 1.0, 1.0), 60, 1.5),
    };

    // Banner slide in from right
    let slide = (t * 3.0).min(1.0);
    let banner_x = 20.0 - slide * 28.0;
    let fade = if t > cin.duration - 0.5 { (cin.duration - t) * 2.0 } else { 1.0 };
    let fade = fade.clamp(0.0, 1.0);

    let label = format!("ACHIEVEMENT: {}", cin.data.text_primary);
    let color = Vec4::new(banner_color.x * fade, banner_color.y * fade, banner_color.z * fade, fade);
    render_text_at(engine, &label, banner_x, 8.0, color, glow * fade);

    // Particles
    let active_particles = ((t * particle_count as f32 / 0.5) as usize).min(particle_count);
    for i in 0..active_particles {
        let seed_f = i as f32 * 97.3 + frame as f32 * 0.08;
        let x = banner_x + (seed_f.sin()) * 8.0;
        let y = 8.0 + seed_f.cos() * 2.0;
        engine.spawn_glyph(Glyph {
            character: ['✦', '★', '·', '+'][i % 4],
            position: Vec3::new(x, y, 0.5),
            color: Vec4::new(banner_color.x, banner_color.y, banner_color.z, fade * 0.6),
            emission: glow * fade * 0.5,
            layer: RenderLayer::Overlay, ..Default::default()
        });
    }

    // Omega: rainbow border cycle
    if rarity >= 5 && t < cin.duration - 0.5 {
        let hue = (frame as f32 * 0.05) % 1.0;
        let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
        for i in 0..40 {
            let edge_x = if i < 10 { -18.0 + i as f32 * 3.6 }
                else if i < 20 { 18.0 }
                else if i < 30 { 18.0 - (i - 20) as f32 * 3.6 }
                else { -18.0 };
            let edge_y = if i < 10 { 10.0 }
                else if i < 20 { 10.0 - (i - 10) as f32 * 2.0 }
                else if i < 30 { -10.0 }
                else { -10.0 + (i - 30) as f32 * 2.0 };
            engine.spawn_glyph(Glyph {
                character: '█',
                position: Vec3::new(edge_x, edge_y, 3.0),
                color: Vec4::new(r, g, b, 0.4),
                emission: 0.8,
                layer: RenderLayer::Overlay, ..Default::default()
            });
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MISERY MILESTONES (2-3 seconds)
// ═══════════════════════════════════════════════════════════════════════════════

pub fn misery_milestone(value: u32) -> CinematicState {
    let (text, duration) = match value {
        5000 => ("SPITE UNLOCKED", 2.5),
        10000 => ("DEFIANCE ACTIVATED", 2.5),
        25000 => ("COSMIC JOKE", 2.5),
        50000 => ("TRANSCENDENT MISERY", 3.0),
        100000 => ("PUBLISHED FAILURE", 3.5),
        _ => ("MISERY MILESTONE", 2.0),
    };
    CinematicState::new(
        CinematicKind::MiseryMilestone,
        duration,
        CinematicData {
            text_primary: text.to_string(),
            milestone: value,
            ..Default::default()
        },
    )
}

pub fn render_misery_milestone(cin: &CinematicState, engine: &mut ProofEngine, state: &GameState) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = cin.elapsed;
    let frame = state.frame;

    // Dark red crack particles from center
    let crack_count = (t * 10.0).min(20.0) as usize;
    for i in 0..crack_count {
        let angle = (i as f32 / crack_count.max(1) as f32) * std::f32::consts::TAU;
        let r = t * 5.0;
        let fade = (1.0 - t / cin.duration).max(0.0);
        engine.spawn_glyph(Glyph {
            character: if i % 3 == 0 { '─' } else { '·' },
            position: Vec3::new(angle.cos() * r, angle.sin() * r * 0.5, 1.0),
            color: Vec4::new(0.8 * fade, 0.1 * fade, 0.1 * fade, fade),
            emission: fade * 0.6,
            layer: RenderLayer::Overlay, ..Default::default()
        });
    }

    // Milestone text
    if t > 0.5 {
        let text_fade = ((t - 0.5) * 2.0).min(1.0);
        let pulse = ((frame as f32 * 0.12).sin() * 0.2 + 0.8).max(0.0);
        let color = Vec4::new(0.9 * pulse * text_fade, 0.1 * text_fade, 0.15 * text_fade, text_fade);
        render_centered(engine, &cin.data.text_primary, 2.0, color, text_fade);
    }

    // Milestone-specific effects
    if cin.data.milestone >= 25000 && t > 1.0 {
        // Desaturation pulse for Cosmic Joke
        let desat = ((t - 1.0) * 3.0).sin().abs() * 0.3;
        // Visual represented as dim overlay
        render_centered(engine, "Ha.", 0.0, Vec4::new(0.3, 0.3, 0.3, desat), 0.2);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CORRUPTION MILESTONES (every 50 kills)
// ═══════════════════════════════════════════════════════════════════════════════

pub fn corruption_milestone(kills: u32) -> CinematicState {
    CinematicState::new(
        CinematicKind::CorruptionMilestone,
        1.5,
        CinematicData {
            text_primary: format!("CORRUPTION: {}", kills),
            milestone: kills,
            ..Default::default()
        },
    )
}

pub fn render_corruption_milestone(cin: &CinematicState, engine: &mut ProofEngine, state: &GameState) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = cin.elapsed;

    // Purple pulse ring from center
    let ring_r = t * 15.0;
    let fade = (1.0 - t / cin.duration).max(0.0);
    let points = 24;
    for i in 0..points {
        let angle = (i as f32 / points as f32) * std::f32::consts::TAU;
        let x = angle.cos() * ring_r;
        let y = angle.sin() * ring_r * 0.5;
        engine.spawn_glyph(Glyph {
            character: '·',
            position: Vec3::new(x, y, 1.0),
            color: Vec4::new(0.5 * fade, 0.1 * fade, 0.8 * fade, fade * 0.6),
            emission: fade * 0.5,
            layer: RenderLayer::Overlay, ..Default::default()
        });
    }

    // Text
    if t > 0.2 {
        let text_fade = ((t - 0.2) * 3.0).min(1.0) * fade;
        render_centered(engine, &cin.data.text_primary, 2.0,
            Vec4::new(0.6 * text_fade, 0.2 * text_fade, 0.9 * text_fade, text_fade), 0.6);
        render_centered(engine, "The engines mutate.", 0.5,
            Vec4::new(theme.dim.x * text_fade, theme.dim.y * text_fade, theme.dim.z * text_fade, text_fade * 0.7), 0.3);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// NEMESIS REVEAL (3 seconds)
// ═══════════════════════════════════════════════════════════════════════════════

pub fn nemesis_reveal(nemesis_name: &str, player_name: &str) -> CinematicState {
    CinematicState::new(
        CinematicKind::NemesisReveal,
        3.0,
        CinematicData {
            text_primary: nemesis_name.to_string(),
            text_secondary: format!("Slayer of {}", player_name),
            ..Default::default()
        },
    )
}

pub fn render_nemesis_reveal(cin: &CinematicState, engine: &mut ProofEngine, state: &GameState) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = cin.elapsed;

    // Darken
    let darken = (t * 2.0).min(0.6);
    render_centered(engine, "", 0.0, Vec4::new(0.0, 0.0, 0.0, darken), 0.0);

    // "Something remembers you..." typewriter
    if t > 0.5 && t < 2.0 {
        let text = "Something remembers you...";
        let chars = ((t - 0.5) * 12.0) as usize;
        let shown: String = text.chars().take(chars.min(text.len())).collect();
        render_centered(engine, &shown, 4.0,
            Vec4::new(0.7, 0.1, 0.1, 0.8), 0.5);
    }

    // Nemesis name
    if t > 1.5 {
        let fade = ((t - 1.5) * 2.0).min(1.0);
        render_centered(engine, &cin.data.text_primary, 2.0,
            Vec4::new(theme.danger.x * fade, theme.danger.y * fade, theme.danger.z * fade, fade), 0.9);
        render_centered(engine, &cin.data.text_secondary, 0.5,
            Vec4::new(theme.warn.x * fade, theme.warn.y * fade, theme.warn.z * fade, fade * 0.7), 0.4);
    }

    // "It has not forgotten."
    if t > 2.3 {
        let fade = ((t - 2.3) * 3.0).min(1.0);
        render_centered(engine, "It has not forgotten.", -1.0,
            Vec4::new(theme.dim.x * fade, theme.dim.y * fade, theme.dim.z * fade, fade), 0.3);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MASTER RENDER DISPATCH
// ═══════════════════════════════════════════════════════════════════════════════

/// Render the active cinematic if any. Returns true if a cinematic is playing.
pub fn render_active_cinematic(cin: &CinematicState, engine: &mut ProofEngine, state: &GameState) {
    match cin.kind {
        CinematicKind::BossEntrance => render_boss_entrance(cin, engine, state),
        CinematicKind::FloorTransition => render_floor_transition(cin, engine, state),
        CinematicKind::LevelUp => render_level_up(cin, engine, state),
        CinematicKind::AchievementUnlock => render_achievement(cin, engine, state),
        CinematicKind::MiseryMilestone => render_misery_milestone(cin, engine, state),
        CinematicKind::CorruptionMilestone => render_corruption_milestone(cin, engine, state),
        CinematicKind::NemesisReveal => render_nemesis_reveal(cin, engine, state),
        CinematicKind::BossPhaseTransition => {
            // Phase transition: brief text announcement
            let t = cin.elapsed;
            let fade = if t < 0.5 { t * 2.0 } else if t < 1.5 { 1.0 } else { (2.0 - t) * 2.0 };
            let fade = fade.clamp(0.0, 1.0);
            let theme = &THEMES[state.theme_idx % THEMES.len()];
            render_centered(engine, &cin.data.text_primary, 3.0,
                Vec4::new(theme.heading.x * fade, theme.heading.y * fade, theme.heading.z * fade, fade), 0.9);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

fn render_centered(engine: &mut ProofEngine, text: &str, y: f32, color: Vec4, emission: f32) {
    let x = -(text.len() as f32 * 0.225);
    render_text_at(engine, text, x, y, color, emission);
}

fn render_text_at(engine: &mut ProofEngine, text: &str, x: f32, y: f32, color: Vec4, emission: f32) {
    for (i, ch) in text.chars().enumerate() {
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x + i as f32 * 0.45, y, 2.0),
            color, emission,
            layer: RenderLayer::Overlay,
            ..Default::default()
        });
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let h6 = h * 6.0;
    let hi = h6 as u32;
    let f = h6.fract();
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match hi % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}
