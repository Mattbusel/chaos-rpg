//! Status effect visual rendering system.
//!
//! 15+ status effects with unique glyph-based visuals.
//! All rendering is immediate-mode: overlays are spawned fresh each frame.

use proof_engine::prelude::*;
use glam::{Vec3, Vec4};

// ── Status effect enum ──────────────────────────────────────────────────────

/// Every status effect that can be visually rendered on an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusEffect {
    Burning,
    Frozen,
    Shocked,
    Poisoned,
    Bleeding,
    Cursed,
    Blessed,
    Hasted,
    Slowed,
    Confused,
    Feared,
    Charmed,
    Silenced,
    Invincible,
    Regenerating,
}

impl StatusEffect {
    /// Try to match a status name string to a known effect.
    pub fn from_name(name: &str) -> Option<Self> {
        let n = name.to_lowercase();
        if n.contains("burn") || n.contains("fire") || n.contains("ignite") { return Some(Self::Burning); }
        if n.contains("frozen") || n.contains("freeze") || n.contains("ice") { return Some(Self::Frozen); }
        if n.contains("shock") || n.contains("stun") || n.contains("electr") { return Some(Self::Shocked); }
        if n.contains("poison") || n.contains("toxic") || n.contains("venom") { return Some(Self::Poisoned); }
        if n.contains("bleed") || n.contains("blood") { return Some(Self::Bleeding); }
        if n.contains("curse") || n.contains("doom") || n.contains("hex") { return Some(Self::Cursed); }
        if n.contains("bless") || n.contains("protect") { return Some(Self::Blessed); }
        if n.contains("haste") || n.contains("speed") || n.contains("quick") { return Some(Self::Hasted); }
        if n.contains("slow") || n.contains("heavy") || n.contains("anchor") { return Some(Self::Slowed); }
        if n.contains("confus") || n.contains("dizzy") { return Some(Self::Confused); }
        if n.contains("fear") || n.contains("terror") { return Some(Self::Feared); }
        if n.contains("charm") || n.contains("love") || n.contains("seduc") { return Some(Self::Charmed); }
        if n.contains("silenc") || n.contains("mute") { return Some(Self::Silenced); }
        if n.contains("invincib") || n.contains("invuln") || n.contains("immune") { return Some(Self::Invincible); }
        if n.contains("regen") || n.contains("heal over") { return Some(Self::Regenerating); }
        None
    }
}

// ── Active status on an entity ──────────────────────────────────────────────

/// A status effect applied to a specific entity, tracked for visual rendering.
#[derive(Debug, Clone)]
pub struct ActiveStatus {
    pub effect: StatusEffect,
    pub entity_pos: Vec3,
    pub timer: f32,
    pub apply_flash: f32,   // flash timer when first applied
    pub remove_flash: f32,  // flash timer when being removed
}

impl ActiveStatus {
    pub fn new(effect: StatusEffect, entity_pos: Vec3) -> Self {
        Self {
            effect,
            entity_pos,
            timer: 0.0,
            apply_flash: 0.3, // 0.3s apply flash
            remove_flash: 0.0,
        }
    }
}

// ── Status Visual Manager ───────────────────────────────────────────────────

/// Tracks all active statuses per entity and renders overlays each frame.
pub struct StatusVisualManager {
    /// Active statuses keyed by (entity index, effect).
    pub active: Vec<(usize, ActiveStatus)>,
}

impl StatusVisualManager {
    pub fn new() -> Self {
        Self { active: Vec::new() }
    }

    /// Apply a status effect to an entity. Shows apply animation.
    pub fn apply_status(&mut self, entity_id: usize, effect: StatusEffect, pos: Vec3) {
        // Remove existing same-type status on this entity
        self.active.retain(|(eid, s)| !(*eid == entity_id && s.effect == effect));
        self.active.push((entity_id, ActiveStatus::new(effect, pos)));
    }

    /// Remove a status effect from an entity. Triggers remove animation.
    pub fn remove_status(&mut self, entity_id: usize, effect: StatusEffect) {
        for (eid, status) in &mut self.active {
            if *eid == entity_id && status.effect == effect {
                status.remove_flash = 0.3;
            }
        }
    }

    /// Update entity position for an entity (call each frame).
    pub fn update_position(&mut self, entity_id: usize, pos: Vec3) {
        for (eid, status) in &mut self.active {
            if *eid == entity_id {
                status.entity_pos = pos;
            }
        }
    }

