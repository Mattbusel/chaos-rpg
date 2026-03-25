//! Fluid spell effects — Navier-Stokes fluid on the combat arena floor.
//!
//! Wires proof-engine's Eulerian fluid grid to render spell residue as
//! density-mapped glyphs on the arena floor. Each spell element has unique
//! fluid behavior: fire rises and fades, ice spreads slowly and persists,
//! poison pulses, arcane swirls, etc.
//!
//! # Rendering
//!
//! Each grid cell maps to one glyph on the arena floor:
//! - Character chosen by density: empty < 0.1, ░ 0.1-0.3, ▒ 0.3-0.6, ▓ 0.6-0.9, █ 0.9+
//! - Color from the FluidType's gradient mapped to density
//! - Emission proportional to density for fire and arcane (they glow)

use glam::{Vec2, Vec3, Vec4};
use proof_engine::physics::fluid::FluidGrid;
use proof_engine::glyph::GlyphId;
use std::collections::HashMap;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Constants
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Default grid resolution for the arena fluid.
const DEFAULT_GRID_WIDTH: usize = 48;
const DEFAULT_GRID_HEIGHT: usize = 32;
/// World-space size of one fluid cell.
const CELL_SIZE: f32 = 0.5;
/// Minimum density to render a glyph.
const RENDER_THRESHOLD: f32 = 0.1;
/// Density at which glyph is fully opaque.
const FULL_DENSITY: f32 = 0.9;
/// Base injection density for spell hits.
const SPELL_INJECT_DENSITY: f32 = 2.0;
/// Bleed tick injection density.
const BLEED_INJECT_DENSITY: f32 = 0.6;
/// Poison cloud injection radius (in grid cells).
const POISON_CLOUD_RADIUS: usize = 4;
/// Holy radiate speed (cells per second).
const HOLY_RADIATE_SPEED: f32 = 3.0;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Fluid type
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Element-specific fluid behaviors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FluidType {
    /// Red, drips downward, pools in low areas.
    Blood,
    /// Orange-red, rises, fades over time, emits light.
    Fire,
    /// Blue-white, spreads slowly, persists, dims nearby light.
    Ice,
    /// Green, spreads moderately, pulses.
    Poison,
    /// Purple, swirls (add vorticity), glows.
    Arcane,
    /// Gold, radiates outward in circles, heals on contact.
    Holy,
    /// Deep purple, creeps along edges, avoids center.
    Dark,
    /// Blue, flows quickly, transparent.
    Water,
}

impl FluidType {
    /// Base color for this fluid type (RGBA).
    pub fn base_color(self) -> Vec4 {
        match self {
            Self::Blood  => Vec4::new(0.8, 0.05, 0.05, 0.9),
            Self::Fire   => Vec4::new(1.0, 0.4, 0.05, 0.85),
            Self::Ice    => Vec4::new(0.7, 0.85, 1.0, 0.7),
            Self::Poison => Vec4::new(0.2, 0.9, 0.15, 0.8),
            Self::Arcane => Vec4::new(0.6, 0.2, 1.0, 0.85),
            Self::Holy   => Vec4::new(1.0, 0.9, 0.3, 0.9),
            Self::Dark   => Vec4::new(0.3, 0.05, 0.4, 0.85),
            Self::Water  => Vec4::new(0.2, 0.5, 0.9, 0.5),
        }
    }

    /// Secondary color for gradient (high density end).
    pub fn bright_color(self) -> Vec4 {
        match self {
            Self::Blood  => Vec4::new(1.0, 0.1, 0.1, 1.0),
            Self::Fire   => Vec4::new(1.0, 0.9, 0.3, 1.0),
            Self::Ice    => Vec4::new(0.9, 0.95, 1.0, 0.9),
            Self::Poison => Vec4::new(0.4, 1.0, 0.3, 1.0),
            Self::Arcane => Vec4::new(0.8, 0.5, 1.0, 1.0),
            Self::Holy   => Vec4::new(1.0, 1.0, 0.8, 1.0),
            Self::Dark   => Vec4::new(0.5, 0.1, 0.7, 1.0),
            Self::Water  => Vec4::new(0.4, 0.7, 1.0, 0.7),
        }
    }

    /// Whether this fluid type emits light (glow).
    pub fn emits_light(self) -> bool {
        matches!(self, Self::Fire | Self::Arcane | Self::Holy)
    }

