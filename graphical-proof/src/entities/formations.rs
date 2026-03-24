//! Formation library for entity rendering.
//!
//! Provides 20+ formation shapes, interpolation between formations, breathing
//! animations, rotation, spring-based cohesion, hit reactions, and
//! class-formation mapping for player entity states.

use glam::{Vec3, Vec4};
use std::f32::consts::{PI, TAU};

// ── Formation shape enum ─────────────────────────────────────────────────────

/// All available formation shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormationShape {
    Shield,
    Star,
    Crescent,
    Cross,
    Diamond,
    Ring,
    Arrow,
    Helix,
    Spiral,
    Grid,
    Vee,
    Line,
    Cluster,
    Swarm,
    Crown,
    Skull,
    Heart,
    Triangle,
    Pentagon,
    Random,
    Snake,
    Semicircle,
    DualBlades,
    Pentagram,
}

// ── Position generation ──────────────────────────────────────────────────────

impl FormationShape {
    /// Generate `count` glyph positions at the given `scale` for this shape.
    /// All positions are relative to entity center (0,0,0).
    pub fn generate_positions(&self, count: usize, scale: f32) -> Vec<Vec3> {
        if count == 0 {
            return Vec::new();
        }
        match self {
            FormationShape::Shield => gen_shield(count, scale),
            FormationShape::Star => gen_star(count, scale),
            FormationShape::Crescent => gen_crescent(count, scale),
            FormationShape::Cross => gen_cross(count, scale),
            FormationShape::Diamond => gen_diamond(count, scale),
            FormationShape::Ring => gen_ring(count, scale),
            FormationShape::Arrow => gen_arrow(count, scale),
            FormationShape::Helix => gen_helix(count, scale),
            FormationShape::Spiral => gen_spiral(count, scale),
            FormationShape::Grid => gen_grid(count, scale),
            FormationShape::Vee => gen_vee(count, scale),
            FormationShape::Line => gen_line(count, scale),
            FormationShape::Cluster => gen_cluster(count, scale),
            FormationShape::Swarm => gen_swarm(count, scale),
            FormationShape::Crown => gen_crown(count, scale),
            FormationShape::Skull => gen_skull(count, scale),
            FormationShape::Heart => gen_heart(count, scale),
            FormationShape::Triangle => gen_triangle(count, scale),
            FormationShape::Pentagon => gen_pentagon(count, scale),
            FormationShape::Random => gen_random(count, scale),
            FormationShape::Snake => gen_snake(count, scale),
            FormationShape::Semicircle => gen_semicircle(count, scale),
            FormationShape::DualBlades => gen_dual_blades(count, scale),
            FormationShape::Pentagram => gen_pentagram(count, scale),
        }
    }
}

// ── Primitive generators ─────────────────────────────────────────────────────

fn gen_shield(count: usize, scale: f32) -> Vec<Vec3> {
    // Rounded rectangle / shield shape: wider at top, tapered at bottom
    let mut pts = Vec::with_capacity(count);
    let rows = ((count as f32).sqrt() * 1.3).ceil() as i32;
    let placed = &mut 0usize;
    for row in 0..rows {
        if *placed >= count {
            break;
        }
        let t = row as f32 / rows as f32; // 0=top, 1=bottom
        // Width tapers: full at top, narrow at bottom
        let half_w = (1.0 - t * 0.6) * scale;
        let cols = ((1.0 - t * 0.5) * (count as f32 / rows as f32)).ceil() as i32;
        let cols = cols.max(1);
        for col in 0..cols {
            if *placed >= count {
                break;
            }
            let x = if cols > 1 {
                (col as f32 / (cols - 1).max(1) as f32 - 0.5) * 2.0 * half_w
            } else {
                0.0
            };
            let y = (0.5 - t) * scale * 2.0;
            pts.push(Vec3::new(x, y, 0.0));
            *placed += 1;
        }
    }
    // Pad remaining
    while pts.len() < count {
        let i = pts.len();
        let angle = i as f32 * 2.399; // golden angle
        let r = 0.3 * scale;
        pts.push(Vec3::new(angle.cos() * r, angle.sin() * r, 0.0));
    }
    pts.truncate(count);
    pts
}

