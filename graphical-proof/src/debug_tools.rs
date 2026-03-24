//! Debug tools for Chaos RPG — profiler overlays, force field visualization,
//! entity inspector, lighting debug, shader graph debug, and developer console.
//!
//! Toggle debug mode with F12. Switch overlays with F1-F5. Open console with backtick.
//! All rendering is done via proof-engine glyphs on the UI/Overlay layers.

use std::collections::VecDeque;

use proof_engine::prelude::*;
use glam::{Vec2, Vec3, Vec4};

use crate::state::GameState;
use crate::theme::THEMES;

// ═════════════════════════════════════════════════════════════════════════════
// Constants
// ═════════════════════════════════════════════════════════════════════════════

/// Maximum number of frame time samples in the ring buffer.
const FRAME_HISTORY_SIZE: usize = 120;
/// Maximum number of console output lines.
const CONSOLE_MAX_OUTPUT: usize = 200;
/// Maximum number of console command history entries.
const CONSOLE_MAX_HISTORY: usize = 100;
/// Character width for overlay text rendering (world units).
const CHAR_W: f32 = 0.40;
/// Line height for overlay text rendering (world units, negative = downward).
const LINE_H: f32 = -0.65;
/// Console background alpha.
const CONSOLE_BG_ALPHA: f32 = 0.85;
/// Force field grid sampling interval in pixels.
const FIELD_GRID_STEP: usize = 4;
/// Maximum autocomplete suggestions shown.
const MAX_AUTOCOMPLETE: usize = 8;

// ═════════════════════════════════════════════════════════════════════════════
// Color constants
// ═════════════════════════════════════════════════════════════════════════════

const COLOR_GREEN: Vec4 = Vec4::new(0.2, 1.0, 0.3, 1.0);
const COLOR_YELLOW: Vec4 = Vec4::new(1.0, 0.9, 0.2, 1.0);
const COLOR_RED: Vec4 = Vec4::new(1.0, 0.2, 0.2, 1.0);
const COLOR_WHITE: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
const COLOR_DIM_WHITE: Vec4 = Vec4::new(0.7, 0.7, 0.7, 0.8);
const COLOR_CYAN: Vec4 = Vec4::new(0.2, 0.9, 1.0, 1.0);
const COLOR_MAGENTA: Vec4 = Vec4::new(1.0, 0.3, 0.8, 1.0);
const COLOR_ORANGE: Vec4 = Vec4::new(1.0, 0.6, 0.1, 1.0);
const COLOR_CONSOLE_BG: Vec4 = Vec4::new(0.02, 0.02, 0.06, CONSOLE_BG_ALPHA);
const COLOR_CONSOLE_INPUT: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
const COLOR_CONSOLE_OUTPUT: Vec4 = Vec4::new(0.3, 1.0, 0.4, 0.9);
const COLOR_CONSOLE_ERROR: Vec4 = Vec4::new(1.0, 0.3, 0.3, 0.9);
const COLOR_CONSOLE_WARN: Vec4 = Vec4::new(1.0, 0.8, 0.2, 0.9);
const COLOR_CONSOLE_INFO: Vec4 = Vec4::new(0.4, 0.7, 1.0, 0.9);
const COLOR_CONSOLE_SUCCESS: Vec4 = Vec4::new(0.2, 1.0, 0.5, 0.9);
const COLOR_AUTOCOMPLETE_BG: Vec4 = Vec4::new(0.05, 0.05, 0.12, 0.95);
const COLOR_AUTOCOMPLETE_HL: Vec4 = Vec4::new(0.1, 0.3, 0.5, 0.95);
const COLOR_INSPECTOR_BG: Vec4 = Vec4::new(0.03, 0.03, 0.08, 0.9);
const COLOR_INSPECTOR_BORDER: Vec4 = Vec4::new(0.3, 0.8, 1.0, 0.7);
const COLOR_FIELD_ARROW_DIM: Vec4 = Vec4::new(0.2, 0.4, 0.6, 0.4);
const COLOR_FIELD_ARROW_BRIGHT: Vec4 = Vec4::new(0.4, 0.9, 1.0, 0.9);
const COLOR_FIELD_SOURCE: Vec4 = Vec4::new(1.0, 0.6, 0.2, 0.9);
const COLOR_LIGHT_STAR: Vec4 = Vec4::new(1.0, 1.0, 0.6, 1.0);
const COLOR_SHADOW_LINE: Vec4 = Vec4::new(0.3, 0.3, 0.4, 0.5);

// ═════════════════════════════════════════════════════════════════════════════
// DebugOverlay enum
// ═════════════════════════════════════════════════════════════════════════════

/// Which debug overlay is currently displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugOverlayKind {
    /// No overlay (debug HUD still shows basic info).
    None,
    /// F1: Frame profiler with FPS graph and entity counts.
    Profiler,
    /// F2: Force field direction/strength visualization.
    ForceFields,
    /// F3: Click-to-inspect entity panel.
    EntityInspector,
    /// F4: Light source and shadow debug.
    Lighting,
    /// F5: Shader graph node/connection inspector.
    ShaderGraph,
}

impl DebugOverlayKind {
    /// Map function key index (1-5) to overlay variant.
    pub fn from_fkey(index: u8) -> Self {
        match index {
            1 => Self::Profiler,
            2 => Self::ForceFields,
            3 => Self::EntityInspector,
            4 => Self::Lighting,
            5 => Self::ShaderGraph,
            _ => Self::None,
        }
    }

    /// Human-readable label for the overlay.
    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Profiler => "Profiler [F1]",
            Self::ForceFields => "Force Fields [F2]",
            Self::EntityInspector => "Entity Inspector [F3]",
            Self::Lighting => "Lighting [F4]",
            Self::ShaderGraph => "Shader Graph [F5]",
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// DebugGlyph — intermediate representation before engine submission
// ═════════════════════════════════════════════════════════════════════════════

/// A single glyph to be rendered as part of a debug overlay.
/// Collected into a Vec and then submitted to the engine in one batch.
#[derive(Clone, Debug)]
pub struct DebugGlyph {
    pub character: char,
    pub position: Vec3,
    pub color: Vec4,
    pub emission: f32,
    pub layer: RenderLayer,
    pub scale: Vec2,
    pub blend_mode: BlendMode,
}

impl DebugGlyph {
    /// Create a simple UI-layer debug glyph.
    pub fn ui(ch: char, x: f32, y: f32, color: Vec4) -> Self {
        Self {
            character: ch,
            position: Vec3::new(x, y, 10.0),
            color,
            emission: 0.3,
            layer: RenderLayer::Overlay,
            scale: Vec2::ONE,
            blend_mode: BlendMode::Normal,
        }
    }

    /// Create a glyph with custom emission.
    pub fn ui_glow(ch: char, x: f32, y: f32, color: Vec4, emission: f32) -> Self {
        Self {
            character: ch,
            position: Vec3::new(x, y, 10.0),
            color,
            emission,
            layer: RenderLayer::Overlay,
            scale: Vec2::ONE,
            blend_mode: BlendMode::Normal,
        }
    }

    /// Create a background/fill glyph (block character with blend).
    pub fn bg(ch: char, x: f32, y: f32, color: Vec4) -> Self {
        Self {
            character: ch,
            position: Vec3::new(x, y, 9.5),
            color,
            emission: 0.0,
            layer: RenderLayer::Overlay,
            scale: Vec2::ONE,
            blend_mode: BlendMode::Normal,
        }
    }

