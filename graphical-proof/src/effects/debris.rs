//! Rigid body debris system — physics-driven debris from combat events.
//!
//! Wraps proof-engine's rigid body and debris systems to produce bouncing,
//! rotating debris pieces that fly off enemies on death, scatter from crits,
//! and explode from crafting destruction.
//!
//! Each `DebrisPiece` has position, velocity, angular velocity, rotation,
//! mass, restitution (bounciness), friction, and lifetime. Pieces collide
//! with the arena floor and walls, fade over time, and are culled when
//! expired.

use glam::{Vec2, Vec3, Vec4};
use proof_engine::glyph::{GlyphId, RenderLayer, BlendMode};
use std::f32::consts::TAU;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Constants
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Default gravity (pixels/sec²).
const GRAVITY: Vec2 = Vec2::new(0.0, -15.0);
/// Default floor Y position.
const DEFAULT_FLOOR_Y: f32 = -4.0;
/// Default arena left wall X.
const DEFAULT_WALL_LEFT: f32 = -12.0;
/// Default arena right wall X.
const DEFAULT_WALL_RIGHT: f32 = 12.0;
/// Minimum velocity before a piece is considered "at rest".
const REST_THRESHOLD: f32 = 0.1;
/// Maximum debris pieces alive at once (performance cap).
const MAX_DEBRIS: usize = 256;
/// Default fade start (seconds before expiry to begin fading).
const DEFAULT_FADE_TIME: f32 = 0.8;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Debris material — determines physical properties
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Material type for a debris piece (affects restitution, friction, visuals).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebrisMaterial {
    /// Stone: low bounce, medium friction, gray.
    Stone,
    /// Metal: medium bounce, low friction, silver sparks.
    Metal,
    /// Flesh: very low bounce, high friction, red droplets.
    Flesh,
    /// Bone: medium bounce, medium friction, white.
    Bone,
    /// Wood: low bounce, high friction, brown.
    Wood,
    /// Crystal: high bounce, low friction, colored shards.
    Crystal,
    /// Magical: medium bounce, low friction, glowing.
    Magic,
    /// Fire: embers that rise briefly before falling.
    Ember,
}

impl DebrisMaterial {
    /// Bounciness (coefficient of restitution).
    pub fn restitution(self) -> f32 {
        match self {
            Self::Stone   => 0.3,
            Self::Metal   => 0.6,
            Self::Flesh   => 0.1,
            Self::Bone    => 0.45,
            Self::Wood    => 0.25,
            Self::Crystal => 0.7,
            Self::Magic   => 0.5,
            Self::Ember   => 0.2,
        }
    }

    /// Friction coefficient.
    pub fn friction(self) -> f32 {
        match self {
            Self::Stone   => 0.5,
            Self::Metal   => 0.3,
            Self::Flesh   => 0.8,
            Self::Bone    => 0.5,
            Self::Wood    => 0.7,
            Self::Crystal => 0.2,
            Self::Magic   => 0.1,
            Self::Ember   => 0.1,
        }
    }

    /// Mass multiplier.
    pub fn mass(self) -> f32 {
        match self {
            Self::Stone   => 2.0,
            Self::Metal   => 3.0,
            Self::Flesh   => 0.5,
            Self::Bone    => 1.0,
            Self::Wood    => 0.8,
            Self::Crystal => 0.6,
            Self::Magic   => 0.3,
            Self::Ember   => 0.1,
        }
    }

    /// Default lifetime in seconds.
    pub fn lifetime(self) -> f32 {
        match self {
            Self::Stone   => 4.0,
            Self::Metal   => 5.0,
            Self::Flesh   => 2.5,
            Self::Bone    => 3.5,
            Self::Wood    => 3.0,
            Self::Crystal => 4.5,
            Self::Magic   => 2.0,
            Self::Ember   => 1.5,
        }
    }

    /// Emission (glow) intensity.
    pub fn emission(self) -> f32 {
        match self {
            Self::Magic => 0.7,
            Self::Ember => 1.0,
            Self::Crystal => 0.3,
            _ => 0.0,
        }
    }