fn gen_star(count: usize, scale: f32) -> Vec<Vec3> {
    let points = 5;
    let mut pts = Vec::with_capacity(count);
    // Center
    pts.push(Vec3::ZERO);
    // Outer and inner points
    let n = points * 2;
    let outer_r = scale;
    let inner_r = scale * 0.4;
    for i in 0..n {
        if pts.len() >= count {
            break;
        }
        let angle = i as f32 / n as f32 * TAU - PI / 2.0;
        let r = if i % 2 == 0 { outer_r } else { inner_r };
        pts.push(Vec3::new(angle.cos() * r, angle.sin() * r, 0.0));
    }
    // Fill remaining along star edges
    let mut idx = 0;
    while pts.len() < count {
        let a1 = idx as f32 / n as f32 * TAU - PI / 2.0;
        let a2 = (idx + 1) as f32 / n as f32 * TAU - PI / 2.0;
        let r1 = if idx % 2 == 0 { outer_r } else { inner_r };
        let r2 = if (idx + 1) % 2 == 0 { outer_r } else { inner_r };
        let t = 0.5;
        let x = (a1.cos() * r1 + a2.cos() * r2) * t;
        let y = (a1.sin() * r1 + a2.sin() * r2) * t;
        pts.push(Vec3::new(x, y, 0.0));
        idx = (idx + 1) % n;
    }
    pts.truncate(count);
    pts
}

fn gen_crescent(count: usize, scale: f32) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / count as f32;
        let angle = t * PI + PI * 0.5; // half circle
        let outer = scale;
        let inner = scale * 0.5;
        // Place on outer arc, offset from inner
        let r = outer - (inner * (1.0 - (angle - PI * 0.5).sin().abs() * 0.4));
        pts.push(Vec3::new(angle.cos() * r, angle.sin() * r * 0.8, 0.0));
    }
    pts
}

fn gen_cross(count: usize, scale: f32) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(count);
    // Center
    pts.push(Vec3::ZERO);
    let arm_len = (count as f32 / 4.0).ceil() as usize;
    let spacing = scale / arm_len.max(1) as f32;
    // Four arms
    let dirs = [
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(-1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
    ];
    for d in &dirs {
        for step in 1..=arm_len {
            if pts.len() >= count {
                break;
            }
            pts.push(*d * step as f32 * spacing);
        }
    }
    pts.truncate(count);
    pts
}

fn gen_diamond(count: usize, scale: f32) -> Vec<Vec3> {
    let size = ((count as f32).sqrt() * 0.7).ceil() as i32;
    let spacing = scale / size.max(1) as f32;
    let mut pts = Vec::new();
    for y in -size..=size {
        for x in -size..=size {
            if x.abs() + y.abs() <= size {
                pts.push(Vec3::new(x as f32 * spacing, y as f32 * spacing, 0.0));
            }
        }
    }
    // If we got more than needed, truncate; if fewer, pad
    while pts.len() < count {
        let i = pts.len();
        let a = i as f32 * 2.399;
        pts.push(Vec3::new(a.cos() * scale * 0.3, a.sin() * scale * 0.3, 0.0));
    }
    pts.truncate(count);
    pts
}

fn gen_ring(count: usize, scale: f32) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(count);
    for i in 0..count {
        let angle = i as f32 / count as f32 * TAU;
        pts.push(Vec3::new(angle.cos() * scale, angle.sin() * scale, 0.0));
    }
    pts
}

fn gen_arrow(count: usize, scale: f32) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(count);
    // Arrowhead (top third) + shaft (bottom two thirds)
    let head_count = count / 3;
    let shaft_count = count - head_count;
    // Arrowhead: V shape pointing up
    for i in 0..head_count {
        let t = i as f32 / head_count.max(1) as f32 - 0.5;
        let x = t * scale * 1.5;
        let y = scale - t.abs() * scale * 1.5;
        pts.push(Vec3::new(x, y, 0.0));
    }
    // Shaft: vertical line
    for i in 0..shaft_count {
        let t = i as f32 / shaft_count.max(1) as f32;
        let y = -t * scale * 1.5;
        pts.push(Vec3::new(0.0, y, 0.0));
    }
    pts.truncate(count);
    pts
}

fn gen_helix(count: usize, scale: f32) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(count);
    let turns = 2.0;
    for i in 0..count {
        let t = i as f32 / count as f32;
        let angle = t * turns * TAU;
        let y = (t - 0.5) * scale * 3.0;
        let r = scale * 0.5;
        // Two interleaved strands
        if i % 2 == 0 {
            pts.push(Vec3::new(angle.cos() * r, y, 0.0));
        } else {
            pts.push(Vec3::new((angle + PI).cos() * r, y, 0.0));
        }
    }
    pts
}