    /// Tick all status timers and remove fully faded-out statuses.
    pub fn update(&mut self, dt: f32) {
        for (_eid, status) in &mut self.active {
            status.timer += dt;
            if status.apply_flash > 0.0 {
                status.apply_flash = (status.apply_flash - dt).max(0.0);
            }
            if status.remove_flash > 0.0 {
                status.remove_flash = (status.remove_flash - dt).max(0.0);
            }
        }
        // Remove statuses that have finished their remove flash
        self.active.retain(|(_eid, s)| s.remove_flash <= 0.0 || s.apply_flash > 0.0 || s.remove_flash > 0.01);
    }

    /// Clear all statuses for an entity.
    pub fn clear_entity(&mut self, entity_id: usize) {
        self.active.retain(|(eid, _)| *eid != entity_id);
    }

    /// Render all active status visuals.
    pub fn render(&self, engine: &mut ProofEngine, frame: u64) {
        for (_eid, status) in &self.active {
            // Apply flash overlay
            if status.apply_flash > 0.0 {
                render_apply_flash(engine, status, frame);
            }
            // Remove flash overlay
            if status.remove_flash > 0.0 && status.remove_flash < 0.29 {
                render_remove_flash(engine, status, frame);
                continue; // Don't render normal visual during removal
            }
            // Normal status visual
            render_status(engine, status, frame);
        }
    }
}

// ── Apply flash ─────────────────────────────────────────────────────────────

