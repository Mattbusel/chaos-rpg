//! Boss-specific magical visual effects.
//!
//! Each boss gets a unique magical visual profile rendered via glyph spawning.
//! 10 boss profiles with distinct spell visuals, aura effects, and combat overlays.

use proof_engine::prelude::*;
use glam::{Vec3, Vec4};
use crate::state::GameState;

// ── Boss magic profiles ─────────────────────────────────────────────────────

/// Identifies which boss magic profile to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossMagicProfile {
    Mirror,           // Boss 1: copies player spells
    Null,             // Boss 6: erases UI elements
    Committee,        // Boss 9: 5 judge elemental beams
    FibonacciHydra,   // Boss 3: split visuals
    Eigenstate,       // Boss 4: quantum superposition
    Ouroboros,        // Boss 7: reversed visuals
    AlgorithmReborn,  // Boss 12: 3-phase escalation
    ChaosWeaver,      // Custom: random visual glitches
    VoidSerpent,      // Custom: growing void border
    PrimeFactorial,   // Custom: number factorization visuals
}

impl BossMagicProfile {
    /// Map from boss_id to profile.
    pub fn from_boss_id(id: u8) -> Option<Self> {
        match id {
            1  => Some(Self::Mirror),
            6  => Some(Self::Null),
            9  => Some(Self::Committee),
            3  => Some(Self::FibonacciHydra),
            4  => Some(Self::Eigenstate),
            7  => Some(Self::Ouroboros),
            12 => Some(Self::AlgorithmReborn),
            13 => Some(Self::ChaosWeaver),
            14 => Some(Self::VoidSerpent),
            15 => Some(Self::PrimeFactorial),
            _  => None,
        }
    }
}

// ── Boss magic renderer ─────────────────────────────────────────────────────

/// Renders boss-specific magical overlays each frame.
pub struct BossMagicRenderer {
    /// Cached last spell name from player (for mirror boss).
    pub last_player_spell: String,
    /// Phase tracker for Algorithm Reborn.
    pub algorithm_phase: u8,
    /// Void border size for VoidSerpent.
    pub void_border: f32,
    /// Scramble seed for ChaosWeaver.
    pub scramble_seed: u64,
}

impl BossMagicRenderer {
    pub fn new() -> Self {
        Self {
            last_player_spell: String::new(),
            algorithm_phase: 1,
            void_border: 0.0,
            scramble_seed: 0,
        }
    }

    /// Update boss magic state each frame.
    pub fn update(&mut self, dt: f32, state: &GameState) {
        // Track algorithm phase
        if let Some(12) = state.boss_id {
            self.algorithm_phase = if state.boss_turn < 5 { 1 }
                else if state.boss_turn < 10 { 2 }
                else { 3 };
        }

        // Grow void border for VoidSerpent
        if let Some(14) = state.boss_id {
            self.void_border = (self.void_border + dt * 0.1).min(2.0);
        }

        // Update scramble seed for ChaosWeaver
        if let Some(13) = state.boss_id {
            self.scramble_seed = state.frame;
        }

        // Track last player spell for Mirror
        if !state.last_spell_name.is_empty() {
            self.last_player_spell = state.last_spell_name.clone();
        }
    }

    /// Render boss magic overlay for the current frame.
    pub fn render(&self, engine: &mut ProofEngine, state: &GameState) {
        let boss_id = match state.boss_id {
            Some(id) => id,
            None => return,
        };
        let profile = match BossMagicProfile::from_boss_id(boss_id) {
            Some(p) => p,
            None => return,
        };
        let frame = state.frame;
        let turn = state.boss_turn;

        match profile {
            BossMagicProfile::Mirror          => render_mirror(engine, state, frame),
            BossMagicProfile::Null            => render_null(engine, state, frame, turn),
            BossMagicProfile::Committee       => render_committee(engine, state, frame, turn),
            BossMagicProfile::FibonacciHydra  => render_fibonacci_hydra(engine, state, frame, turn),
            BossMagicProfile::Eigenstate      => render_eigenstate(engine, state, frame),
            BossMagicProfile::Ouroboros       => render_ouroboros(engine, state, frame, turn),
            BossMagicProfile::AlgorithmReborn => render_algorithm_reborn(engine, state, frame, turn, self.algorithm_phase),
            BossMagicProfile::ChaosWeaver     => render_chaos_weaver(engine, state, frame, self.scramble_seed),
            BossMagicProfile::VoidSerpent     => render_void_serpent(engine, state, frame, self.void_border),
            BossMagicProfile::PrimeFactorial  => render_prime_factorial(engine, state, frame, turn),
        }
    }
}

// ── Mirror boss ─────────────────────────────────────────────────────────────
// Creates reversed copy of player's last spell: mirror-flipped colors.

