//! Floor navigation / world map screen.
//!
//! Vertical floor list with biome info, difficulty, completion, boss markers,
//! floor preview, lore snippets, and ambient particles.

use proof_engine::prelude::*;
use crate::state::GameState;
use crate::theme::{Theme, THEMES};
use crate::ui_render;
use super::{color_lerp, rgb, rgb_a};

// ═══════════════════════════════════════════════════════════════════════════════
// FLOOR DESCRIPTORS
// ═══════════════════════════════════════════════════════════════════════════════

/// Get biome icon character for a floor range.
fn biome_icon(floor: u32) -> char {
    match floor_biome_name(floor) {
        "Ruins" => '\u{25A0}',
        "Crypt" => '\u{2620}',
        "Library" => '\u{2261}',
        "Forge" => '\u{2666}',
        "Garden" => '\u{2663}',
        "Void" => '\u{2726}',
        "Chaos" => '?',
        "Abyss" => '\u{2605}',
        "Cathedral" => '\u{2660}',
        "Laboratory" => '+',
        _ => '.',
    }
}

/// Approximate biome name from floor number (matches proof-engine generation).
fn floor_biome_name(floor: u32) -> &'static str {
    match floor {
        1..=10 => "Ruins",
        11..=20 => "Crypt",
        21..=30 => "Library",
        31..=40 => "Forge",
        41..=50 => "Garden",
        51..=60 => "Void",
        61..=70 => "Chaos",
        71..=80 => "Abyss",
        81..=90 => "Cathedral",
        91..=100 => "Laboratory",
        _ => "Void",
    }
}

/// Biome accent color for background particles.
fn biome_accent_color(floor: u32) -> Vec4 {
    match floor_biome_name(floor) {
        "Ruins" => rgb(140, 120, 100),
        "Crypt" => rgb(90, 90, 120),
        "Library" => rgb(180, 140, 80),
        "Forge" => rgb(255, 120, 40),
        "Garden" => rgb(80, 180, 60),
        "Void" => rgb(120, 60, 200),
        "Chaos" => rgb(220, 50, 80),
        "Abyss" => rgb(40, 40, 80),
        "Cathedral" => rgb(220, 200, 100),
        "Laboratory" => rgb(80, 200, 180),
        _ => rgb(128, 128, 128),
    }
}

/// Difficulty in stars (1-5) based on floor.
fn difficulty_stars(floor: u32) -> u32 {
    match floor {
        1..=10 => 1,
        11..=25 => 2,
        26..=50 => 3,
        51..=75 => 4,
        76..=100 => 5,
        _ => 5,
    }
}

/// Whether this floor is a boss floor.
fn is_boss_floor(floor: u32) -> bool {
    floor == 25 || floor == 50 || floor == 75 || floor == 100
}