fn gen_spiral(count: usize, scale: f32) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(count);
    let turns = 2.5;
    for i in 0..count {
        let t = i as f32 / count as f32;
        let angle = t * turns * TAU;
        let r = t * scale;
        pts.push(Vec3::new(angle.cos() * r, angle.sin() * r, 0.0));
    }
    pts
}

fn gen_grid(count: usize, scale: f32) -> Vec<Vec3> {
    let cols = (count as f32).sqrt().ceil() as i32;
    let rows = ((count as f32) / cols as f32).ceil() as i32;
    let spacing = scale * 2.0 / cols.max(1) as f32;
    let mut pts = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            if pts.len() >= count {
                break;
            }
            let x = (col as f32 - (cols - 1) as f32 * 0.5) * spacing;
            let y = (row as f32 - (rows - 1) as f32 * 0.5) * spacing;
            pts.push(Vec3::new(x, y, 0.0));
        }
    }
    pts.truncate(count);
    pts
}

fn gen_vee(count: usize, scale: f32) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(count);
    let half = count / 2;
    for i in 0..=half {
        if pts.len() >= count {
            break;
        }
        let t = i as f32 / half.max(1) as f32;
        // Left arm
        pts.push(Vec3::new(-t * scale, -t * scale, 0.0));
    }
    for i in 1..=half {
        if pts.len() >= count {
            break;
        }
        let t = i as f32 / half.max(1) as f32;
        // Right arm
        pts.push(Vec3::new(t * scale, -t * scale, 0.0));
    }
    pts.truncate(count);
    pts
}

fn gen_line(count: usize, scale: f32) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / count.max(1) as f32 - 0.5;
        pts.push(Vec3::new(t * scale * 2.0, 0.0, 0.0));
    }
    pts
}

fn gen_cluster(count: usize, scale: f32) -> Vec<Vec3> {
    // Gaussian-distributed cluster around center
    let mut pts = Vec::with_capacity(count);
    let golden_angle = TAU / (1.0 + 5.0f32.sqrt()) * 0.5;
    for i in 0..count {
        let r = (i as f32).sqrt() * scale * 0.3;
        let angle = i as f32 * golden_angle;
        pts.push(Vec3::new(angle.cos() * r, angle.sin() * r, 0.0));
    }
    pts
}

fn gen_swarm(count: usize, scale: f32) -> Vec<Vec3> {
    // Pseudo-random scatter using deterministic hash
    let mut pts = Vec::with_capacity(count);
    for i in 0..count {
        let hash = ((i as u32).wrapping_mul(2654435761)) as f32 / u32::MAX as f32;
        let hash2 = (((i as u32 + 7919).wrapping_mul(2246822519))) as f32 / u32::MAX as f32;
        let angle = hash * TAU;
        let r = hash2.sqrt() * scale;
        pts.push(Vec3::new(angle.cos() * r, angle.sin() * r, 0.0));
    }
    pts
}

fn gen_crown(count: usize, scale: f32) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(count);
    let peaks = 5;
    for i in 0..count {
        let t = i as f32 / count as f32;
        let x = (t - 0.5) * scale * 2.0;
        // Crown: base + peaks
        let peak_phase = (t * peaks as f32 * TAU).sin();
        let y = scale * 0.3 + peak_phase.max(0.0) * scale * 0.7;
        pts.push(Vec3::new(x, y, 0.0));
    }
    pts
}

fn gen_skull(count: usize, scale: f32) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(count);
    let cranium = count * 2 / 3;
    let jaw = count - cranium;
    // Cranium: upper half circle
    for i in 0..cranium {
        let t = i as f32 / cranium.max(1) as f32;
        let angle = t * PI;
        let r = scale;
        pts.push(Vec3::new(angle.cos() * r, angle.sin() * r * 0.8 + scale * 0.2, 0.0));
    }
    // Jaw: smaller arc below
    for i in 0..jaw {
        let t = i as f32 / jaw.max(1) as f32;
        let angle = PI + t * PI * 0.6 + PI * 0.2;
        let r = scale * 0.6;
        pts.push(Vec3::new(angle.cos() * r, angle.sin() * r * 0.5 - scale * 0.1, 0.0));
    }
    pts.truncate(count);
    pts
}

fn gen_heart(count: usize, scale: f32) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / count as f32 * TAU;
        // Heart parametric curve
        let x = 16.0 * t.sin().powi(3);
        let y = 13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos();
        pts.push(Vec3::new(x * scale * 0.06, y * scale * 0.06, 0.0));
    }
    pts
}