fn render_mirror(engine: &mut ProofEngine, state: &GameState, frame: u64) {
    // Mirror reflection line
    let mirror_pulse = ((frame as f32 * 0.08).sin() * 0.2 + 0.7).max(0.0);
    for i in 0..20 {
        let y = -5.0 + i as f32 * 0.5;
        engine.spawn_glyph(Glyph {
            character: '|',
            position: Vec3::new(0.0, y, -0.5),
            color: Vec4::new(0.5 * mirror_pulse, 0.8 * mirror_pulse, 1.0 * mirror_pulse, 0.4),
            emission: mirror_pulse * 0.3,
            layer: RenderLayer::Overlay,
            ..Default::default()
        });
    }

    // Mirrored spell echo — show reversed color glyphs on boss side
    // when player casts a spell
    if !state.last_spell_name.is_empty() && state.spell_beam_timer > 0.0 {
        let spell_color = state.spell_beam_color;
        // Invert the color
        let inv_color = Vec4::new(1.0 - spell_color.x, 1.0 - spell_color.y, 1.0 - spell_color.z, spell_color.w);

        // Mirror projectile on boss side (positive X)
        let mirror_t = (1.0 - state.spell_beam_timer / 0.5).clamp(0.0, 1.0);
        let mx = 2.0 + mirror_t * 4.0;
        let my = ((mirror_t * std::f32::consts::PI).sin()) * 1.0 + 2.0;

        let mirror_glyphs = ['◐', '◑', '●', '○'];
        for i in 0..4 {
            let offset = i as f32 * 0.3;
            let fade = (1.0 - mirror_t * 0.5).max(0.0);
            engine.spawn_glyph(Glyph {
                character: mirror_glyphs[i],
                position: Vec3::new(mx + offset, my, 0.0),
                color: Vec4::new(inv_color.x, inv_color.y, inv_color.z, fade * 0.7),
                emission: fade * 0.6,
                glow_color: Vec3::new(inv_color.x, inv_color.y, inv_color.z),
                glow_radius: fade * 1.0,
                layer: RenderLayer::Particle,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }

        // "REFLECTED" text
        let reflect_alpha = (1.0 - mirror_t).max(0.0);
        if reflect_alpha > 0.2 {
            render_boss_text(engine, "REFLECTED", 2.0, 4.0,
                Vec4::new(inv_color.x, inv_color.y, inv_color.z, reflect_alpha * 0.5),
                reflect_alpha * 0.4);
        }
    }
}

// ── Null boss ───────────────────────────────────────────────────────────────
// Erases UI elements with expanding black rectangles, void zones blank screen.

fn render_null(engine: &mut ProofEngine, state: &GameState, frame: u64, turn: u32) {
    let null_progress = (turn as f32 / 10.0).min(1.0);

    // Void zones that blank out screen areas
    let void_count = (null_progress * 5.0) as usize;
    for i in 0..void_count {
        let seed = i as f32 * 37.1;
        let vx = seed.sin() * 6.0;
        let vy = seed.cos() * 4.0;
        let void_size = 0.5 + null_progress * 1.5;

        // Black rectangle expanding over UI
        for dx in 0..((void_size * 3.0) as usize) {
            for dy in 0..((void_size * 2.0) as usize) {
                engine.spawn_glyph(Glyph {
                    character: ' ',
                    position: Vec3::new(
                        vx - void_size + dx as f32 * 0.5,
                        vy - void_size * 0.5 + dy as f32 * 0.5,
                        0.8,
                    ),
                    color: Vec4::new(0.0, 0.0, 0.0, null_progress * 0.8),
                    emission: 0.0,
                    layer: RenderLayer::Overlay,
                    ..Default::default()
                });
            }
        }
    }

    // Void border creeping in from edges
    let border_depth = null_progress * 1.5;
    let border_chars = [' ', '░', '▒', '▓', '█'];
    for i in 0..40 {
        let t = i as f32 * 0.5;
        // Top edge
        for d in 0..((border_depth * 2.0) as usize).max(1) {
            let depth_frac = d as f32 / (border_depth * 2.0).max(1.0);
            let ch_idx = ((1.0 - depth_frac) * 4.0) as usize;
            engine.spawn_glyph(Glyph {
                character: border_chars[ch_idx.min(4)],
                position: Vec3::new(-10.0 + t, 5.4 - d as f32 * 0.4, 0.9),
                color: Vec4::new(0.0, 0.0, 0.0, null_progress * 0.6),
                emission: 0.0,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }
    }

    // "NULL" text glitching
    if turn >= 5 {
        let glitch = ((frame as f32 * 0.5).sin() * 0.5 + 0.5).max(0.0);
        let null_text = if (frame / 10) % 3 == 0 { "N U L L" } else if (frame / 10) % 3 == 1 { "N_L_" } else { "      " };
        render_boss_text(engine, null_text, -2.0, 0.0,
            Vec4::new(0.3, 0.3, 0.3, glitch * 0.5),
            glitch * 0.2);
    }
}

// ── Committee boss ──────────────────────────────────────────────────────────
// 5 elemental beams from each judge, vote visual with colored lights.

fn render_committee(engine: &mut ProofEngine, state: &GameState, frame: u64, turn: u32) {
    let judge_elements = [
        (Vec4::new(1.0, 0.3, 0.05, 1.0), '🜂'),  // Fire judge
        (Vec4::new(0.2, 0.6, 1.0, 1.0), '❄'),     // Ice judge
        (Vec4::new(1.0, 1.0, 0.3, 1.0), '⚡'),     // Lightning judge
        (Vec4::new(0.1, 0.8, 0.2, 1.0), '☠'),     // Poison judge
        (Vec4::new(0.3, 0.05, 0.5, 1.0), '◐'),    // Shadow judge
    ];

    // 5 judge positions across top
    for (j, (color, glyph_char)) in judge_elements.iter().enumerate() {
        let jx = -4.0 + j as f32 * 2.0;
        let jy = 4.5;

        // Judge glyph
        engine.spawn_glyph(Glyph {
            character: *glyph_char,
            position: Vec3::new(jx, jy, 0.0),
            color: *color,
            emission: 0.6,
            glow_color: Vec3::new(color.x, color.y, color.z),
            glow_radius: 1.0,
            layer: RenderLayer::Overlay,
            ..Default::default()
        });

        // Elemental beam from judge toward target area
        let beam_active = ((frame / 20 + j as u64) * 7919) % 5 < 3;
        if beam_active {
            let target_y = -2.0;
            let segs = 8;
            for s in 0..segs {
                let st = s as f32 / segs as f32;
                let sx = jx + ((frame as f32 * 0.3 + s as f32 * 5.0 + j as f32 * 3.0).sin()) * 0.3;
                let sy = jy - (jy - target_y) * st;
                let fade = 1.0 - st * 0.5;
                engine.spawn_glyph(Glyph {
                    character: '|',
                    position: Vec3::new(sx, sy, 0.0),
                    color: Vec4::new(color.x, color.y, color.z, fade * 0.5),
                    emission: fade * 0.6,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }
        }

        // Vote visual — colored light sequencing
        let vote_phase = (frame / 30) % 5;
        if vote_phase == j as u64 {
            // This judge is currently voting
            let vote_pulse = ((frame as f32 * 0.3).sin() * 0.3 + 0.7).max(0.0);
            engine.spawn_glyph(Glyph {
                character: '●',
                position: Vec3::new(jx, jy - 0.6, 0.0),
                scale: Vec2::splat(0.5),
                color: Vec4::new(color.x, color.y, color.z, vote_pulse),
                emission: vote_pulse * 1.0,
                glow_color: Vec3::new(color.x, color.y, color.z),
                glow_radius: vote_pulse * 1.5,
                layer: RenderLayer::Overlay,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }
    }

    // "DELIBERATING" text
    if (frame / 60) % 2 == 0 {
        let dots = ".".repeat(((frame / 15) % 4) as usize);
        let text = format!("DELIBERATING{}", dots);
        render_boss_text(engine, &text, -2.5, 3.5,
            Vec4::new(0.7, 0.7, 0.7, 0.4), 0.3);
    }
}

// ── Fibonacci Hydra ─────────────────────────────────────────────────────────
// Split visual: entity tears in half with energy bridge, then separates.

fn render_fibonacci_hydra(engine: &mut ProofEngine, state: &GameState, frame: u64, turn: u32) {
    // Number of heads based on turn (Fibonacci-style growth)
    let heads = (turn as usize / 3 + 1).min(8);
    let boss_x = 5.0;
    let boss_y = 1.0;

    for h in 0..heads {
        let head_angle = (h as f32 / heads as f32) * std::f32::consts::TAU * 0.6 - 0.5;
        let head_r = 1.5 + h as f32 * 0.3;
        let hx = boss_x + head_angle.cos() * head_r;
        let hy = boss_y + head_angle.sin() * head_r;

        // Head glyph
        let head_pulse = ((frame as f32 * 0.1 + h as f32 * 1.5).sin() * 0.2 + 0.8).max(0.0);
        engine.spawn_glyph(Glyph {
            character: 'H',
            position: Vec3::new(hx, hy, 0.0),
            scale: Vec2::splat(0.6),
            color: Vec4::new(0.8, 0.6, 0.1, head_pulse),
            emission: head_pulse * 0.5,
            glow_color: Vec3::new(0.8, 0.6, 0.1),
            glow_radius: 0.8,
            layer: RenderLayer::Entity,
            ..Default::default()
        });

        // Neck connecting to body
        let segs = 4;
        for s in 1..segs {
            let st = s as f32 / segs as f32;
            let nx = boss_x + (hx - boss_x) * st;
            let ny = boss_y + (hy - boss_y) * st;
            engine.spawn_glyph(Glyph {
                character: '~',
                position: Vec3::new(nx, ny, 0.0),
                color: Vec4::new(0.6, 0.5, 0.1, 0.5),
                emission: 0.2,
                layer: RenderLayer::Entity,
                ..Default::default()
            });
        }
    }

    // Split animation — energy bridge between splitting heads
    if turn % 3 == 2 && heads < 8 {
        let split_t = (frame % 60) as f32 / 60.0;
        let split_x = boss_x;
        let split_y = boss_y + 1.0;

        // Tear effect — glyphs separating
        let sep = split_t * 1.5;
        engine.spawn_glyph(Glyph {
            character: '<',
            position: Vec3::new(split_x - sep, split_y, 0.0),
            color: Vec4::new(1.0, 0.8, 0.2, 1.0 - split_t),
            emission: (1.0 - split_t) * 0.8,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
        engine.spawn_glyph(Glyph {
            character: '>',
            position: Vec3::new(split_x + sep, split_y, 0.0),
            color: Vec4::new(1.0, 0.8, 0.2, 1.0 - split_t),
            emission: (1.0 - split_t) * 0.8,
            layer: RenderLayer::Particle,
            ..Default::default()
        });

        // Energy bridge between halves
        let bridge_segs = 5;
        for s in 0..bridge_segs {
            let bt = s as f32 / bridge_segs as f32;
            let bx = split_x - sep + bt * sep * 2.0;
            let by = split_y + ((bt * std::f32::consts::PI).sin()) * 0.3;
            let bridge_fade = (1.0 - split_t * 0.6).max(0.0);
            engine.spawn_glyph(Glyph {
                character: '-',
                position: Vec3::new(bx, by, 0.0),
                color: Vec4::new(1.0, 0.9, 0.3, bridge_fade * 0.5),
                emission: bridge_fade * 0.6,
                layer: RenderLayer::Particle,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }
    }

    // Golden ratio spiral in background
    let phi: f32 = 1.618033988749895;
    for i in 0..20 {
        let angle = i as f32 * 2.399963; // golden angle
        let r = (i as f32).sqrt() * 0.8;
        let x = boss_x + angle.cos() * r;
        let y = boss_y + angle.sin() * r * 0.5 - 2.0;
        let fade = 1.0 - (i as f32 / 20.0);
        engine.spawn_glyph(Glyph {
            character: if i % 3 == 0 { 'F' } else { '.' },
            position: Vec3::new(x, y, -0.5),
            color: Vec4::new(1.0 * fade, 0.85 * fade, 0.2 * fade, fade * 0.3),
            emission: fade * 0.15,
            layer: RenderLayer::Background,
            ..Default::default()
        });
    }
}

// ── Eigenstate ──────────────────────────────────────────────────────────────
// Quantum superposition: two overlapping translucent entities, observation collapse.

fn render_eigenstate(engine: &mut ProofEngine, state: &GameState, frame: u64) {
    let boss_x = 5.0;
    let boss_y = 2.0;

    // Two superimposed states
    let is_observed = (frame / 8) % 2 == 0;
    let state_a_alpha = if is_observed { 0.8 } else { 0.3 };
    let state_b_alpha = if is_observed { 0.3 } else { 0.8 };

    // State A: large entity
    let offset_a = ((frame as f32 * 0.05).sin()) * 0.3;
    engine.spawn_glyph(Glyph {
        character: '█',
        position: Vec3::new(boss_x + offset_a, boss_y + 0.2, 0.0),
        scale: Vec2::splat(1.2),
        color: Vec4::new(1.0, 0.3, 0.3, state_a_alpha),
        emission: state_a_alpha * 0.5,
        layer: RenderLayer::Entity,
        ..Default::default()
    });

    // State B: small entity
    let offset_b = ((frame as f32 * 0.07).cos()) * 0.3;
    engine.spawn_glyph(Glyph {
        character: '·',
        position: Vec3::new(boss_x + offset_b, boss_y - 0.2, 0.0),
        scale: Vec2::splat(0.5),
        color: Vec4::new(0.3, 0.3, 1.0, state_b_alpha),
        emission: state_b_alpha * 0.5,
        layer: RenderLayer::Entity,
        ..Default::default()
    });

    // Quantum shimmer — interference pattern between states
    for i in 0..12 {
        let qx = boss_x + ((frame as f32 * 0.08 + i as f32 * 0.7).sin()) * 1.5;
        let qy = boss_y + ((frame as f32 * 0.06 + i as f32 * 1.1).cos()) * 1.0;
        let interference = ((frame as f32 * 0.1 + i as f32 * 3.0).sin() * 0.5 + 0.5).max(0.0);
        engine.spawn_glyph(Glyph {
            character: if (frame + i as u64) % 3 == 0 { '░' } else { '▒' },
            position: Vec3::new(qx, qy, 0.0),
            color: Vec4::new(0.5, 0.3, 0.8, interference * 0.3),
            emission: interference * 0.2,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    // Observation collapse flash
    if (frame % 16) == 0 {
        engine.spawn_glyph(Glyph {
            character: '*',
            position: Vec3::new(boss_x, boss_y, 0.0),
            scale: Vec2::splat(1.5),
            color: Vec4::new(1.0, 1.0, 1.0, 0.4),
            emission: 1.0,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }

    // Superposition label
    let label = if is_observed { "|1>" } else { "|0>" };
    render_boss_text(engine, label, boss_x - 0.5, boss_y + 1.5,
        Vec4::new(0.7, 0.5, 1.0, 0.5), 0.3);
}

// ── Ouroboros ────────────────────────────────────────────────────────────────
// Reversed healing/damage visuals, ouroboros ring.

fn render_ouroboros(engine: &mut ProofEngine, state: &GameState, frame: u64, turn: u32) {
    let boss_x = 5.0;
    let boss_y = 2.0;

    // Ouroboros ring — snake eating its tail
    let ring_points = 24;
    let ring_r = 3.0;
    let cycle_progress = (turn % 3) as f32 / 3.0;

    for i in 0..ring_points {
        let frac = i as f32 / ring_points as f32;
        let angle = frac * std::f32::consts::TAU + frame as f32 * 0.03;
        let rx = boss_x + angle.cos() * ring_r;
        let ry = boss_y + angle.sin() * ring_r * 0.5;

        // Snake body segments
        let is_head = i == 0;
        let is_tail = i == ring_points - 1;
        let snake_char = if is_head { '@' }
            else if is_tail { '~' }
            else if frac <= cycle_progress { 'S' }
            else { '.' };

        let color = if frac <= cycle_progress {
            Vec4::new(0.2, 0.9, 0.3, 0.7) // green = approaching heal
        } else {
            Vec4::new(0.4, 0.4, 0.4, 0.3)
        };

        engine.spawn_glyph(Glyph {
            character: snake_char,
            position: Vec3::new(rx, ry, 0.0),
            color,
            emission: if frac <= cycle_progress { 0.4 } else { 0.1 },
            layer: RenderLayer::Overlay,
            ..Default::default()
        });
    }

    // Reversed damage/heal numbers
    // Damage shows as green (looks like healing but it's damage)
    if state.enemy_flash > 0.0 {
        let flash_t = state.enemy_flash;
        let reversed_color = Vec4::new(0.2, 0.9, 0.3, flash_t); // green for damage
        engine.spawn_glyph(Glyph {
            character: '+',
            position: Vec3::new(boss_x + 1.0, boss_y + 1.5, 0.0),
            color: reversed_color,
            emission: flash_t * 0.8,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }

    // Healing shows as red (looks like damage but it's healing)
    if cycle_progress > 0.9 {
        let heal_flash = ((frame as f32 * 0.3).sin() * 0.3 + 0.7).max(0.0);
        let reversed_heal_color = Vec4::new(0.9, 0.15, 0.15, heal_flash); // red for healing
        render_boss_text(engine, "HEALED", boss_x - 1.0, boss_y + 2.0,
            reversed_heal_color, heal_flash * 0.5);
    }

    // Infinity loop hint
    let inf_pulse = ((frame as f32 * 0.05).sin() * 0.2 + 0.5).max(0.0);
    engine.spawn_glyph(Glyph {
        character: '8',
        position: Vec3::new(boss_x, boss_y, 0.0),
        rotation: std::f32::consts::FRAC_PI_2,
        color: Vec4::new(0.5, 0.8, 0.5, inf_pulse),
        emission: inf_pulse * 0.3,
        layer: RenderLayer::Overlay,
        ..Default::default()
    });
}

// ── Algorithm Reborn ────────────────────────────────────────────────────────
// Phase 1: standard, Phase 2: prediction lines, Phase 3: full chaos.

fn render_algorithm_reborn(
    engine: &mut ProofEngine, state: &GameState, frame: u64, turn: u32, phase: u8,
) {
    let boss_x = 5.0;
    let boss_y = 2.0;

    match phase {
        1 => {
            // Phase 1: Standard — scanning lines
            let scan_y = -5.0 + ((frame as f32 * 0.03) % 10.0);
            for i in 0..30 {
                let x = -8.0 + i as f32 * 0.6;
                engine.spawn_glyph(Glyph {
                    character: '-',
                    position: Vec3::new(x, scan_y, -0.5),
                    color: Vec4::new(0.3, 0.8, 0.3, 0.2),
                    emission: 0.1,
                    layer: RenderLayer::Background,
                    ..Default::default()
                });
            }
            render_boss_text(engine, "SCANNING...", -2.0, -4.5,
                Vec4::new(0.3, 0.8, 0.3, 0.4), 0.3);
        }
        2 => {
            // Phase 2: Prediction lines — ghost arrows showing predicted player actions
            let predict_count = 4;
            for p in 0..predict_count {
                let seed = p as f32 * 23.7 + turn as f32 * 7.1;
                // Predicted action position
                let px = -5.0 + (seed.sin() + 1.0) * 3.0;
                let py = -1.0 + (seed.cos() + 1.0) * 2.0;

                // Ghost arrow
                let arrow_alpha = ((frame as f32 * 0.1 + p as f32 * 2.0).sin() * 0.2 + 0.4).max(0.0);
                engine.spawn_glyph(Glyph {
                    character: '>',
                    position: Vec3::new(px, py, 0.0),
                    color: Vec4::new(0.8, 0.4, 1.0, arrow_alpha),
                    emission: arrow_alpha * 0.5,
                    layer: RenderLayer::Overlay,
                    ..Default::default()
                });

                // Dotted prediction line from boss to predicted position
                let segs = 6;
                for s in 0..segs {
                    let st = s as f32 / segs as f32;
                    let lx = boss_x + (px - boss_x) * st;
                    let ly = boss_y + (py - boss_y) * st;
                    engine.spawn_glyph(Glyph {
                        character: '.',
                        position: Vec3::new(lx, ly, 0.0),
                        color: Vec4::new(0.6, 0.3, 0.9, arrow_alpha * 0.3),
                        emission: arrow_alpha * 0.2,
                        layer: RenderLayer::Overlay,
                        ..Default::default()
                    });
                }
            }
            render_boss_text(engine, "PREDICTING YOUR MOVES...", -4.0, -4.5,
                Vec4::new(0.8, 0.4, 1.0, 0.5), 0.4);
        }
        _ => {
            // Phase 3: Full chaos — all elements simultaneously, screen distortion
            let elements_colors = [
                Vec4::new(1.0, 0.3, 0.05, 1.0),  // fire
                Vec4::new(0.2, 0.6, 1.0, 1.0),    // ice
                Vec4::new(1.0, 1.0, 0.3, 1.0),    // lightning
                Vec4::new(0.1, 0.8, 0.2, 1.0),    // poison
                Vec4::new(0.3, 0.05, 0.5, 1.0),   // shadow
                Vec4::new(1.0, 0.95, 0.5, 1.0),   // holy
                Vec4::new(0.4, 0.2, 1.0, 1.0),    // arcane
                Vec4::new(1.0, 0.0, 0.5, 1.0),    // chaos
            ];

            // All elements simultaneously
            for (e, color) in elements_colors.iter().enumerate() {
                let angle = (e as f32 / 8.0) * std::f32::consts::TAU + frame as f32 * 0.1;
                let r = 3.0 + ((frame as f32 * 0.05 + e as f32).sin()) * 0.5;
                let ex = boss_x + angle.cos() * r;
                let ey = boss_y + angle.sin() * r * 0.5;
                engine.spawn_glyph(Glyph {
                    character: '*',
                    position: Vec3::new(ex, ey, 0.0),
                    color: Vec4::new(color.x, color.y, color.z, 0.6),
                    emission: 0.8,
                    glow_color: Vec3::new(color.x, color.y, color.z),
                    glow_radius: 1.5,
                    layer: RenderLayer::Particle,
                    blend_mode: BlendMode::Additive,
                    ..Default::default()
                });
            }

            // Granular visual glitches
            let glitch_chars = ['█', '▓', '▒', '░', '#', '!', '?', '@'];
            for i in 0..20 {
                let seed = i as f32 * 67.3 + frame as f32 * 0.9;
                let gx = seed.sin() * 8.0;
                let gy = seed.cos() * 5.0;
                engine.spawn_glyph(Glyph {
                    character: glitch_chars[(frame as usize + i) % glitch_chars.len()],
                    position: Vec3::new(gx, gy, 0.5),
                    color: Vec4::new(
                        (seed * 0.3).sin().abs(),
                        (seed * 0.7).cos().abs() * 0.3,
                        (seed * 1.1).sin().abs(),
                        0.25,
                    ),
                    emission: 0.3,
                    layer: RenderLayer::Overlay,
                    ..Default::default()
                });
            }

            // "I SEE YOU" text
            let pulse = ((frame as f32 * 0.15).sin() * 0.4 + 0.6).max(0.0);
            render_boss_text_centered(engine, "I  S E E  Y O U", 4.5,
                Vec4::new(1.0, 0.2 * pulse, 0.8 * pulse, pulse),
                pulse * 1.2);

            // Screen distortion — subtle warp hint
            engine.add_trauma(0.02);
        }
    }
}

// ── ChaosWeaver ─────────────────────────────────────────────────────────────
// Random visual glitches, element chart scramble, ability icon shuffle.

fn render_chaos_weaver(engine: &mut ProofEngine, state: &GameState, frame: u64, scramble_seed: u64) {
    let boss_x = 5.0;
    let boss_y = 2.0;

    // Random visual glitches across screen
    for i in 0..15 {
        let seed = (scramble_seed.wrapping_mul(7919).wrapping_add(i * 31)) as f32;
        let gx = (seed * 0.001).sin() * 8.0;
        let gy = (seed * 0.0013).cos() * 5.0;

        // Glitch only appears briefly
        if (frame + i) % 7 < 2 {
            let glitch_chars = ['#', '@', '!', '?', '%', '&', '*', '~'];
            engine.spawn_glyph(Glyph {
                character: glitch_chars[(seed as usize) % glitch_chars.len()],
                position: Vec3::new(gx, gy, 0.5),
                color: Vec4::new(
                    (seed * 0.17).sin().abs(),
                    (seed * 0.31).cos().abs(),
                    (seed * 0.47).sin().abs(),
                    0.4,
                ),
                emission: 0.3,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }
    }

    // Scrambled element chart display
    let element_labels = ["FIRE", "ICE", "LGHT", "POIS", "SHDW", "HOLY", "ARCN", "CAOS"];
    let element_colors = [
        Vec4::new(1.0, 0.3, 0.0, 0.5),
        Vec4::new(0.2, 0.6, 1.0, 0.5),
        Vec4::new(1.0, 1.0, 0.3, 0.5),
        Vec4::new(0.1, 0.8, 0.2, 0.5),
        Vec4::new(0.3, 0.0, 0.5, 0.5),
        Vec4::new(1.0, 0.9, 0.3, 0.5),
        Vec4::new(0.4, 0.2, 1.0, 0.5),
        Vec4::new(1.0, 0.0, 0.5, 0.5),
    ];

    for (i, label) in element_labels.iter().enumerate() {
        // Scramble which color goes with which label
        let scrambled_idx = ((i as u64).wrapping_add(scramble_seed / 15)) as usize % element_colors.len();
        let color = element_colors[scrambled_idx];
        let x = -7.5 + (i % 4) as f32 * 4.0;
        let y = -3.5 - (i / 4) as f32 * 0.5;
        render_boss_text(engine, label, x, y, color, 0.2);
    }

    // Ability icons that shuffle positions
    let ability_chars = ['A', 'B', 'C', 'D'];
    for (i, ch) in ability_chars.iter().enumerate() {
        let shuffled_pos = ((i as u64).wrapping_add(scramble_seed / 20)) as usize % 4;
        let ax = -6.0 + shuffled_pos as f32 * 3.0;
        let ay = -5.0;
        engine.spawn_glyph(Glyph {
            character: *ch,
            position: Vec3::new(ax, ay, 0.0),
            color: Vec4::new(0.8, 0.8, 0.8, 0.5),
            emission: 0.3,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }

    // Boss aura — chaotic color cycling
    let aura_count = 8;
    for i in 0..aura_count {
        let angle = (i as f32 / aura_count as f32) * std::f32::consts::TAU + frame as f32 * 0.15;
        let r = 1.5;
        let color_idx = (frame as usize / 4 + i) % element_colors.len();
        engine.spawn_glyph(Glyph {
            character: '.',
            position: Vec3::new(boss_x + angle.cos() * r, boss_y + angle.sin() * r, 0.0),
            color: element_colors[color_idx],
            emission: 0.4,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

// ── VoidSerpent ─────────────────────────────────────────────────────────────
// Growing void border, serpent emerge animation, void projectiles.

fn render_void_serpent(engine: &mut ProofEngine, state: &GameState, frame: u64, void_border: f32) {
    // Growing void border around screen edges
    let border_intensity = void_border.min(2.0);
    if border_intensity > 0.1 {
        let steps = 60;
        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let perimeter = t * 4.0; // 0-4 maps to 4 edges
            let (bx, by) = if perimeter < 1.0 {
                (-8.7 + perimeter * 17.4, 5.4) // top
            } else if perimeter < 2.0 {
                (8.7, 5.4 - (perimeter - 1.0) * 10.8) // right
            } else if perimeter < 3.0 {
                (8.7 - (perimeter - 2.0) * 17.4, -5.4) // bottom
            } else {
                (-8.7, -5.4 + (perimeter - 3.0) * 10.8) // left
            };
            let depth = border_intensity * 0.3;
            let pulse = ((frame as f32 * 0.05 + i as f32 * 0.3).sin() * 0.2 + 0.6).max(0.0);
            engine.spawn_glyph(Glyph {
                character: '░',
                position: Vec3::new(bx, by, 0.8),
                color: Vec4::new(0.05, 0.0, 0.1, pulse * depth),
                emission: 0.0,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }
    }

    // Serpent body — sinusoidal glyph snake
    let serpent_length = 15;
    let serpent_speed = frame as f32 * 0.08;
    for i in 0..serpent_length {
        let st = i as f32 / serpent_length as f32;
        let sx = 3.0 + (serpent_speed + st * 6.0).sin() * 3.0;
        let sy = 1.0 + (serpent_speed * 0.7 + st * 4.0).cos() * 2.0;
        let body_fade = 1.0 - st * 0.5;
        let serpent_char = if i == 0 { '@' }
            else if i == serpent_length - 1 { '~' }
            else { 'S' };

        engine.spawn_glyph(Glyph {
            character: serpent_char,
            position: Vec3::new(sx, sy, 0.0),
            color: Vec4::new(0.2, 0.05, 0.3, body_fade * 0.7),
            emission: body_fade * 0.3,
            glow_color: Vec3::new(0.3, 0.0, 0.5),
            glow_radius: body_fade * 0.8,
            layer: RenderLayer::Entity,
            ..Default::default()
        });
    }

    // Void projectiles — black holes moving toward player
    let proj_count = (state.boss_turn / 2) as usize;
    let proj_count = proj_count.min(4);
    for p in 0..proj_count {
        let proj_t = ((frame as f32 * 0.02 + p as f32 * 0.7) % 1.0);
        let px = 5.0 + ((-5.0) - 5.0) * proj_t; // move from boss to player area
        let py = 1.0 + ((proj_t * std::f32::consts::PI + p as f32).sin()) * 1.5;

        // Black hole core
        engine.spawn_glyph(Glyph {
            character: '●',
            position: Vec3::new(px, py, 0.1),
            scale: Vec2::splat(0.8),
            color: Vec4::new(0.0, 0.0, 0.0, 0.9),
            emission: 0.0,
            layer: RenderLayer::Particle,
            ..Default::default()
        });

        // Accretion ring
        for r in 0..6 {
            let ring_angle = (r as f32 / 6.0) * std::f32::consts::TAU + frame as f32 * 0.2;
            let rr = 0.4;
            engine.spawn_glyph(Glyph {
                character: '.',
                position: Vec3::new(px + ring_angle.cos() * rr, py + ring_angle.sin() * rr, 0.0),
                color: Vec4::new(0.4, 0.1, 0.6, 0.5),
                emission: 0.3,
                layer: RenderLayer::Particle,
                ..Default::default()
            });
        }
    }
}

// ── PrimeFactorial ──────────────────────────────────────────────────────────
// Number glyphs that factor, math formula display, prime check visual.

fn render_prime_factorial(engine: &mut ProofEngine, state: &GameState, frame: u64, turn: u32) {
    let boss_x = 5.0;
    let boss_y = 2.0;

    // Current HP as number display
    let hp = state.enemy.as_ref().map(|e| e.hp).unwrap_or(100);
    let hp_str = format!("{}", hp);

    // Large HP number display above boss
    render_boss_text(engine, &hp_str, boss_x - hp_str.len() as f32 * 0.2, boss_y + 2.0,
        Vec4::new(0.8, 0.8, 1.0, 0.7), 0.6);

    // Factor the number visually — show prime factors splitting
    let mut n = hp.max(1) as u64;
    let mut factors: Vec<u64> = Vec::new();
    let mut d: u64 = 2;
    let mut temp = n;
    while d * d <= temp && factors.len() < 8 {
        while temp % d == 0 {
            factors.push(d);
            temp /= d;
        }
        d += 1;
    }
    if temp > 1 {
        factors.push(temp);
    }

    // Display factors splitting outward from number
    let factor_count = factors.len();
    for (i, factor) in factors.iter().enumerate() {
        let angle = if factor_count > 1 {
            (i as f32 / factor_count as f32) * std::f32::consts::PI - std::f32::consts::FRAC_PI_2
        } else {
            0.0
        };
        let split_r = 1.5 + ((frame as f32 * 0.05).sin()) * 0.3;
        let fx = boss_x + angle.cos() * split_r;
        let fy = boss_y + 0.5 + angle.sin() * split_r;
        let factor_str = format!("{}", factor);
        let is_prime = is_prime_number(*factor);

        let color = if is_prime {
            Vec4::new(1.0, 0.8, 0.2, 0.7) // gold for primes
        } else {
            Vec4::new(0.5, 0.5, 0.8, 0.5) // blue-gray for composites
        };

        render_boss_text(engine, &factor_str, fx, fy, color, 0.4);

        // Multiplication signs between factors
        if i < factor_count - 1 {
            let next_angle = ((i + 1) as f32 / factor_count as f32) * std::f32::consts::PI - std::f32::consts::FRAC_PI_2;
            let mid_angle = (angle + next_angle) / 2.0;
            let mid_r = split_r * 0.7;
            engine.spawn_glyph(Glyph {
                character: 'x',
                position: Vec3::new(
                    boss_x + mid_angle.cos() * mid_r,
                    boss_y + 0.5 + mid_angle.sin() * mid_r,
                    0.0,
                ),
                color: Vec4::new(0.6, 0.6, 0.6, 0.4),
                emission: 0.2,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }
    }

    // Prime check visual — golden glow if HP is prime
    if is_prime_number(hp as u64) {
        let prime_pulse = ((frame as f32 * 0.15).sin() * 0.3 + 0.7).max(0.0);
        engine.spawn_glyph(Glyph {
            character: 'P',
            position: Vec3::new(boss_x + 1.5, boss_y + 2.0, 0.0),
            color: Vec4::new(1.0, 0.85, 0.2, prime_pulse),
            emission: prime_pulse * 0.8,
            glow_color: Vec3::new(1.0, 0.85, 0.2),
            glow_radius: prime_pulse * 1.5,
            layer: RenderLayer::UI,
            ..Default::default()
        });

        // Protective golden aura when prime
        for i in 0..10 {
            let angle = (i as f32 / 10.0) * std::f32::consts::TAU + frame as f32 * 0.05;
            let r = 2.0;
            engine.spawn_glyph(Glyph {
                character: '*',
                position: Vec3::new(boss_x + angle.cos() * r, boss_y + angle.sin() * r, 0.0),
                color: Vec4::new(1.0, 0.85, 0.2, prime_pulse * 0.3),
                emission: prime_pulse * 0.4,
                layer: RenderLayer::Particle,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }
    }

    // Math formula display in background
    let formulas = ["n! = ", "p*q = ", "gcd = ", "mod = "];
    let formula_idx = (frame / 120) as usize % formulas.len();
    let formula = formulas[formula_idx];
    let formula_alpha = ((frame as f32 * 0.02).sin() * 0.15 + 0.2).max(0.0);
    render_boss_text(engine, formula, boss_x - 3.0, boss_y - 2.0,
        Vec4::new(0.5, 0.5, 0.7, formula_alpha), formula_alpha * 0.3);

    // Number glyphs floating in background
    for i in 0..8 {
        let seed = i as f32 * 13.7 + frame as f32 * 0.02;
        let nx = boss_x + seed.sin() * 4.0;
        let ny = boss_y + seed.cos() * 3.0 - 1.0;
        let digit = char::from_digit(((frame / 10 + i) % 10) as u32, 10).unwrap_or('0');
        engine.spawn_glyph(Glyph {
            character: digit,
            position: Vec3::new(nx, ny, -0.5),
            color: Vec4::new(0.4, 0.4, 0.6, 0.2),
            emission: 0.1,
            layer: RenderLayer::Background,
            ..Default::default()
        });
    }
}

// ── Utility functions ───────────────────────────────────────────────────────

fn render_boss_text(
    engine: &mut ProofEngine, text: &str, x: f32, y: f32, color: Vec4, emission: f32,
) {
    crate::magic::render_magic_text(engine, text, x, y, color, emission, RenderLayer::UI);
}

fn render_boss_text_centered(
    engine: &mut ProofEngine, text: &str, y: f32, color: Vec4, emission: f32,
) {
    let x = -(text.len() as f32 * 0.175);
    render_boss_text(engine, text, x, y, color, emission);
}

/// Simple primality check.
fn is_prime_number(n: u64) -> bool {
    if n < 2 { return false; }
    if n < 4 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5u64;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 { return false; }
        i += 6;
    }
    true
}
