//! Soft body glyph clusters — physically responsive entity formations.
//!
//! Wires proof-engine's mass-spring soft body system to entity glyph
//! formations so that entities compress, wobble, recoil, dissolve, and
//! generally behave like squishy mathematical objects.
//!
//! # Game event integration
//!
//! Each combat event maps to a concrete physical response:
//! - **Hit recoil** — impulse pushes glyphs away from the attack vector
//! - **Defend brace** — springs stiffen 3×, entity becomes visually rigid
//! - **Death dissolution** — all springs break, glyphs become free particles
//! - **Low HP wobble** — Perlin noise displaces target offsets
//! - **Crit impact** — one random spring permanently breaks
//! - **Spell channel** — glyphs orbit the center with tangential velocity

use glam::{Vec2, Vec3, Vec4};
use proof_engine::glyph::{GlyphId, Glyph, RenderLayer, BlendMode};
use proof_engine::math::{ForceField, Falloff, AttractorType};
use proof_engine::physics::soft_body::{SoftBody, SoftNode, Spring, SpringKind};
use std::f32::consts::{PI, TAU};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Constants
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Default spring stiffness for entity formations.
const BASE_STIFFNESS: f32 = 400.0;
/// Default damping coefficient.
const BASE_DAMPING: f32 = 0.92;
/// Maximum impulse magnitude per hit (prevents explosion).
const MAX_HIT_IMPULSE: f32 = 8.0;
/// HP threshold below which wobble kicks in (fraction 0-1).
const LOW_HP_THRESHOLD: f32 = 0.30;
/// Noise amplitude scale for low-HP wobble.
const WOBBLE_AMPLITUDE: f32 = 0.5;
/// Stiffness multiplier when defending.
const DEFEND_STIFFNESS_MULT: f32 = 3.0;
/// Tangential velocity for spell channeling (radians/sec equiv).
const CHANNEL_ORBIT_SPEED: f32 = 2.5;
/// Random outward impulse on death dissolution.
const DEATH_BURST_SPEED: f32 = 4.0;
/// Gravity applied to free particles after death.
const DEATH_GRAVITY: Vec2 = Vec2::new(0.0, -6.0);
/// Speed at which springs restore after defend ends.
const BRACE_RESTORE_RATE: f32 = 5.0;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SoftGlyph — a single glyph in the soft entity
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// One glyph within a soft entity's formation.
#[derive(Debug, Clone)]
pub struct SoftGlyph {
    /// Handle to the engine glyph (for rendering).
    pub glyph_id: GlyphId,
    /// Current 2D position (screen-space relative to entity center).
    pub position: Vec2,
    /// Current velocity.
    pub velocity: Vec2,
    /// Where this glyph WANTS to be relative to center-of-mass.
    pub target_offset: Vec2,
    /// Original target offset (before any wobble noise).
    pub base_offset: Vec2,
    /// Mass (affects how much impulses move this glyph).
    pub mass: f32,
    /// Inverse mass (0 = pinned).
    pub inv_mass: f32,
    /// The character this glyph renders.
    pub character: char,
    /// Base color of this glyph.
    pub color: Vec4,
    /// Whether this glyph has been freed (post-death dissolution).
    pub freed: bool,
    /// Per-glyph emission intensity.
    pub emission: f32,
}