    /// Emission intensity multiplier.
    pub fn emission_mult(self) -> f32 {
        match self {
            Self::Fire   => 1.2,
            Self::Arcane => 0.8,
            Self::Holy   => 1.0,
            _            => 0.0,
        }
    }

    /// Viscosity override for the fluid grid.
    pub fn viscosity(self) -> f32 {
        match self {
            Self::Blood  => 2e-3,
            Self::Fire   => 5e-5,
            Self::Ice    => 1e-2,
            Self::Poison => 5e-4,
            Self::Arcane => 1e-4,
            Self::Holy   => 1e-4,
            Self::Dark   => 3e-3,
            Self::Water  => 5e-5,
        }
    }

    /// Decay rate (how fast density fades).
    pub fn decay(self) -> f32 {
        match self {
            Self::Blood  => 0.998,
            Self::Fire   => 0.985,
            Self::Ice    => 0.999,
            Self::Poison => 0.993,
            Self::Arcane => 0.990,
            Self::Holy   => 0.992,
            Self::Dark   => 0.997,
            Self::Water  => 0.996,
        }
    }

    /// Gravity bias for this fluid type.
    pub fn gravity_bias(self) -> Vec2 {
        match self {
            Self::Blood  => Vec2::new(0.0, -2.0),   // drips down
            Self::Fire   => Vec2::new(0.0, 3.0),    // rises up
            Self::Ice    => Vec2::ZERO,               // stays put
            Self::Poison => Vec2::new(0.0, -0.5),   // slight sink
            Self::Arcane => Vec2::ZERO,               // swirls
            Self::Holy   => Vec2::ZERO,               // radiates
            Self::Dark   => Vec2::new(0.0, -0.3),   // creeps low
            Self::Water  => Vec2::new(0.0, -3.0),   // flows down
        }
    }

