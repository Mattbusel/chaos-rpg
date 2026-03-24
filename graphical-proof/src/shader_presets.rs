//! Shader graph presets — per-theme, per-floor, per-boss, per-status, corruption.
//!
//! Drives the engine's EffectsController with game-state-reactive parameters.
//! Every frame, `apply_shader_state()` updates bloom, grain, chromatic aberration,
//! vignette, scanlines, distortion, color grade, and motion blur based on:
//!   - Current theme (5 visual identities)
//!   - Floor depth (clean → degraded → VHS at 100+)
//!   - Corruption level (glitch frequency and intensity)
//!   - Active boss (per-boss shader overrides)
//!   - Status effects (tint, desaturation, distortion)

use proof_engine::prelude::*;
use proof_engine::effects::EffectsController;
use proof_engine::render::postfx::{
    bloom::BloomParams,
    grain::GrainParams,
    scanlines::ScanlineParams,
    chromatic::ChromaticParams,
    distortion::DistortionParams,
    motion_blur::MotionBlurParams,
    color_grade::ColorGradeParams,
};
use crate::state::GameState;
use crate::theme::THEMES;

// ═══════════════════════════════════════════════════════════════════════════════
// PER-THEME SHADER PRESETS
// ═══════════════════════════════════════════════════════════════════════════════

/// Base shader parameters for each theme.
pub struct ThemeShaderPreset {
    pub bloom_intensity: f32,
    pub bloom_threshold: f32,
    pub chromatic: f32,
    pub vignette: f32,
    pub grain: f32,
    pub scanlines: bool,
    pub scanline_intensity: f32,
    pub color_tint: Vec3,     // RGB multiplier
    pub saturation: f32,      // 0.0 = grayscale, 1.0 = normal
    pub contrast: f32,        // 1.0 = normal
    pub motion_blur: f32,
}