fn render_apply_flash(engine: &mut ProofEngine, status: &ActiveStatus, _frame: u64) {
    let pos = status.entity_pos;
    let t = 1.0 - (status.apply_flash / 0.3); // 0 to 1
    let color = status_color(status.effect);
    let flash_r = 0.5 + t * 2.0;
    let flash_alpha = (1.0 - t).max(0.0);

    for i in 0..8 {
        let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
        engine.spawn_glyph(Glyph {
            character: '*',
            position: Vec3::new(pos.x + angle.cos() * flash_r, pos.y + angle.sin() * flash_r, 0.0),
            color: Vec4::new(color.x, color.y, color.z, flash_alpha * 0.6),
            emission: flash_alpha * 1.0,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
}

// ── Remove flash ────────────────────────────────────────────────────────────

fn render_remove_flash(engine: &mut ProofEngine, status: &ActiveStatus, _frame: u64) {
    let pos = status.entity_pos;
    let t = 1.0 - (status.remove_flash / 0.3);
    let color = status_color(status.effect);

    // Fading outward burst
    let r = 1.0 + t * 3.0;
    let alpha = (1.0 - t).max(0.0);
    for i in 0..6 {
        let angle = (i as f32 / 6.0) * std::f32::consts::TAU;
        engine.spawn_glyph(Glyph {
            character: '~',
            position: Vec3::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r, 0.0),
            color: Vec4::new(color.x, color.y, color.z, alpha * 0.4),
            emission: alpha * 0.3,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

// ── Individual status renderers ─────────────────────────────────────────────

fn render_status(engine: &mut ProofEngine, status: &ActiveStatus, frame: u64) {
    let pos = status.entity_pos;
    let t = status.timer;

    match status.effect {
        StatusEffect::Burning => render_burning(engine, pos, t, frame),
        StatusEffect::Frozen => render_frozen(engine, pos, t, frame),
        StatusEffect::Shocked => render_shocked(engine, pos, t, frame),
        StatusEffect::Poisoned => render_poisoned(engine, pos, t, frame),
        StatusEffect::Bleeding => render_bleeding(engine, pos, t, frame),
        StatusEffect::Cursed => render_cursed(engine, pos, t, frame),
        StatusEffect::Blessed => render_blessed(engine, pos, t, frame),
        StatusEffect::Hasted => render_hasted(engine, pos, t, frame),
        StatusEffect::Slowed => render_slowed(engine, pos, t, frame),
        StatusEffect::Confused => render_confused(engine, pos, t, frame),
        StatusEffect::Feared => render_feared(engine, pos, t, frame),
        StatusEffect::Charmed => render_charmed(engine, pos, t, frame),
        StatusEffect::Silenced => render_silenced(engine, pos, t, frame),
        StatusEffect::Invincible => render_invincible(engine, pos, t, frame),
        StatusEffect::Regenerating => render_regenerating(engine, pos, t, frame),
    }
}

// ── Burning: orange/red flicker, rising ember particles ─────────────────────

fn render_burning(engine: &mut ProofEngine, pos: Vec3, t: f32, frame: u64) {
    // Flicker tint on entity area
    let flicker = ((frame as f32 * 0.3).sin() * 0.3 + 0.7).max(0.0);
    engine.spawn_glyph(Glyph {
        character: '▒',
        position: pos,
        color: Vec4::new(1.0, 0.3 * flicker, 0.05, 0.2 * flicker),
        emission: flicker * 0.3,
        layer: RenderLayer::Overlay,
        blend_mode: BlendMode::Additive,
        ..Default::default()
    });

    // Rising ember particles
    for i in 0..4 {
        let x_off = ((frame as f32 * 0.2 + i as f32 * 3.1).sin()) * 0.8;
        let y_cycle = (t * 1.5 + i as f32 * 0.7) % 2.0;
        let y_off = y_cycle;
        let fade = (1.0 - y_cycle / 2.0).max(0.0);
        engine.spawn_glyph(Glyph {
            character: '.',
            position: Vec3::new(pos.x + x_off, pos.y + y_off + 0.5, 0.0),
            color: Vec4::new(1.0, 0.5, 0.1, fade * 0.6),
            emission: fade * 0.5,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    // Periodic damage tick flash
    if (frame % 30) < 3 {
        engine.spawn_glyph(Glyph {
            character: '!',
            position: Vec3::new(pos.x + 0.8, pos.y + 0.5, 0.0),
            color: Vec4::new(1.0, 0.2, 0.0, 0.7),
            emission: 1.0,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
}

// ── Frozen: blue tint, surrounding ice crystals, crack animation ────────────

fn render_frozen(engine: &mut ProofEngine, pos: Vec3, t: f32, frame: u64) {
    // Blue tint overlay
    engine.spawn_glyph(Glyph {
        character: '░',
        position: pos,
        color: Vec4::new(0.3, 0.6, 1.0, 0.25),
        emission: 0.1,
        layer: RenderLayer::Overlay,
        blend_mode: BlendMode::Additive,
        ..Default::default()
    });

    // Surrounding ice crystal glyphs
    let crystal_count = 6;
    for i in 0..crystal_count {
        let angle = (i as f32 / crystal_count as f32) * std::f32::consts::TAU
            + (frame as f32 * 0.02); // slow rotation
        let r = 1.0;
        let ice_chars = ['❄', '✧', '◇', '✱', '◆', '*'];
        let pulse = ((frame as f32 * 0.08 + i as f32 * 1.5).sin() * 0.2 + 0.8).max(0.0);
        engine.spawn_glyph(Glyph {
            character: ice_chars[i % ice_chars.len()],
            position: Vec3::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r, 0.0),
            color: Vec4::new(0.5, 0.85, 1.0, 0.5 * pulse),
            emission: 0.3 * pulse,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    // Crack animation when breaking free (after 3+ seconds)
    if t > 3.0 {
        let crack_progress = ((t - 3.0) * 2.0).min(1.0);
        let crack_chars = ['/', '\\', '|', '-'];
        for i in 0..4 {
            if i as f32 / 4.0 > crack_progress { break; }
            let cx = pos.x + ((i as f32 * 2.7).sin()) * 0.5;
            let cy = pos.y + ((i as f32 * 3.1).cos()) * 0.5;
            engine.spawn_glyph(Glyph {
                character: crack_chars[i % crack_chars.len()],
                position: Vec3::new(cx, cy, 0.0),
                color: Vec4::new(0.8, 0.9, 1.0, 0.6),
                emission: 0.5,
                layer: RenderLayer::Overlay,
                ..Default::default()
            });
        }
    }
}

// ── Shocked: yellow sparks randomly appearing ───────────────────────────────

fn render_shocked(engine: &mut ProofEngine, pos: Vec3, _t: f32, frame: u64) {
    // Random spark glyphs appearing briefly
    for i in 0..3 {
        if (frame + i * 7) % 5 != 0 { continue; }
        let seed = (frame as f32 * 0.7 + i as f32 * 19.3);
        let sx = pos.x + seed.sin() * 1.2;
        let sy = pos.y + seed.cos() * 0.8;
        let spark_chars = ['⚡', '*', '\'', '`', '.'];
        engine.spawn_glyph(Glyph {
            character: spark_chars[(frame as usize + i as usize) % spark_chars.len()],
            position: Vec3::new(sx, sy, 0.0),
            color: Vec4::new(1.0, 1.0, 0.3, 0.7),
            emission: 1.0,
            glow_color: Vec3::new(1.0, 1.0, 0.5),
            glow_radius: 0.8,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }

    // Brief paralysis flash (every second)
    if frame % 60 < 4 {
        engine.spawn_glyph(Glyph {
            character: '▓',
            position: pos,
            color: Vec4::new(1.0, 1.0, 0.5, 0.3),
            emission: 0.5,
            layer: RenderLayer::Overlay,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
}

// ── Poisoned: green pulse, dripping particles ───────────────────────────────

fn render_poisoned(engine: &mut ProofEngine, pos: Vec3, t: f32, frame: u64) {
    // Green pulse on entity
    let pulse = ((t * 2.0).sin() * 0.3 + 0.5).max(0.0);
    engine.spawn_glyph(Glyph {
        character: '░',
        position: pos,
        color: Vec4::new(0.1, 0.8, 0.2, pulse * 0.25),
        emission: pulse * 0.2,
        layer: RenderLayer::Overlay,
        blend_mode: BlendMode::Additive,
        ..Default::default()
    });

    // Dripping particles downward
    for i in 0..3 {
        let x_off = ((i as f32 * 2.3 + frame as f32 * 0.1).sin()) * 0.6;
        let drip_cycle = (t * 1.2 + i as f32 * 0.5) % 1.5;
        let y_off = -drip_cycle;
        let fade = (1.0 - drip_cycle / 1.5).max(0.0);
        engine.spawn_glyph(Glyph {
            character: '.',
            position: Vec3::new(pos.x + x_off, pos.y + y_off - 0.5, 0.0),
            color: Vec4::new(0.15, 0.8, 0.2, fade * 0.5),
            emission: fade * 0.2,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    // Periodic damage flash
    if (frame % 45) < 3 {
        engine.spawn_glyph(Glyph {
            character: '☠',
            position: Vec3::new(pos.x + 0.7, pos.y + 0.3, 0.0),
            scale: Vec2::splat(0.6),
            color: Vec4::new(0.2, 0.9, 0.1, 0.7),
            emission: 0.8,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

// ── Bleeding: red drops falling, blood trail ────────────────────────────────

fn render_bleeding(engine: &mut ProofEngine, pos: Vec3, t: f32, frame: u64) {
    // Red drops falling from entity
    for i in 0..4 {
        let x_off = ((i as f32 * 3.7 + frame as f32 * 0.05).sin()) * 0.5;
        let drop_cycle = (t * 1.0 + i as f32 * 0.4) % 1.2;
        let y_off = -drop_cycle * 1.5;
        let fade = (1.0 - drop_cycle / 1.2).max(0.0);
        let drop_chars = ['.',  ':', '\''];
        engine.spawn_glyph(Glyph {
            character: drop_chars[i % drop_chars.len()],
            position: Vec3::new(pos.x + x_off, pos.y + y_off - 0.3, 0.0),
            color: Vec4::new(0.8, 0.05, 0.05, fade * 0.7),
            emission: fade * 0.3,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    // Blood trail on ground (accumulated)
    let trail_count = ((t * 2.0) as usize).min(6);
    for i in 0..trail_count {
        let tx = pos.x + ((i as f32 * 7.1).sin()) * 1.5;
        let ty = pos.y - 1.5;
        engine.spawn_glyph(Glyph {
            character: '.',
            position: Vec3::new(tx, ty, -0.1),
            color: Vec4::new(0.6, 0.02, 0.02, 0.3),
            emission: 0.05,
            layer: RenderLayer::World,
            ..Default::default()
        });
    }
}

// ── Cursed: purple aura, inverted colors briefly, doom countdown ────────────

fn render_cursed(engine: &mut ProofEngine, pos: Vec3, t: f32, frame: u64) {
    // Purple aura
    let aura_pulse = ((t * 1.5).sin() * 0.2 + 0.6).max(0.0);
    for i in 0..8 {
        let angle = (i as f32 / 8.0) * std::f32::consts::TAU + t * 0.5;
        let r = 1.2 + ((t * 2.0 + i as f32).sin()) * 0.2;
        engine.spawn_glyph(Glyph {
            character: '.',
            position: Vec3::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r, 0.0),
            color: Vec4::new(0.6, 0.1, 0.8, aura_pulse * 0.4),
            emission: aura_pulse * 0.3,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    // Brief inverted color flash (every 2 seconds)
    if (frame % 120) < 6 {
        engine.spawn_glyph(Glyph {
            character: '█',
            position: pos,
            color: Vec4::new(0.5, 0.0, 0.7, 0.15),
            emission: 0.0,
            layer: RenderLayer::Overlay,
            blend_mode: BlendMode::Multiply,
            ..Default::default()
        });
    }

    // Doom countdown glyph
    let doom_num = (10.0 - t).max(0.0) as u32;
    let doom_char = char::from_digit(doom_num % 10, 10).unwrap_or('0');
    let urgency = if doom_num < 3 {
        ((frame as f32 * 0.4).sin() * 0.4 + 0.6).max(0.0)
    } else {
        0.5
    };
    engine.spawn_glyph(Glyph {
        character: doom_char,
        position: Vec3::new(pos.x, pos.y + 1.2, 0.0),
        scale: Vec2::splat(0.6),
        color: Vec4::new(0.7, 0.1, 0.9, urgency),
        emission: urgency * 0.5,
        layer: RenderLayer::UI,
        ..Default::default()
    });
}

// ── Blessed: golden glow, upward sparkle particles ──────────────────────────

fn render_blessed(engine: &mut ProofEngine, pos: Vec3, t: f32, frame: u64) {
    // Golden glow overlay
    let glow_pulse = ((t * 1.0).sin() * 0.15 + 0.5).max(0.0);
    engine.spawn_glyph(Glyph {
        character: '░',
        position: pos,
        color: Vec4::new(1.0, 0.9, 0.3, glow_pulse * 0.2),
        emission: glow_pulse * 0.4,
        glow_color: Vec3::new(1.0, 0.85, 0.3),
        glow_radius: 1.5,
        layer: RenderLayer::Overlay,
        blend_mode: BlendMode::Additive,
        ..Default::default()
    });

    // Upward sparkle particles
    for i in 0..3 {
        let x_off = ((t * 1.5 + i as f32 * 2.1).sin()) * 0.8;
        let sparkle_cycle = (t * 1.5 + i as f32 * 0.6) % 1.5;
        let y_off = sparkle_cycle;
        let fade = (1.0 - sparkle_cycle / 1.5).max(0.0);
        let sparkle_chars = ['*', '✧', '+', '.'];
        engine.spawn_glyph(Glyph {
            character: sparkle_chars[i % sparkle_chars.len()],
            position: Vec3::new(pos.x + x_off, pos.y + y_off + 0.5, 0.0),
            color: Vec4::new(1.0, 0.95, 0.5, fade * 0.5),
            emission: fade * 0.6,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }

    // Protection indicator
    engine.spawn_glyph(Glyph {
        character: '+',
        position: Vec3::new(pos.x, pos.y + 1.3, 0.0),
        scale: Vec2::splat(0.5),
        color: Vec4::new(1.0, 0.9, 0.3, 0.5),
        emission: 0.4,
        layer: RenderLayer::UI,
        ..Default::default()
    });
}

// ── Hasted: blue motion lines, afterimage trail ─────────────────────────────

fn render_hasted(engine: &mut ProofEngine, pos: Vec3, _t: f32, frame: u64) {
    // Motion lines behind entity (to the left, suggesting rightward speed)
    for i in 0..4 {
        let y_off = (i as f32 - 1.5) * 0.3;
        let len = 1.0 + i as f32 * 0.3;
        let alpha = 0.3 - i as f32 * 0.05;
        for s in 0..3 {
            let x_off = -0.8 - s as f32 * 0.4 - len * 0.3;
            let line_fade = alpha * (1.0 - s as f32 / 3.0);
            engine.spawn_glyph(Glyph {
                character: '-',
                position: Vec3::new(pos.x + x_off, pos.y + y_off, 0.0),
                color: Vec4::new(0.3, 0.6, 1.0, line_fade),
                emission: line_fade * 0.4,
                layer: RenderLayer::Particle,
                ..Default::default()
            });
        }
    }

    // Afterimage trail (ghostly copy offset behind)
    let ghost_alpha = ((frame as f32 * 0.15).sin() * 0.15 + 0.15).max(0.0);
    engine.spawn_glyph(Glyph {
        character: '▒',
        position: Vec3::new(pos.x - 0.5, pos.y, 0.0),
        color: Vec4::new(0.3, 0.5, 1.0, ghost_alpha),
        emission: ghost_alpha * 0.3,
        layer: RenderLayer::Particle,
        ..Default::default()
    });
}

// ── Slowed: gray tint, heavy visual, anchor glyph ──────────────────────────

fn render_slowed(engine: &mut ProofEngine, pos: Vec3, t: f32, _frame: u64) {
    // Gray tint overlay
    engine.spawn_glyph(Glyph {
        character: '░',
        position: pos,
        color: Vec4::new(0.4, 0.4, 0.4, 0.2),
        emission: 0.0,
        layer: RenderLayer::Overlay,
        blend_mode: BlendMode::Multiply,
        ..Default::default()
    });

    // Heavy visual — downward lines
    for i in 0..3 {
        let x_off = (i as f32 - 1.0) * 0.4;
        engine.spawn_glyph(Glyph {
            character: '|',
            position: Vec3::new(pos.x + x_off, pos.y - 0.8, 0.0),
            color: Vec4::new(0.5, 0.5, 0.5, 0.3),
            emission: 0.05,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    // Anchor glyph below entity
    let bob = ((t * 0.8).sin()) * 0.1;
    engine.spawn_glyph(Glyph {
        character: '#',
        position: Vec3::new(pos.x, pos.y - 1.2 + bob, 0.0),
        scale: Vec2::splat(0.7),
        color: Vec4::new(0.5, 0.5, 0.5, 0.5),
        emission: 0.1,
        layer: RenderLayer::Particle,
        ..Default::default()
    });
}

// ── Confused: spiral glyphs above, scrambled direction indicators ───────────

fn render_confused(engine: &mut ProofEngine, pos: Vec3, t: f32, frame: u64) {
    // Spiral glyphs above entity
    let spiral_count = 3;
    for i in 0..spiral_count {
        let angle = t * 3.0 + (i as f32 / spiral_count as f32) * std::f32::consts::TAU;
        let r = 0.6;
        let spiral_chars = ['@', '?', '*'];
        engine.spawn_glyph(Glyph {
            character: spiral_chars[i % spiral_chars.len()],
            position: Vec3::new(pos.x + angle.cos() * r, pos.y + 1.3 + angle.sin() * 0.2, 0.0),
            color: Vec4::new(0.8, 0.8, 0.2, 0.6),
            emission: 0.4,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    // Scrambled movement direction arrows
    let arrow_chars = ['^', 'v', '<', '>'];
    for i in 0..4 {
        let random_dir = (frame as usize + i * 17) % 4;
        let (dx, dy) = match i {
            0 => (0.0, 0.8),
            1 => (0.0, -0.8),
            2 => (-0.8, 0.0),
            _ => (0.8, 0.0),
        };
        engine.spawn_glyph(Glyph {
            character: arrow_chars[random_dir],
            position: Vec3::new(pos.x + dx, pos.y + dy, 0.0),
            scale: Vec2::splat(0.4),
            color: Vec4::new(0.7, 0.7, 0.2, 0.3),
            emission: 0.2,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }
}

// ── Feared: entity shrinks, shaking, retreat arrows ─────────────────────────

fn render_feared(engine: &mut ProofEngine, pos: Vec3, t: f32, frame: u64) {
    // Shaking effect — jitter overlay
    let jx = ((frame as f32 * 1.5).sin()) * 0.1;
    let jy = ((frame as f32 * 1.8).cos()) * 0.08;
    engine.spawn_glyph(Glyph {
        character: '!',
        position: Vec3::new(pos.x + jx, pos.y + 1.0 + jy, 0.0),
        scale: Vec2::splat(0.5),
        color: Vec4::new(1.0, 0.3, 0.3, 0.6),
        emission: 0.5,
        layer: RenderLayer::Particle,
        ..Default::default()
    });

    // Retreat direction arrows (pointing away from center)
    let flee_angle = std::f32::consts::PI; // default: flee leftward
    for i in 0..3 {
        let arrow_offset = i as f32 * 0.4;
        let ax = pos.x + flee_angle.cos() * (0.8 + arrow_offset);
        let ay = pos.y + flee_angle.sin() * (0.3 + arrow_offset * 0.2);
        let fade = 1.0 - i as f32 / 3.0;
        engine.spawn_glyph(Glyph {
            character: '<',
            position: Vec3::new(ax, ay, 0.0),
            scale: Vec2::splat(0.4),
            color: Vec4::new(1.0, 0.4, 0.4, fade * 0.4),
            emission: fade * 0.2,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    // Slight shrink indicator — smaller scale overlay
    engine.spawn_glyph(Glyph {
        character: '.',
        position: Vec3::new(pos.x + jx, pos.y + jy, 0.0),
        scale: Vec2::splat(0.7),
        color: Vec4::new(0.8, 0.6, 0.6, 0.15),
        emission: 0.0,
        layer: RenderLayer::Overlay,
        ..Default::default()
    });
}

// ── Charmed: pink hearts, entity faces charmer ──────────────────────────────

fn render_charmed(engine: &mut ProofEngine, pos: Vec3, t: f32, frame: u64) {
    // Pink hearts floating up
    for i in 0..3 {
        let x_off = ((t * 1.2 + i as f32 * 2.5).sin()) * 0.7;
        let heart_cycle = (t * 0.8 + i as f32 * 0.5) % 2.0;
        let y_off = heart_cycle;
        let fade = (1.0 - heart_cycle / 2.0).max(0.0);
        let heart_chars = ['<', '3', '*']; // Approximate hearts
        engine.spawn_glyph(Glyph {
            character: heart_chars[i % heart_chars.len()],
            position: Vec3::new(pos.x + x_off, pos.y + y_off + 0.5, 0.0),
            color: Vec4::new(1.0, 0.4, 0.6, fade * 0.6),
            emission: fade * 0.5,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    // Pink tint on entity
    let pulse = ((t * 1.5).sin() * 0.1 + 0.15).max(0.0);
    engine.spawn_glyph(Glyph {
        character: '░',
        position: pos,
        color: Vec4::new(1.0, 0.4, 0.6, pulse),
        emission: pulse * 0.3,
        layer: RenderLayer::Overlay,
        blend_mode: BlendMode::Additive,
        ..Default::default()
    });
}

// ── Silenced: 'X' over mouth area, muted colors ────────────────────────────

fn render_silenced(engine: &mut ProofEngine, pos: Vec3, _t: f32, _frame: u64) {
    // X over mouth area
    engine.spawn_glyph(Glyph {
        character: 'X',
        position: Vec3::new(pos.x, pos.y - 0.2, 0.1),
        scale: Vec2::splat(0.5),
        color: Vec4::new(0.8, 0.2, 0.2, 0.6),
        emission: 0.3,
        layer: RenderLayer::Overlay,
        ..Default::default()
    });

    // Muted color overlay — gray wash
    engine.spawn_glyph(Glyph {
        character: '▒',
        position: pos,
        color: Vec4::new(0.3, 0.3, 0.3, 0.2),
        emission: 0.0,
        layer: RenderLayer::Overlay,
        blend_mode: BlendMode::Multiply,
        ..Default::default()
    });

    // Small "no sound" indicator
    engine.spawn_glyph(Glyph {
        character: '~',
        position: Vec3::new(pos.x + 0.5, pos.y - 0.1, 0.0),
        scale: Vec2::splat(0.3),
        color: Vec4::new(0.5, 0.5, 0.5, 0.3),
        emission: 0.0,
        layer: RenderLayer::Overlay,
        ..Default::default()
    });
}

// ── Invincible: bright golden shield aura ───────────────────────────────────

fn render_invincible(engine: &mut ProofEngine, pos: Vec3, t: f32, frame: u64) {
    // Bright golden shield aura ring
    let shield_count = 12;
    for i in 0..shield_count {
        let angle = (i as f32 / shield_count as f32) * std::f32::consts::TAU
            + frame as f32 * 0.05;
        let r = 1.3;
        let shimmer = ((frame as f32 * 0.15 + i as f32 * 1.0).sin() * 0.2 + 0.8).max(0.0);
        engine.spawn_glyph(Glyph {
            character: '|',
            position: Vec3::new(pos.x + angle.cos() * r, pos.y + angle.sin() * r, 0.0),
            color: Vec4::new(1.0, 0.85, 0.2, shimmer * 0.6),
            emission: shimmer * 0.8,
            glow_color: Vec3::new(1.0, 0.85, 0.2),
            glow_radius: shimmer * 1.0,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }

    // Golden glow overlay
    engine.spawn_glyph(Glyph {
        character: '░',
        position: pos,
        color: Vec4::new(1.0, 0.9, 0.3, 0.15),
        emission: 0.5,
        glow_color: Vec3::new(1.0, 0.85, 0.3),
        glow_radius: 2.0,
        layer: RenderLayer::Overlay,
        blend_mode: BlendMode::Additive,
        ..Default::default()
    });

    // "0" damage indicator when hit (flash periodically to show invincibility)
    if (frame % 40) < 5 {
        engine.spawn_glyph(Glyph {
            character: '0',
            position: Vec3::new(pos.x + 1.0, pos.y + 0.8, 0.0),
            scale: Vec2::splat(0.5),
            color: Vec4::new(1.0, 0.9, 0.3, 0.5),
            emission: 0.6,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}

// ── Regenerating: green pulse, rising '+' glyphs ────────────────────────────

fn render_regenerating(engine: &mut ProofEngine, pos: Vec3, t: f32, frame: u64) {
    // Green pulse on entity
    let pulse = ((t * 2.0).sin() * 0.15 + 0.3).max(0.0);
    engine.spawn_glyph(Glyph {
        character: '░',
        position: pos,
        color: Vec4::new(0.2, 0.9, 0.3, pulse * 0.2),
        emission: pulse * 0.3,
        layer: RenderLayer::Overlay,
        blend_mode: BlendMode::Additive,
        ..Default::default()
    });

    // Rising '+' glyphs
    for i in 0..3 {
        let x_off = ((t * 1.0 + i as f32 * 2.5).sin()) * 0.6;
        let plus_cycle = (t * 1.0 + i as f32 * 0.7) % 1.5;
        let y_off = plus_cycle;
        let fade = (1.0 - plus_cycle / 1.5).max(0.0);
        engine.spawn_glyph(Glyph {
            character: '+',
            position: Vec3::new(pos.x + x_off, pos.y + y_off + 0.5, 0.0),
            color: Vec4::new(0.2, 0.9, 0.3, fade * 0.5),
            emission: fade * 0.4,
            layer: RenderLayer::Particle,
            ..Default::default()
        });
    }

    // Periodic heal tick flash
    if (frame % 50) < 3 {
        engine.spawn_glyph(Glyph {
            character: '+',
            position: Vec3::new(pos.x + 0.8, pos.y + 0.5, 0.0),
            scale: Vec2::splat(0.6),
            color: Vec4::new(0.3, 1.0, 0.4, 0.7),
            emission: 0.8,
            layer: RenderLayer::Particle,
            blend_mode: BlendMode::Additive,
            ..Default::default()
        });
    }
}

// ── Utility ─────────────────────────────────────────────────────────────────

/// Base color for a status effect (used for flash overlays).
fn status_color(effect: StatusEffect) -> Vec4 {
    match effect {
        StatusEffect::Burning      => Vec4::new(1.0, 0.4, 0.05, 1.0),
        StatusEffect::Frozen       => Vec4::new(0.3, 0.7, 1.0, 1.0),
        StatusEffect::Shocked      => Vec4::new(1.0, 1.0, 0.3, 1.0),
        StatusEffect::Poisoned     => Vec4::new(0.15, 0.8, 0.2, 1.0),
        StatusEffect::Bleeding     => Vec4::new(0.8, 0.05, 0.05, 1.0),
        StatusEffect::Cursed       => Vec4::new(0.6, 0.1, 0.8, 1.0),
        StatusEffect::Blessed      => Vec4::new(1.0, 0.9, 0.3, 1.0),
        StatusEffect::Hasted       => Vec4::new(0.3, 0.6, 1.0, 1.0),
        StatusEffect::Slowed       => Vec4::new(0.5, 0.5, 0.5, 1.0),
        StatusEffect::Confused     => Vec4::new(0.8, 0.8, 0.2, 1.0),
        StatusEffect::Feared       => Vec4::new(1.0, 0.3, 0.3, 1.0),
        StatusEffect::Charmed      => Vec4::new(1.0, 0.4, 0.6, 1.0),
        StatusEffect::Silenced     => Vec4::new(0.5, 0.5, 0.5, 1.0),
        StatusEffect::Invincible   => Vec4::new(1.0, 0.85, 0.2, 1.0),
        StatusEffect::Regenerating => Vec4::new(0.2, 0.9, 0.3, 1.0),
    }
}