fn gen_triangle(count: usize, scale: f32) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(count);
    let verts = [
        Vec3::new(0.0, scale, 0.0),
        Vec3::new(-scale * 0.866, -scale * 0.5, 0.0),
        Vec3::new(scale * 0.866, -scale * 0.5, 0.0),
    ];
    let per_side = count / 3;
    let remainder = count % 3;
    for side in 0..3 {
        let a = verts[side];
        let b = verts[(side + 1) % 3];
        let n = per_side + if side < remainder { 1 } else { 0 };
        for i in 0..n {
            let t = i as f32 / n.max(1) as f32;
            pts.push(a + (b - a) * t);
        }
    }
    pts.truncate(count);
    pts
}

fn gen_pentagon(count: usize, scale: f32) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(count);
    let sides = 5;
    let per_side = count / sides;
    let remainder = count % sides;
    for side in 0..sides {
        let a1 = side as f32 / sides as f32 * TAU - PI / 2.0;
        let a2 = (side + 1) as f32 / sides as f32 * TAU - PI / 2.0;
        let p1 = Vec3::new(a1.cos() * scale, a1.sin() * scale, 0.0);
        let p2 = Vec3::new(a2.cos() * scale, a2.sin() * scale, 0.0);
        let n = per_side + if side < remainder { 1 } else { 0 };
        for i in 0..n {
            let t = i as f32 / n.max(1) as f32;
            pts.push(p1 + (p2 - p1) * t);
        }
    }
    pts.truncate(count);
    pts
}

fn gen_random(count: usize, scale: f32) -> Vec<Vec3> {
    // Deterministic pseudo-random using golden ratio scatter
    let mut pts = Vec::with_capacity(count);
    for i in 0..count {
        let h1 = ((i as u32).wrapping_mul(2654435761)) as f32 / u32::MAX as f32;
        let h2 = (((i as u32).wrapping_add(1)).wrapping_mul(2246822519)) as f32 / u32::MAX as f32;
        let x = (h1 - 0.5) * scale * 2.0;
        let y = (h2 - 0.5) * scale * 2.0;
        pts.push(Vec3::new(x, y, 0.0));
    }
    pts
}

fn gen_snake(count: usize, scale: f32) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / count as f32;
        let x = (t - 0.5) * scale * 4.0;
        let y = (t * TAU * 1.5).sin() * scale * 0.6;
        pts.push(Vec3::new(x, y, 0.0));
    }
    pts
}

fn gen_semicircle(count: usize, scale: f32) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / count.max(1) as f32;
        let angle = t * PI;
        pts.push(Vec3::new(angle.cos() * scale, angle.sin() * scale * 0.5, 0.0));
    }
    pts
}

fn gen_dual_blades(count: usize, scale: f32) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(count);
    let half = count / 2;
    // Left blade (angled line)
    for i in 0..half {
        let t = i as f32 / half.max(1) as f32;
        pts.push(Vec3::new(-scale * 0.3 - t * scale * 0.5, (t - 0.5) * scale * 1.5, 0.0));
    }
    // Right blade
    for i in 0..(count - half) {
        let t = i as f32 / (count - half).max(1) as f32;
        pts.push(Vec3::new(scale * 0.3 + t * scale * 0.5, (t - 0.5) * scale * 1.5, 0.0));
    }
    pts
}

fn gen_pentagram(count: usize, scale: f32) -> Vec<Vec3> {
    // 5-pointed star drawn with connecting inner lines
    let mut pts = Vec::with_capacity(count);
    let outer_r = scale;
    let inner_r = scale * 0.38;
    let n = 10; // alternate outer/inner
    let per_edge = count / n;
    let remainder = count % n;
    for edge in 0..n {
        let a1 = edge as f32 / n as f32 * TAU - PI / 2.0;
        let a2 = (edge + 1) as f32 / n as f32 * TAU - PI / 2.0;
        let r1 = if edge % 2 == 0 { outer_r } else { inner_r };
        let r2 = if (edge + 1) % 2 == 0 { outer_r } else { inner_r };
        let p1 = Vec3::new(a1.cos() * r1, a1.sin() * r1, 0.0);
        let p2 = Vec3::new(a2.cos() * r2, a2.sin() * r2, 0.0);
        let seg_count = per_edge + if edge < remainder { 1 } else { 0 };
        for i in 0..seg_count {
            let t = i as f32 / seg_count.max(1) as f32;
            pts.push(p1 + (p2 - p1) * t);
        }
    }
    pts.truncate(count);
    pts
}

// ── Formation animation utilities ────────────────────────────────────────────