/// Get the shader preset for a theme index.
pub fn theme_preset(theme_idx: usize) -> ThemeShaderPreset {
    match theme_idx % 5 {
        // VOID PROTOCOL: chromatic aberration + deep vignette + grain + blue-purple grade
        0 => ThemeShaderPreset {
            bloom_intensity: 1.2,
            bloom_threshold: 0.7,
            chromatic: 0.003,
            vignette: 0.4,
            grain: 0.08,
            scanlines: false,
            scanline_intensity: 0.0,
            color_tint: Vec3::new(0.85, 0.8, 1.1),  // blue-purple
            saturation: 1.0,
            contrast: 1.05,
            motion_blur: 0.0,
        },
        // BLOOD PACT: high contrast + red tint + heavy vignette + grain
        1 => ThemeShaderPreset {
            bloom_intensity: 0.8,
            bloom_threshold: 0.8,
            chromatic: 0.001,
            vignette: 0.6,
            grain: 0.12,
            scanlines: false,
            scanline_intensity: 0.0,
            color_tint: Vec3::new(1.15, 0.9, 0.85),  // red tint
            saturation: 1.1,
            contrast: 1.2,
            motion_blur: 0.0,
        },
        // EMERALD ENGINE: CRT scanlines + green grade + bloom boost
        2 => ThemeShaderPreset {
            bloom_intensity: 1.3,
            bloom_threshold: 0.6,
            chromatic: 0.002,
            vignette: 0.3,
            grain: 0.04,
            scanlines: true,
            scanline_intensity: 0.15,
            color_tint: Vec3::new(0.85, 1.1, 0.9),  // green
            saturation: 1.0,
            contrast: 1.1,
            motion_blur: 0.0,
        },
        // SOLAR FORGE: warm golden hour + lens flare hint + bloom
        3 => ThemeShaderPreset {
            bloom_intensity: 1.4,
            bloom_threshold: 0.5,
            chromatic: 0.0,
            vignette: 0.5,
            grain: 0.03,
            scanlines: false,
            scanline_intensity: 0.0,
            color_tint: Vec3::new(1.15, 1.05, 0.85),  // warm gold
            saturation: 1.1,
            contrast: 1.0,
            motion_blur: 0.0,
        },
        // GLACIAL ABYSS: desaturation + blue tint + motion blur + crisp contrast
        _ => ThemeShaderPreset {
            bloom_intensity: 0.9,
            bloom_threshold: 0.75,
            chromatic: 0.001,
            vignette: 0.35,
            grain: 0.02,
            scanlines: false,
            scanline_intensity: 0.0,
            color_tint: Vec3::new(0.85, 0.95, 1.15),  // blue
            saturation: 0.7,
            contrast: 1.15,
            motion_blur: 0.15,
        },
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PER-FLOOR SHADER EVOLUTION
// ═══════════════════════════════════════════════════════════════════════════════

/// Floor-depth visual degradation parameters.
pub struct FloorShaderMods {
    pub vignette_add: f32,
    pub grain_add: f32,
    pub chromatic_add: f32,
    pub distortion: f32,
    pub crt_curvature: bool,
    pub vhs_tracking: bool,
}

pub fn floor_shader_mods(floor: u32) -> FloorShaderMods {
    match floor {
        0..=10 => FloorShaderMods {
            vignette_add: 0.0, grain_add: 0.0, chromatic_add: 0.0,
            distortion: 0.0, crt_curvature: false, vhs_tracking: false,
        },
        11..=25 => FloorShaderMods {
            vignette_add: 0.05, grain_add: 0.0, chromatic_add: 0.0,
            distortion: 0.0, crt_curvature: false, vhs_tracking: false,
        },
        26..=50 => FloorShaderMods {
            vignette_add: 0.1, grain_add: 0.04, chromatic_add: 0.001,
            distortion: 0.0, crt_curvature: false, vhs_tracking: false,
        },
        51..=75 => FloorShaderMods {
            vignette_add: 0.15, grain_add: 0.08, chromatic_add: 0.002,
            distortion: 0.01, crt_curvature: false, vhs_tracking: false,
        },
        76..=99 => FloorShaderMods {
            vignette_add: 0.25, grain_add: 0.15, chromatic_add: 0.005,
            distortion: 0.03, crt_curvature: false, vhs_tracking: false,
        },
        _ => FloorShaderMods {
            vignette_add: 0.35, grain_add: 0.2, chromatic_add: 0.008,
            distortion: 0.05, crt_curvature: true, vhs_tracking: true,
        },
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CORRUPTION SHADER EFFECTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Corruption-driven visual degradation.
pub struct CorruptionShaderMods {
    pub glitch_chance: f32,     // probability per frame of a glitch (0.0-1.0)
    pub chromatic_add: f32,
    pub color_shift: Vec3,      // additive color shift (toward purple)
    pub desaturation: f32,      // subtracted from saturation
    pub vhs_tracking: bool,
}

pub fn corruption_shader_mods(corruption: u32) -> CorruptionShaderMods {
    match corruption {
        0..=99 => CorruptionShaderMods {
            glitch_chance: 0.0, chromatic_add: 0.0,
            color_shift: Vec3::ZERO, desaturation: 0.0, vhs_tracking: false,
        },
        100..=199 => CorruptionShaderMods {
            glitch_chance: 1.0 / 300.0,  // ~1 in 300 frames
            chromatic_add: 0.0,
            color_shift: Vec3::ZERO, desaturation: 0.0, vhs_tracking: false,
        },
        200..=299 => CorruptionShaderMods {
            glitch_chance: 1.0 / 100.0,
            chromatic_add: 0.001,
            color_shift: Vec3::new(0.02, -0.01, 0.03),  // slight purple
            desaturation: 0.0, vhs_tracking: false,
        },
        300..=399 => CorruptionShaderMods {
            glitch_chance: 1.0 / 30.0,
            chromatic_add: 0.003,
            color_shift: Vec3::new(0.04, -0.02, 0.06),
            desaturation: 0.05, vhs_tracking: false,
        },
        _ => CorruptionShaderMods {
            glitch_chance: 1.0 / 15.0,
            chromatic_add: 0.006,
            color_shift: Vec3::new(0.06, -0.03, 0.08),
            desaturation: 0.15, vhs_tracking: true,
        },
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PER-BOSS SHADER OVERRIDES
// ═══════════════════════════════════════════════════════════════════════════════

/// Boss-specific shader parameters (applied on top of theme+floor+corruption).
pub struct BossShaderOverride {
    pub hue_rotation: f32,       // degrees (0 = none, 180 = inversion)
    pub saturation_mult: f32,    // multiplier (1.0 = unchanged)
    pub bloom_mult: f32,
    pub chromatic_mult: f32,
    pub distortion_override: Option<f32>,
    pub vignette_override: Option<f32>,
    pub grain_override: Option<f32>,
    pub scanlines_override: Option<bool>,
    /// The Null: progressive effect stripping (0.0 = full effects, 1.0 = nothing)
    pub null_strip_progress: f32,
    /// Algorithm Reborn Phase 3: glitch + VHS
    pub glitch_intensity: f32,
    /// Ouroboros: radial distortion centered on boss
    pub radial_distortion: f32,
    /// Fibonacci Hydra: golden spiral overlay opacity
    pub spiral_overlay: f32,
}

impl Default for BossShaderOverride {
    fn default() -> Self {
        Self {
            hue_rotation: 0.0, saturation_mult: 1.0, bloom_mult: 1.0,
            chromatic_mult: 1.0, distortion_override: None,
            vignette_override: None, grain_override: None,
            scanlines_override: None, null_strip_progress: 0.0,
            glitch_intensity: 0.0, radial_distortion: 0.0, spiral_overlay: 0.0,
        }
    }
}

pub fn boss_shader_override(boss_id: u8, turn: u32) -> BossShaderOverride {
    let mut o = BossShaderOverride::default();
    match boss_id {
        // The Mirror: vertical split (handled in boss_visuals, shader just adds slight bloom)
        1 => { o.bloom_mult = 1.3; }

        // The Null: progressive stripping
        6 => {
            let progress = (turn as f32 / 10.0).min(1.0);
            o.null_strip_progress = progress;
            o.bloom_mult = 1.0 - progress;
            o.saturation_mult = 1.0 - progress * 0.8;
            o.grain_override = Some(0.0);
            if turn >= 5 { o.scanlines_override = Some(false); }
        }

        // The Paradox: hue inversion
        11 => { o.hue_rotation = 180.0; }

        // Algorithm Reborn Phase 3: full glitch
        12 => {
            if turn >= 10 {
                o.glitch_intensity = 0.5 + (turn as f32 - 10.0) * 0.05;
                o.chromatic_mult = 3.0;
                o.distortion_override = Some(0.08);
                o.scanlines_override = Some(true);
            }
        }

        // Ouroboros: radial distortion
        7 => {
            let cycle = (turn % 3) as f32 / 3.0;
            o.radial_distortion = cycle * 0.03;
        }

        // Fibonacci Hydra: golden spiral overlay
        3 => {
            let splits = (turn / 3 + 1).min(8) as f32;
            o.spiral_overlay = (splits / 8.0) * 0.15;
            o.bloom_mult = 1.0 + splits * 0.1;
        }

        // The Accountant: cold fluorescent (handled by lighting, shader just desaturates)
        2 => {
            o.saturation_mult = 0.85;
        }

        _ => {} // Default for other bosses
    }
    o
}

// ═══════════════════════════════════════════════════════════════════════════════
// STATUS EFFECT SHADER MODIFICATIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Shader modifications from active status effects on the player.
pub struct StatusShaderMods {
    pub green_tint: f32,        // poison
    pub desaturation_add: f32,  // cursed
    pub wave_distortion: f32,   // confused
    pub chromatic_add: f32,     // feared
    pub vignette_add: f32,      // feared
    pub fov_zoom: f32,          // feared (slight zoom in)
    pub warm_tint: f32,         // blessed
    pub bloom_add: f32,         // blessed
    pub motion_blur_add: f32,   // hasted
    pub motion_blur_remove: bool, // slowed (sharp/frozen look)
}

impl Default for StatusShaderMods {
    fn default() -> Self {
        Self {
            green_tint: 0.0, desaturation_add: 0.0, wave_distortion: 0.0,
            chromatic_add: 0.0, vignette_add: 0.0, fov_zoom: 0.0,
            warm_tint: 0.0, bloom_add: 0.0, motion_blur_add: 0.0,
            motion_blur_remove: false,
        }
    }
}

/// Compute shader modifications from player status effects.
pub fn status_shader_mods(state: &GameState) -> StatusShaderMods {
    let mut mods = StatusShaderMods::default();

    if let Some(ref player) = state.player {
        for effect in &player.status_effects {
            let name = format!("{:?}", effect);
            let name_lower = name.to_lowercase();

            if name_lower.contains("poison") {
                mods.green_tint = 0.06;
            }
            if name_lower.contains("curse") {
                mods.desaturation_add = 0.15;
                mods.vignette_add = 0.1;
            }
            if name_lower.contains("stun") || name_lower.contains("confus") {
                mods.wave_distortion = 0.015;
            }
            if name_lower.contains("fear") {
                mods.chromatic_add = 0.004;
                mods.vignette_add += 0.15;
                mods.fov_zoom = 0.05;
            }
            if name_lower.contains("shield") || name_lower.contains("empow") {
                mods.warm_tint = 0.04;
                mods.bloom_add = 0.3;
            }
            if name_lower.contains("regen") {
                mods.bloom_add += 0.1;
            }
        }
    }

    mods
}

// ═══════════════════════════════════════════════════════════════════════════════
// MASTER SHADER APPLICATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Apply all shader state to the effects controller based on current game state.
/// Call this every frame from the main game loop.
pub fn apply_shader_state(state: &GameState, effects: &mut EffectsController) {
    let theme_idx = state.theme_idx;
    let floor = state.floor_num;
    let corruption = state.player.as_ref().map(|p| p.corruption).unwrap_or(0);
    let frame = state.frame;

    // Layer 1: Theme base
    let theme = theme_preset(theme_idx);

    // Layer 2: Floor depth evolution
    let floor_mods = floor_shader_mods(floor);

    // Layer 3: Corruption degradation
    let corr_mods = corruption_shader_mods(corruption);

    // Layer 4: Boss overrides (if in boss fight)
    let boss_override = state.boss_id
        .map(|id| boss_shader_override(id, state.boss_turn))
        .unwrap_or_default();

    // Layer 5: Status effects
    let status_mods = status_shader_mods(state);

    // ── Compose final parameters ─────────────────────────────────────────────

    // Bloom
    let bloom_intensity = (theme.bloom_intensity + status_mods.bloom_add) * boss_override.bloom_mult;
    effects.bloom = BloomParams {
        intensity: bloom_intensity * (1.0 - boss_override.null_strip_progress),
        threshold: theme.bloom_threshold,
        ..effects.bloom.clone()
    };

    // Grain
    let grain = boss_override.grain_override.unwrap_or(
        theme.grain + floor_mods.grain_add
    );
    effects.grain = GrainParams {
        intensity: grain * (1.0 - boss_override.null_strip_progress),
        ..effects.grain.clone()
    };

    // Scanlines
    let scanlines_on = boss_override.scanlines_override.unwrap_or(
        theme.scanlines || floor_mods.crt_curvature
    );
    effects.scanlines = ScanlineParams {
        enabled: scanlines_on && boss_override.null_strip_progress < 0.7,
        intensity: theme.scanline_intensity,
        ..effects.scanlines.clone()
    };

    // Chromatic aberration (uses red_offset / blue_offset fields)
    let chromatic = theme.chromatic
        + floor_mods.chromatic_add
        + corr_mods.chromatic_add
        + status_mods.chromatic_add;
    let chromatic = chromatic * boss_override.chromatic_mult * (1.0 - boss_override.null_strip_progress);
    effects.chromatic.red_offset = chromatic;
    effects.chromatic.blue_offset = chromatic * 1.2;
    effects.chromatic.enabled = chromatic > 0.0005;

    // Distortion (uses scale field)
    let distortion = boss_override.distortion_override.unwrap_or(
        floor_mods.distortion + boss_override.radial_distortion + status_mods.wave_distortion
    );
    effects.distortion.scale = distortion * (1.0 - boss_override.null_strip_progress);
    effects.distortion.enabled = distortion > 0.001;

    // Motion blur (uses scale field)
    let motion_blur = if status_mods.motion_blur_remove {
        0.0
    } else {
        theme.motion_blur + status_mods.motion_blur_add
    };
    effects.motion_blur.scale = motion_blur;
    effects.motion_blur.enabled = motion_blur > 0.01;

    // Color grade (uses tint, saturation, contrast fields — no hue_rotation)
    let saturation = (theme.saturation - corr_mods.desaturation - status_mods.desaturation_add)
        * boss_override.saturation_mult;
    let tint = Vec3::new(
        theme.color_tint.x + corr_mods.color_shift.x + status_mods.warm_tint,
        theme.color_tint.y + corr_mods.color_shift.y + status_mods.green_tint,
        theme.color_tint.z + corr_mods.color_shift.z,
    );
    effects.color_grade.saturation = saturation.max(0.0);
    effects.color_grade.contrast = theme.contrast;
    effects.color_grade.tint = tint;

    // ── Corruption glitch (random frame UV offset) ──
    if corr_mods.glitch_chance > 0.0 {
        let hash = (frame.wrapping_mul(2654435761) >> 16) as f32 / 65536.0;
        if hash < corr_mods.glitch_chance {
            effects.distortion.scale += 0.1;
            effects.chromatic.red_offset += 0.01;
            effects.chromatic.blue_offset += 0.012;
        }
    }

    // ── Algorithm Reborn Phase 3 glitch ──
    if boss_override.glitch_intensity > 0.0 {
        let glitch_hash = ((frame.wrapping_mul(1103515245).wrapping_add(12345)) >> 16) as f32 / 65536.0;
        if glitch_hash < boss_override.glitch_intensity * 0.3 {
            effects.distortion.scale += boss_override.glitch_intensity * 0.15;
            effects.chromatic.red_offset += boss_override.glitch_intensity * 0.02;
            effects.chromatic.blue_offset += boss_override.glitch_intensity * 0.025;
        }
    }
}