impl SoftGlyph {
    pub fn new(glyph_id: GlyphId, character: char, offset: Vec2, color: Vec4, mass: f32) -> Self {
        Self {
            glyph_id,
            position: offset,
            velocity: Vec2::ZERO,
            target_offset: offset,
            base_offset: offset,
            mass,
            inv_mass: if mass > 0.0 { 1.0 / mass } else { 0.0 },
            character,
            color,
            freed: false,
            emission: 0.0,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// EntitySpring — connects two glyphs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A spring connecting two glyphs in the soft entity.
#[derive(Debug, Clone)]
pub struct EntitySpring {
    /// Index of glyph A in the entity's glyph array.
    pub glyph_a: usize,
    /// Index of glyph B.
    pub glyph_b: usize,
    /// Natural rest length of this spring.
    pub rest_length: f32,
    /// Current stiffness (may be modified by defend/brace).
    pub stiffness: f32,
    /// Base stiffness (before any multipliers).
    pub base_stiffness: f32,
    /// Damping coefficient along the spring axis.
    pub damping: f32,
    /// Whether this spring has been permanently broken (crit impact).
    pub broken: bool,
}

impl EntitySpring {
    pub fn new(glyph_a: usize, glyph_b: usize, rest_length: f32, stiffness: f32) -> Self {
        Self {
            glyph_a,
            glyph_b,
            rest_length,
            stiffness,
            base_stiffness: stiffness,
            damping: 8.0,
            broken: false,
        }
    }

    /// Compute the spring force on glyph A from this connection.
    fn force_on_a(&self, pa: Vec2, pb: Vec2, va: Vec2, vb: Vec2) -> Vec2 {
        if self.broken {
            return Vec2::ZERO;
        }
        let delta = pb - pa;
        let dist = delta.length();
        if dist < 1e-6 {
            return Vec2::ZERO;
        }
        let dir = delta / dist;
        let stretch = dist - self.rest_length;
        let spring_force = self.stiffness * stretch;
        let rel_vel = (vb - va).dot(dir);
        let damping_force = self.damping * rel_vel;
        dir * (spring_force + damping_force)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SoftEntityState — tracks active effects
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Active modifier state on a soft entity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityEffect {
    /// No special effect active.
    None,
    /// Defending — springs are stiffened.
    Bracing,
    /// Channeling a spell — glyphs orbit.
    Channeling,
    /// Dissolving — springs broken, particles free.
    Dissolving,
}

/// Tracks which death attractor (if any) to apply to freed glyphs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeathAttractor {
    None,
    Lorenz,
    Rossler,
    Aizawa,
    Thomas,
    Scatter,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SoftEntity — the main struct
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A soft-body entity whose glyphs are connected by springs and respond
/// physically to combat events.
#[derive(Debug, Clone)]
pub struct SoftEntity {
    /// All glyphs composing this entity.
    pub glyphs: Vec<SoftGlyph>,
    /// Springs connecting glyphs.
    pub springs: Vec<EntitySpring>,
    /// World-space center of this entity.
    pub center: Vec2,
    /// Computed center of mass (updated each tick).
    pub center_of_mass: Vec2,
    /// Binding strength (proportional to HP/maxHP, 0.0-1.0).
    pub binding_strength: f32,
    /// Global damping (velocity decay per frame).
    pub damping: f32,
    /// Active effect modifier.
    pub active_effect: EntityEffect,
    /// Timer for brace restoration.
    pub brace_timer: f32,
    /// Timer for channeling animation.
    pub channel_timer: f32,
    /// Death attractor type.
    pub death_attractor: DeathAttractor,
    /// Dissolution timer (seconds since death).
    pub dissolve_timer: f32,
    /// Current HP fraction (0.0-1.0) for wobble calculation.
    pub hp_fraction: f32,
    /// Accumulated time for noise sampling.
    pub time: f32,
    /// Simple RNG state for deterministic randomness.
    rng_state: u32,
}

impl SoftEntity {
    // ════════════════════════════════════════════════════════════════════════
    // Construction
    // ════════════════════════════════════════════════════════════════════════

    /// Create a new soft entity at `center` from a set of glyph offsets,
    /// characters, and colors.
    pub fn new(
        center: Vec2,
        offsets: &[Vec2],
        characters: &[char],
        colors: &[Vec4],
        glyph_ids: &[GlyphId],
    ) -> Self {
        let n = offsets.len();
        let mass_per = 1.0 / n.max(1) as f32;

        let mut glyphs: Vec<SoftGlyph> = (0..n)
            .map(|i| {
                SoftGlyph::new(
                    glyph_ids.get(i).copied().unwrap_or(GlyphId(i as u32)),
                    characters.get(i).copied().unwrap_or('◆'),
                    offsets[i],
                    colors.get(i).copied().unwrap_or(Vec4::ONE),
                    mass_per,
                )
            })
            .collect();

        // Build springs: connect each glyph to its nearest neighbors
        let mut springs = Vec::new();
        let max_spring_dist = Self::compute_max_spring_distance(offsets);

        for i in 0..n {
            for j in (i + 1)..n {
                let dist = (offsets[i] - offsets[j]).length();
                if dist <= max_spring_dist {
                    springs.push(EntitySpring::new(i, j, dist, BASE_STIFFNESS));
                }
            }
        }

        // If no springs were created (very sparse formation), connect everything
        // to the nearest neighbor at minimum
        if springs.is_empty() && n > 1 {
            for i in 0..n {
                let mut best_j = 0;
                let mut best_d = f32::MAX;
                for j in 0..n {
                    if i == j { continue; }
                    let d = (offsets[i] - offsets[j]).length();
                    if d < best_d {
                        best_d = d;
                        best_j = j;
                    }
                }
                if i < best_j {
                    springs.push(EntitySpring::new(i, best_j, best_d, BASE_STIFFNESS));
                }
            }
        }

        Self {
            glyphs,
            springs,
            center,
            center_of_mass: center,
            binding_strength: 1.0,
            damping: BASE_DAMPING,
            active_effect: EntityEffect::None,
            brace_timer: 0.0,
            channel_timer: 0.0,
            death_attractor: DeathAttractor::None,
            dissolve_timer: 0.0,
            hp_fraction: 1.0,
            time: 0.0,
            rng_state: 42,
        }
    }

    /// Build from a proof-engine formation shape.
    pub fn from_formation(
        center: Vec2,
        shape: &crate::entities::formations::FormationShape,
        count: usize,
        scale: f32,
        characters: &[char],
        colors: &[Vec4],
        glyph_ids: &[GlyphId],
    ) -> Self {
        let positions_3d = shape.generate_positions(count, scale);
        let offsets: Vec<Vec2> = positions_3d.iter().map(|p| Vec2::new(p.x, p.y)).collect();
        Self::new(center, &offsets, characters, colors, glyph_ids)
    }

    /// Compute a reasonable max spring distance from the formation offsets.
    /// Uses 1.8× the median nearest-neighbor distance.
    fn compute_max_spring_distance(offsets: &[Vec2]) -> f32 {
        if offsets.len() < 2 {
            return f32::MAX;
        }
        let mut nn_dists: Vec<f32> = offsets
            .iter()
            .map(|a| {
                offsets
                    .iter()
                    .filter(|b| *b != a)
                    .map(|b| (*a - *b).length())
                    .fold(f32::MAX, f32::min)
            })
            .collect();
        nn_dists.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = nn_dists[nn_dists.len() / 2];
        median * 1.8
    }

    // ════════════════════════════════════════════════════════════════════════
    // Physics tick
    // ════════════════════════════════════════════════════════════════════════

    /// Advance the soft entity physics by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        self.time += dt;

        match self.active_effect {
            EntityEffect::Dissolving => self.tick_dissolve(dt),
            EntityEffect::Channeling => {
                self.channel_timer += dt;
                self.tick_channel(dt);
                self.tick_springs(dt);
                self.tick_integration(dt);
            }
            EntityEffect::Bracing => {
                self.brace_timer += dt;
                self.tick_springs(dt);
                self.tick_integration(dt);
            }
            EntityEffect::None => {
                self.tick_wobble(dt);
                self.tick_springs(dt);
                self.tick_integration(dt);
            }
        }

        self.update_center_of_mass();
    }

    /// Apply spring forces.
    fn tick_springs(&mut self, dt: f32) {
        // Collect forces from springs
        let n = self.glyphs.len();
        let mut forces = vec![Vec2::ZERO; n];

        for spring in &self.springs {
            if spring.broken { continue; }
            let pa = self.glyphs[spring.glyph_a].position;
            let pb = self.glyphs[spring.glyph_b].position;
            let va = self.glyphs[spring.glyph_a].velocity;
            let vb = self.glyphs[spring.glyph_b].velocity;
            let f = spring.force_on_a(pa, pb, va, vb);
            forces[spring.glyph_a] += f;
            forces[spring.glyph_b] -= f;
        }

        // Return-to-target force (acts like an anchor spring to formation position)
        let anchor_k = BASE_STIFFNESS * 0.3 * self.binding_strength;
        for (i, g) in self.glyphs.iter().enumerate() {
            if g.freed { continue; }
            let target = self.center + g.target_offset;
            let to_target = target - g.position;
            forces[i] += to_target * anchor_k;
        }

        // Apply forces
        for (i, g) in self.glyphs.iter_mut().enumerate() {
            if g.freed { continue; }
            g.velocity += forces[i] * g.inv_mass * dt;
        }
    }

    /// Integration: apply velocity, damping.
    fn tick_integration(&mut self, dt: f32) {
        for g in &mut self.glyphs {
            if g.freed { continue; }
            g.velocity *= self.damping;
            g.position += g.velocity * dt;
        }
    }

    /// Low-HP wobble: add Perlin-like noise to target offsets.
    fn tick_wobble(&mut self, _dt: f32) {
        if self.hp_fraction >= LOW_HP_THRESHOLD {
            // Restore base offsets
            for g in &mut self.glyphs {
                g.target_offset = g.base_offset;
            }
            return;
        }

        let wobble_intensity = (1.0 - self.hp_fraction / LOW_HP_THRESHOLD) * WOBBLE_AMPLITUDE;
        let t = self.time;

        for (i, g) in self.glyphs.iter_mut().enumerate() {
            let seed = i as f32 * 1.618;
            let nx = simple_noise(t * 3.0 + seed) * wobble_intensity;
            let ny = simple_noise(t * 2.7 + seed + 100.0) * wobble_intensity;
            g.target_offset = g.base_offset + Vec2::new(nx, ny);
        }
    }

    /// Spell channel: add tangential velocity so glyphs orbit.
    fn tick_channel(&mut self, dt: f32) {
        let ct = self.channel_timer;
        for g in &mut self.glyphs {
            if g.freed { continue; }
            let to_center = self.center - g.position;
            let dist = to_center.length();
            if dist < 0.01 { continue; }

            // Tangential direction (perpendicular to radial)
            let radial = to_center / dist;
            let tangent = Vec2::new(-radial.y, radial.x);

            // Orbit speed scales with distance from center
            let orbit_mag = CHANNEL_ORBIT_SPEED * (0.5 + dist * 0.5);
            g.velocity += tangent * orbit_mag * dt;

            // Gentle emission pulse
            g.emission = 0.3 + 0.3 * (ct * 4.0 + dist * 2.0).sin();
        }
    }

    /// Death dissolution: free particles, apply attractor.
    fn tick_dissolve(&mut self, dt: f32) {
        self.dissolve_timer += dt;

        for g in &mut self.glyphs {
            if !g.freed { continue; }

            // Apply attractor force
            let attractor_force = match self.death_attractor {
                DeathAttractor::Lorenz => lorenz_force(g.position, self.dissolve_timer),
                DeathAttractor::Rossler => rossler_force(g.position, self.dissolve_timer),
                DeathAttractor::Aizawa => aizawa_force(g.position, self.dissolve_timer),
                DeathAttractor::Thomas => thomas_force(g.position, self.dissolve_timer),
                DeathAttractor::Scatter | DeathAttractor::None => Vec2::ZERO,
            };

            g.velocity += (attractor_force + DEATH_GRAVITY) * dt;
            g.velocity *= 0.98; // light drag
            g.position += g.velocity * dt;

            // Fade out over time
            let fade = (1.0 - self.dissolve_timer / 3.0).max(0.0);
            g.color.w = fade;
            g.emission = fade * 0.5;
        }
    }

    /// Recompute center of mass from glyph positions.
    fn update_center_of_mass(&mut self) {
        let mut total_mass = 0.0_f32;
        let mut weighted_pos = Vec2::ZERO;
        for g in &self.glyphs {
            if g.freed { continue; }
            weighted_pos += g.position * g.mass;
            total_mass += g.mass;
        }
        if total_mass > 0.0 {
            self.center_of_mass = weighted_pos / total_mass;
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // Game event handlers
    // ════════════════════════════════════════════════════════════════════════

    /// Hit recoil: impulse pushes glyphs away from the attack direction.
    /// The entity compresses on the hit side and bulges on the opposite side,
    /// then springs back.
    pub fn on_hit(&mut self, attack_direction: Vec2, damage: f32) {
        let impulse_mag = (damage / 100.0).min(MAX_HIT_IMPULSE);
        let dir = if attack_direction.length_squared() > 0.001 {
            attack_direction.normalize()
        } else {
            Vec2::new(1.0, 0.0)
        };

        for g in &mut self.glyphs {
            if g.freed { continue; }

            // Glyphs on the hit side get pushed inward (compress)
            // Glyphs on the far side get pushed outward (bulge)
            let to_glyph = (g.position - self.center).normalize_or_zero();
            let facing = to_glyph.dot(dir);

            // Hit side: compress inward; far side: bulge outward
            let impulse = if facing > 0.0 {
                // Hit side — push with the attack
                dir * impulse_mag * (0.5 + facing * 0.5)
            } else {
                // Far side — push outward from center
                to_glyph * impulse_mag * 0.3
            };

            g.velocity += impulse * g.inv_mass;
        }
    }

    /// Defend brace: temporarily increase all spring stiffness by 3×.
    /// Entity becomes visually rigid and tight.
    pub fn on_defend_start(&mut self) {
        self.active_effect = EntityEffect::Bracing;
        self.brace_timer = 0.0;
        for spring in &mut self.springs {
            spring.stiffness = spring.base_stiffness * DEFEND_STIFFNESS_MULT;
        }
        // Tighten damping so it looks solid
        self.damping = 0.80;
    }

    /// End defend: restore spring stiffness.
    pub fn on_defend_end(&mut self) {
        self.active_effect = EntityEffect::None;
        for spring in &mut self.springs {
            spring.stiffness = spring.base_stiffness;
        }
        self.damping = BASE_DAMPING;
    }

    /// Death dissolution: set binding strength to 0, break all springs,
    /// free each glyph as a particle with outward impulse.
    pub fn on_death(&mut self, attractor: DeathAttractor) {
        self.active_effect = EntityEffect::Dissolving;
        self.binding_strength = 0.0;
        self.death_attractor = attractor;
        self.dissolve_timer = 0.0;

        // Break all springs
        for spring in &mut self.springs {
            spring.broken = true;
        }

        // Free all glyphs with random outward impulse
        for g in &mut self.glyphs {
            g.freed = true;
            let to_glyph = (g.position - self.center).normalize_or_zero();

            // Random angle perturbation
            let angle = self.next_rng_f32() * TAU;
            let random_dir = Vec2::new(angle.cos(), angle.sin());

            g.velocity += (to_glyph * 0.7 + random_dir * 0.3) * DEATH_BURST_SPEED;
        }
    }

    /// Update HP fraction — triggers wobble when low.
    pub fn set_hp_fraction(&mut self, fraction: f32) {
        self.hp_fraction = fraction.clamp(0.0, 1.0);
        self.binding_strength = self.hp_fraction;

        // Scale spring stiffness with HP
        if self.active_effect != EntityEffect::Bracing {
            let stiff_scale = 0.3 + 0.7 * self.hp_fraction;
            for spring in &mut self.springs {
                spring.stiffness = spring.base_stiffness * stiff_scale;
            }
        }
    }

    /// Crit impact: one random spring breaks permanently. The entity
    /// has a visible gap/loose section for the rest of the fight.
    pub fn on_crit(&mut self) {
        // Find unbroken springs
        let unbroken: Vec<usize> = self.springs
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.broken)
            .map(|(i, _)| i)
            .collect();

        if let Some(&idx) = unbroken.get(self.next_rng_usize() % unbroken.len().max(1)) {
            self.springs[idx].broken = true;
        }
    }

    /// Begin spell channeling: glyphs orbit the center.
    pub fn on_channel_start(&mut self) {
        self.active_effect = EntityEffect::Channeling;
        self.channel_timer = 0.0;
    }

    /// End spell channeling.
    pub fn on_channel_end(&mut self) {
        self.active_effect = EntityEffect::None;
        self.channel_timer = 0.0;
        // Kill orbit velocity gently
        for g in &mut self.glyphs {
            g.velocity *= 0.3;
            g.emission = 0.0;
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // Query
    // ════════════════════════════════════════════════════════════════════════

    /// Whether this entity has fully dissolved (all glyphs faded out).
    pub fn is_dissolved(&self) -> bool {
        self.active_effect == EntityEffect::Dissolving && self.dissolve_timer > 3.0
    }

    /// Get glyph world positions (for rendering or collision).
    pub fn glyph_world_positions(&self) -> Vec<Vec3> {
        self.glyphs
            .iter()
            .map(|g| Vec3::new(g.position.x, g.position.y, 0.0))
            .collect()
    }

    /// Number of unbroken springs remaining.
    pub fn intact_springs(&self) -> usize {
        self.springs.iter().filter(|s| !s.broken).count()
    }

    /// Number of living (non-freed) glyphs.
    pub fn living_glyphs(&self) -> usize {
        self.glyphs.iter().filter(|g| !g.freed).count()
    }

    // ════════════════════════════════════════════════════════════════════════
    // Render output
    // ════════════════════════════════════════════════════════════════════════

    /// Produce glyph render data for the engine. Returns (position, character,
    /// color, emission) tuples.
    pub fn render_data(&self) -> Vec<SoftGlyphRender> {
        self.glyphs
            .iter()
            .filter(|g| g.color.w > 0.01)
            .map(|g| SoftGlyphRender {
                glyph_id: g.glyph_id,
                position: Vec3::new(g.position.x, g.position.y, 0.0),
                character: g.character,
                color: g.color,
                emission: g.emission,
                freed: g.freed,
            })
            .collect()
    }

    /// Produce spring debug lines (for debug overlay).
    pub fn spring_lines(&self) -> Vec<(Vec2, Vec2, bool)> {
        self.springs
            .iter()
            .map(|s| {
                let pa = self.glyphs[s.glyph_a].position;
                let pb = self.glyphs[s.glyph_b].position;
                (pa, pb, s.broken)
            })
            .collect()
    }

    // ════════════════════════════════════════════════════════════════════════
    // Internal RNG
    // ════════════════════════════════════════════════════════════════════════

    fn next_rng(&mut self) -> u32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        self.rng_state
    }

    fn next_rng_f32(&mut self) -> f32 {
        (self.next_rng() & 0x00FF_FFFF) as f32 / 0x0100_0000 as f32
    }

    fn next_rng_usize(&mut self) -> usize {
        self.next_rng() as usize
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Render output struct
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Per-glyph rendering data emitted by `SoftEntity::render_data()`.
#[derive(Debug, Clone)]
pub struct SoftGlyphRender {
    pub glyph_id: GlyphId,
    pub position: Vec3,
    pub character: char,
    pub color: Vec4,
    pub emission: f32,
    pub freed: bool,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// SoftEntityManager — manages all active soft entities in combat
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Identifies a soft entity in the manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoftEntityId(pub u32);

/// Manages all soft entities in the current combat scene.
pub struct SoftEntityManager {
    entities: Vec<(SoftEntityId, SoftEntity)>,
    next_id: u32,
}

impl SoftEntityManager {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            next_id: 0,
        }
    }

    /// Spawn a new soft entity and return its ID.
    pub fn spawn(&mut self, entity: SoftEntity) -> SoftEntityId {
        let id = SoftEntityId(self.next_id);
        self.next_id += 1;
        self.entities.push((id, entity));
        id
    }

    /// Get a reference to a soft entity by ID.
    pub fn get(&self, id: SoftEntityId) -> Option<&SoftEntity> {
        self.entities.iter().find(|(eid, _)| *eid == id).map(|(_, e)| e)
    }

    /// Get a mutable reference to a soft entity by ID.
    pub fn get_mut(&mut self, id: SoftEntityId) -> Option<&mut SoftEntity> {
        self.entities.iter_mut().find(|(eid, _)| *eid == id).map(|(_, e)| e)
    }

    /// Tick all entities.
    pub fn tick(&mut self, dt: f32) {
        for (_, entity) in &mut self.entities {
            entity.tick(dt);
        }
        // Remove fully dissolved entities
        self.entities.retain(|(_, e)| !e.is_dissolved());
    }

    /// Despawn by ID.
    pub fn despawn(&mut self, id: SoftEntityId) {
        self.entities.retain(|(eid, _)| *eid != id);
    }

    /// Collect all render data from all entities.
    pub fn render_all(&self) -> Vec<(SoftEntityId, Vec<SoftGlyphRender>)> {
        self.entities
            .iter()
            .map(|(id, e)| (*id, e.render_data()))
            .collect()
    }

    /// Number of active entities.
    pub fn count(&self) -> usize {
        self.entities.len()
    }

    /// Clear all entities.
    pub fn clear(&mut self) {
        self.entities.clear();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Combat event bridge — translates game events to soft entity physics
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// High-level combat event that the game system dispatches.
#[derive(Debug, Clone)]
pub enum SoftEntityEvent {
    /// Entity took damage from a direction.
    Hit {
        target: SoftEntityId,
        direction: Vec2,
        damage: f32,
        is_crit: bool,
    },
    /// Entity started defending.
    DefendStart { target: SoftEntityId },
    /// Entity stopped defending.
    DefendEnd { target: SoftEntityId },
    /// Entity died. Attractor name determines dissolution visual.
    Death {
        target: SoftEntityId,
        killer_element: String,
    },
    /// Entity HP changed.
    HpChanged {
        target: SoftEntityId,
        hp_fraction: f32,
    },
    /// Entity started channeling a spell.
    ChannelStart { target: SoftEntityId },
    /// Entity finished channeling.
    ChannelEnd { target: SoftEntityId },
}

/// Process a batch of soft entity events.
pub fn process_events(manager: &mut SoftEntityManager, events: &[SoftEntityEvent]) {
    for event in events {
        match event {
            SoftEntityEvent::Hit { target, direction, damage, is_crit } => {
                if let Some(entity) = manager.get_mut(*target) {
                    entity.on_hit(*direction, *damage);
                    if *is_crit {
                        entity.on_crit();
                    }
                }
            }
            SoftEntityEvent::DefendStart { target } => {
                if let Some(entity) = manager.get_mut(*target) {
                    entity.on_defend_start();
                }
            }
            SoftEntityEvent::DefendEnd { target } => {
                if let Some(entity) = manager.get_mut(*target) {
                    entity.on_defend_end();
                }
            }
            SoftEntityEvent::Death { target, killer_element } => {
                let attractor = element_to_attractor(killer_element);
                if let Some(entity) = manager.get_mut(*target) {
                    entity.on_death(attractor);
                }
            }
            SoftEntityEvent::HpChanged { target, hp_fraction } => {
                if let Some(entity) = manager.get_mut(*target) {
                    entity.set_hp_fraction(*hp_fraction);
                }
            }
            SoftEntityEvent::ChannelStart { target } => {
                if let Some(entity) = manager.get_mut(*target) {
                    entity.on_channel_start();
                }
            }
            SoftEntityEvent::ChannelEnd { target } => {
                if let Some(entity) = manager.get_mut(*target) {
                    entity.on_channel_end();
                }
            }
        }
    }
}

/// Map an element string to a death attractor.
fn element_to_attractor(element: &str) -> DeathAttractor {
    match element.to_lowercase().as_str() {
        "chaos" | "entropy" | "void" => DeathAttractor::Lorenz,
        "fire" | "lightning" | "temporal" => DeathAttractor::Rossler,
        "ice" | "gravity" | "dark" | "shadow" => DeathAttractor::Aizawa,
        "holy" | "radiant" | "arcane" => DeathAttractor::Thomas,
        _ => DeathAttractor::Scatter,
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Attractor force functions (2D projections of 3D attractors)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Lorenz attractor force (projected to 2D).
fn lorenz_force(pos: Vec2, t: f32) -> Vec2 {
    let sigma = 10.0;
    let rho = 28.0;
    let beta = 8.0 / 3.0;
    let x = pos.x * 0.1;
    let y = pos.y * 0.1;
    let z = (t * 0.5).sin() * 5.0; // oscillating z
    let dx = sigma * (y - x);
    let dy = x * (rho - z) - y;
    Vec2::new(dx, dy) * 0.3
}

/// Rossler attractor force (projected to 2D).
fn rossler_force(pos: Vec2, t: f32) -> Vec2 {
    let a = 0.2;
    let b = 0.2;
    let c = 5.7;
    let x = pos.x * 0.1;
    let y = pos.y * 0.1;
    let z = (t * 0.3).sin() * 3.0;
    let dx = -y - z;
    let dy = x + a * y;
    Vec2::new(dx, dy) * 0.4
}

/// Aizawa attractor force (projected to 2D).
fn aizawa_force(pos: Vec2, t: f32) -> Vec2 {
    let a = 0.95;
    let b = 0.7;
    let x = pos.x * 0.1;
    let y = pos.y * 0.1;
    let z = (t * 0.4).cos() * 2.0;
    let dx = (z - b) * x - y;
    let dy = x + (z - b) * y;
    Vec2::new(dx, dy) * 0.35
}

/// Thomas attractor force (projected to 2D).
fn thomas_force(pos: Vec2, _t: f32) -> Vec2 {
    let b = 0.208186;
    let x = pos.x * 0.2;
    let y = pos.y * 0.2;
    let dx = -b * x + y.sin();
    let dy = -b * y + x.sin();
    Vec2::new(dx, dy) * 0.5
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Simple noise (deterministic, fast, no dependency)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Simple value noise in [-1, 1] range.
fn simple_noise(x: f32) -> f32 {
    let xi = x.floor() as i32;
    let xf = x - x.floor();
    let t = xf * xf * (3.0 - 2.0 * xf); // smoothstep
    let a = hash_f32(xi);
    let b = hash_f32(xi + 1);
    a + (b - a) * t
}

/// Integer hash → float in [-1, 1].
fn hash_f32(n: i32) -> f32 {
    let n = (n as u32).wrapping_mul(0x9E3779B9);
    let n = n ^ (n >> 16);
    let n = n.wrapping_mul(0x85EBCA6B);
    (n & 0x00FF_FFFF) as f32 / 0x0080_0000 as f32 - 1.0
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Tests
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entity() -> SoftEntity {
        let offsets = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.5, 0.5),
        ];
        let chars = vec!['◆', '◇', '○', '●', '★'];
        let colors = vec![Vec4::ONE; 5];
        let ids: Vec<GlyphId> = (0..5).map(|i| GlyphId(i)).collect();
        SoftEntity::new(Vec2::new(5.0, 5.0), &offsets, &chars, &colors, &ids)
    }

    #[test]
    fn test_construction() {
        let e = test_entity();
        assert_eq!(e.glyphs.len(), 5);
        assert!(e.springs.len() > 0);
        assert_eq!(e.hp_fraction, 1.0);
        assert_eq!(e.active_effect, EntityEffect::None);
    }

    #[test]
    fn test_tick_stable() {
        let mut e = test_entity();
        for _ in 0..100 {
            e.tick(1.0 / 60.0);
        }
        // Entity should remain roughly in place
        for g in &e.glyphs {
            assert!((g.position - (e.center + g.base_offset)).length() < 2.0,
                "glyph drifted too far");
        }
    }

    #[test]
    fn test_hit_recoil() {
        let mut e = test_entity();
        let before: Vec<Vec2> = e.glyphs.iter().map(|g| g.position).collect();
        e.on_hit(Vec2::new(1.0, 0.0), 50.0);
        e.tick(1.0 / 60.0);
        // At least some glyphs should have moved
        let moved = e.glyphs.iter().zip(before.iter())
            .any(|(g, b)| (g.position - *b).length() > 0.01);
        assert!(moved, "glyphs should move after hit");
    }

    #[test]
    fn test_defend_stiffens() {
        let mut e = test_entity();
        let base_stiff = e.springs[0].stiffness;
        e.on_defend_start();
        assert!((e.springs[0].stiffness - base_stiff * DEFEND_STIFFNESS_MULT).abs() < 0.1);
        e.on_defend_end();
        assert!((e.springs[0].stiffness - base_stiff).abs() < 0.1);
    }

    #[test]
    fn test_death_dissolution() {
        let mut e = test_entity();
        e.on_death(DeathAttractor::Lorenz);
        assert_eq!(e.active_effect, EntityEffect::Dissolving);
        assert!(e.springs.iter().all(|s| s.broken));
        assert!(e.glyphs.iter().all(|g| g.freed));

        // Tick until dissolved
        for _ in 0..200 {
            e.tick(1.0 / 60.0);
        }
        assert!(e.is_dissolved());
    }

    #[test]
    fn test_crit_breaks_spring() {
        let mut e = test_entity();
        let initial_intact = e.intact_springs();
        e.on_crit();
        assert_eq!(e.intact_springs(), initial_intact - 1);
    }

    #[test]
    fn test_low_hp_wobble() {
        let mut e = test_entity();
        e.set_hp_fraction(0.1);
        e.tick(0.5);
        // Target offsets should differ from base offsets
        let wobbled = e.glyphs.iter().any(|g| (g.target_offset - g.base_offset).length() > 0.01);
        assert!(wobbled, "low HP should cause wobble");
    }

    #[test]
    fn test_channel_orbit() {
        let mut e = test_entity();
        e.on_channel_start();
        for _ in 0..30 {
            e.tick(1.0 / 60.0);
        }
        // Glyphs should have nonzero velocity from orbiting
        let has_velocity = e.glyphs.iter().any(|g| g.velocity.length() > 0.01);
        assert!(has_velocity, "channeling should impart orbital velocity");
        assert!(e.glyphs.iter().any(|g| g.emission > 0.0), "channeling should emit");
        e.on_channel_end();
        assert_eq!(e.active_effect, EntityEffect::None);
    }

    #[test]
    fn test_manager_lifecycle() {
        let mut mgr = SoftEntityManager::new();
        let e = test_entity();
        let id = mgr.spawn(e);
        assert_eq!(mgr.count(), 1);
        assert!(mgr.get(id).is_some());
        mgr.despawn(id);
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_event_processing() {
        let mut mgr = SoftEntityManager::new();
        let e = test_entity();
        let id = mgr.spawn(e);

        let events = vec![
            SoftEntityEvent::Hit {
                target: id,
                direction: Vec2::X,
                damage: 30.0,
                is_crit: false,
            },
            SoftEntityEvent::HpChanged {
                target: id,
                hp_fraction: 0.7,
            },
        ];
        process_events(&mut mgr, &events);

        let entity = mgr.get(id).unwrap();
        assert!((entity.hp_fraction - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_simple_noise_range() {
        for i in 0..100 {
            let v = simple_noise(i as f32 * 0.1);
            assert!(v >= -1.0 && v <= 1.0, "noise out of range: {v}");
        }
    }
}