/// Interpolate between two formation position sets. `t` in [0, 1].
/// If the two sets differ in length, extra positions fade to/from origin.
pub fn interpolate_formations(from: &[Vec3], to: &[Vec3], t: f32) -> Vec<Vec3> {
    let len = from.len().max(to.len());
    let t = t.clamp(0.0, 1.0);
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let a = from.get(i).copied().unwrap_or(Vec3::ZERO);
        let b = to.get(i).copied().unwrap_or(Vec3::ZERO);
        result.push(a + (b - a) * t);
    }
    result
}

/// Apply breathing animation to formation positions.
/// Returns new positions with sinusoidal scale oscillation.
pub fn apply_breathing(positions: &[Vec3], time: f32, rate: f32, depth: f32) -> Vec<Vec3> {
    let scale = 1.0 + (time * rate * TAU).sin() * depth;
    positions.iter().map(|p| *p * scale).collect()
}

/// Apply slow rotation to formation positions around Z axis.
pub fn apply_rotation(positions: &[Vec3], time: f32, speed: f32) -> Vec<Vec3> {
    let angle = time * speed;
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    positions
        .iter()
        .map(|p| Vec3::new(p.x * cos_a - p.y * sin_a, p.x * sin_a + p.y * cos_a, p.z))
        .collect()
}

/// Spring-based cohesion: push glyphs toward their target positions.
/// `cohesion` in [0, 1]: 1.0 = tight formation, 0.0 = fully dispersed.
/// `current` = actual glyph positions, `target` = formation targets.
/// Returns corrected positions.
pub fn apply_cohesion_spring(
    current: &[Vec3],
    target: &[Vec3],
    cohesion: f32,
    spring_k: f32,
    dt: f32,
) -> Vec<Vec3> {
    let strength = cohesion.clamp(0.0, 1.0) * spring_k;
    current
        .iter()
        .zip(target.iter())
        .map(|(c, t)| {
            let delta = *t - *c;
            *c + delta * (strength * dt).min(1.0)
        })
        .collect()
}

/// Hit reaction: briefly expand formation outward from center, then contract.
/// `reaction_t` in [0, 1]: 0 = no reaction, peaks at ~0.3, settles at 1.0.
pub fn apply_hit_reaction(positions: &[Vec3], reaction_t: f32, magnitude: f32) -> Vec<Vec3> {
    if reaction_t <= 0.0 || reaction_t >= 1.0 {
        return positions.to_vec();
    }
    // Impulse curve: fast expand, slow contract
    let impulse = if reaction_t < 0.3 {
        reaction_t / 0.3
    } else {
        1.0 - (reaction_t - 0.3) / 0.7
    };
    let expand = impulse * magnitude;
    positions
        .iter()
        .map(|p| {
            let len = p.length();
            if len < 0.001 {
                *p
            } else {
                *p * (1.0 + expand / len.max(0.1))
            }
        })
        .collect()
}

/// Apply movement lean: shift formation in movement direction with trailing spring.
/// `velocity` = entity movement direction, `lean_factor` = how much glyphs trail.
pub fn apply_movement_lean(
    positions: &[Vec3],
    velocity: Vec3,
    lean_factor: f32,
) -> Vec<Vec3> {
    if velocity.length_squared() < 0.0001 {
        return positions.to_vec();
    }
    let dir = velocity.normalize();
    positions
        .iter()
        .enumerate()
        .map(|(i, p)| {
            // Trailing glyphs (farther from front) lean more
            let dot = p.dot(dir);
            let trail = (-dot).max(0.0) * lean_factor;
            *p - dir * trail * 0.1
        })
        .collect()
}

/// Apply HP-based drift: low HP causes glyphs to wobble away from targets.
/// `hp_frac` in [0, 1], `time` for animation phase.
pub fn apply_hp_drift(positions: &[Vec3], hp_frac: f32, time: f32) -> Vec<Vec3> {
    if hp_frac > 0.8 {
        return positions.to_vec();
    }
    let drift_strength = (1.0 - hp_frac) * 0.5;
    positions
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let phase = time * 2.0 + i as f32 * 1.7;
            let dx = phase.sin() * drift_strength;
            let dy = (phase * 1.3 + 0.5).cos() * drift_strength;
            *p + Vec3::new(dx, dy, 0.0)
        })
        .collect()
}

// ── Status effect color modifiers ────────────────────────────────────────────