/// Lore snippet for a floor range.
fn lore_snippet(floor: u32) -> &'static str {
    match floor {
        1..=10 => "The old ruins crumble under forgotten equations...",
        11..=20 => "Bones of mathematicians line the corridors.",
        21..=30 => "Infinite shelves hold theorems never proven.",
        31..=40 => "The forge burns with the heat of computation.",
        41..=50 => "Nature reclaims what logic abandoned.",
        51..=60 => "Reality thins. The void whispers axioms.",
        61..=70 => "Chaos reigns. Every step rewrites the rules.",
        71..=80 => "The abyss below holds no bottom, only recursion.",
        81..=90 => "Golden arches frame the final proofs.",
        91..=100 => "At the apex, creation and destruction are one.",
        _ => "Beyond the known...",
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WORLD MAP STATE
// ═══════════════════════════════════════════════════════════════════════════════

/// Persistent state for the world map screen.
pub struct WorldMap {
    /// Which floor entry is currently highlighted.
    pub cursor: u32,
    /// Scroll offset for the floor list.
    pub scroll_offset: f32,
    /// Target scroll offset (for smooth scrolling).
    pub target_scroll: f32,
    /// Accumulated time for animations.
    pub time: f32,
    /// Staircase descent animation timer (>0 while descending).
    pub descent_timer: f32,
    /// Floor we are descending FROM.
    pub descent_from: u32,
    /// Floor we are descending TO.
    pub descent_to: u32,
}

impl WorldMap {
    pub fn new() -> Self {
        Self {
            cursor: 1,
            scroll_offset: 0.0,
            target_scroll: 0.0,
            time: 0.0,
            descent_timer: 0.0,
            descent_from: 0,
            descent_to: 0,
        }
    }

    /// Update world map logic (input, animations). Returns true if a floor was selected.
    pub fn update(
        &mut self,
        state: &GameState,
        engine: &mut ProofEngine,
        dt: f32,
    ) -> Option<WorldMapAction> {
        self.time += dt;

        // Descent animation
        if self.descent_timer > 0.0 {
            self.descent_timer = (self.descent_timer - dt).max(0.0);
            if self.descent_timer <= 0.0 {
                return Some(WorldMapAction::DescendTo(self.descent_to));
            }
            return None;
        }

        let key_up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
        let key_down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);
        let key_enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);
        let key_esc = engine.input.just_pressed(Key::Escape);

        let max_floor = state.floor_num.max(1);

        if key_up && self.cursor > 1 {
            self.cursor -= 1;
        }
        if key_down && self.cursor < max_floor.min(100) {
            self.cursor += 1;
        }
        if key_enter {
            if self.cursor != state.floor_num {
                // Start descent animation
                self.descent_from = state.floor_num;
                self.descent_to = self.cursor;
                self.descent_timer = 0.8;
            } else {
                return Some(WorldMapAction::EnterFloor(self.cursor));
            }
        }
        if key_esc {
            return Some(WorldMapAction::Back);
        }

        // Smooth scroll to keep cursor in view
        self.target_scroll = (self.cursor as f32 - 5.0).max(0.0);
        let lerp = 1.0 - (1.0 - 0.1_f32).powf(dt * 60.0);
        self.scroll_offset += (self.target_scroll - self.scroll_offset) * lerp;

        None
    }

    /// Render the world map screen.
    pub fn render(
        &self,
        state: &GameState,
        engine: &mut ProofEngine,
    ) {
        let theme = &THEMES[state.theme_idx % THEMES.len()];
        let max_floor = state.floor_num.max(1);

        // ── Background particles (biome-specific) ───────────────────────────
        self.render_ambient_particles(engine, theme);

        // ── Descent animation overlay ───────────────────────────────────────
        if self.descent_timer > 0.0 {
            self.render_descent_animation(engine, theme);
            return;
        }

        // ── Header ──────────────────────────────────────────────────────────
        ui_render::heading_centered(engine, "DUNGEON MAP", 5.0, theme.heading);

        // Progress indicator
        let progress_label = format!(
            "Depth: {}/100  {}",
            max_floor,
            if is_boss_floor(max_floor) { "[BOSS FLOOR]" } else { "" }
        );
        ui_render::small(engine, &progress_label, -6.0, 4.3, theme.accent);

        // Boss floor markers
        let boss_bar = format!(
            "Bosses: {} {} {} {}",
            if max_floor >= 25 { "[25]" } else { " 25 " },
            if max_floor >= 50 { "[50]" } else { " 50 " },
            if max_floor >= 75 { "[75]" } else { " 75 " },
            if max_floor >= 100 { "[100]" } else { " 100" },
        );
        ui_render::small(engine, &boss_bar, -5.0, 3.8, theme.dim);

        // ── Floor list (left side) ──────────────────────────────────────────
        let visible_count = 10_u32;
        let start = (self.scroll_offset as u32).max(1);
        let end = (start + visible_count).min(max_floor + 1).min(101);

        for floor in start..end {
            let i = (floor - start) as f32;
            let y = 3.0 - i * 0.7;
            let is_selected = floor == self.cursor;
            let is_current = floor == state.floor_num;

            // Floor entry
            let icon = biome_icon(floor);
            let stars = difficulty_stars(floor);
            let star_str: String = (0..stars).map(|_| '*').collect();
            let boss_marker = if is_boss_floor(floor) { " [BOSS]" } else { "" };
            let status = if floor < state.floor_num {
                " (cleared)"
            } else if floor == state.floor_num {
                " <-- YOU"
            } else {
                ""
            };

            let label = format!(
                "{} {} F{:>3} {} {}{}",
                if is_selected { ">" } else { " " },
                icon,
                floor,
                star_str,
                floor_biome_name(floor),
                boss_marker,
            );

            let color = if is_selected {
                theme.selected
            } else if is_current {
                theme.accent
            } else if is_boss_floor(floor) {
                theme.danger
            } else if floor < state.floor_num {
                theme.dim
            } else {
                theme.primary
            };

            let emission = if is_selected { 0.7 } else if is_current { 0.5 } else { 0.3 };
            ui_render::text(engine, &label, -8.5, y, color, 0.3, emission);

            // Status suffix
            if !status.is_empty() {
                let status_x = -8.5 + label.len() as f32 * 0.255 + 0.2;
                ui_render::text(engine, status, status_x, y, theme.muted, 0.22, 0.2);
            }
        }

        // ── Floor preview (right side) ──────────────────────────────────────
        self.render_floor_preview(state, engine, theme);

        // ── Lore snippet ────────────────────────────────────────────────────
        let lore = lore_snippet(self.cursor);
        ui_render::small(engine, lore, -8.0, -3.8, theme.dim);

        // ── Controls ────────────────────────────────────────────────────────
        ui_render::small(engine, "[Up/Down] Navigate  [Enter] Go  [Esc] Back", -8.0, -4.5, theme.muted);
    }

    // ── Floor preview ───────────────────────────────────────────────────────

    fn render_floor_preview(
        &self,
        state: &GameState,
        engine: &mut ProofEngine,
        theme: &Theme,
    ) {
        // Small tile preview of the current floor layout
        let bridge = match &state.dungeon_bridge {
            Some(b) => b,
            None => {
                // No bridge: show placeholder
                ui_render::text(engine, "No map data", 2.5, 2.0, theme.dim, 0.3, 0.2);
                return;
            }
        };

        // Only show preview if cursor matches current floor
        if self.cursor != state.floor_num {
            // Show biome info instead
            let biome = floor_biome_name(self.cursor);
            let accent = biome_accent_color(self.cursor);
            ui_render::text(engine, biome, 3.0, 3.0, accent, 0.45, 0.6);
            let stars = difficulty_stars(self.cursor);
            let star_label = format!("Difficulty: {}", "*".repeat(stars as usize));
            ui_render::small(engine, &star_label, 3.0, 2.3, theme.primary);
            if is_boss_floor(self.cursor) {
                ui_render::text(engine, "!! BOSS FLOOR !!", 3.0, 1.6, theme.danger, 0.35, 0.8);
            }
            return;
        }

        let grid = match bridge.get_current_map() {
            Some(g) => g,
            None => return,
        };

        // Render a small-scale version of the tile grid
        let preview_x = 2.5_f32;
        let preview_y = 3.0_f32;
        let preview_scale = 0.06_f32;

        // Sample every N tiles for the preview
        let step = (grid.width / 40).max(1);
        for ty in (0..grid.height).step_by(step) {
            for tx in (0..grid.width).step_by(step) {
                let tile = grid.get(tx as i32, ty as i32);
                if tile.visibility == crate::dungeon_bridge::TileVisibility::Unseen { continue; }

                let px = preview_x + tx as f32 * preview_scale;
                let py = preview_y - ty as f32 * preview_scale;

                let color = match tile.tile_type {
                    crate::dungeon_bridge::TileBridge::Wall => theme.dim,
                    crate::dungeon_bridge::TileBridge::Floor
                    | crate::dungeon_bridge::TileBridge::Corridor => theme.primary,
                    crate::dungeon_bridge::TileBridge::Door => rgb(160, 100, 40),
                    crate::dungeon_bridge::TileBridge::StairsDown => rgb(100, 200, 255),
                    crate::dungeon_bridge::TileBridge::Chest => rgb(255, 200, 50),
                    crate::dungeon_bridge::TileBridge::Shrine => theme.accent,
                    crate::dungeon_bridge::TileBridge::Water => rgb(60, 120, 220),
                    crate::dungeon_bridge::TileBridge::Lava => rgb(255, 100, 20),
                    _ => theme.muted,
                };

                engine.spawn_glyph(Glyph {
                    character: '\u{25A0}',
                    position: Vec3::new(px, py, 0.5),
                    scale: Vec2::splat(preview_scale * 0.9),
                    color,
                    emission: 0.2,
                    layer: RenderLayer::UI,
                    ..Default::default()
                });
            }
        }

        // Player dot on preview
        let (ppx, ppy) = grid.player_start;
        let dot_x = preview_x + ppx as f32 * preview_scale;
        let dot_y = preview_y - ppy as f32 * preview_scale;
        let pulse = (self.time * 3.0).sin() * 0.3 + 0.7;
        engine.spawn_glyph(Glyph {
            character: '\u{25CF}',
            position: Vec3::new(dot_x, dot_y, 1.0),
            scale: Vec2::splat(preview_scale * 2.0),
            color: rgb_a(255, 255, 255, pulse),
            emission: pulse,
            layer: RenderLayer::UI,
            ..Default::default()
        });

        // Preview label
        ui_render::text(engine, "PREVIEW", preview_x, preview_y + 1.2, theme.dim, 0.22, 0.3);
    }

    // ── Descent animation ───────────────────────────────────────────────────

    fn render_descent_animation(
        &self,
        engine: &mut ProofEngine,
        theme: &Theme,
    ) {
        let t = 1.0 - self.descent_timer / 0.8;
        let t = t.clamp(0.0, 1.0);

        // Staircase visual: rows of '>' descending
        let num_steps = 12;
        for i in 0..num_steps {
            let step_t = (t * num_steps as f32 - i as f32).clamp(0.0, 1.0);
            let x = -2.0 + i as f32 * 0.4;
            let y = 3.0 - i as f32 * 0.6 - step_t * 0.3;
            let alpha = if step_t > 0.0 { 1.0 } else { 0.2 };
            let color = color_lerp(theme.dim, theme.accent, step_t);
            let color = Vec4::new(color.x, color.y, color.z, alpha);
            engine.spawn_glyph(Glyph {
                character: '>',
                position: Vec3::new(x, y, 0.0),
                scale: Vec2::splat(0.5),
                color,
                emission: step_t * 0.6,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }

        // Floor label
        let interp_floor = self.descent_from as f32 + (self.descent_to as f32 - self.descent_from as f32) * t;
        let label = format!("Descending to Floor {}...", interp_floor as u32);
        ui_render::text_centered(engine, &label, -2.0, theme.heading, 0.45, 0.7);

        // Biome name of target
        let target_biome = floor_biome_name(self.descent_to);
        ui_render::text_centered(engine, target_biome, -3.0, biome_accent_color(self.descent_to), 0.6, 0.5);
    }

    // ── Ambient particles ───────────────────────────────────────────────────

    fn render_ambient_particles(
        &self,
        engine: &mut ProofEngine,
        _theme: &Theme,
    ) {
        let accent = biome_accent_color(self.cursor);
        let particle_count = 20;

        for i in 0..particle_count {
            let seed = i as f32 * 1.618;
            let x = ((seed * 7.3 + self.time * 0.1).sin() * 9.0).clamp(-8.5, 8.5);
            let y = ((seed * 3.7 + self.time * 0.15).cos() * 6.0).clamp(-5.0, 5.0);
            let alpha = ((seed * 11.0 + self.time * 0.5).sin() * 0.5 + 0.5) * 0.15;

            engine.spawn_glyph(Glyph {
                character: '\u{00B7}',
                position: Vec3::new(x, y, -5.0),
                scale: Vec2::splat(0.3),
                color: Vec4::new(accent.x, accent.y, accent.z, alpha),
                emission: alpha * 0.3,
                layer: RenderLayer::World,
                ..Default::default()
            });
        }
    }
}

/// Actions the world map can produce.
pub enum WorldMapAction {
    /// Enter the specified floor (already on it).
    EnterFloor(u32),
    /// Descend/travel to a different floor.
    DescendTo(u32),
    /// Go back to previous screen.
    Back,
}