    /// Convert to engine Glyph for spawning.
    pub fn to_glyph(&self) -> Glyph {
        Glyph {
            character: self.character,
            position: self.position,
            color: self.color,
            emission: self.emission,
            layer: self.layer,
            scale: self.scale,
            blend_mode: self.blend_mode,
            ..Default::default()
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Helper: render a text string as DebugGlyphs
// ═════════════════════════════════════════════════════════════════════════════

/// Emit a string as a row of debug glyphs starting at (x, y).
fn text_glyphs(out: &mut Vec<DebugGlyph>, text: &str, x: f32, y: f32, color: Vec4) {
    for (i, ch) in text.chars().enumerate() {
        out.push(DebugGlyph::ui(ch, x + i as f32 * CHAR_W, y, color));
    }
}

/// Emit a string with custom emission.
fn text_glyphs_glow(
    out: &mut Vec<DebugGlyph>,
    text: &str,
    x: f32,
    y: f32,
    color: Vec4,
    emission: f32,
) {
    for (i, ch) in text.chars().enumerate() {
        out.push(DebugGlyph::ui_glow(ch, x + i as f32 * CHAR_W, y, color, emission));
    }
}

/// Emit a filled rectangle of block characters for backgrounds.
fn rect_glyphs(out: &mut Vec<DebugGlyph>, x: f32, y: f32, cols: usize, rows: usize, color: Vec4) {
    for row in 0..rows {
        for col in 0..cols {
            out.push(DebugGlyph::bg(
                '\u{2588}', // full block
                x + col as f32 * CHAR_W,
                y + row as f32 * LINE_H,
                color,
            ));
        }
    }
}

/// Compute the screen-space anchor for the top-left corner of the overlay area.
/// Returns (origin_x, origin_y) in world coordinates.
fn overlay_origin(engine: &ProofEngine) -> (f32, f32) {
    let cam_pos = engine.camera.target.position();
    let cam_z = engine.camera.position.position().z;
    let fov_rad = engine.camera.fov.position.to_radians();
    let half_h = cam_z * (fov_rad * 0.5).tan();
    let aspect = 16.0 / 9.0;
    let half_w = half_h * aspect;
    (cam_pos.x - half_w + 0.5, cam_pos.y + half_h - 0.5)
}

// ═════════════════════════════════════════════════════════════════════════════
// ProfilerOverlay
// ═════════════════════════════════════════════════════════════════════════════

/// Profiler overlay: frame time ring buffer, entity/particle/glyph counts,
/// FPS bar chart rendered with block characters.
pub struct ProfilerOverlay {
    /// Ring buffer of frame times in seconds.
    frame_times: VecDeque<f32>,
    /// Current counts sampled from the engine.
    pub glyph_count: usize,
    pub particle_count: usize,
    pub field_count: usize,
    pub entity_count: usize,
    pub fluid_count: usize,
    pub debris_count: usize,
    pub cloth_count: usize,
    /// Smoothed FPS for display.
    smoothed_fps: f32,
}

impl ProfilerOverlay {
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(FRAME_HISTORY_SIZE + 1),
            glyph_count: 0,
            particle_count: 0,
            field_count: 0,
            entity_count: 0,
            fluid_count: 0,
            debris_count: 0,
            cloth_count: 0,
            smoothed_fps: 60.0,
        }
    }

    /// Sample all counts from the engine and push a frame time sample.
    pub fn update(&mut self, dt: f32, engine: &ProofEngine) {
        // Push frame time
        self.frame_times.push_back(dt);
        if self.frame_times.len() > FRAME_HISTORY_SIZE {
            self.frame_times.pop_front();
        }

        // Sample counts from engine scene
        self.glyph_count = engine.scene.stats.glyph_count;
        self.particle_count = engine.scene.stats.particle_count;
        self.field_count = engine.scene.stats.field_count;
        self.entity_count = engine.scene.stats.entity_count;

        // Fluid, debris, cloth are sub-categories we estimate from particles
        // (proof-engine does not track these separately; we use heuristics)
        self.fluid_count = 0;
        self.debris_count = 0;
        self.cloth_count = 0;

        // Smoothed FPS (exponential moving average)
        let instant_fps = if dt > 0.0 { 1.0 / dt } else { 999.0 };
        self.smoothed_fps += (instant_fps - self.smoothed_fps) * 0.1;
    }

    /// Compute min/avg/max frame times from the ring buffer.
    fn frame_stats(&self) -> (f32, f32, f32) {
        if self.frame_times.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        let mut min = f32::MAX;
        let mut max = 0.0_f32;
        let mut sum = 0.0_f32;
        for &t in &self.frame_times {
            min = min.min(t);
            max = max.max(t);
            sum += t;
        }
        let avg = sum / self.frame_times.len() as f32;
        (min, avg, max)
    }

    /// FPS color based on current smoothed FPS.
    fn fps_color(&self) -> Vec4 {
        if self.smoothed_fps >= 58.0 {
            COLOR_GREEN
        } else if self.smoothed_fps >= 30.0 {
            COLOR_YELLOW
        } else {
            COLOR_RED
        }
    }

    /// Color for a single frame time sample.
    fn sample_color(dt: f32) -> Vec4 {
        let fps = if dt > 0.0 { 1.0 / dt } else { 999.0 };
        if fps >= 58.0 {
            COLOR_GREEN
        } else if fps >= 30.0 {
            COLOR_YELLOW
        } else {
            COLOR_RED
        }
    }

    /// Render the profiler overlay as a list of debug glyphs.
    pub fn render(&self, origin_x: f32, origin_y: f32) -> Vec<DebugGlyph> {
        let mut out = Vec::with_capacity(800);
        let x0 = origin_x;
        let mut y = origin_y;

        // ── Title ──
        text_glyphs(&mut out, "=== PROFILER ===", x0, y, COLOR_CYAN);
        y += LINE_H;

        // ── FPS ──
        let fps_text = format!("FPS: {:.0}", self.smoothed_fps);
        text_glyphs(&mut out, &fps_text, x0, y, self.fps_color());
        y += LINE_H;

        // ── Frame time stats ──
        let (fmin, favg, fmax) = self.frame_stats();
        let stats_text = format!(
            "Frame: min {:.1}ms  avg {:.1}ms  max {:.1}ms",
            fmin * 1000.0,
            favg * 1000.0,
            fmax * 1000.0,
        );
        text_glyphs(&mut out, &stats_text, x0, y, COLOR_DIM_WHITE);
        y += LINE_H;

        // ── Entity counts ──
        let counts = format!(
            "Glyphs: {}  Particles: {}  Entities: {}  Fields: {}",
            self.glyph_count, self.particle_count, self.entity_count, self.field_count,
        );
        text_glyphs(&mut out, &counts, x0, y, COLOR_DIM_WHITE);
        y += LINE_H;

        let extra_counts = format!(
            "Fluid: {}  Debris: {}  Cloth: {}",
            self.fluid_count, self.debris_count, self.cloth_count,
        );
        text_glyphs(&mut out, &extra_counts, x0, y, COLOR_DIM_WHITE);
        y += LINE_H * 1.5;

        // ── FPS bar chart (using block characters) ──
        // Each bar represents one frame. Height proportional to frame time.
        // Max bar height = 8 characters.
        text_glyphs(&mut out, "Frame Times:", x0, y, COLOR_CYAN);
        y += LINE_H;

        let bar_max_height = 8;
        let max_dt = 0.050; // 50ms = worst expected; anything above is clamped

        // Render bars from left to right
        let bar_x_start = x0;
        let bar_y_base = y + (bar_max_height as f32 * LINE_H);

        for (i, &dt) in self.frame_times.iter().enumerate() {
            let normalized = (dt as f64 / max_dt).min(1.0) as f32;
            let bar_height = (normalized * bar_max_height as f32).ceil() as usize;
            let bar_height = bar_height.max(1).min(bar_max_height);
            let color = Self::sample_color(dt);

            let bx = bar_x_start + i as f32 * CHAR_W * 0.35;
            for row in 0..bar_height {
                let by = bar_y_base - row as f32 * LINE_H * 0.5;
                let block = if row == bar_height - 1 { '\u{2584}' } else { '\u{2588}' };
                out.push(DebugGlyph::ui(block, bx, by, color));
            }
        }

        // Move y past the bar chart
        y = bar_y_base + LINE_H;

        // ── 60fps / 30fps reference lines ──
        let fps_60_height = ((1.0 / 60.0) / max_dt as f32).min(1.0) * bar_max_height as f32;
        let fps_30_height = ((1.0 / 30.0) / max_dt as f32).min(1.0) * bar_max_height as f32;

        let ref_y_60 = bar_y_base - fps_60_height * LINE_H.abs() * 0.5;
        let ref_y_30 = bar_y_base - fps_30_height * LINE_H.abs() * 0.5;

        let chart_right = bar_x_start + FRAME_HISTORY_SIZE as f32 * CHAR_W * 0.35 + CHAR_W;
        text_glyphs(&mut out, "60", chart_right, ref_y_60, COLOR_GREEN);
        text_glyphs(&mut out, "30", chart_right, ref_y_30, COLOR_YELLOW);

        out
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// ForceFieldOverlay
// ═════════════════════════════════════════════════════════════════════════════

/// Arrow characters indexed by direction octant.
const ARROW_CHARS: [char; 8] = [
    '\u{2192}', // 0: right     →
    '\u{2197}', // 1: up-right  ↗
    '\u{2191}', // 2: up        ↑
    '\u{2196}', // 3: up-left   ↖
    '\u{2190}', // 4: left      ←
    '\u{2199}', // 5: down-left ↙
    '\u{2193}', // 6: down      ↓
    '\u{2198}', // 7: down-right↘
];

/// Force field visualization overlay: samples the scene's force fields on a grid,
/// rendering arrow glyphs showing direction and magnitude.
pub struct ForceFieldOverlay {
    /// Cached grid of sampled forces. Rebuilt each frame.
    samples: Vec<FieldSample>,
    /// Pulse timer for source markers.
    pulse_timer: f32,
    /// Grid columns and rows.
    grid_cols: usize,
    grid_rows: usize,
}

/// A single sampled force at a grid point.
struct FieldSample {
    world_x: f32,
    world_y: f32,
    force_x: f32,
    force_y: f32,
    magnitude: f32,
}

impl ForceFieldOverlay {
    pub fn new() -> Self {
        Self {
            samples: Vec::with_capacity(256),
            pulse_timer: 0.0,
            grid_cols: 0,
            grid_rows: 0,
        }
    }

    /// Sample force fields from the engine scene on a grid.
    pub fn update(&mut self, dt: f32, engine: &ProofEngine, state: &GameState) {
        self.pulse_timer += dt;

        let w = state.screen_width as f32;
        let h = state.screen_height as f32;

        // Compute the visible world rect from camera
        let cam_pos = engine.camera.target.position();
        let cam_z = engine.camera.position.position().z;
        let fov_rad = engine.camera.fov.position.to_radians();
        let half_h = cam_z * (fov_rad * 0.5).tan();
        let aspect = w / h.max(1.0);
        let half_w = half_h * aspect;

        let world_left = cam_pos.x - half_w;
        let world_right = cam_pos.x + half_w;
        let world_bottom = cam_pos.y - half_h;
        let world_top = cam_pos.y + half_h;

        // Grid sampling
        let step_x = (world_right - world_left) / (w / FIELD_GRID_STEP as f32);
        let step_y = (world_top - world_bottom) / (h / FIELD_GRID_STEP as f32);

        self.grid_cols = ((world_right - world_left) / step_x.max(0.01)) as usize;
        self.grid_rows = ((world_top - world_bottom) / step_y.max(0.01)) as usize;

        // Clamp grid to prevent excessive sampling
        let max_samples = 2000;
        let grid_cols = self.grid_cols.min(80);
        let grid_rows = self.grid_rows.min(max_samples / grid_cols.max(1));

        self.samples.clear();
        self.samples.reserve(grid_cols * grid_rows);

        for row in 0..grid_rows {
            for col in 0..grid_cols {
                let wx = world_left + col as f32 * step_x + step_x * 0.5;
                let wy = world_bottom + row as f32 * step_y + step_y * 0.5;
                let pos = Vec3::new(wx, wy, 0.0);

                // Sum all force fields at this point
                let mut total = Vec3::ZERO;
                for (_, field) in &engine.scene.fields {
                    let f = field.force_at(pos, 1.0, 0.0, engine.scene.time);
                    total += f;
                }

                let mag = (total.x * total.x + total.y * total.y).sqrt();
                if mag > 0.001 {
                    self.samples.push(FieldSample {
                        world_x: wx,
                        world_y: wy,
                        force_x: total.x,
                        force_y: total.y,
                        magnitude: mag,
                    });
                }
            }
        }
    }

    /// Render force field arrows and source markers.
    pub fn render(&self, engine: &ProofEngine) -> Vec<DebugGlyph> {
        let mut out = Vec::with_capacity(self.samples.len() + 64);

        // ── Grid arrows ──
        let max_mag = self
            .samples
            .iter()
            .map(|s| s.magnitude)
            .fold(0.0_f32, f32::max)
            .max(0.01);

        for sample in &self.samples {
            // Direction → octant
            let angle = sample.force_y.atan2(sample.force_x);
            let octant = (((angle + std::f32::consts::PI) / (std::f32::consts::PI / 4.0)) as usize) % 8;
            let arrow = ARROW_CHARS[octant];

            // Intensity based on magnitude
            let t = (sample.magnitude / max_mag).clamp(0.0, 1.0);
            let color = Vec4::new(
                COLOR_FIELD_ARROW_DIM.x + (COLOR_FIELD_ARROW_BRIGHT.x - COLOR_FIELD_ARROW_DIM.x) * t,
                COLOR_FIELD_ARROW_DIM.y + (COLOR_FIELD_ARROW_BRIGHT.y - COLOR_FIELD_ARROW_DIM.y) * t,
                COLOR_FIELD_ARROW_DIM.z + (COLOR_FIELD_ARROW_BRIGHT.z - COLOR_FIELD_ARROW_DIM.z) * t,
                COLOR_FIELD_ARROW_DIM.w + (COLOR_FIELD_ARROW_BRIGHT.w - COLOR_FIELD_ARROW_DIM.w) * t,
            );

            out.push(DebugGlyph {
                character: arrow,
                position: Vec3::new(sample.world_x, sample.world_y, 8.0),
                color,
                emission: t * 0.5,
                layer: RenderLayer::Overlay,
                scale: Vec2::ONE,
                blend_mode: BlendMode::Additive,
            });
        }

        // ── Field source positions (pulsing circles) ──
        let pulse = (self.pulse_timer * 3.0).sin() * 0.3 + 0.7;
        for (_, field) in &engine.scene.fields {
            let center = field_center(field);
            if let Some(c) = center {
                let circle_chars = ['\u{25CB}', '\u{25CF}', '\u{25C9}']; // ○ ● ◉
                let idx = ((self.pulse_timer * 2.0) as usize) % circle_chars.len();
                out.push(DebugGlyph {
                    character: circle_chars[idx],
                    position: Vec3::new(c.x, c.y, 8.5),
                    color: Vec4::new(
                        COLOR_FIELD_SOURCE.x * pulse,
                        COLOR_FIELD_SOURCE.y * pulse,
                        COLOR_FIELD_SOURCE.z * pulse,
                        COLOR_FIELD_SOURCE.w,
                    ),
                    emission: pulse,
                    layer: RenderLayer::Overlay,
                    scale: Vec2::splat(1.5),
                    blend_mode: BlendMode::Additive,
                });
            }
        }

        // ── Legend ──
        let (ox, oy) = overlay_origin_from(engine);
        let ly = oy;
        text_glyphs(&mut out, "=== FORCE FIELDS ===", ox, ly, COLOR_CYAN);
        let field_count = engine.scene.fields.len();
        let sample_count = self.samples.len();
        let info = format!("Fields: {}  Samples: {}", field_count, sample_count);
        text_glyphs(&mut out, &info, ox, ly + LINE_H, COLOR_DIM_WHITE);

        out
    }
}

/// Extract the center position from a ForceField variant, if it has one.
fn field_center(field: &ForceField) -> Option<Vec3> {
    match field {
        ForceField::Gravity { center, .. }
        | ForceField::Vortex { center, .. }
        | ForceField::Repulsion { center, .. }
        | ForceField::Electromagnetic { center, .. }
        | ForceField::HeatSource { center, .. }
        | ForceField::MathField { center, .. }
        | ForceField::StrangeAttractor { center, .. }
        | ForceField::EntropyField { center, .. }
        | ForceField::Damping { center, .. }
        | ForceField::Pulsing { center, .. }
        | ForceField::Shockwave { center, .. }
        | ForceField::Warp { center, .. }
        | ForceField::Tidal { center, .. }
        | ForceField::MagneticDipole { center, .. }
        | ForceField::Saddle { center, .. } => Some(*center),
        ForceField::Flow { .. } | ForceField::Wind { .. } => None,
    }
}

/// Overlay origin helper that takes just the engine reference.
fn overlay_origin_from(engine: &ProofEngine) -> (f32, f32) {
    overlay_origin(engine)
}

// ═════════════════════════════════════════════════════════════════════════════
// EntityInspector
// ═════════════════════════════════════════════════════════════════════════════

/// Entity inspector: select and inspect individual entities.
pub struct EntityInspector {
    /// Index of the currently selected entity (into engine.scene.entities).
    pub selected_index: usize,
    /// Whether an entity is actively selected for inspection.
    pub has_selection: bool,
    /// Cached info about the selected entity.
    cached_name: String,
    cached_hp: i64,
    cached_max_hp: i64,
    cached_position: Vec3,
    cached_glyph_count: usize,
    cached_formation: String,
    cached_cohesion: f32,
    cached_status_effects: Vec<String>,
}

impl EntityInspector {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            has_selection: false,
            cached_name: String::new(),
            cached_hp: 0,
            cached_max_hp: 1,
            cached_position: Vec3::ZERO,
            cached_glyph_count: 0,
            cached_formation: String::new(),
            cached_cohesion: 0.0,
            cached_status_effects: Vec::new(),
        }
    }

    /// Handle entity cycling with arrow keys and mouse click selection.
    pub fn handle_input(&mut self, engine: &ProofEngine) {
        let entity_count = engine.scene.entities.len();
        if entity_count == 0 {
            self.has_selection = false;
            return;
        }

        // Arrow keys to cycle
        if engine.input.just_pressed(Key::Right) || engine.input.just_pressed(Key::Down) {
            self.selected_index = (self.selected_index + 1) % entity_count;
            self.has_selection = true;
        }
        if engine.input.just_pressed(Key::Left) || engine.input.just_pressed(Key::Up) {
            if self.selected_index == 0 {
                self.selected_index = entity_count.saturating_sub(1);
            } else {
                self.selected_index -= 1;
            }
            self.has_selection = true;
        }

        // Mouse click to select nearest entity
        if engine.input.mouse_left_just_pressed {
            let mouse_world = Vec3::new(
                engine.input.mouse_ndc.x * 20.0 + engine.camera.target.position().x,
                engine.input.mouse_ndc.y * 12.0 + engine.camera.target.position().y,
                0.0,
            );

            let mut best_dist = f32::MAX;
            let mut best_idx = 0;
            for (i, (_, entity)) in engine.scene.entities.iter().enumerate() {
                let d = (entity.position - mouse_world).length();
                if d < best_dist {
                    best_dist = d;
                    best_idx = i;
                }
            }
            if best_dist < 5.0 {
                self.selected_index = best_idx;
                self.has_selection = true;
            }
        }

        // Ensure index is valid
        if self.selected_index >= entity_count {
            self.selected_index = entity_count.saturating_sub(1);
        }
    }

    /// Update cached entity info from the engine.
    pub fn update(&mut self, engine: &ProofEngine, state: &GameState) {
        if !self.has_selection {
            return;
        }

        let entities = &engine.scene.entities;
        if self.selected_index >= entities.len() {
            self.has_selection = false;
            return;
        }

        let (_, entity) = &entities[self.selected_index];
        self.cached_name = entity.name.clone();
        self.cached_position = entity.position;
        self.cached_glyph_count = entity.formation.len();
        self.cached_formation = format!("{} glyphs", entity.formation.len());
        self.cached_cohesion = entity.cohesion;

        // Pull HP from game state if this is the player or enemy
        self.cached_status_effects.clear();
        if let Some(ref enemy) = state.enemy {
            if self.cached_name.contains(&enemy.name) {
                self.cached_hp = enemy.hp;
                self.cached_max_hp = enemy.max_hp;
                return;
            }
        }
        if let Some(ref player) = state.player {
            if self.cached_name.contains("player") || self.cached_name.contains("Player") {
                self.cached_hp = player.current_hp;
                self.cached_max_hp = player.max_hp;
                return;
            }
        }

        // Default: use entity glyph count as a proxy for "health"
        self.cached_hp = self.cached_glyph_count as i64;
        self.cached_max_hp = self.cached_glyph_count.max(1) as i64;
    }

    /// Render the inspector panel and selection highlight.
    pub fn render(&self, engine: &ProofEngine) -> Vec<DebugGlyph> {
        let mut out = Vec::with_capacity(256);

        let (ox, oy) = overlay_origin_from(engine);
        text_glyphs(&mut out, "=== ENTITY INSPECTOR ===", ox, oy, COLOR_CYAN);

        let entity_count = engine.scene.entities.len();
        let info = format!(
            "Entities: {}  Selected: {}/{}",
            entity_count,
            if self.has_selection { self.selected_index + 1 } else { 0 },
            entity_count,
        );
        text_glyphs(&mut out, &info, ox, oy + LINE_H, COLOR_DIM_WHITE);
        text_glyphs(
            &mut out,
            "[<-/-> cycle]  [click to select]",
            ox,
            oy + LINE_H * 2.0,
            COLOR_DIM_WHITE,
        );

        if !self.has_selection || self.selected_index >= entity_count {
            text_glyphs(
                &mut out,
                "No entity selected",
                ox,
                oy + LINE_H * 4.0,
                COLOR_YELLOW,
            );
            return out;
        }

        // ── Highlight the selected entity ──
        let (_, entity) = &engine.scene.entities[self.selected_index];
        let ep = entity.position;

        // Bright outline corners around the entity
        let corners = [
            ('\u{250C}', -1.0, 1.0),  // ┌ top-left
            ('\u{2510}', 1.5, 1.0),   // ┐ top-right
            ('\u{2514}', -1.0, -1.0),  // └ bottom-left
            ('\u{2518}', 1.5, -1.0),   // ┘ bottom-right
        ];
        for (ch, dx, dy) in corners {
            out.push(DebugGlyph {
                character: ch,
                position: Vec3::new(ep.x + dx, ep.y + dy, 9.0),
                color: COLOR_INSPECTOR_BORDER,
                emission: 0.8,
                layer: RenderLayer::Overlay,
                scale: Vec2::ONE,
                blend_mode: BlendMode::Additive,
            });
        }

        // ── Floating info panel near entity ──
        let panel_x = ep.x + 2.5;
        let panel_y = ep.y + 1.5;
        let panel_w = 30;
        let panel_h = 10;
        rect_glyphs(&mut out, panel_x - CHAR_W, panel_y + LINE_H * 0.3, panel_w, panel_h, COLOR_INSPECTOR_BG);

        let mut py = panel_y;

        // Name
        let name_line = format!(">> {}", self.cached_name);
        text_glyphs(&mut out, &name_line, panel_x, py, COLOR_CYAN);
        py += LINE_H;

        // HP bar
        let hp_pct = if self.cached_max_hp > 0 {
            self.cached_hp as f32 / self.cached_max_hp as f32
        } else {
            0.0
        };
        let hp_bar_len = 16;
        let filled = (hp_pct * hp_bar_len as f32) as usize;
        let hp_color = if hp_pct > 0.6 {
            COLOR_GREEN
        } else if hp_pct > 0.3 {
            COLOR_YELLOW
        } else {
            COLOR_RED
        };
        let hp_text = format!("HP: {}/{}", self.cached_hp, self.cached_max_hp);
        text_glyphs(&mut out, &hp_text, panel_x, py, hp_color);
        py += LINE_H;

        let mut bar = String::with_capacity(hp_bar_len + 2);
        bar.push('[');
        for i in 0..hp_bar_len {
            bar.push(if i < filled { '\u{2588}' } else { '\u{2591}' });
        }
        bar.push(']');
        text_glyphs(&mut out, &bar, panel_x, py, hp_color);
        py += LINE_H;

        // Formation
        let form_line = format!("Formation: {}", self.cached_formation);
        text_glyphs(&mut out, &form_line, panel_x, py, COLOR_DIM_WHITE);
        py += LINE_H;

        // Cohesion
        let coh_line = format!("Cohesion: {:.2}", self.cached_cohesion);
        text_glyphs(&mut out, &coh_line, panel_x, py, COLOR_DIM_WHITE);
        py += LINE_H;

        // Position / Velocity
        let pos_line = format!(
            "Pos: ({:.1}, {:.1})",
            self.cached_position.x, self.cached_position.y,
        );
        text_glyphs(&mut out, &pos_line, panel_x, py, COLOR_DIM_WHITE);
        py += LINE_H;

        // Glyph count
        let gc_line = format!("Glyphs: {}", self.cached_glyph_count);
        text_glyphs(&mut out, &gc_line, panel_x, py, COLOR_DIM_WHITE);
        py += LINE_H;

        // Status effects
        if !self.cached_status_effects.is_empty() {
            text_glyphs(&mut out, "Status:", panel_x, py, COLOR_ORANGE);
            py += LINE_H;
            for effect in &self.cached_status_effects {
                let eff_line = format!("  * {}", effect);
                text_glyphs(&mut out, &eff_line, panel_x, py, COLOR_MAGENTA);
                py += LINE_H;
            }
        }

        out
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// LightingOverlay
// ═════════════════════════════════════════════════════════════════════════════

/// Lighting debug overlay: shows light source positions, radii, and shadow info.
pub struct LightingOverlay {
    /// Number of lights detected this frame.
    pub light_count: usize,
    /// Number of shadow casters detected this frame.
    pub shadow_caster_count: usize,
    /// Maximum lights the engine supports.
    pub max_lights: usize,
    /// Cached light info for rendering.
    lights: Vec<LightInfo>,
}

struct LightInfo {
    position: Vec3,
    color: Vec4,
    radius: f32,
    shadow_dir: Option<Vec2>,
}

impl LightingOverlay {
    pub fn new() -> Self {
        Self {
            light_count: 0,
            shadow_caster_count: 0,
            max_lights: 64,
            lights: Vec::new(),
        }
    }

    /// Scan the scene for light sources (glyphs with emission > 0.5).
    pub fn update(&mut self, engine: &ProofEngine) {
        self.lights.clear();
        self.light_count = 0;
        self.shadow_caster_count = 0;

        // In proof-engine, lights are glyphs with emission > 0.
        // We scan the glyph pool for high-emission glyphs as "lights".
        // The actual light system is in the render pipeline, but we can
        // approximate from glyph data.

        // Count entities that cast shadows (any entity with mass > 0)
        for (_, entity) in &engine.scene.entities {
            self.shadow_caster_count += 1;
        }

        // Scan force fields for heat sources (which produce light)
        for (_, field) in &engine.scene.fields {
            match field {
                ForceField::HeatSource { center, temperature, radius } => {
                    self.lights.push(LightInfo {
                        position: *center,
                        color: Vec4::new(1.0, 0.7 + temperature * 0.001, 0.4, 1.0),
                        radius: *radius,
                        shadow_dir: Some(Vec2::new(0.0, -1.0)),
                    });
                    self.light_count += 1;
                }
                _ => {}
            }
        }

        // Treat scene zones as potential light sources
        for zone in &engine.scene.zones {
            // AmbientZone uses min/max AABB; compute center and approximate radius
            let zone_center = (zone.min + zone.max) * 0.5;
            let zone_radius = (zone.max - zone.min).length() * 0.5;
            self.lights.push(LightInfo {
                position: Vec3::new(zone_center.x, zone_center.y, 0.0),
                color: Vec4::new(
                    zone.ambient_color.x,
                    zone.ambient_color.y,
                    zone.ambient_color.z,
                    0.7,
                ),
                radius: zone_radius,
                shadow_dir: None,
            });
            self.light_count += 1;
        }
    }

    /// Render light stars, radius circles, and shadow lines.
    pub fn render(&self, engine: &ProofEngine) -> Vec<DebugGlyph> {
        let mut out = Vec::with_capacity(128);

        let (ox, oy) = overlay_origin_from(engine);

        // ── Legend ──
        text_glyphs(&mut out, "=== LIGHTING DEBUG ===", ox, oy, COLOR_CYAN);
        let info = format!(
            "Lights: {}/{}  Shadow casters: {}",
            self.light_count, self.max_lights, self.shadow_caster_count,
        );
        text_glyphs(&mut out, &info, ox, oy + LINE_H, COLOR_DIM_WHITE);

        // ── Light markers ──
        for light in &self.lights {
            // Star at light position
            out.push(DebugGlyph {
                character: '\u{2605}', // ★
                position: Vec3::new(light.position.x, light.position.y, 9.0),
                color: Vec4::new(
                    light.color.x,
                    light.color.y,
                    light.color.z,
                    1.0,
                ),
                emission: 1.5,
                layer: RenderLayer::Overlay,
                scale: Vec2::splat(1.3),
                blend_mode: BlendMode::Additive,
            });

            // Radius circle (using dots)
            let circle_points = 24;
            for i in 0..circle_points {
                let angle = (i as f32 / circle_points as f32) * std::f32::consts::TAU;
                let cx = light.position.x + angle.cos() * light.radius;
                let cy = light.position.y + angle.sin() * light.radius;
                out.push(DebugGlyph {
                    character: '\u{00B7}', // ·
                    position: Vec3::new(cx, cy, 8.5),
                    color: Vec4::new(light.color.x * 0.5, light.color.y * 0.5, light.color.z * 0.5, 0.3),
                    emission: 0.2,
                    layer: RenderLayer::Overlay,
                    scale: Vec2::ONE,
                    blend_mode: BlendMode::Additive,
                });
            }

            // Shadow direction line
            if let Some(dir) = light.shadow_dir {
                let line_len = 8;
                for j in 0..line_len {
                    let t = j as f32 / line_len as f32;
                    let lx = light.position.x + dir.x * t * 5.0;
                    let ly = light.position.y + dir.y * t * 5.0;
                    out.push(DebugGlyph {
                        character: '\u{2500}', // ─
                        position: Vec3::new(lx, ly, 8.0),
                        color: Vec4::new(
                            COLOR_SHADOW_LINE.x,
                            COLOR_SHADOW_LINE.y,
                            COLOR_SHADOW_LINE.z,
                            COLOR_SHADOW_LINE.w * (1.0 - t),
                        ),
                        emission: 0.0,
                        layer: RenderLayer::Overlay,
                        scale: Vec2::ONE,
                        blend_mode: BlendMode::Normal,
                    });
                }
            }
        }

        out
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// ShaderGraphOverlay
// ═════════════════════════════════════════════════════════════════════════════

/// Shader graph debug: displays active shader preset info, node counts,
/// connection counts, and estimated instruction cost.
pub struct ShaderGraphOverlay {
    pub active_preset: String,
    pub node_count: usize,
    pub connection_count: usize,
    pub estimated_instructions: usize,
    pub intermediate_values: Vec<(String, f32)>,
}

impl ShaderGraphOverlay {
    pub fn new() -> Self {
        Self {
            active_preset: "default".to_string(),
            node_count: 0,
            connection_count: 0,
            estimated_instructions: 0,
            intermediate_values: Vec::new(),
        }
    }

    /// Update shader stats from the engine/game state.
    pub fn update(&mut self, state: &GameState) {
        // The shader graph system is managed through shader_presets module.
        // We read approximate values from state.
        let theme = &THEMES[state.theme_idx % THEMES.len()];
        self.active_preset = theme.name.to_string();

        // Estimate node counts from theme settings
        // (In a full implementation this would query the actual shader graph)
        self.node_count = 12 + (theme.bloom_intensity * 4.0) as usize;
        self.connection_count = self.node_count * 2 - 3;
        self.estimated_instructions = self.node_count * 8 + self.connection_count * 2;

        self.intermediate_values.clear();
        self.intermediate_values
            .push(("bloom_intensity".to_string(), theme.bloom_intensity));
        self.intermediate_values
            .push(("chromatic_ab".to_string(), theme.chromatic_aberration));
        self.intermediate_values
            .push(("vignette".to_string(), theme.vignette_strength));
        self.intermediate_values
            .push(("chaos_brightness".to_string(), theme.chaos_field_brightness));
        self.intermediate_values
            .push(("corruption".to_string(), state.corruption_frac()));
    }

    /// Render shader graph info as debug glyphs.
    pub fn render(&self, engine: &ProofEngine) -> Vec<DebugGlyph> {
        let mut out = Vec::with_capacity(128);

        let (ox, oy) = overlay_origin_from(engine);
        let mut y = oy;

        text_glyphs(&mut out, "=== SHADER GRAPH ===", ox, y, COLOR_CYAN);
        y += LINE_H;

        let preset = format!("Active Preset: {}", self.active_preset);
        text_glyphs(&mut out, &preset, ox, y, COLOR_WHITE);
        y += LINE_H;

        let nodes = format!("Nodes: {}  Connections: {}", self.node_count, self.connection_count);
        text_glyphs(&mut out, &nodes, ox, y, COLOR_DIM_WHITE);
        y += LINE_H;

        let cost = format!("Est. Instructions: {}", self.estimated_instructions);
        text_glyphs(&mut out, &cost, ox, y, COLOR_DIM_WHITE);
        y += LINE_H * 1.5;

        // Intermediate values
        text_glyphs(&mut out, "Intermediate Values:", ox, y, COLOR_ORANGE);
        y += LINE_H;

        for (name, value) in &self.intermediate_values {
            let line = format!("  {} = {:.4}", name, value);
            text_glyphs(&mut out, &line, ox, y, COLOR_DIM_WHITE);
            y += LINE_H;
        }

        out
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// DebugConsole
// ═════════════════════════════════════════════════════════════════════════════

/// Developer console with command input, history, and autocomplete.
pub struct DebugConsole {
    /// Whether the console is visible.
    pub visible: bool,
    /// Current input buffer.
    pub input_buffer: String,
    /// Command history (most recent last).
    pub history: Vec<String>,
    /// Output lines with per-line color.
    pub output_lines: VecDeque<(String, [f32; 4])>,
    /// Cursor position within input_buffer.
    pub cursor_pos: usize,
    /// Current autocomplete suggestions.
    pub autocomplete_suggestions: Vec<String>,
    /// Currently selected autocomplete index.
    autocomplete_index: usize,
    /// History browsing index (-1 = current input, 0 = most recent, etc.).
    history_index: Option<usize>,
    /// Saved input buffer when browsing history.
    saved_input: String,

    // ── Cheat state ──
    pub god_mode: bool,
    pub noclip: bool,
    pub show_fps: bool,

    /// Debug force field IDs (for `field clear`).
    debug_field_ids: Vec<FieldId>,
}

/// All known console commands for autocomplete.
const COMMANDS: &[&str] = &[
    "set corruption",
    "set hp",
    "set mp",
    "set floor",
    "spawn boss",
    "spawn enemy",
    "shader preset",
    "field gravity",
    "field vortex",
    "field clear",
    "particles burst",
    "weather lightning",
    "weather rain",
    "music vibe",
    "timeline play",
    "kill all",
    "god",
    "noclip",
    "fps",
    "help",
    "clear",
];

/// Boss names known to the `spawn boss` command.
const BOSS_NAMES: &[&str] = &[
    "mirror",
    "null",
    "committee",
    "fibonacci_hydra",
    "eigenstate",
    "ouroboros",
    "algorithm_reborn",
    "chaos_weaver",
    "void_serpent",
    "prime_factorial",
];

/// Music vibes known to the `music vibe` command.
const MUSIC_VIBES: &[&str] = &[
    "combat", "boss", "exploration", "title", "shop", "shrine", "chaos",
];

/// Timeline names known to the `timeline play` command.
const TIMELINE_NAMES: &[&str] = &["death", "victory", "boss_intro"];

impl DebugConsole {
    pub fn new() -> Self {
        Self {
            visible: false,
            input_buffer: String::new(),
            history: Vec::new(),
            output_lines: VecDeque::with_capacity(CONSOLE_MAX_OUTPUT + 1),
            cursor_pos: 0,
            autocomplete_suggestions: Vec::new(),
            autocomplete_index: 0,
            history_index: None,
            saved_input: String::new(),
            god_mode: false,
            noclip: false,
            show_fps: false,
            debug_field_ids: Vec::new(),
        }
    }

    /// Toggle console visibility.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.input_buffer.clear();
            self.cursor_pos = 0;
            self.autocomplete_suggestions.clear();
            self.history_index = None;
        }
    }

    /// Push an output line with a color.
    pub fn push_output(&mut self, text: String, color: [f32; 4]) {
        self.output_lines.push_back((text, color));
        if self.output_lines.len() > CONSOLE_MAX_OUTPUT {
            self.output_lines.pop_front();
        }
    }

    /// Push a green info output.
    fn info(&mut self, text: String) {
        self.push_output(text, COLOR_CONSOLE_OUTPUT.to_array());
    }

    /// Push a red error output.
    fn error(&mut self, text: String) {
        self.push_output(text, COLOR_CONSOLE_ERROR.to_array());
    }

    /// Push a yellow warning output.
    fn warn(&mut self, text: String) {
        self.push_output(text, COLOR_CONSOLE_WARN.to_array());
    }

    /// Push a blue info output.
    fn blue_info(&mut self, text: String) {
        self.push_output(text, COLOR_CONSOLE_INFO.to_array());
    }

    /// Push a success output.
    fn success(&mut self, text: String) {
        self.push_output(text, COLOR_CONSOLE_SUCCESS.to_array());
    }

    /// Update autocomplete suggestions based on current input.
    fn update_autocomplete(&mut self) {
        self.autocomplete_suggestions.clear();
        self.autocomplete_index = 0;

        if self.input_buffer.is_empty() {
            return;
        }

        let input_lower = self.input_buffer.to_lowercase();

        for &cmd in COMMANDS {
            if cmd.starts_with(&input_lower) || cmd.contains(&input_lower) {
                self.autocomplete_suggestions.push(cmd.to_string());
            }
        }

        // For specific sub-commands, add argument completions
        if input_lower.starts_with("spawn boss ") {
            let partial = &input_lower["spawn boss ".len()..];
            self.autocomplete_suggestions.clear();
            for &name in BOSS_NAMES {
                if name.starts_with(partial) {
                    self.autocomplete_suggestions
                        .push(format!("spawn boss {}", name));
                }
            }
        } else if input_lower.starts_with("music vibe ") {
            let partial = &input_lower["music vibe ".len()..];
            self.autocomplete_suggestions.clear();
            for &name in MUSIC_VIBES {
                if name.starts_with(partial) {
                    self.autocomplete_suggestions
                        .push(format!("music vibe {}", name));
                }
            }
        } else if input_lower.starts_with("timeline play ") {
            let partial = &input_lower["timeline play ".len()..];
            self.autocomplete_suggestions.clear();
            for &name in TIMELINE_NAMES {
                if name.starts_with(partial) {
                    self.autocomplete_suggestions
                        .push(format!("timeline play {}", name));
                }
            }
        }

        self.autocomplete_suggestions.truncate(MAX_AUTOCOMPLETE);
    }

    /// Map a Key to a character for text input.
    fn key_to_char(key: Key, shift: bool) -> Option<char> {
        match key {
            Key::A => Some(if shift { 'A' } else { 'a' }),
            Key::B => Some(if shift { 'B' } else { 'b' }),
            Key::C => Some(if shift { 'C' } else { 'c' }),
            Key::D => Some(if shift { 'D' } else { 'd' }),
            Key::E => Some(if shift { 'E' } else { 'e' }),
            Key::F => Some(if shift { 'F' } else { 'f' }),
            Key::G => Some(if shift { 'G' } else { 'g' }),
            Key::H => Some(if shift { 'H' } else { 'h' }),
            Key::I => Some(if shift { 'I' } else { 'i' }),
            Key::J => Some(if shift { 'J' } else { 'j' }),
            Key::K => Some(if shift { 'K' } else { 'k' }),
            Key::L => Some(if shift { 'L' } else { 'l' }),
            Key::M => Some(if shift { 'M' } else { 'm' }),
            Key::N => Some(if shift { 'N' } else { 'n' }),
            Key::O => Some(if shift { 'O' } else { 'o' }),
            Key::P => Some(if shift { 'P' } else { 'p' }),
            Key::Q => Some(if shift { 'Q' } else { 'q' }),
            Key::R => Some(if shift { 'R' } else { 'r' }),
            Key::S => Some(if shift { 'S' } else { 's' }),
            Key::T => Some(if shift { 'T' } else { 't' }),
            Key::U => Some(if shift { 'U' } else { 'u' }),
            Key::V => Some(if shift { 'V' } else { 'v' }),
            Key::W => Some(if shift { 'W' } else { 'w' }),
            Key::X => Some(if shift { 'X' } else { 'x' }),
            Key::Y => Some(if shift { 'Y' } else { 'y' }),
            Key::Z => Some(if shift { 'Z' } else { 'z' }),
            Key::Num0 => Some(if shift { ')' } else { '0' }),
            Key::Num1 => Some(if shift { '!' } else { '1' }),
            Key::Num2 => Some(if shift { '@' } else { '2' }),
            Key::Num3 => Some(if shift { '#' } else { '3' }),
            Key::Num4 => Some(if shift { '$' } else { '4' }),
            Key::Num5 => Some(if shift { '%' } else { '5' }),
            Key::Num6 => Some(if shift { '^' } else { '6' }),
            Key::Num7 => Some(if shift { '&' } else { '7' }),
            Key::Num8 => Some(if shift { '*' } else { '8' }),
            Key::Num9 => Some(if shift { '(' } else { '9' }),
            Key::Space => Some(' '),
            Key::Period => Some(if shift { '>' } else { '.' }),
            Key::Comma => Some(if shift { '<' } else { ',' }),
            Key::Minus => Some(if shift { '_' } else { '-' }),
            Key::Equals => Some(if shift { '+' } else { '=' }),
            Key::Slash => Some(if shift { '?' } else { '/' }),
            Key::Backslash => Some(if shift { '|' } else { '\\' }),
            Key::Semicolon => Some(if shift { ':' } else { ';' }),
            Key::Quote => Some(if shift { '"' } else { '\'' }),
            Key::LBracket => Some(if shift { '{' } else { '[' }),
            Key::RBracket => Some(if shift { '}' } else { ']' }),
            _ => None,
        }
    }

    /// Handle keyboard input for the console. Returns true if input was consumed.
    pub fn handle_input(&mut self, engine: &ProofEngine) -> bool {
        if !self.visible {
            return false;
        }

        let shift = engine.input.shift();

        // Enter: execute command
        if engine.input.just_pressed(Key::Enter) {
            let cmd = self.input_buffer.clone();
            if !cmd.is_empty() {
                self.push_output(format!("> {}", cmd), COLOR_CONSOLE_INPUT.to_array());
                self.history.push(cmd.clone());
                if self.history.len() > CONSOLE_MAX_HISTORY {
                    self.history.remove(0);
                }
                self.history_index = None;
                self.input_buffer.clear();
                self.cursor_pos = 0;
                self.autocomplete_suggestions.clear();
                // Command execution is deferred to execute() call from manager
                return true;
            }
        }

        // Backspace
        if engine.input.just_pressed(Key::Backspace) {
            if self.cursor_pos > 0 {
                self.cursor_pos -= 1;
                self.input_buffer.remove(self.cursor_pos);
                self.update_autocomplete();
            }
            return true;
        }

        // Delete
        if engine.input.just_pressed(Key::Delete) {
            if self.cursor_pos < self.input_buffer.len() {
                self.input_buffer.remove(self.cursor_pos);
                self.update_autocomplete();
            }
            return true;
        }

        // Tab: autocomplete
        if engine.input.just_pressed(Key::Tab) {
            if !self.autocomplete_suggestions.is_empty() {
                let suggestion = self.autocomplete_suggestions
                    [self.autocomplete_index % self.autocomplete_suggestions.len()]
                    .clone();
                self.input_buffer = suggestion;
                self.cursor_pos = self.input_buffer.len();
                self.autocomplete_index += 1;
                self.update_autocomplete();
            }
            return true;
        }

        // Up: history previous
        if engine.input.just_pressed(Key::Up) {
            if !self.history.is_empty() {
                match self.history_index {
                    None => {
                        self.saved_input = self.input_buffer.clone();
                        self.history_index = Some(self.history.len() - 1);
                    }
                    Some(idx) if idx > 0 => {
                        self.history_index = Some(idx - 1);
                    }
                    _ => {}
                }
                if let Some(idx) = self.history_index {
                    self.input_buffer = self.history[idx].clone();
                    self.cursor_pos = self.input_buffer.len();
                }
            }
            return true;
        }

        // Down: history next
        if engine.input.just_pressed(Key::Down) {
            if let Some(idx) = self.history_index {
                if idx + 1 < self.history.len() {
                    self.history_index = Some(idx + 1);
                    self.input_buffer = self.history[idx + 1].clone();
                } else {
                    self.history_index = None;
                    self.input_buffer = self.saved_input.clone();
                }
                self.cursor_pos = self.input_buffer.len();
            }
            return true;
        }

        // Left/Right cursor movement
        if engine.input.just_pressed(Key::Left) {
            if self.cursor_pos > 0 {
                self.cursor_pos -= 1;
            }
            return true;
        }
        if engine.input.just_pressed(Key::Right) {
            if self.cursor_pos < self.input_buffer.len() {
                self.cursor_pos += 1;
            }
            return true;
        }

        // Home/End
        if engine.input.just_pressed(Key::Home) {
            self.cursor_pos = 0;
            return true;
        }
        if engine.input.just_pressed(Key::End) {
            self.cursor_pos = self.input_buffer.len();
            return true;
        }

        // Escape closes console
        if engine.input.just_pressed(Key::Escape) {
            self.visible = false;
            return true;
        }

        // Character input: iterate all just-pressed keys
        for &key in &engine.input.keys_just_pressed {
            if let Some(ch) = Self::key_to_char(key, shift) {
                self.input_buffer.insert(self.cursor_pos, ch);
                self.cursor_pos += 1;
                self.update_autocomplete();
            }
        }

        true // Console always consumes input while visible
    }

    /// Execute a command string, modifying game state as needed.
    /// Returns the response message.
    pub fn execute(&mut self, command: &str, state: &mut GameState, engine: &mut ProofEngine) {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "set" if parts.len() >= 3 => {
                self.execute_set(&parts[1..], state);
            }
            "spawn" if parts.len() >= 3 => {
                self.execute_spawn(&parts[1..], state, engine);
            }
            "shader" if parts.len() >= 3 && parts[1] == "preset" => {
                self.execute_shader_preset(parts[2], state);
            }
            "field" if parts.len() >= 2 => {
                self.execute_field(&parts[1..], engine);
            }
            "particles" if parts.len() >= 2 => {
                self.execute_particles(&parts[1..], engine);
            }
            "weather" if parts.len() >= 2 => {
                self.execute_weather(parts[1], engine);
            }
            "music" if parts.len() >= 3 && parts[1] == "vibe" => {
                self.execute_music_vibe(parts[2], engine);
            }
            "timeline" if parts.len() >= 3 && parts[1] == "play" => {
                self.execute_timeline(parts[2], state);
            }
            "kill" if parts.len() >= 2 && parts[1] == "all" => {
                self.execute_kill_all(state);
            }
            "god" => {
                self.god_mode = !self.god_mode;
                let status = if self.god_mode { "ON" } else { "OFF" };
                self.success(format!("God mode: {}", status));
            }
            "noclip" => {
                self.noclip = !self.noclip;
                let status = if self.noclip { "ON" } else { "OFF" };
                self.success(format!("Noclip: {}", status));
            }
            "fps" => {
                self.show_fps = !self.show_fps;
                let status = if self.show_fps { "ON" } else { "OFF" };
                self.success(format!("FPS counter: {}", status));
            }
            "help" => {
                self.execute_help();
            }
            "clear" => {
                self.output_lines.clear();
                self.info("Console cleared.".to_string());
            }
            _ => {
                self.error(format!("Unknown command: '{}'. Type 'help' for commands.", command));
            }
        }
    }

    fn execute_set(&mut self, args: &[&str], state: &mut GameState) {
        if args.len() < 2 {
            self.error("Usage: set <property> <value>".to_string());
            return;
        }

        match args[0] {
            "corruption" => {
                if let Ok(n) = args[1].parse::<u32>() {
                    if let Some(ref mut player) = state.player {
                        player.kills = n;
                        self.success(format!("Corruption set to {}", n));
                    } else {
                        self.error("No player active.".to_string());
                    }
                } else {
                    self.error("Invalid number for corruption.".to_string());
                }
            }
            "hp" => {
                if let Ok(n) = args[1].parse::<i64>() {
                    if let Some(ref mut player) = state.player {
                        player.current_hp = n.max(0);
                        self.success(format!("HP set to {}", n));
                    } else {
                        self.error("No player active.".to_string());
                    }
                } else {
                    self.error("Invalid number for HP.".to_string());
                }
            }
            "mp" => {
                if let Ok(n) = args[1].parse::<i64>() {
                    state.current_mana = n.max(0);
                    self.success(format!("MP set to {}", n));
                } else {
                    self.error("Invalid number for MP.".to_string());
                }
            }
            "floor" => {
                if let Ok(n) = args[1].parse::<u32>() {
                    state.floor_num = n.max(1);
                    state.floor = None; // Force regeneration
                    self.success(format!("Floor set to {} (regenerating dungeon)", n));
                } else {
                    self.error("Invalid number for floor.".to_string());
                }
            }
            _ => {
                self.error(format!(
                    "Unknown property '{}'. Valid: corruption, hp, mp, floor",
                    args[0]
                ));
            }
        }
    }

    fn execute_spawn(&mut self, args: &[&str], state: &mut GameState, _engine: &mut ProofEngine) {
        match args[0] {
            "boss" => {
                if args.len() < 2 {
                    self.error(format!("Usage: spawn boss <name>. Valid: {:?}", BOSS_NAMES));
                    return;
                }
                let name = args[1].to_lowercase();
                let boss_id = match name.as_str() {
                    "mirror" => Some(0u8),
                    "null" => Some(1),
                    "committee" => Some(2),
                    "fibonacci_hydra" => Some(3),
                    "eigenstate" => Some(4),
                    "ouroboros" => Some(5),
                    "algorithm_reborn" => Some(6),
                    "chaos_weaver" => Some(7),
                    "void_serpent" => Some(8),
                    "prime_factorial" => Some(9),
                    _ => None,
                };
                if let Some(id) = boss_id {
                    state.boss_id = Some(id);
                    state.is_boss_fight = true;
                    state.boss_turn = 0;
                    state.boss_extra = 0;
                    state.boss_extra2 = 0;
                    state.boss_entrance_timer = 2.0;
                    state.boss_entrance_name = name.clone();
                    self.success(format!("Spawning boss: {}", name));
                } else {
                    self.error(format!(
                        "Unknown boss '{}'. Valid: {:?}",
                        name, BOSS_NAMES
                    ));
                }
            }
            "enemy" => {
                if args.len() < 2 {
                    self.error("Usage: spawn enemy <tier 1-5>".to_string());
                    return;
                }
                if let Ok(tier) = args[1].parse::<u32>() {
                    if (1..=5).contains(&tier) {
                        // Generate a test enemy at the given tier
                        let enemy = chaos_rpg_core::enemy::generate_enemy(
                            state.floor_num * tier,
                            state.seed + tier as u64,
                        );
                        state.enemy = Some(enemy);
                        state.is_boss_fight = false;
                        self.success(format!("Spawned tier {} test enemy", tier));
                    } else {
                        self.error("Tier must be 1-5.".to_string());
                    }
                } else {
                    self.error("Invalid tier number.".to_string());
                }
            }
            _ => {
                self.error(format!(
                    "Unknown spawn type '{}'. Valid: boss, enemy",
                    args[0]
                ));
            }
        }
    }

    fn execute_shader_preset(&mut self, name: &str, state: &mut GameState) {
        // Map preset name to theme index
        let theme_idx = match name.to_lowercase().as_str() {
            "void" | "void_protocol" => Some(0),
            "blood" | "blood_pact" => Some(1),
            "emerald" | "emerald_engine" => Some(2),
            "solar" | "solar_forge" => Some(3),
            "glacial" | "glacial_abyss" => Some(4),
            _ => None,
        };
        if let Some(idx) = theme_idx {
            state.theme_idx = idx;
            let name = THEMES[idx].name;
            self.success(format!("Shader preset: {}", name));
        } else {
            self.error(format!(
                "Unknown shader preset '{}'. Valid: void, blood, emerald, solar, glacial",
                name
            ));
        }
    }

    fn execute_field(&mut self, args: &[&str], engine: &mut ProofEngine) {
        if args.is_empty() {
            self.error("Usage: field <gravity|vortex|clear> [args...]".to_string());
            return;
        }

        match args[0] {
            "gravity" => {
                if args.len() < 4 {
                    self.error("Usage: field gravity <x> <y> <strength>".to_string());
                    return;
                }
                let x = args[1].parse::<f32>().unwrap_or(0.0);
                let y = args[2].parse::<f32>().unwrap_or(0.0);
                let strength = args[3].parse::<f32>().unwrap_or(5.0);
                let field = ForceField::Gravity {
                    center: Vec3::new(x, y, 0.0),
                    strength,
                    falloff: Falloff::InverseSquare,
                };
                let id = engine.add_field(field);
                self.debug_field_ids.push(id);
                self.success(format!(
                    "Added gravity field at ({}, {}) strength {}",
                    x, y, strength
                ));
            }
            "vortex" => {
                if args.len() < 4 {
                    self.error("Usage: field vortex <x> <y> <strength>".to_string());
                    return;
                }
                let x = args[1].parse::<f32>().unwrap_or(0.0);
                let y = args[2].parse::<f32>().unwrap_or(0.0);
                let strength = args[3].parse::<f32>().unwrap_or(5.0);
                let field = ForceField::Vortex {
                    center: Vec3::new(x, y, 0.0),
                    axis: Vec3::Z,
                    strength,
                    radius: 10.0,
                };
                let id = engine.add_field(field);
                self.debug_field_ids.push(id);
                self.success(format!(
                    "Added vortex field at ({}, {}) strength {}",
                    x, y, strength
                ));
            }
            "clear" => {
                let count = self.debug_field_ids.len();
                for id in self.debug_field_ids.drain(..) {
                    engine.remove_field(id);
                }
                self.success(format!("Removed {} debug fields", count));
            }
            _ => {
                self.error(format!(
                    "Unknown field type '{}'. Valid: gravity, vortex, clear",
                    args[0]
                ));
            }
        }
    }

    fn execute_particles(&mut self, args: &[&str], engine: &mut ProofEngine) {
        if args.is_empty() || args[0] != "burst" {
            self.error("Usage: particles burst <x> <y> <count>".to_string());
            return;
        }

        if args.len() < 4 {
            self.error("Usage: particles burst <x> <y> <count>".to_string());
            return;
        }

        let x = args[1].parse::<f32>().unwrap_or(0.0);
        let y = args[2].parse::<f32>().unwrap_or(0.0);
        let count = args[3].parse::<usize>().unwrap_or(50);
        let count = count.min(500); // Safety cap

        // Spawn particle burst as individual glyphs with random velocities
        let particle_chars = ['*', '.', '+', '\u{00B7}', '\u{2022}'];
        for i in 0..count {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
            let speed = 2.0 + (i % 7) as f32 * 0.5;
            let ch = particle_chars[i % particle_chars.len()];
            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(x, y, 0.0),
                velocity: Vec3::new(angle.cos() * speed, angle.sin() * speed, 0.0),
                color: Vec4::new(1.0, 0.8 - (i % 5) as f32 * 0.1, 0.2, 1.0),
                emission: 0.8,
                lifetime: 1.5 + (i % 3) as f32 * 0.3,
                layer: RenderLayer::Particle,
                blend_mode: BlendMode::Additive,
                ..Default::default()
            });
        }

        self.success(format!("Burst {} particles at ({}, {})", count, x, y));
    }

    fn execute_weather(&mut self, weather_type: &str, engine: &mut ProofEngine) {
        match weather_type {
            "lightning" => {
                // Flash effect: spawn a bright screen-covering glyph that fades
                engine.spawn_glyph(Glyph {
                    character: '\u{2588}',
                    position: Vec3::new(0.0, 0.0, 5.0),
                    color: Vec4::new(0.9, 0.95, 1.0, 0.8),
                    emission: 3.0,
                    lifetime: 0.15,
                    layer: RenderLayer::Overlay,
                    blend_mode: BlendMode::Additive,
                    scale: Vec2::splat(100.0),
                    ..Default::default()
                });
                engine.add_trauma(0.4);
                self.success("Lightning flash triggered".to_string());
            }
            "rain" => {
                // Spawn rain particles (downward-moving dots)
                for i in 0..80 {
                    let x = (i as f32 / 80.0) * 44.0 - 22.0;
                    let y = 15.0 + (i % 7) as f32 * 2.0;
                    engine.spawn_glyph(Glyph {
                        character: '|',
                        position: Vec3::new(x, y, 1.0),
                        velocity: Vec3::new(0.0, -8.0 - (i % 5) as f32 * 1.0, 0.0),
                        color: Vec4::new(0.4, 0.5, 0.8, 0.4),
                        emission: 0.1,
                        lifetime: 3.0 + (i % 4) as f32,
                        layer: RenderLayer::Particle,
                        blend_mode: BlendMode::Normal,
                        scale: Vec2::new(0.3, 1.5),
                        ..Default::default()
                    });
                }
                self.success("Rain toggled".to_string());
            }
            _ => {
                self.error(format!(
                    "Unknown weather type '{}'. Valid: lightning, rain",
                    weather_type
                ));
            }
        }
    }

    fn execute_music_vibe(&mut self, vibe_name: &str, engine: &mut ProofEngine) {
        let vibe = match vibe_name.to_lowercase().as_str() {
            "combat" => Some(MusicVibe::Combat),
            "boss" => Some(MusicVibe::BossFight),
            "exploration" => Some(MusicVibe::Exploration),
            "title" => Some(MusicVibe::Title),
            "shop" => Some(MusicVibe::Exploration),   // map to closest available
            "shrine" => Some(MusicVibe::Exploration),  // map to closest available
            "chaos" => Some(MusicVibe::Combat),        // map to closest available
            _ => None,
        };

        if let Some(v) = vibe {
            engine.emit_audio(AudioEvent::SetMusicVibe(v));
            self.success(format!("Music vibe: {}", vibe_name));
        } else {
            self.error(format!(
                "Unknown vibe '{}'. Valid: {:?}",
                vibe_name, MUSIC_VIBES
            ));
        }
    }

    fn execute_timeline(&mut self, timeline_name: &str, state: &mut GameState) {
        match timeline_name.to_lowercase().as_str() {
            "death" => {
                state.death_cinematic_done = false;
                state.screen = crate::state::AppScreen::GameOver;
                self.success("Playing death timeline".to_string());
            }
            "victory" => {
                state.screen = crate::state::AppScreen::Victory;
                self.success("Playing victory timeline".to_string());
            }
            "boss_intro" => {
                state.boss_entrance_timer = 3.0;
                state.boss_entrance_name = "Debug Boss".to_string();
                self.success("Playing boss_intro timeline".to_string());
            }
            _ => {
                self.error(format!(
                    "Unknown timeline '{}'. Valid: {:?}",
                    timeline_name, TIMELINE_NAMES
                ));
            }
        }
    }

    fn execute_kill_all(&mut self, state: &mut GameState) {
        if let Some(ref mut enemy) = state.enemy {
            enemy.hp = 0;
            self.success("All enemies killed.".to_string());
        } else {
            self.warn("No enemies to kill.".to_string());
        }
        state.gauntlet_enemies.clear();
    }

    fn execute_help(&mut self) {
        self.blue_info("=== Debug Console Commands ===".to_string());
        self.info("  set corruption <N>      Set corruption level".to_string());
        self.info("  set hp <N>              Set player HP".to_string());
        self.info("  set mp <N>              Set player MP".to_string());
        self.info("  set floor <N>           Change floor (regenerates dungeon)".to_string());
        self.info("  spawn boss <name>       Start boss encounter".to_string());
        self.info("  spawn enemy <tier>      Spawn test enemy (tier 1-5)".to_string());
        self.info("  shader preset <name>    Switch shader graph preset".to_string());
        self.info("  field gravity <x> <y> <str>  Add gravity field".to_string());
        self.info("  field vortex <x> <y> <str>   Add vortex field".to_string());
        self.info("  field clear             Remove all debug fields".to_string());
        self.info("  particles burst <x> <y> <n>  Spawn particle burst".to_string());
        self.info("  weather lightning       Trigger lightning flash".to_string());
        self.info("  weather rain            Toggle rain particles".to_string());
        self.info("  music vibe <name>       Switch music vibe".to_string());
        self.info("  timeline play <name>    Play named timeline".to_string());
        self.info("  kill all                Kill all enemies".to_string());
        self.info("  god                     Toggle invincibility".to_string());
        self.info("  noclip                  Toggle walking through walls".to_string());
        self.info("  fps                     Toggle FPS counter".to_string());
        self.info("  help                    Show this help".to_string());
        self.info("  clear                   Clear console output".to_string());
    }

    /// Get the last command from history (for deferred execution).
    pub fn last_command(&self) -> Option<&str> {
        self.history.last().map(|s| s.as_str())
    }

    /// Render the console overlay.
    pub fn render(&self, engine: &ProofEngine) -> Vec<DebugGlyph> {
        if !self.visible {
            return Vec::new();
        }

        let mut out = Vec::with_capacity(512);

        let (ox, oy) = overlay_origin_from(engine);

        // Console takes the bottom half of the screen
        let console_x = ox;
        let console_y = oy + LINE_H * 2.0; // Start a couple lines below top
        let console_w = 80;
        let visible_lines = 20;

        // ── Semi-transparent background ──
        rect_glyphs(
            &mut out,
            console_x - CHAR_W,
            console_y,
            console_w + 2,
            visible_lines + 3,
            COLOR_CONSOLE_BG,
        );

        // ── Title bar ──
        text_glyphs(
            &mut out,
            " CHAOS RPG DEBUG CONSOLE [ESC to close] ",
            console_x,
            console_y,
            COLOR_CYAN,
        );

        // ── Output lines (show last N) ──
        let start_idx = if self.output_lines.len() > visible_lines {
            self.output_lines.len() - visible_lines
        } else {
            0
        };

        for (i, (text, color)) in self.output_lines.iter().skip(start_idx).enumerate() {
            let line_y = console_y + LINE_H * (i as f32 + 1.5);
            let color_vec = Vec4::new(color[0], color[1], color[2], color[3]);
            // Truncate long lines
            let display_text: String = text.chars().take(console_w).collect();
            text_glyphs(&mut out, &display_text, console_x, line_y, color_vec);
        }

        // ── Input line ──
        let input_y = console_y + LINE_H * (visible_lines as f32 + 1.5);
        text_glyphs(&mut out, "> ", console_x, input_y, COLOR_GREEN);

        let input_display: String = self.input_buffer.chars().take(console_w - 3).collect();
        text_glyphs(
            &mut out,
            &input_display,
            console_x + CHAR_W * 2.0,
            input_y,
            COLOR_CONSOLE_INPUT,
        );

        // ── Cursor (blinking) ──
        let blink = (engine.scene.time * 3.0).sin() > 0.0;
        if blink {
            let cursor_x = console_x + CHAR_W * (2.0 + self.cursor_pos as f32);
            out.push(DebugGlyph::ui('\u{2588}', cursor_x, input_y, Vec4::new(1.0, 1.0, 1.0, 0.7)));
        }

        // ── Autocomplete dropdown ──
        if !self.autocomplete_suggestions.is_empty() {
            let ac_y = input_y + LINE_H;
            let ac_count = self.autocomplete_suggestions.len().min(MAX_AUTOCOMPLETE);

            // Background for autocomplete
            rect_glyphs(
                &mut out,
                console_x + CHAR_W,
                ac_y,
                40,
                ac_count,
                COLOR_AUTOCOMPLETE_BG,
            );

            for (i, suggestion) in self.autocomplete_suggestions.iter().take(ac_count).enumerate() {
                let sy = ac_y + LINE_H * i as f32;
                let is_selected = i == self.autocomplete_index % ac_count;

                if is_selected {
                    // Highlight bar
                    rect_glyphs(
                        &mut out,
                        console_x + CHAR_W,
                        sy,
                        40,
                        1,
                        COLOR_AUTOCOMPLETE_HL,
                    );
                }

                let color = if is_selected { COLOR_WHITE } else { COLOR_DIM_WHITE };
                text_glyphs(&mut out, suggestion, console_x + CHAR_W * 2.0, sy, color);
            }
        }

        out
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// DebugMode — top-level state
// ═════════════════════════════════════════════════════════════════════════════

/// Top-level debug mode state.
pub struct DebugMode {
    /// Whether debug mode is enabled (F12 toggle).
    pub enabled: bool,
    /// Currently active overlay.
    pub active_overlay: DebugOverlayKind,
    /// The developer console.
    pub console: DebugConsole,
}

impl DebugMode {
    pub fn new() -> Self {
        Self {
            enabled: false,
            active_overlay: DebugOverlayKind::None,
            console: DebugConsole::new(),
        }
    }

    /// Toggle debug mode on/off (F12).
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        if !self.enabled {
            self.active_overlay = DebugOverlayKind::None;
            self.console.visible = false;
        }
    }

    /// Set the active overlay by F-key index (1-5).
    pub fn set_overlay(&mut self, key_index: u8) {
        let new_overlay = DebugOverlayKind::from_fkey(key_index);
        if self.active_overlay == new_overlay {
            // Toggle off if pressing same key
            self.active_overlay = DebugOverlayKind::None;
        } else {
            self.active_overlay = new_overlay;
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// DebugToolsManager — owns all debug systems
// ═════════════════════════════════════════════════════════════════════════════

/// Central manager for all debug tools. Owns overlays, console, and the
/// top-level debug mode toggle.
pub struct DebugToolsManager {
    pub mode: DebugMode,
    pub profiler: ProfilerOverlay,
    pub force_fields: ForceFieldOverlay,
    pub inspector: EntityInspector,
    pub lighting: LightingOverlay,
    pub shader_graph: ShaderGraphOverlay,
    /// Pending command to execute (set when Enter is pressed in console).
    pending_command: Option<String>,
}

impl DebugToolsManager {
    pub fn new() -> Self {
        Self {
            mode: DebugMode::new(),
            profiler: ProfilerOverlay::new(),
            force_fields: ForceFieldOverlay::new(),
            inspector: EntityInspector::new(),
            lighting: LightingOverlay::new(),
            shader_graph: ShaderGraphOverlay::new(),
            pending_command: None,
        }
    }

    /// Handle input. Returns true if input was consumed by debug tools
    /// (meaning the game should NOT process it).
    pub fn handle_input(&mut self, engine: &ProofEngine) -> bool {
        // F12: toggle debug mode
        if engine.input.just_pressed(Key::F12) {
            self.mode.toggle();
            return true;
        }

        // If debug mode is off, nothing else to handle
        if !self.mode.enabled {
            return false;
        }

        // Backtick: toggle console
        if engine.input.just_pressed(Key::Backtick) {
            self.mode.console.toggle();
            return true;
        }

        // If console is visible, it consumes all input
        if self.mode.console.visible {
            // Check if Enter was pressed to extract the command before handle_input
            let enter_pressed = engine.input.just_pressed(Key::Enter);
            let cmd = if enter_pressed && !self.mode.console.input_buffer.is_empty() {
                Some(self.mode.console.input_buffer.clone())
            } else {
                None
            };

            self.mode.console.handle_input(engine);

            if let Some(cmd) = cmd {
                self.pending_command = Some(cmd);
            }

            return true;
        }

        // F1-F5: switch overlays
        if engine.input.just_pressed(Key::F1) {
            self.mode.set_overlay(1);
            return true;
        }
        if engine.input.just_pressed(Key::F2) {
            self.mode.set_overlay(2);
            return true;
        }
        if engine.input.just_pressed(Key::F3) {
            self.mode.set_overlay(3);
            return true;
        }
        if engine.input.just_pressed(Key::F4) {
            self.mode.set_overlay(4);
            return true;
        }
        if engine.input.just_pressed(Key::F5) {
            self.mode.set_overlay(5);
            return true;
        }

        // Entity inspector: handle entity cycling when active
        if self.mode.active_overlay == DebugOverlayKind::EntityInspector {
            self.inspector.handle_input(engine);
            // Only consume arrow keys when inspector is active
            if engine.input.just_pressed(Key::Left)
                || engine.input.just_pressed(Key::Right)
                || engine.input.just_pressed(Key::Up)
                || engine.input.just_pressed(Key::Down)
            {
                return true;
            }
        }

        false
    }

    /// Tick all active overlays. Also executes any pending console command.
    pub fn update(&mut self, dt: f32, state: &mut GameState, engine: &mut ProofEngine) {
        if !self.mode.enabled {
            return;
        }

        // Execute pending console command
        if let Some(cmd) = self.pending_command.take() {
            self.mode.console.execute(&cmd, state, engine);
        }

        // Always update the profiler (lightweight)
        self.profiler.update(dt, engine);

        // Update the active overlay
        match self.mode.active_overlay {
            DebugOverlayKind::ForceFields => {
                self.force_fields.update(dt, engine, state);
            }
            DebugOverlayKind::EntityInspector => {
                self.inspector.update(engine, state);
            }
            DebugOverlayKind::Lighting => {
                self.lighting.update(engine);
            }
            DebugOverlayKind::ShaderGraph => {
                self.shader_graph.update(state);
            }
            _ => {}
        }

        // Apply god mode
        if self.mode.console.god_mode {
            if let Some(ref mut player) = state.player {
                player.current_hp = player.max_hp;
            }
        }
    }

    /// Render all active debug overlays. Returns glyphs to be submitted to the engine.
    pub fn render(&self, engine: &ProofEngine) -> Vec<DebugGlyph> {
        if !self.mode.enabled {
            return Vec::new();
        }

        let mut out = Vec::with_capacity(1024);

        let (ox, oy) = overlay_origin(engine);

        // ── Debug mode indicator ──
        text_glyphs_glow(
            &mut out,
            "[DEBUG MODE]",
            ox,
            oy,
            COLOR_CYAN,
            0.8,
        );
        let overlay_label = format!("Overlay: {}", self.mode.active_overlay.label());
        text_glyphs(&mut out, &overlay_label, ox + CHAR_W * 14.0, oy, COLOR_DIM_WHITE);

        // ── FPS counter (always shown in debug mode, or when console fps is on) ──
        if self.mode.console.show_fps || self.mode.active_overlay == DebugOverlayKind::Profiler {
            let fps_text = format!("{:.0} FPS", self.profiler.smoothed_fps);
            let fps_color = self.profiler.fps_color();
            // Top-right corner
            let right_x = ox + 35.0;
            text_glyphs(&mut out, &fps_text, right_x, oy, fps_color);
        }

        // ── Active overlay ──
        let overlay_y = oy + LINE_H * 2.0;
        match self.mode.active_overlay {
            DebugOverlayKind::Profiler => {
                out.extend(self.profiler.render(ox, overlay_y));
            }
            DebugOverlayKind::ForceFields => {
                out.extend(self.force_fields.render(engine));
            }
            DebugOverlayKind::EntityInspector => {
                out.extend(self.inspector.render(engine));
            }
            DebugOverlayKind::Lighting => {
                out.extend(self.lighting.render(engine));
            }
            DebugOverlayKind::ShaderGraph => {
                out.extend(self.shader_graph.render(engine));
            }
            DebugOverlayKind::None => {}
        }

        // ── Console (rendered on top of everything) ──
        if self.mode.console.visible {
            out.extend(self.mode.console.render(engine));
        }

        // ── Cheat indicators ──
        let mut cheat_x = ox;
        let cheat_y = oy + LINE_H;
        if self.mode.console.god_mode {
            text_glyphs_glow(&mut out, "[GOD]", cheat_x, cheat_y, COLOR_YELLOW, 1.0);
            cheat_x += CHAR_W * 6.0;
        }
        if self.mode.console.noclip {
            text_glyphs_glow(&mut out, "[NOCLIP]", cheat_x, cheat_y, COLOR_MAGENTA, 1.0);
            cheat_x += CHAR_W * 9.0;
        }
        let _ = cheat_x; // suppress unused warning

        out
    }

    /// Returns true when the console is open, meaning game input should be blocked.
    pub fn is_consuming_input(&self) -> bool {
        self.mode.enabled && self.mode.console.visible
    }

    /// Submit all debug glyphs to the engine for rendering.
    pub fn submit_to_engine(&self, engine: &mut ProofEngine) {
        let glyphs = self.render(engine);
        for dg in glyphs {
            engine.spawn_glyph(dg.to_glyph());
        }
    }
}