    /// Vorticity (swirl) strength override.
    pub fn vorticity(self) -> f32 {
        match self {
            Self::Arcane => 2.0,
            Self::Holy   => 0.5,
            Self::Dark   => 0.8,
            _            => 0.3,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FluidLayer — one layer of fluid on the arena
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// A single fluid layer (one per active element type).
struct FluidLayer {
    fluid_type: FluidType,
    grid: FluidGrid,
    /// How long this layer has been alive.
    age: f32,
    /// Per-layer pulse timer (for poison pulsing).
    pulse_timer: f32,
}

impl FluidLayer {
    fn new(fluid_type: FluidType, width: usize, height: usize, dx: f32) -> Self {
        let mut grid = FluidGrid::new(width, height, dx);
        grid.viscosity = fluid_type.viscosity();
        grid.decay = fluid_type.decay();
        grid.gravity = fluid_type.gravity_bias();
        grid.vorticity_strength = fluid_type.vorticity();
        Self {
            fluid_type,
            grid,
            age: 0.0,
            pulse_timer: 0.0,
        }
    }

    /// Inject density at a world position.
    fn inject(&mut self, world_x: f32, world_y: f32, density: f32, origin_x: f32, origin_y: f32) {
        let gx = ((world_x - origin_x) / self.grid.dx).round() as i32;
        let gy = ((world_y - origin_y) / self.grid.dx).round() as i32;
        let gx = gx.clamp(1, self.grid.width as i32 - 2) as usize;
        let gy = gy.clamp(1, self.grid.height as i32 - 2) as usize;

        let idx = self.grid.idx(gx, gy);
        self.grid.density[idx] += density;

        // Set color channels
        let color = self.fluid_type.base_color();
        self.grid.color_r[idx] = (self.grid.color_r[idx] + color.x * density).min(3.0);
        self.grid.color_g[idx] = (self.grid.color_g[idx] + color.y * density).min(3.0);
        self.grid.color_b[idx] = (self.grid.color_b[idx] + color.z * density).min(3.0);

        // Add type-specific velocity at injection point
        let bias = self.fluid_type.gravity_bias();
        self.grid.add_velocity(gx, gy, bias.x * 0.5, bias.y * 0.5);
    }

    /// Inject in a radius (for AOE effects like poison cloud).
    fn inject_radius(
        &mut self,
        world_x: f32,
        world_y: f32,
        radius: usize,
        density: f32,
        origin_x: f32,
        origin_y: f32,
    ) {
        let cx = ((world_x - origin_x) / self.grid.dx).round() as i32;
        let cy = ((world_y - origin_y) / self.grid.dx).round() as i32;
        let r = radius as i32;

        for dy in -r..=r {
            for dx in -r..=r {
                let dist_sq = (dx * dx + dy * dy) as f32;
                let r_sq = (r * r) as f32;
                if dist_sq > r_sq { continue; }

                let gx = (cx + dx).clamp(1, self.grid.width as i32 - 2) as usize;
                let gy = (cy + dy).clamp(1, self.grid.height as i32 - 2) as usize;

                let falloff = 1.0 - (dist_sq / r_sq).sqrt();
                let d = density * falloff;
                let idx = self.grid.idx(gx, gy);
                self.grid.density[idx] += d;

                let color = self.fluid_type.base_color();
                self.grid.color_r[idx] = (self.grid.color_r[idx] + color.x * d).min(3.0);
                self.grid.color_g[idx] = (self.grid.color_g[idx] + color.y * d).min(3.0);
                self.grid.color_b[idx] = (self.grid.color_b[idx] + color.z * d).min(3.0);
            }
        }
    }

    /// Add vorticity (swirl) at a point.
    fn add_swirl(&mut self, world_x: f32, world_y: f32, strength: f32, origin_x: f32, origin_y: f32) {
        let gx = ((world_x - origin_x) / self.grid.dx).round() as i32;
        let gy = ((world_y - origin_y) / self.grid.dx).round() as i32;
        let gx = gx.clamp(2, self.grid.width as i32 - 3) as usize;
        let gy = gy.clamp(2, self.grid.height as i32 - 3) as usize;

        // Create a small vortex: clockwise velocity around the point
        self.grid.add_velocity(gx + 1, gy, 0.0, -strength);
        self.grid.add_velocity(gx - 1, gy, 0.0, strength);
        self.grid.add_velocity(gx, gy + 1, strength, 0.0);
        self.grid.add_velocity(gx, gy - 1, -strength, 0.0);
    }

    /// Step the simulation.
    fn step(&mut self, dt: f32) {
        self.age += dt;
        self.pulse_timer += dt;
        self.grid.step(dt);

        // Type-specific post-step behaviors
        match self.fluid_type {
            FluidType::Poison => {
                // Pulse: periodically boost density in active cells
                if self.pulse_timer > 1.5 {
                    self.pulse_timer = 0.0;
                    let w = self.grid.width;
                    let h = self.grid.height;
                    for y in 0..h {
                        for x in 0..w {
                            let idx = y * w + x;
                            if self.grid.density[idx] > 0.2 {
                                self.grid.density[idx] *= 1.15;
                            }
                        }
                    }
                }
            }
            FluidType::Dark => {
                // Creep toward edges: push fluid away from center
                let cx = self.grid.width as f32 * 0.5;
                let cy = self.grid.height as f32 * 0.5;
                let w = self.grid.width;
                let h = self.grid.height;
                for y in 1..h - 1 {
                    for x in 1..w - 1 {
                        let idx = y * w + x;
                        if self.grid.density[idx] < 0.1 { continue; }
                        let dx = x as f32 - cx;
                        let dy = y as f32 - cy;
                        let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                        let push = 0.3 / dist;
                        self.grid.add_velocity(x, y, dx * push * dt, dy * push * dt);
                    }
                }
            }
            FluidType::Arcane => {
                // Continuous swirl at high-density cells
                let w = self.grid.width;
                let h = self.grid.height;
                for y in 2..h - 2 {
                    for x in 2..w - 2 {
                        let idx = y * w + x;
                        if self.grid.density[idx] > 0.3 {
                            let angle = self.age * 2.0 + (x as f32 * 0.3) + (y as f32 * 0.7);
                            let swirl_vx = angle.cos() * 0.5;
                            let swirl_vy = angle.sin() * 0.5;
                            self.grid.add_velocity(x, y, swirl_vx * dt, swirl_vy * dt);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Check if this layer has any visible density remaining.
    fn is_empty(&self) -> bool {
        self.grid.density.iter().all(|d| *d < RENDER_THRESHOLD * 0.5)
    }

    /// Total density (for deciding when to clean up).
    fn total_density(&self) -> f32 {
        self.grid.density.iter().sum()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ArenaFluid — main manager
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Manages all fluid layers on the combat arena floor.
pub struct ArenaFluid {
    /// Active fluid layers (one per element type in play).
    layers: HashMap<FluidType, FluidLayer>,
    /// Grid dimensions.
    pub width: usize,
    pub height: usize,
    /// Cell size in world units.
    pub cell_size: f32,
    /// World-space origin of the grid (bottom-left corner).
    pub origin: Vec2,
    /// Accumulated time.
    pub time: f32,
}

impl ArenaFluid {
    /// Create a new arena fluid system.
    pub fn new() -> Self {
        let origin_x = -(DEFAULT_GRID_WIDTH as f32 * CELL_SIZE) * 0.5;
        let origin_y = -(DEFAULT_GRID_HEIGHT as f32 * CELL_SIZE) * 0.5;
        Self {
            layers: HashMap::new(),
            width: DEFAULT_GRID_WIDTH,
            height: DEFAULT_GRID_HEIGHT,
            cell_size: CELL_SIZE,
            origin: Vec2::new(origin_x, origin_y),
            time: 0.0,
        }
    }

    /// Create with custom dimensions.
    pub fn with_size(width: usize, height: usize, cell_size: f32) -> Self {
        let origin_x = -(width as f32 * cell_size) * 0.5;
        let origin_y = -(height as f32 * cell_size) * 0.5;
        Self {
            layers: HashMap::new(),
            width,
            height,
            cell_size,
            origin: Vec2::new(origin_x, origin_y),
            time: 0.0,
        }
    }

    /// Get or create a fluid layer for a given type.
    fn layer_mut(&mut self, fluid_type: FluidType) -> &mut FluidLayer {
        self.layers.entry(fluid_type).or_insert_with(|| {
            FluidLayer::new(fluid_type, self.width, self.height, self.cell_size)
        })
    }

    // ════════════════════════════════════════════════════════════════════════
    // Injection (combat events)
    // ════════════════════════════════════════════════════════════════════════

    /// Inject fluid at a world position from a spell impact.
    pub fn inject_spell(&mut self, fluid_type: FluidType, world_pos: Vec2, intensity: f32) {
        let density = SPELL_INJECT_DENSITY * intensity;
        let ox = self.origin.x;
        let oy = self.origin.y;

        let layer = self.layer_mut(fluid_type);
        layer.inject(world_pos.x, world_pos.y, density, ox, oy);

        // Type-specific injection behavior
        match fluid_type {
            FluidType::Fire => {
                // Fire spreads upward velocity
                layer.inject(world_pos.x, world_pos.y + self.cell_size, density * 0.5, ox, oy);
            }
            FluidType::Ice => {
                // Ice spreads in a small cross
                let cs = self.cell_size;
                for &(dx, dy) in &[(cs, 0.0), (-cs, 0.0), (0.0, cs), (0.0, -cs)] {
                    layer.inject(world_pos.x + dx, world_pos.y + dy, density * 0.3, ox, oy);
                }
            }
            FluidType::Arcane => {
                // Arcane adds swirl
                layer.add_swirl(world_pos.x, world_pos.y, 3.0, ox, oy);
            }
            _ => {}
        }
    }

    /// Inject a poison cloud (AOE).
    pub fn inject_poison_cloud(&mut self, center: Vec2, intensity: f32) {
        let density = SPELL_INJECT_DENSITY * intensity * 0.6;
        let ox = self.origin.x;
        let oy = self.origin.y;
        let layer = self.layer_mut(FluidType::Poison);
        layer.inject_radius(center.x, center.y, POISON_CLOUD_RADIUS, density, ox, oy);
    }

    /// Inject a holy radial burst.
    pub fn inject_holy_radiate(&mut self, center: Vec2, intensity: f32) {
        let density = SPELL_INJECT_DENSITY * intensity * 0.8;
        let ox = self.origin.x;
        let oy = self.origin.y;
        let layer = self.layer_mut(FluidType::Holy);

        // Inject in expanding ring
        let radius = (self.time * HOLY_RADIATE_SPEED) % 8.0;
        let steps = 16;
        for i in 0..steps {
            let angle = (i as f32 / steps as f32) * std::f32::consts::TAU;
            let wx = center.x + angle.cos() * radius * self.cell_size;
            let wy = center.y + angle.sin() * radius * self.cell_size;
            layer.inject(wx, wy, density * 0.3, ox, oy);
        }
    }

    /// Inject blood at an entity position (bleed status).
    pub fn inject_bleed(&mut self, entity_pos: Vec2) {
        let ox = self.origin.x;
        let oy = self.origin.y;
        let layer = self.layer_mut(FluidType::Blood);
        layer.inject(entity_pos.x, entity_pos.y, BLEED_INJECT_DENSITY, ox, oy);
        // Blood drips downward
        layer.inject(entity_pos.x, entity_pos.y - self.cell_size, BLEED_INJECT_DENSITY * 0.3, ox, oy);
    }

    /// Inject dark fluid (creeping shadow).
    pub fn inject_dark(&mut self, pos: Vec2, intensity: f32) {
        let ox = self.origin.x;
        let oy = self.origin.y;
        let layer = self.layer_mut(FluidType::Dark);
        layer.inject_radius(pos.x, pos.y, 3, SPELL_INJECT_DENSITY * intensity * 0.5, ox, oy);
    }

    /// Inject water (environmental).
    pub fn inject_water(&mut self, pos: Vec2, amount: f32) {
        let ox = self.origin.x;
        let oy = self.origin.y;
        let layer = self.layer_mut(FluidType::Water);
        layer.inject(pos.x, pos.y, amount, ox, oy);
    }

    /// Boss mechanic: Ouroboros healing flow from damage zone to boss.
    pub fn inject_boss_heal_flow(&mut self, from: Vec2, to: Vec2, intensity: f32) {
        let ox = self.origin.x;
        let oy = self.origin.y;
        let layer = self.layer_mut(FluidType::Holy);

        // Inject along a line from damage zone to boss
        let dir = to - from;
        let steps = 10;
        for i in 0..steps {
            let t = i as f32 / steps as f32;
            let p = from + dir * t;
            layer.inject(p.x, p.y, intensity * 0.3, ox, oy);

            // Add velocity toward the boss
            let vel_dir = dir.normalize_or_zero();
            let gx = ((p.x - ox) / self.cell_size).round() as i32;
            let gy = ((p.y - oy) / self.cell_size).round() as i32;
            let gx = gx.clamp(1, self.width as i32 - 2) as usize;
            let gy = gy.clamp(1, self.height as i32 - 2) as usize;
            layer.grid.add_velocity(gx, gy, vel_dir.x * 5.0, vel_dir.y * 5.0);
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // Simulation
    // ════════════════════════════════════════════════════════════════════════

    /// Step all fluid layers.
    pub fn tick(&mut self, dt: f32) {
        self.time += dt;
        for layer in self.layers.values_mut() {
            layer.step(dt);
        }
        // Remove empty layers
        self.layers.retain(|_, layer| !layer.is_empty());
    }

    /// Clear all fluid.
    pub fn clear(&mut self) {
        self.layers.clear();
    }

    /// Whether any visible fluid exists.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Number of active fluid layers.
    pub fn active_layer_count(&self) -> usize {
        self.layers.len()
    }

    // ════════════════════════════════════════════════════════════════════════
    // Rendering
    // ════════════════════════════════════════════════════════════════════════

    /// Produce per-cell rendering data for all fluid layers.
    /// Each cell with density above threshold produces a `FluidCellRender`.
    pub fn render_cells(&self) -> Vec<FluidCellRender> {
        let mut cells = Vec::new();

        for layer in self.layers.values() {
            let w = layer.grid.width;
            let h = layer.grid.height;
            let ft = layer.fluid_type;

            for y in 0..h {
                for x in 0..w {
                    let idx = y * w + x;
                    let density = layer.grid.density[idx];
                    if density < RENDER_THRESHOLD { continue; }

                    let norm_density = (density / FULL_DENSITY).clamp(0.0, 1.0);

                    // Character by density
                    let character = density_to_char(density);

                    // Color interpolation between base and bright
                    let base = ft.base_color();
                    let bright = ft.bright_color();
                    let color = Vec4::new(
                        base.x + (bright.x - base.x) * norm_density,
                        base.y + (bright.y - base.y) * norm_density,
                        base.z + (bright.z - base.z) * norm_density,
                        base.w + (bright.w - base.w) * norm_density,
                    );

                    // Emission for glowing fluids
                    let emission = if ft.emits_light() {
                        norm_density * ft.emission_mult()
                    } else {
                        0.0
                    };

                    // Poison pulse modulation
                    let emission = if ft == FluidType::Poison {
                        let pulse = (layer.pulse_timer * 3.0).sin() * 0.3 + 0.3;
                        emission + pulse * norm_density * 0.4
                    } else {
                        emission
                    };

                    // World position
                    let wx = self.origin.x + (x as f32 + 0.5) * self.cell_size;
                    let wy = self.origin.y + (y as f32 + 0.5) * self.cell_size;

                    cells.push(FluidCellRender {
                        position: Vec3::new(wx, wy, -0.1), // slightly below arena floor
                        character,
                        color,
                        emission,
                        density,
                        fluid_type: ft,
                    });
                }
            }
        }

        cells
    }

    /// Get the dominant fluid type at a world position (for gameplay: damage zones, etc.).
    pub fn fluid_at(&self, world_pos: Vec2) -> Option<(FluidType, f32)> {
        let mut best: Option<(FluidType, f32)> = None;
        for (ft, layer) in &self.layers {
            let gx = ((world_pos.x - self.origin.x) / self.cell_size).round() as i32;
            let gy = ((world_pos.y - self.origin.y) / self.cell_size).round() as i32;
            if gx < 0 || gy < 0 { continue; }
            let gx = gx as usize;
            let gy = gy as usize;
            if gx >= layer.grid.width || gy >= layer.grid.height { continue; }
            let idx = gy * layer.grid.width + gx;
            let d = layer.grid.density[idx];
            if d > RENDER_THRESHOLD {
                if best.map_or(true, |(_, bd)| d > bd) {
                    best = Some((*ft, d));
                }
            }
        }
        best
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Cell render output
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Render data for one fluid cell on the arena floor.
#[derive(Debug, Clone)]
pub struct FluidCellRender {
    /// World-space position of this cell.
    pub position: Vec3,
    /// Density-mapped character.
    pub character: char,
    /// Interpolated color.
    pub color: Vec4,
    /// Emission intensity (glow).
    pub emission: f32,
    /// Raw density value.
    pub density: f32,
    /// Which fluid type this cell belongs to.
    pub fluid_type: FluidType,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Density → character mapping
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Map fluid density to a glyph character.
fn density_to_char(density: f32) -> char {
    if density < 0.1 {
        ' '
    } else if density < 0.3 {
        '░'
    } else if density < 0.6 {
        '▒'
    } else if density < 0.9 {
        '▓'
    } else {
        '█'
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Combat event bridge
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Map a spell element string to a fluid type.
pub fn element_to_fluid(element: &str) -> Option<FluidType> {
    match element.to_lowercase().as_str() {
        "blood" | "bleed" => Some(FluidType::Blood),
        "fire" => Some(FluidType::Fire),
        "ice" | "frost" | "cold" => Some(FluidType::Ice),
        "poison" | "toxic" => Some(FluidType::Poison),
        "arcane" | "chaos" | "entropy" => Some(FluidType::Arcane),
        "holy" | "radiant" | "light" => Some(FluidType::Holy),
        "dark" | "shadow" | "void" | "necrotic" => Some(FluidType::Dark),
        "water" => Some(FluidType::Water),
        _ => None,
    }
}

/// High-level combat event for the fluid system.
#[derive(Debug, Clone)]
pub enum FluidEvent {
    /// A spell hit at a position with an element.
    SpellHit {
        element: String,
        position: Vec2,
        intensity: f32,
    },
    /// Bleed status tick on an entity.
    BleedTick { entity_position: Vec2 },
    /// Poison cloud AOE.
    PoisonCloud {
        center: Vec2,
        intensity: f32,
    },
    /// Boss healing flow (Ouroboros mechanic).
    BossHealFlow {
        from: Vec2,
        to: Vec2,
        intensity: f32,
    },
    /// Environmental water.
    WaterSplash {
        position: Vec2,
        amount: f32,
    },
}

/// Process a batch of fluid events.
pub fn process_fluid_events(arena: &mut ArenaFluid, events: &[FluidEvent]) {
    for event in events {
        match event {
            FluidEvent::SpellHit { element, position, intensity } => {
                if let Some(ft) = element_to_fluid(element) {
                    arena.inject_spell(ft, *position, *intensity);
                }
            }
            FluidEvent::BleedTick { entity_position } => {
                arena.inject_bleed(*entity_position);
            }
            FluidEvent::PoisonCloud { center, intensity } => {
                arena.inject_poison_cloud(*center, *intensity);
            }
            FluidEvent::BossHealFlow { from, to, intensity } => {
                arena.inject_boss_heal_flow(*from, *to, *intensity);
            }
            FluidEvent::WaterSplash { position, amount } => {
                arena.inject_water(*position, *amount);
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
    fn test_density_to_char() {
        assert_eq!(density_to_char(0.05), ' ');
        assert_eq!(density_to_char(0.2), '░');
        assert_eq!(density_to_char(0.45), '▒');
        assert_eq!(density_to_char(0.75), '▓');
        assert_eq!(density_to_char(1.5), '█');
    }

    #[test]
    fn test_arena_fluid_inject_and_render() {
        let mut arena = ArenaFluid::with_size(16, 16, 1.0);
        arena.inject_spell(FluidType::Fire, Vec2::ZERO, 1.0);

        let cells = arena.render_cells();
        assert!(!cells.is_empty(), "should have visible cells after injection");
        assert!(cells.iter().any(|c| c.fluid_type == FluidType::Fire));
    }

    #[test]
    fn test_arena_fluid_tick_decays() {
        let mut arena = ArenaFluid::with_size(16, 16, 1.0);
        arena.inject_spell(FluidType::Fire, Vec2::ZERO, 1.0);

        let initial = arena.render_cells().len();
        for _ in 0..500 {
            arena.tick(1.0 / 60.0);
        }
        let final_count = arena.render_cells().len();
        assert!(final_count <= initial, "fire should fade over time");
    }

    #[test]
    fn test_poison_cloud_radius() {
        let mut arena = ArenaFluid::with_size(32, 32, 0.5);
        arena.inject_poison_cloud(Vec2::ZERO, 1.0);

        let cells = arena.render_cells();
        assert!(cells.len() > 1, "poison cloud should affect multiple cells");
    }

    #[test]
    fn test_fluid_at_query() {
        let mut arena = ArenaFluid::with_size(16, 16, 1.0);
        arena.inject_spell(FluidType::Ice, Vec2::ZERO, 2.0);

        let result = arena.fluid_at(Vec2::ZERO);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, FluidType::Ice);
    }

    #[test]
    fn test_empty_arena() {
        let arena = ArenaFluid::new();
        assert!(arena.is_empty());
        assert_eq!(arena.render_cells().len(), 0);
    }

    #[test]
    fn test_element_to_fluid_mapping() {
        assert_eq!(element_to_fluid("fire"), Some(FluidType::Fire));
        assert_eq!(element_to_fluid("ice"), Some(FluidType::Ice));
        assert_eq!(element_to_fluid("poison"), Some(FluidType::Poison));
        assert_eq!(element_to_fluid("arcane"), Some(FluidType::Arcane));
        assert_eq!(element_to_fluid("holy"), Some(FluidType::Holy));
        assert_eq!(element_to_fluid("dark"), Some(FluidType::Dark));
        assert_eq!(element_to_fluid("blood"), Some(FluidType::Blood));
        assert_eq!(element_to_fluid("physical"), None);
    }

    #[test]
    fn test_multiple_layers() {
        let mut arena = ArenaFluid::with_size(16, 16, 1.0);
        arena.inject_spell(FluidType::Fire, Vec2::new(2.0, 0.0), 1.0);
        arena.inject_spell(FluidType::Ice, Vec2::new(-2.0, 0.0), 1.0);

        assert_eq!(arena.active_layer_count(), 2);
        let cells = arena.render_cells();
        assert!(cells.iter().any(|c| c.fluid_type == FluidType::Fire));
        assert!(cells.iter().any(|c| c.fluid_type == FluidType::Ice));
    }

    #[test]
    fn test_event_processing() {
        let mut arena = ArenaFluid::with_size(16, 16, 1.0);
        let events = vec![
            FluidEvent::SpellHit {
                element: "fire".to_string(),
                position: Vec2::ZERO,
                intensity: 1.0,
            },
            FluidEvent::BleedTick {
                entity_position: Vec2::new(3.0, 0.0),
            },
        ];
        process_fluid_events(&mut arena, &events);
        assert_eq!(arena.active_layer_count(), 2);
    }
}