    /// Typical characters for this material.
    pub fn characters(self) -> &'static [char] {
        match self {
            Self::Stone   => &['▪', '◼', '◾', '■'],
            Self::Metal   => &['✦', '✧', '⚙', '◆'],
            Self::Flesh   => &['•', '·', '∙', ','],
            Self::Bone    => &['†', '‡', '⸸', '╬'],
            Self::Wood    => &['╱', '╲', '│', '─'],
            Self::Crystal => &['◇', '◈', '❖', '✶'],
            Self::Magic   => &['✴', '✳', '✵', '⁂'],
            Self::Ember   => &['*', '∗', '⁎', '·'],
        }
    }

    /// Base color for this material.
    pub fn base_color(self) -> Vec4 {
        match self {
            Self::Stone   => Vec4::new(0.5, 0.5, 0.5, 1.0),
            Self::Metal   => Vec4::new(0.75, 0.78, 0.8, 1.0),
            Self::Flesh   => Vec4::new(0.8, 0.15, 0.1, 0.9),
            Self::Bone    => Vec4::new(0.9, 0.88, 0.8, 1.0),
            Self::Wood    => Vec4::new(0.55, 0.35, 0.15, 1.0),
            Self::Crystal => Vec4::new(0.6, 0.8, 1.0, 0.85),
            Self::Magic   => Vec4::new(0.7, 0.3, 1.0, 0.9),
            Self::Ember   => Vec4::new(1.0, 0.6, 0.1, 0.9),
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// DebrisPiece
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A single piece of debris bouncing around the arena.
#[derive(Debug, Clone)]
pub struct DebrisPiece {
    /// Engine glyph handle (if tracked).
    pub glyph_id: Option<GlyphId>,
    /// Character to render.
    pub character: char,
    /// Current position.
    pub position: Vec2,
    /// Current velocity.
    pub velocity: Vec2,
    /// Angular velocity (radians/sec).
    pub angular_velocity: f32,
    /// Current rotation (radians).
    pub rotation: f32,
    /// Mass (affects bounce energy).
    pub mass: f32,
    /// Coefficient of restitution (bounciness, 0-1).
    pub restitution: f32,
    /// Friction coefficient.
    pub friction: f32,
    /// Time remaining before removal.
    pub lifetime: f32,
    /// Maximum lifetime (for fade calculation).
    pub max_lifetime: f32,
    /// Time before expiry at which fading begins.
    pub fade_start: f32,
    /// Material type.
    pub material: DebrisMaterial,
    /// Color.
    pub color: Vec4,
    /// Emission intensity.
    pub emission: f32,
    /// Whether this piece is at rest (on the floor, not moving).
    pub at_rest: bool,
}

impl DebrisPiece {
    pub fn new(character: char, position: Vec2, velocity: Vec2, material: DebrisMaterial) -> Self {
        let lt = material.lifetime();
        Self {
            glyph_id: None,
            character,
            position,
            velocity,
            angular_velocity: (velocity.x * 3.0).clamp(-15.0, 15.0),
            rotation: 0.0,
            mass: material.mass(),
            restitution: material.restitution(),
            friction: material.friction(),
            lifetime: lt,
            max_lifetime: lt,
            fade_start: DEFAULT_FADE_TIME,
            material,
            color: material.base_color(),
            emission: material.emission(),
            at_rest: false,
        }
    }

    /// Current alpha (1.0 = fully visible, 0.0 = expired).
    pub fn alpha(&self) -> f32 {
        if self.lifetime > self.fade_start {
            1.0
        } else {
            (self.lifetime / self.fade_start).max(0.0)
        }
    }

    /// Whether this piece has expired.
    pub fn is_expired(&self) -> bool {
        self.lifetime <= 0.0
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// DebrisSystem — manages all debris
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Manages all debris pieces in the arena.
pub struct DebrisSystem {
    /// All active debris pieces.
    pub pieces: Vec<DebrisPiece>,
    /// Gravity acceleration.
    pub gravity: Vec2,
    /// Floor Y position (pieces bounce here).
    pub floor_y: f32,
    /// Wall positions [left, right].
    pub walls: [f32; 2],
    /// Simple RNG state.
    rng_state: u32,
}

impl DebrisSystem {
    pub fn new() -> Self {
        Self {
            pieces: Vec::new(),
            gravity: GRAVITY,
            floor_y: DEFAULT_FLOOR_Y,
            walls: [DEFAULT_WALL_LEFT, DEFAULT_WALL_RIGHT],
            rng_state: 12345,
        }
    }

    /// Set arena bounds.
    pub fn set_bounds(&mut self, floor_y: f32, wall_left: f32, wall_right: f32) {
        self.floor_y = floor_y;
        self.walls = [wall_left, wall_right];
    }

    // ════════════════════════════════════════════════════════════════════════
    // Spawning
    // ════════════════════════════════════════════════════════════════════════

    /// Spawn a single debris piece.
    pub fn spawn(&mut self, piece: DebrisPiece) {
        if self.pieces.len() >= MAX_DEBRIS {
            // Remove oldest piece to make room
            if let Some(oldest) = self.pieces.iter().position(|p| p.at_rest) {
                self.pieces.swap_remove(oldest);
            } else {
                self.pieces.swap_remove(0);
            }
        }
        self.pieces.push(piece);
    }

    /// Enemy death: each glyph becomes a debris piece with random outward velocity.
    pub fn spawn_enemy_death(
        &mut self,
        center: Vec2,
        glyphs: &[char],
        colors: &[Vec4],
        material: DebrisMaterial,
        overkill_damage: f32,
    ) {
        let speed = (overkill_damage / 50.0).clamp(2.0, 12.0);
        let n = glyphs.len();

        for i in 0..n {
            let angle = (i as f32 / n.max(1) as f32) * TAU + self.next_rng_f32() * 0.5;
            let vel = Vec2::new(angle.cos(), angle.sin()) * speed * (0.7 + self.next_rng_f32() * 0.6);
            let ch = glyphs[i];
            let mut piece = DebrisPiece::new(ch, center, vel, material);
            if i < colors.len() {
                piece.color = colors[i];
            }
            piece.angular_velocity = (self.next_rng_f32() - 0.5) * 20.0;
            self.spawn(piece);
        }
    }

    /// Crit hit: 3-5 small debris pieces fly off the enemy.
    pub fn spawn_crit_sparks(
        &mut self,
        impact_pos: Vec2,
        attack_dir: Vec2,
        material: DebrisMaterial,
    ) {
        let count = 3 + (self.next_rng() % 3) as usize;
        let chars = material.characters();
        let base_color = material.base_color();

        for i in 0..count {
            let ch = chars[i % chars.len()];
            let spread = (self.next_rng_f32() - 0.5) * 1.5;
            let perp = Vec2::new(-attack_dir.y, attack_dir.x);
            let vel = (attack_dir * (3.0 + self.next_rng_f32() * 4.0))
                + perp * spread * 3.0
                + Vec2::new(0.0, 2.0 + self.next_rng_f32() * 3.0);

            let mut piece = DebrisPiece::new(ch, impact_pos, vel, material);
            piece.color = base_color;
            piece.lifetime *= 0.5; // sparks are short-lived
            piece.max_lifetime = piece.lifetime;
            self.spawn(piece);
        }
    }

    /// Crafting destruction: item glyphs explode from a crafting bench position.
    pub fn spawn_crafting_destroy(
        &mut self,
        bench_center: Vec2,
        item_chars: &[char],
        item_color: Vec4,
    ) {
        for (i, &ch) in item_chars.iter().enumerate() {
            let angle = (i as f32 / item_chars.len().max(1) as f32) * TAU;
            let speed = 4.0 + self.next_rng_f32() * 3.0;
            let vel = Vec2::new(angle.cos(), angle.sin()) * speed
                + Vec2::new(0.0, 3.0); // upward bias

            let mut piece = DebrisPiece::new(ch, bench_center, vel, DebrisMaterial::Magic);
            piece.color = item_color;
            piece.emission = 0.5;
            self.spawn(piece);
        }
    }

    /// Item shatter: breaks into pieces for each modifier line.
    pub fn spawn_item_shatter(
        &mut self,
        origin: Vec2,
        modifier_chars: &[char],
        modifier_colors: &[Vec4],
    ) {
        for (i, &ch) in modifier_chars.iter().enumerate() {
            let angle = (i as f32 / modifier_chars.len().max(1) as f32) * TAU
                + self.next_rng_f32() * 0.3;
            let speed = 3.0 + self.next_rng_f32() * 2.5;
            let vel = Vec2::new(angle.cos(), angle.sin()) * speed;

            let mut piece = DebrisPiece::new(ch, origin, vel, DebrisMaterial::Crystal);
            if i < modifier_colors.len() {
                piece.color = modifier_colors[i];
            }
            piece.emission = 0.3;
            self.spawn(piece);
        }
    }

    /// Environmental debris: falling rocks, trap debris.
    pub fn spawn_environmental(
        &mut self,
        origin: Vec2,
        count: usize,
        material: DebrisMaterial,
        upward_force: f32,
    ) {
        let chars = material.characters();
        for i in 0..count {
            let ch = chars[i % chars.len()];
            let angle = self.next_rng_f32() * TAU;
            let speed = 1.0 + self.next_rng_f32() * upward_force;
            let vel = Vec2::new(
                angle.cos() * speed * 0.5,
                speed,
            );
            let piece = DebrisPiece::new(ch, origin, vel, material);
            self.spawn(piece);
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // Simulation
    // ════════════════════════════════════════════════════════════════════════

    /// Step the physics simulation.
    pub fn tick(&mut self, dt: f32) {
        for piece in &mut self.pieces {
            if piece.is_expired() { continue; }
            piece.lifetime -= dt;

            if piece.at_rest {
                continue;
            }

            // Apply gravity
            piece.velocity += self.gravity * dt;

            // Ember special: slight upward force early in life
            if piece.material == DebrisMaterial::Ember && piece.lifetime > piece.max_lifetime * 0.5 {
                piece.velocity.y += 8.0 * dt;
            }

            // Integrate position
            piece.position += piece.velocity * dt;
            piece.rotation += piece.angular_velocity * dt;

            // Floor collision
            if piece.position.y <= self.floor_y {
                piece.position.y = self.floor_y;
                piece.velocity.y = -piece.velocity.y * piece.restitution;
                piece.velocity.x *= 1.0 - piece.friction * dt * 5.0;
                piece.angular_velocity *= 0.8;

                // Check if at rest
                if piece.velocity.length() < REST_THRESHOLD {
                    piece.at_rest = true;
                    piece.velocity = Vec2::ZERO;
                    piece.angular_velocity = 0.0;
                }
            }

            // Wall collisions
            if piece.position.x <= self.walls[0] {
                piece.position.x = self.walls[0];
                piece.velocity.x = -piece.velocity.x * piece.restitution;
                piece.angular_velocity = -piece.angular_velocity * 0.7;
            }
            if piece.position.x >= self.walls[1] {
                piece.position.x = self.walls[1];
                piece.velocity.x = -piece.velocity.x * piece.restitution;
                piece.angular_velocity = -piece.angular_velocity * 0.7;
            }
        }

        // Remove expired pieces
        self.pieces.retain(|p| !p.is_expired());
    }

    /// Clear all debris.
    pub fn clear(&mut self) {
        self.pieces.clear();
    }

    /// Number of active debris pieces.
    pub fn count(&self) -> usize {
        self.pieces.len()
    }

    // ════════════════════════════════════════════════════════════════════════
    // Rendering
    // ════════════════════════════════════════════════════════════════════════

    /// Produce render data for all visible debris.
    pub fn render_data(&self) -> Vec<DebrisRender> {
        self.pieces
            .iter()
            .filter(|p| !p.is_expired())
            .map(|p| {
                let alpha = p.alpha();
                let mut color = p.color;
                color.w *= alpha;

                DebrisRender {
                    position: Vec3::new(p.position.x, p.position.y, 0.05),
                    character: p.character,
                    color,
                    rotation: p.rotation,
                    emission: p.emission * alpha,
                    material: p.material,
                    at_rest: p.at_rest,
                }
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
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Render output
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Per-debris-piece rendering data.
#[derive(Debug, Clone)]
pub struct DebrisRender {
    pub position: Vec3,
    pub character: char,
    pub color: Vec4,
    pub rotation: f32,
    pub emission: f32,
    pub material: DebrisMaterial,
    pub at_rest: bool,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Combat event bridge
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// High-level debris event from the game.
#[derive(Debug, Clone)]
pub enum DebrisEvent {
    /// Enemy died — scatter entity glyphs as debris.
    EnemyDeath {
        center: Vec2,
        glyphs: Vec<char>,
        colors: Vec<Vec4>,
        material: DebrisMaterial,
        overkill_damage: f32,
    },
    /// Critical hit — sparks/blood fly off.
    CritHit {
        impact_pos: Vec2,
        attack_dir: Vec2,
        material: DebrisMaterial,
    },
    /// Crafting item destroyed.
    CraftingDestroy {
        bench_center: Vec2,
        item_chars: Vec<char>,
        item_color: Vec4,
    },
    /// Item shattered into modifier pieces.
    ItemShatter {
        origin: Vec2,
        modifier_chars: Vec<char>,
        modifier_colors: Vec<Vec4>,
    },
    /// Environmental trap debris.
    Environmental {
        origin: Vec2,
        count: usize,
        material: DebrisMaterial,
        upward_force: f32,
    },
}

/// Map enemy type to debris material.
pub fn enemy_to_material(enemy_type: &str) -> DebrisMaterial {
    match enemy_type.to_lowercase().as_str() {
        "skeleton" | "lich" | "undead" | "death_knight" => DebrisMaterial::Bone,
        "golem" | "gargoyle" | "elemental" | "construct" => DebrisMaterial::Stone,
        "slime" | "ooze" | "blob" | "jelly" => DebrisMaterial::Flesh,
        "knight" | "warrior" | "guard" | "automaton" | "mech" => DebrisMaterial::Metal,
        "treant" | "plant" | "mushroom" | "vine" => DebrisMaterial::Wood,
        "crystal" | "gem" | "prism" | "shard" => DebrisMaterial::Crystal,
        "mage" | "wizard" | "sorcerer" | "warlock" | "witch" => DebrisMaterial::Magic,
        "fire" | "flame" | "infernal" | "phoenix" => DebrisMaterial::Ember,
        _ => DebrisMaterial::Flesh,
    }
}

/// Process a batch of debris events.
pub fn process_debris_events(system: &mut DebrisSystem, events: &[DebrisEvent]) {
    for event in events {
        match event {
            DebrisEvent::EnemyDeath { center, glyphs, colors, material, overkill_damage } => {
                system.spawn_enemy_death(*center, glyphs, colors, *material, *overkill_damage);
            }
            DebrisEvent::CritHit { impact_pos, attack_dir, material } => {
                system.spawn_crit_sparks(*impact_pos, *attack_dir, *material);
            }
            DebrisEvent::CraftingDestroy { bench_center, item_chars, item_color } => {
                system.spawn_crafting_destroy(*bench_center, item_chars, *item_color);
            }
            DebrisEvent::ItemShatter { origin, modifier_chars, modifier_colors } => {
                system.spawn_item_shatter(*origin, modifier_chars, modifier_colors);
            }
            DebrisEvent::Environmental { origin, count, material, upward_force } => {
                system.spawn_environmental(*origin, *count, *material, *upward_force);
            }
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Tests
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_and_tick() {
        let mut sys = DebrisSystem::new();
        let piece = DebrisPiece::new('▪', Vec2::new(0.0, 2.0), Vec2::new(3.0, 5.0), DebrisMaterial::Stone);
        sys.spawn(piece);
        assert_eq!(sys.count(), 1);

        // Tick until it hits the floor
        for _ in 0..120 {
            sys.tick(1.0 / 60.0);
        }
        // Piece should still be alive but have bounced
        assert_eq!(sys.count(), 1);
        let p = &sys.pieces[0];
        assert!(p.position.y >= sys.floor_y - 0.01);
    }

    #[test]
    fn test_floor_bounce() {
        let mut sys = DebrisSystem::new();
        let piece = DebrisPiece::new('◆', Vec2::new(0.0, 0.0), Vec2::new(0.0, -10.0), DebrisMaterial::Metal);
        sys.spawn(piece);
        sys.tick(1.0);
        // Should have bounced off the floor
        assert!(sys.pieces[0].position.y >= sys.floor_y);
    }

    #[test]
    fn test_wall_collision() {
        let mut sys = DebrisSystem::new();
        let piece = DebrisPiece::new('◆', Vec2::new(11.0, 0.0), Vec2::new(10.0, 0.0), DebrisMaterial::Metal);
        sys.spawn(piece);
        sys.tick(0.5);
        // Should have bounced off right wall
        assert!(sys.pieces[0].position.x <= sys.walls[1] + 0.01);
    }

    #[test]
    fn test_enemy_death_spawns_debris() {
        let mut sys = DebrisSystem::new();
        let glyphs = vec!['◆', '◇', '○', '●', '★'];
        let colors = vec![Vec4::ONE; 5];
        sys.spawn_enemy_death(Vec2::ZERO, &glyphs, &colors, DebrisMaterial::Bone, 100.0);
        assert_eq!(sys.count(), 5);
    }

    #[test]
    fn test_crit_sparks() {
        let mut sys = DebrisSystem::new();
        sys.spawn_crit_sparks(Vec2::ZERO, Vec2::new(1.0, 0.0), DebrisMaterial::Metal);
        assert!(sys.count() >= 3 && sys.count() <= 5);
    }

    #[test]
    fn test_lifetime_expiry() {
        let mut sys = DebrisSystem::new();
        let mut piece = DebrisPiece::new('·', Vec2::ZERO, Vec2::ZERO, DebrisMaterial::Ember);
        piece.lifetime = 0.5;
        piece.max_lifetime = 0.5;
        sys.spawn(piece);

        for _ in 0..60 {
            sys.tick(1.0 / 60.0);
        }
        assert_eq!(sys.count(), 0, "expired debris should be removed");
    }

    #[test]
    fn test_alpha_fade() {
        let mut piece = DebrisPiece::new('·', Vec2::ZERO, Vec2::ZERO, DebrisMaterial::Stone);
        piece.lifetime = 2.0;
        assert!((piece.alpha() - 1.0).abs() < 0.01);
        piece.lifetime = 0.4; // within fade_start (0.8)
        assert!(piece.alpha() < 1.0);
        assert!(piece.alpha() > 0.0);
        piece.lifetime = 0.0;
        assert!(piece.alpha() < 0.01);
    }

    #[test]
    fn test_max_debris_cap() {
        let mut sys = DebrisSystem::new();
        for i in 0..MAX_DEBRIS + 50 {
            let piece = DebrisPiece::new('·', Vec2::ZERO, Vec2::ZERO, DebrisMaterial::Stone);
            sys.spawn(piece);
        }
        assert!(sys.count() <= MAX_DEBRIS);
    }

    #[test]
    fn test_render_data() {
        let mut sys = DebrisSystem::new();
        sys.spawn(DebrisPiece::new('★', Vec2::new(1.0, 2.0), Vec2::ZERO, DebrisMaterial::Magic));
        let renders = sys.render_data();
        assert_eq!(renders.len(), 1);
        assert_eq!(renders[0].character, '★');
        assert!(renders[0].emission > 0.0);
    }

    #[test]
    fn test_event_processing() {
        let mut sys = DebrisSystem::new();
        let events = vec![
            DebrisEvent::CritHit {
                impact_pos: Vec2::ZERO,
                attack_dir: Vec2::X,
                material: DebrisMaterial::Metal,
            },
            DebrisEvent::Environmental {
                origin: Vec2::new(0.0, 5.0),
                count: 4,
                material: DebrisMaterial::Stone,
                upward_force: 3.0,
            },
        ];
        process_debris_events(&mut sys, &events);
        assert!(sys.count() >= 7); // 3-5 sparks + 4 rocks
    }

    #[test]
    fn test_enemy_material_mapping() {
        assert_eq!(enemy_to_material("skeleton"), DebrisMaterial::Bone);
        assert_eq!(enemy_to_material("golem"), DebrisMaterial::Stone);
        assert_eq!(enemy_to_material("slime"), DebrisMaterial::Flesh);
        assert_eq!(enemy_to_material("knight"), DebrisMaterial::Metal);
        assert_eq!(enemy_to_material("treant"), DebrisMaterial::Wood);
        assert_eq!(enemy_to_material("unknown_creature"), DebrisMaterial::Flesh);
    }
}