/// Apply burning visual: orange flicker on glyph colors.
pub fn color_burning(base: Vec4, time: f32, glyph_idx: usize) -> Vec4 {
    let flicker = ((time * 8.0 + glyph_idx as f32 * 2.1).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    Vec4::new(
        (base.x + flicker * 0.4).min(1.0),
        base.y * (0.4 + flicker * 0.3),
        base.z * 0.2,
        base.w,
    )
}

/// Apply frozen visual: blue tint.
pub fn color_frozen(base: Vec4, time: f32, glyph_idx: usize) -> Vec4 {
    let shimmer = ((time * 1.5 + glyph_idx as f32 * 0.8).sin() * 0.15 + 0.85).clamp(0.0, 1.0);
    Vec4::new(
        base.x * 0.3 * shimmer,
        base.y * 0.4 * shimmer,
        (base.z * 0.5 + 0.5).min(1.0),
        base.w,
    )
}

/// Apply poisoned visual: green pulse.
pub fn color_poisoned(base: Vec4, time: f32, _glyph_idx: usize) -> Vec4 {
    let pulse = ((time * 3.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    Vec4::new(
        base.x * (0.5 + pulse * 0.2),
        (base.y + pulse * 0.4).min(1.0),
        base.z * 0.3,
        base.w,
    )
}

/// Apply blessed visual: golden glow.
pub fn color_blessed(base: Vec4, time: f32, glyph_idx: usize) -> Vec4 {
    let glow = ((time * 2.0 + glyph_idx as f32 * 0.5).sin() * 0.3 + 0.7).clamp(0.0, 1.0);
    Vec4::new(
        (base.x + glow * 0.3).min(1.0),
        (base.y + glow * 0.25).min(1.0),
        base.z * 0.5,
        base.w,
    )
}

// ── Player state to formation mapping ────────────────────────────────────────

/// Player animation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAnimState {
    Idle,
    Combat,
    Cast,
    Hurt,
    Death,
    Move,
    LevelUp,
}

/// Archetype visual profile (maps from CharacterClass categories).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassArchetype {
    Warrior,
    Mage,
    Rogue,
    Cleric,
    Necromancer,
    Berserker,
}

impl ClassArchetype {
    /// Idle formation for this archetype.
    pub fn idle_formation(&self) -> FormationShape {
        match self {
            ClassArchetype::Warrior => FormationShape::Shield,
            ClassArchetype::Mage => FormationShape::Pentagram,
            ClassArchetype::Rogue => FormationShape::Crescent,
            ClassArchetype::Cleric => FormationShape::Cross,
            ClassArchetype::Necromancer => FormationShape::Skull,
            ClassArchetype::Berserker => FormationShape::Arrow,
        }
    }

    /// Combat stance formation.
    pub fn combat_formation(&self) -> FormationShape {
        match self {
            ClassArchetype::Warrior => FormationShape::Diamond,
            ClassArchetype::Mage => FormationShape::Star,
            ClassArchetype::Rogue => FormationShape::DualBlades,
            ClassArchetype::Cleric => FormationShape::Ring,
            ClassArchetype::Necromancer => FormationShape::Ring,
            ClassArchetype::Berserker => FormationShape::Vee,
        }
    }

    /// Casting pose formation.
    pub fn cast_formation(&self) -> FormationShape {
        match self {
            ClassArchetype::Warrior => FormationShape::Shield,
            ClassArchetype::Mage => FormationShape::Ring,
            ClassArchetype::Rogue => FormationShape::Crescent,
            ClassArchetype::Cleric => FormationShape::Star,
            ClassArchetype::Necromancer => FormationShape::Pentagram,
            ClassArchetype::Berserker => FormationShape::Triangle,
        }
    }

    /// Hurt recoil formation.
    pub fn hurt_formation(&self) -> FormationShape {
        match self {
            ClassArchetype::Warrior => FormationShape::Cluster,
            ClassArchetype::Mage => FormationShape::Swarm,
            ClassArchetype::Rogue => FormationShape::Swarm,
            ClassArchetype::Cleric => FormationShape::Cluster,
            ClassArchetype::Necromancer => FormationShape::Swarm,
            ClassArchetype::Berserker => FormationShape::Random,
        }
    }

    /// Death dissolution formation.
    pub fn death_formation(&self) -> FormationShape {
        match self {
            ClassArchetype::Warrior => FormationShape::Random,
            ClassArchetype::Mage => FormationShape::Spiral,
            ClassArchetype::Rogue => FormationShape::Line,
            ClassArchetype::Cleric => FormationShape::Ring,
            ClassArchetype::Necromancer => FormationShape::Spiral,
            ClassArchetype::Berserker => FormationShape::Random,
        }
    }

    /// Movement formation.
    pub fn move_formation(&self) -> FormationShape {
        match self {
            ClassArchetype::Warrior => FormationShape::Arrow,
            ClassArchetype::Mage => FormationShape::Diamond,
            ClassArchetype::Rogue => FormationShape::Line,
            ClassArchetype::Cleric => FormationShape::Diamond,
            ClassArchetype::Necromancer => FormationShape::Crescent,
            ClassArchetype::Berserker => FormationShape::Arrow,
        }
    }

    /// Formation for the given animation state.
    pub fn formation_for_state(&self, state: PlayerAnimState) -> FormationShape {
        match state {
            PlayerAnimState::Idle => self.idle_formation(),
            PlayerAnimState::Combat => self.combat_formation(),
            PlayerAnimState::Cast => self.cast_formation(),
            PlayerAnimState::Hurt => self.hurt_formation(),
            PlayerAnimState::Death => self.death_formation(),
            PlayerAnimState::Move => self.move_formation(),
            PlayerAnimState::LevelUp => FormationShape::Star,
        }
    }

    /// Target glyph count for this archetype.
    pub fn base_glyph_count(&self) -> usize {
        match self {
            ClassArchetype::Warrior => 12,
            ClassArchetype::Mage => 15,
            ClassArchetype::Rogue => 10,
            ClassArchetype::Cleric => 12,
            ClassArchetype::Necromancer => 14,
            ClassArchetype::Berserker => 12,
        }
    }

    /// Formation scale for this archetype.
    pub fn formation_scale(&self) -> f32 {
        match self {
            ClassArchetype::Warrior => 1.2,
            ClassArchetype::Mage => 1.4,
            ClassArchetype::Rogue => 1.0,
            ClassArchetype::Cleric => 1.2,
            ClassArchetype::Necromancer => 1.3,
            ClassArchetype::Berserker => 1.3,
        }
    }

    /// Pulse (breathing) rate for this archetype.
    pub fn pulse_rate(&self) -> f32 {
        match self {
            ClassArchetype::Warrior => 0.8,
            ClassArchetype::Mage => 1.2,
            ClassArchetype::Rogue => 1.5,
            ClassArchetype::Cleric => 0.7,
            ClassArchetype::Necromancer => 0.6,
            ClassArchetype::Berserker => 1.8,
        }
    }

    /// Pulse depth for this archetype.
    pub fn pulse_depth(&self) -> f32 {
        match self {
            ClassArchetype::Warrior => 0.03,
            ClassArchetype::Mage => 0.06,
            ClassArchetype::Rogue => 0.04,
            ClassArchetype::Cleric => 0.05,
            ClassArchetype::Necromancer => 0.07,
            ClassArchetype::Berserker => 0.08,
        }
    }
}

// ── Level-up effect ──────────────────────────────────────────────────────────

/// Compute formation for level-up visual effect.
/// `effect_t` in [0, 1]: 0 = start, 1 = end.
/// Returns (positions, glow_intensity).
pub fn level_up_formation(
    base: &[Vec3],
    effect_t: f32,
    new_glyph_pos: Vec3,
) -> (Vec<Vec3>, f32) {
    let glow = if effect_t < 0.5 {
        effect_t / 0.5
    } else {
        1.0 - (effect_t - 0.5) / 0.5
    };

    // During effect, formation tightens then relaxes to new shape
    let tighten = if effect_t < 0.4 {
        1.0 - effect_t / 0.4 * 0.3
    } else {
        0.7 + (effect_t - 0.4) / 0.6 * 0.3
    };

    let mut positions: Vec<Vec3> = base.iter().map(|p| *p * tighten).collect();

    // New glyph fades in from beyond the formation
    let new_t = (effect_t * 2.0 - 0.5).clamp(0.0, 1.0);
    let spawn_pos = new_glyph_pos * (3.0 - new_t * 2.0);
    positions.push(spawn_pos);

    (positions, glow)
}

// ── Spawn and death animations ───────────────────────────────────────────────

/// Enemy spawn animation: glyphs appear from center outward.
/// `spawn_t` in [0, 1]: 0 = start, 1 = fully formed.
pub fn spawn_animation(target_positions: &[Vec3], spawn_t: f32) -> Vec<Vec3> {
    let t = spawn_t.clamp(0.0, 1.0);
    target_positions
        .iter()
        .enumerate()
        .map(|(i, p)| {
            // Stagger: inner glyphs appear first
            let dist = p.length();
            let max_dist = target_positions
                .iter()
                .map(|pp| pp.length())
                .fold(0.0f32, f32::max)
                .max(0.001);
            let normalized_dist = dist / max_dist;
            let local_t = ((t - normalized_dist * 0.5) * 2.0).clamp(0.0, 1.0);
            // Ease out
            let eased = 1.0 - (1.0 - local_t).powi(3);
            *p * eased
        })
        .collect()
}

/// Death dissolution: glyphs scatter outward.
/// `death_t` in [0, 1]: 0 = alive, 1 = fully dissolved.
/// Returns (positions, alpha_multiplier).
pub fn death_dissolution(
    positions: &[Vec3],
    death_t: f32,
    scatter_dir_seed: u32,
) -> (Vec<Vec3>, f32) {
    let t = death_t.clamp(0.0, 1.0);
    let alpha = (1.0 - t).max(0.0);
    let scatter_strength = t * 3.0;

    let result = positions
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let seed = (i as u32).wrapping_add(scatter_dir_seed);
            let angle = (seed.wrapping_mul(2654435761)) as f32 / u32::MAX as f32 * TAU;
            let scatter = Vec3::new(angle.cos(), angle.sin(), 0.0) * scatter_strength;
            // Also drift downward (gravity)
            let gravity = Vec3::new(0.0, -t * t * 2.0, 0.0);
            *p + scatter + gravity
        })
        .collect();

    (result, alpha)
}

/// Element-specific death effect modifier.
/// Returns (color_shift, extra_scatter).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementalDeathStyle {
    /// Fire: embers float upward
    Fire,
    /// Ice: shatter outward sharply
    Ice,
    /// Lightning: zigzag scatter
    Lightning,
    /// Poison: drip downward
    Poison,
    /// Shadow: fade to dark center
    Shadow,
    /// Holy: dissolve in light burst
    Holy,
    /// Default: standard dissolution
    Default,
}

impl ElementalDeathStyle {
    /// Modify a position during death animation based on element.
    pub fn modify_death_pos(&self, base_pos: Vec3, death_t: f32, idx: usize) -> Vec3 {
        let t = death_t.clamp(0.0, 1.0);
        match self {
            ElementalDeathStyle::Fire => {
                // Embers float up
                base_pos + Vec3::new(
                    (idx as f32 * 1.7 + t * 3.0).sin() * t * 0.5,
                    t * 2.0,
                    0.0,
                )
            }
            ElementalDeathStyle::Ice => {
                // Sharp outward shatter
                let dir = if base_pos.length() > 0.01 {
                    base_pos.normalize()
                } else {
                    Vec3::new(1.0, 0.0, 0.0)
                };
                base_pos + dir * t * t * 4.0
            }
            ElementalDeathStyle::Lightning => {
                // Zigzag
                let zigzag = ((idx as f32 + t * 10.0) * 5.0).sin() * t;
                base_pos + Vec3::new(zigzag, zigzag * 0.5, 0.0)
            }
            ElementalDeathStyle::Poison => {
                // Drip down
                base_pos + Vec3::new(
                    (idx as f32 * 2.3).sin() * t * 0.3,
                    -t * t * 3.0,
                    0.0,
                )
            }
            ElementalDeathStyle::Shadow => {
                // Contract to center then vanish
                base_pos * (1.0 - t * 0.8)
            }
            ElementalDeathStyle::Holy => {
                // Burst outward in all directions
                let angle = idx as f32 * 2.399;
                base_pos + Vec3::new(angle.cos(), angle.sin(), 0.0) * t * 2.5
            }
            ElementalDeathStyle::Default => base_pos,
        }
    }

    /// Death color for this element at given time.
    pub fn death_color(&self, base: Vec4, death_t: f32) -> Vec4 {
        let t = death_t.clamp(0.0, 1.0);
        let alpha = (1.0 - t).max(0.0);
        match self {
            ElementalDeathStyle::Fire => Vec4::new(1.0, 0.4 * (1.0 - t), 0.0, alpha),
            ElementalDeathStyle::Ice => Vec4::new(0.6, 0.8, 1.0, alpha),
            ElementalDeathStyle::Lightning => Vec4::new(1.0, 1.0, 0.3, alpha),
            ElementalDeathStyle::Poison => Vec4::new(0.2, 0.8 * (1.0 - t * 0.5), 0.1, alpha),
            ElementalDeathStyle::Shadow => Vec4::new(0.2 * (1.0 - t), 0.0, 0.3 * (1.0 - t), alpha * alpha),
            ElementalDeathStyle::Holy => Vec4::new(1.0, 1.0, 0.8, alpha),
            ElementalDeathStyle::Default => Vec4::new(base.x, base.y, base.z, alpha),
        }
    }
}
