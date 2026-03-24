//! Room event visual system.
//!
//! Renders rich visuals for each room type: shop, shrine, trap, puzzle, rest,
//! treasure, library, forge, chaos rift, and secret rooms.
//! Each room type has unique entity rendering, particle effects, and UI.

use proof_engine::prelude::*;
use crate::state::GameState;
use crate::theme::{Theme, THEMES};
use crate::dungeon_bridge::{BridgeItemKind, RoomInfo};
use crate::ui_render;
use super::{color_lerp, rgb, rgb_a, dim};

// ═══════════════════════════════════════════════════════════════════════════════
// ROOM EVENT RENDERER
// ═══════════════════════════════════════════════════════════════════════════════

/// Persistent state for room event animations.
pub struct RoomEventRenderer {
    /// Accumulated time for animations.
    pub time: f32,
    /// Animation phase for room-specific sequences.
    pub anim_phase: f32,
    /// Whether the current room event animation is complete.
    pub anim_complete: bool,
    /// Chest open animation progress (0..1).
    pub chest_open: f32,
    /// Trap trigger animation progress (0..1).
    pub trap_trigger: f32,
    /// Wall crumble animation for secret rooms (0..1).
    pub crumble_progress: f32,
    /// Campfire flicker seed (randomized on entry).
    pub fire_seed: f32,
}

impl RoomEventRenderer {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            anim_phase: 0.0,
            anim_complete: false,
            chest_open: 0.0,
            trap_trigger: 0.0,
            crumble_progress: 0.0,
            fire_seed: 0.0,
        }
    }

    /// Reset animations for a new room event.
    pub fn reset(&mut self) {
        self.anim_phase = 0.0;
        self.anim_complete = false;
        self.chest_open = 0.0;
        self.trap_trigger = 0.0;
        self.crumble_progress = 0.0;
        self.fire_seed = self.time; // randomize campfire
    }

    /// Update timers. Call each frame.
    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        self.anim_phase += dt;

        // Auto-advance animations
        if self.chest_open < 1.0 && self.chest_open > 0.0 {
            self.chest_open = (self.chest_open + dt * 2.0).min(1.0);
        }
        if self.trap_trigger < 1.0 && self.trap_trigger > 0.0 {
            self.trap_trigger = (self.trap_trigger + dt * 1.5).min(1.0);
        }
        if self.crumble_progress < 1.0 && self.crumble_progress > 0.0 {
            self.crumble_progress = (self.crumble_progress + dt * 1.0).min(1.0);
        }
    }

    /// Start chest open animation.
    pub fn trigger_chest_open(&mut self) {
        if self.chest_open == 0.0 { self.chest_open = 0.01; }
    }

    /// Start trap trigger animation.
    pub fn trigger_trap(&mut self) {
        if self.trap_trigger == 0.0 { self.trap_trigger = 0.01; }
    }

    /// Start secret room crumble animation.
    pub fn trigger_crumble(&mut self) {
        if self.crumble_progress == 0.0 { self.crumble_progress = 0.01; }
    }

    /// Render room event visuals based on the room type determined from state.
    pub fn render(
        &self,
        state: &GameState,
        engine: &mut ProofEngine,
        room: Option<&RoomInfo>,
    ) {
        let theme = &THEMES[state.theme_idx % THEMES.len()];

        // Determine room type from event title or room info
        let room_type_hint = &state.room_event.title;

        if room_type_hint.contains("Shop") {
            self.render_shop(engine, theme, room);
        } else if room_type_hint.contains("Shrine") {
            self.render_shrine(engine, theme, room);
        } else if room_type_hint.contains("Trap") {
            self.render_trap(engine, theme);
        } else if room_type_hint.contains("Puzzle") {
            self.render_puzzle(engine, theme);
        } else if room_type_hint.contains("Rest") || room_type_hint.contains("Campfire") {
            self.render_rest(engine, theme);
        } else if room_type_hint.contains("Treasure") {
            self.render_treasure(engine, theme);
        } else if room_type_hint.contains("Library") {
            self.render_library(engine, theme);
        } else if room_type_hint.contains("Forge") {
            self.render_forge(engine, theme);
        } else if room_type_hint.contains("Chaos") || room_type_hint.contains("Rift") {
            self.render_chaos_rift(engine, theme);
        } else if room_type_hint.contains("Secret") {
            self.render_secret(engine, theme);
        } else {
            // Generic room: just show the title area
            self.render_generic(engine, theme);
        }
    }

    // ── Shop room ───────────────────────────────────────────────────────────

    fn render_shop(
        &self,
        engine: &mut ProofEngine,
        theme: &Theme,
        room: Option<&RoomInfo>,
    ) {
        // Merchant NPC entity
        let merchant_x = -2.0_f32;
        let merchant_y = 1.5_f32;
        let bob = (self.time * 1.5).sin() * 0.05;
        engine.spawn_glyph(Glyph {
            character: '$',
            position: Vec3::new(merchant_x, merchant_y + bob, 0.0),
            scale: Vec2::splat(0.8),
            color: rgb(80, 200, 80),
            emission: 0.6,
            layer: RenderLayer::UI,
            ..Default::default()
        });

        // "MERCHANT" label
        ui_render::text(engine, "MERCHANT", merchant_x - 1.5, merchant_y + 1.0, theme.success, 0.35, 0.5);

        // Item pedestals
        if let Some(room) = room {
            for (i, item) in room.items.iter().enumerate() {
                let pedestal_x = -4.0 + i as f32 * 2.5;
                let pedestal_y = -0.5;

                // Pedestal base
                engine.spawn_glyph(Glyph {
                    character: '\u{2550}',
                    position: Vec3::new(pedestal_x, pedestal_y - 0.3, 0.0),
                    scale: Vec2::splat(0.5),
                    color: theme.muted,
                    emission: 0.2,
                    layer: RenderLayer::UI,
                    ..Default::default()
                });

                // Item on pedestal
                let (ch, color) = match &item.kind {
                    BridgeItemKind::Merchant { items, price_mult } => {
                        let price_tag = format!("{}g x{:.0}%", items, price_mult * 100.0);
                        ui_render::small(engine, &price_tag, pedestal_x - 0.5, pedestal_y - 0.7, theme.gold);
                        ('?', theme.accent)
                    }
                    _ => ('.', theme.primary),
                };

                let item_float = (self.time * 2.0 + i as f32 * 1.5).sin() * 0.08;
                engine.spawn_glyph(Glyph {
                    character: ch,
                    position: Vec3::new(pedestal_x, pedestal_y + 0.3 + item_float, 0.0),
                    scale: Vec2::splat(0.5),
                    color,
                    emission: 0.5,
                    layer: RenderLayer::UI,
                    ..Default::default()
                });
            }
        }

        // Buy/sell hint
        ui_render::small(engine, "[B] Buy  [S] Sell  [Esc] Leave", -6.0, -3.5, theme.muted);
    }

    // ── Shrine room ─────────────────────────────────────────────────────────

    fn render_shrine(
        &self,
        engine: &mut ProofEngine,
        theme: &Theme,
        _room: Option<&RoomInfo>,
    ) {
        let shrine_x = 0.0_f32;
        let shrine_y = 1.0_f32;

        // Glowing shrine entity
        let glow = (self.time * 1.5).sin() * 0.3 + 0.7;
        engine.spawn_glyph(Glyph {
            character: '\u{2726}',
            position: Vec3::new(shrine_x, shrine_y, 0.0),
            scale: Vec2::splat(1.2),
            color: Vec4::new(theme.accent.x * glow, theme.accent.y * glow, theme.accent.z * glow, 1.0),
            emission: glow,
            layer: RenderLayer::UI,
            ..Default::default()
        });

        // Particle halo
        let halo_count = 8;
        for i in 0..halo_count {
            let angle = self.time * 0.8 + i as f32 * std::f32::consts::TAU / halo_count as f32;
            let radius = 1.5 + (self.time * 0.5 + i as f32).sin() * 0.3;
            let px = shrine_x + angle.cos() * radius;
            let py = shrine_y + angle.sin() * radius;
            let alpha = 0.3 + (self.time * 2.0 + i as f32).sin() * 0.2;
            engine.spawn_glyph(Glyph {
                character: '\u{00B7}',
                position: Vec3::new(px, py, 0.1),
                scale: Vec2::splat(0.3),
                color: Vec4::new(theme.accent.x, theme.accent.y, theme.accent.z, alpha),
                emission: alpha * 0.5,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }

        // "SHRINE" label
        ui_render::text_centered(engine, "SHRINE", shrine_y + 2.0, theme.accent, 0.5, 0.6);

        // Buff selection prompt
        ui_render::small(engine, "[1] Heal  [2] Buff  [3] Risk  [Esc] Leave", -7.0, -3.5, theme.muted);
    }

    // ── Trap room ───────────────────────────────────────────────────────────

    fn render_trap(
        &self,
        engine: &mut ProofEngine,
        theme: &Theme,
    ) {
        ui_render::text_centered(engine, "!! TRAP !!", 3.0, theme.danger, 0.6, 0.8);

        if self.trap_trigger > 0.0 {
            // Trap animation: different types based on anim phase
            let trap_type = ((self.fire_seed * 100.0) as u32) % 4;
            match trap_type {
                0 => self.render_trap_pendulum(engine, theme),
                1 => self.render_trap_spikes(engine, theme),
                2 => self.render_trap_arrows(engine, theme),
                _ => self.render_trap_flames(engine, theme),
            }
        } else {
            // Pre-trigger: ominous flicker
            let flicker = (self.time * 8.0).sin() * 0.3 + 0.7;
            for i in 0..5 {
                let x = -3.0 + i as f32 * 1.5;
                let alpha = flicker * (0.5 + (self.time * 3.0 + i as f32 * 2.0).sin() * 0.3);
                engine.spawn_glyph(Glyph {
                    character: '!',
                    position: Vec3::new(x, 0.0, 0.0),
                    scale: Vec2::splat(0.5),
                    color: Vec4::new(0.8, 0.2, 0.1, alpha),
                    emission: alpha * 0.4,
                    layer: RenderLayer::UI,
                    ..Default::default()
                });
            }
        }

        ui_render::small(engine, "[Enter] Continue", -3.0, -3.5, theme.muted);
    }

    fn render_trap_pendulum(&self, engine: &mut ProofEngine, _theme: &Theme) {
        let swing = (self.time * 4.0).sin();
        let x = swing * 3.0;
        let y = 0.0;
        engine.spawn_glyph(Glyph {
            character: '/',
            position: Vec3::new(x, y + 2.0, 0.0),
            scale: Vec2::splat(0.6),
            color: rgb(200, 200, 200),
            emission: 0.4,
            layer: RenderLayer::UI,
            ..Default::default()
        });
        engine.spawn_glyph(Glyph {
            character: '\u{25C6}',
            position: Vec3::new(x, y, 0.0),
            scale: Vec2::splat(0.7),
            color: rgb(180, 180, 180),
            emission: 0.5,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }

    fn render_trap_spikes(&self, engine: &mut ProofEngine, _theme: &Theme) {
        let rise = (self.trap_trigger * std::f32::consts::PI).sin();
        for i in 0..7 {
            let x = -3.0 + i as f32 * 1.0;
            let spike_y = -2.0 + rise * 3.0;
            engine.spawn_glyph(Glyph {
                character: '\u{25B2}',
                position: Vec3::new(x, spike_y, 0.0),
                scale: Vec2::splat(0.5),
                color: rgb(180, 180, 180),
                emission: 0.3 + rise * 0.3,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }
    }

    fn render_trap_arrows(&self, engine: &mut ProofEngine, _theme: &Theme) {
        let arrow_count = 5;
        for i in 0..arrow_count {
            let delay = i as f32 * 0.15;
            let progress = ((self.trap_trigger - delay).max(0.0) * 3.0).min(1.0);
            let x = -6.0 + progress * 12.0;
            let y = 1.5 - i as f32 * 0.6;
            let alpha = if progress > 0.0 && progress < 1.0 { 1.0 } else { 0.3 };
            engine.spawn_glyph(Glyph {
                character: '\u{2192}',
                position: Vec3::new(x, y, 0.0),
                scale: Vec2::splat(0.5),
                color: rgb_a(200, 200, 200, alpha),
                emission: alpha * 0.4,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }
    }

    fn render_trap_flames(&self, engine: &mut ProofEngine, _theme: &Theme) {
        let burst = self.trap_trigger;
        let flame_count = 12;
        for i in 0..flame_count {
            let angle = i as f32 * std::f32::consts::TAU / flame_count as f32 + self.time * 2.0;
            let radius = burst * 4.0;
            let x = angle.cos() * radius;
            let y = angle.sin() * radius;
            let fade = (1.0 - burst).max(0.0);
            let t = i as f32 / flame_count as f32;
            let color = color_lerp(rgb(255, 60, 20), rgb(255, 200, 40), t);
            engine.spawn_glyph(Glyph {
                character: '*',
                position: Vec3::new(x, y, 0.0),
                scale: Vec2::splat(0.4 + burst * 0.3),
                color: Vec4::new(color.x, color.y, color.z, fade),
                emission: fade * 0.7,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }
    }

    // ── Puzzle room ─────────────────────────────────────────────────────────

    fn render_puzzle(
        &self,
        engine: &mut ProofEngine,
        theme: &Theme,
    ) {
        ui_render::text_centered(engine, "PUZZLE", 3.5, theme.accent, 0.5, 0.6);

        // Puzzle board: 4x4 grid of blocks
        let board_x = -1.5_f32;
        let board_y = 1.5_f32;
        let cell_size = 0.7_f32;

        for row in 0..4 {
            for col in 0..4 {
                let x = board_x + col as f32 * cell_size;
                let y = board_y - row as f32 * cell_size;
                let idx = row * 4 + col;
                // Pattern: some blocks lit, some dark (based on time)
                let lit = (self.time * 0.5 + idx as f32 * 0.3).sin() > 0.0;
                let ch = if lit { '\u{25A0}' } else { '\u{25A1}' };
                let color = if lit { theme.accent } else { theme.dim };
                engine.spawn_glyph(Glyph {
                    character: ch,
                    position: Vec3::new(x, y, 0.0),
                    scale: Vec2::splat(cell_size * 0.8),
                    color,
                    emission: if lit { 0.5 } else { 0.1 },
                    layer: RenderLayer::UI,
                    ..Default::default()
                });
            }
        }

        ui_render::small(engine, "[Arrows] Move block  [Enter] Confirm  [Esc] Leave", -7.0, -3.5, theme.muted);
    }

    // ── Rest room ───────────────────────────────────────────────────────────

    fn render_rest(
        &self,
        engine: &mut ProofEngine,
        theme: &Theme,
    ) {
        ui_render::text_centered(engine, "CAMPFIRE", 3.5, rgb(255, 180, 60), 0.5, 0.7);

        let fire_x = 0.0_f32;
        let fire_y = 0.5_f32;

        // Campfire base
        engine.spawn_glyph(Glyph {
            character: 'A',
            position: Vec3::new(fire_x, fire_y - 0.5, 0.0),
            scale: Vec2::splat(0.5),
            color: rgb(100, 70, 40),
            emission: 0.2,
            layer: RenderLayer::UI,
            ..Default::default()
        });

        // Fire particles
        let particle_count = 15;
        for i in 0..particle_count {
            let seed = self.fire_seed + i as f32 * 2.7;
            let life = (self.time * 1.5 + seed) % 1.5;
            let rise = life * 2.0;
            let drift = (seed * 7.3 + self.time * 3.0).sin() * 0.3 * life;
            let alpha = (1.0 - life / 1.5).max(0.0);
            let t = life / 1.5;
            let color = color_lerp(rgb(255, 80, 20), rgb(255, 200, 60), t);
            let ch = if t < 0.5 { '*' } else { '\u{00B7}' };
            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(fire_x + drift, fire_y + rise, 0.1),
                scale: Vec2::splat(0.3 * (1.0 - t * 0.5)),
                color: Vec4::new(color.x, color.y, color.z, alpha),
                emission: alpha * 0.8,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }

        // Warm glow circle
        let glow_pulse = (self.time * 2.0).sin() * 0.1 + 0.4;
        engine.spawn_glyph(Glyph {
            character: '\u{25CF}',
            position: Vec3::new(fire_x, fire_y, -0.5),
            scale: Vec2::splat(3.0),
            color: rgb_a(255, 140, 40, glow_pulse * 0.15),
            emission: glow_pulse * 0.2,
            layer: RenderLayer::World,
            ..Default::default()
        });

        ui_render::small(engine, "[R] Rest (heal)  [C] Craft  [S] Save  [Esc] Leave", -7.0, -3.5, theme.muted);
    }

    // ── Treasure room ───────────────────────────────────────────────────────

    fn render_treasure(
        &self,
        engine: &mut ProofEngine,
        theme: &Theme,
    ) {
        ui_render::text_centered(engine, "TREASURE", 3.5, theme.gold, 0.5, 0.7);

        let chest_x = 0.0_f32;
        let chest_y = 1.0_f32;

        if self.chest_open < 0.01 {
            // Closed chest
            engine.spawn_glyph(Glyph {
                character: '\u{25A0}',
                position: Vec3::new(chest_x, chest_y, 0.0),
                scale: Vec2::splat(0.8),
                color: rgb(180, 140, 40),
                emission: 0.4,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        } else {
            // Open chest animation
            let open = self.chest_open;
            let lid_y = chest_y + open * 0.5;
            // Chest body
            engine.spawn_glyph(Glyph {
                character: '\u{25A1}',
                position: Vec3::new(chest_x, chest_y, 0.0),
                scale: Vec2::splat(0.8),
                color: rgb(180, 140, 40),
                emission: 0.4,
                layer: RenderLayer::UI,
                ..Default::default()
            });
            // Lid
            engine.spawn_glyph(Glyph {
                character: '\u{2550}',
                position: Vec3::new(chest_x, lid_y, 0.1),
                scale: Vec2::splat(0.6),
                color: rgb(200, 160, 60),
                emission: 0.5,
                layer: RenderLayer::UI,
                ..Default::default()
            });

            // Loot scatter
            if open > 0.5 {
                let scatter_t = (open - 0.5) * 2.0;
                let loot_count = 6;
                for i in 0..loot_count {
                    let angle = i as f32 * std::f32::consts::TAU / loot_count as f32;
                    let dist = scatter_t * 2.0;
                    let lx = chest_x + angle.cos() * dist;
                    let ly = chest_y + angle.sin() * dist + scatter_t * 0.5;
                    let sparkle = (self.time * 5.0 + i as f32 * 2.0).sin() * 0.3 + 0.7;
                    engine.spawn_glyph(Glyph {
                        character: '*',
                        position: Vec3::new(lx, ly, 0.2),
                        scale: Vec2::splat(0.3),
                        color: rgb_a(255, 220, 80, sparkle),
                        emission: sparkle * 0.6,
                        layer: RenderLayer::UI,
                        ..Default::default()
                    });
                }
            }
        }

        // Sparkle particles around chest
        for i in 0..8 {
            let seed = i as f32 * 3.14;
            let sparkle_life = (self.time * 2.0 + seed) % 2.0;
            let alpha = ((1.0 - sparkle_life / 2.0) * (sparkle_life * 3.0).min(1.0)).max(0.0);
            let sx = chest_x + (seed * 7.0 + self.time).sin() * 1.5;
            let sy = chest_y + sparkle_life * 0.5 + (seed * 3.0).cos() * 0.5;
            engine.spawn_glyph(Glyph {
                character: '\u{00B7}',
                position: Vec3::new(sx, sy, 0.1),
                scale: Vec2::splat(0.2),
                color: rgb_a(255, 200, 80, alpha),
                emission: alpha * 0.5,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }

        ui_render::small(engine, "[P] Pick up  [Enter] Continue", -5.0, -3.5, theme.muted);
    }

    // ── Library room ────────────────────────────────────────────────────────

    fn render_library(
        &self,
        engine: &mut ProofEngine,
        theme: &Theme,
    ) {
        ui_render::text_centered(engine, "LIBRARY", 3.5, rgb(200, 170, 100), 0.5, 0.6);

        // Bookshelf walls (left and right)
        for side in [-1.0_f32, 1.0] {
            let base_x = side * 5.0;
            for row in 0..5 {
                for col in 0..3 {
                    let x = base_x + col as f32 * 0.5;
                    let y = 2.0 - row as f32 * 0.6;
                    let h = ((row * 3 + col) as f32 * 7.3 + self.time * 0.01).sin();
                    let color = if h > 0.3 {
                        rgb(140, 100, 60)
                    } else {
                        rgb(100, 80, 50)
                    };
                    engine.spawn_glyph(Glyph {
                        character: '\u{2261}',
                        position: Vec3::new(x, y, -0.5),
                        scale: Vec2::splat(0.4),
                        color,
                        emission: 0.15,
                        layer: RenderLayer::UI,
                        ..Default::default()
                    });
                }
            }
        }

        // Scroll entity in center
        let scroll_bob = (self.time * 1.5).sin() * 0.1;
        engine.spawn_glyph(Glyph {
            character: '?',
            position: Vec3::new(0.0, 0.5 + scroll_bob, 0.0),
            scale: Vec2::splat(0.6),
            color: rgb(220, 200, 140),
            emission: 0.5,
            layer: RenderLayer::UI,
            ..Default::default()
        });

        // Dust motes
        for i in 0..10 {
            let seed = i as f32 * 2.3;
            let x = (seed * 5.1 + self.time * 0.05).sin() * 6.0;
            let y = (seed * 3.7 + self.time * 0.08).cos() * 3.0;
            let alpha = (seed * 11.0 + self.time * 0.3).sin() * 0.15 + 0.1;
            engine.spawn_glyph(Glyph {
                character: '\u{00B7}',
                position: Vec3::new(x, y, -1.0),
                scale: Vec2::splat(0.15),
                color: rgb_a(200, 180, 140, alpha.max(0.0)),
                emission: 0.05,
                layer: RenderLayer::World,
                ..Default::default()
            });
        }

        ui_render::small(engine, "[R] Read lore  [L] Learn spell  [Esc] Leave", -7.0, -3.5, theme.muted);
    }

    // ── Forge room ──────────────────────────────────────────────────────────

    fn render_forge(
        &self,
        engine: &mut ProofEngine,
        theme: &Theme,
    ) {
        ui_render::text_centered(engine, "FORGE", 3.5, rgb(255, 140, 40), 0.5, 0.7);

        // Anvil
        engine.spawn_glyph(Glyph {
            character: 'A',
            position: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec2::splat(0.7),
            color: rgb(160, 160, 170),
            emission: 0.3,
            layer: RenderLayer::UI,
            ..Default::default()
        });

        // Forge fire (left side)
        let fire_base_x = -3.0_f32;
        let fire_base_y = 0.0_f32;
        for i in 0..10 {
            let seed = self.fire_seed + i as f32 * 1.7;
            let life = (self.time * 2.0 + seed) % 1.2;
            let rise = life * 1.5;
            let drift = (seed * 5.0 + self.time * 4.0).sin() * 0.2 * life;
            let alpha = (1.0 - life / 1.2).max(0.0);
            let t = life / 1.2;
            let color = color_lerp(rgb(255, 100, 20), rgb(255, 220, 80), t);
            engine.spawn_glyph(Glyph {
                character: '*',
                position: Vec3::new(fire_base_x + drift, fire_base_y + rise, 0.1),
                scale: Vec2::splat(0.25 * (1.0 - t * 0.3)),
                color: Vec4::new(color.x, color.y, color.z, alpha),
                emission: alpha * 0.7,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }

        // Hammer strike animation (periodic)
        let strike_cycle = (self.time * 1.0) % 2.0;
        if strike_cycle < 0.3 {
            let strike_y = 1.5 - strike_cycle / 0.3 * 1.5;
            engine.spawn_glyph(Glyph {
                character: 'T',
                position: Vec3::new(0.5, strike_y, 0.2),
                scale: Vec2::splat(0.5),
                color: rgb(180, 180, 180),
                emission: 0.4,
                layer: RenderLayer::UI,
                ..Default::default()
            });
            // Spark on hit
            if strike_cycle > 0.2 {
                for i in 0..4 {
                    let angle = i as f32 * 1.3 + self.time * 10.0;
                    let dist = (strike_cycle - 0.2) * 5.0;
                    engine.spawn_glyph(Glyph {
                        character: '*',
                        position: Vec3::new(0.5 + angle.cos() * dist, 0.0 + angle.sin() * dist, 0.3),
                        scale: Vec2::splat(0.2),
                        color: rgb_a(255, 200, 60, 0.8),
                        emission: 0.8,
                        layer: RenderLayer::UI,
                        ..Default::default()
                    });
                }
            }
        }

        ui_render::small(engine, "[U] Upgrade  [E] Enchant  [Esc] Leave", -6.0, -3.5, theme.muted);
    }

    // ── Chaos Rift room ─────────────────────────────────────────────────────

    fn render_chaos_rift(
        &self,
        engine: &mut ProofEngine,
        theme: &Theme,
    ) {
        ui_render::text_centered(engine, "CHAOS RIFT", 3.5, theme.danger, 0.5, 0.8);

        // Swirling portal entity
        let portal_x = 0.0_f32;
        let portal_y = 0.5_f32;
        let ring_count = 3;
        for ring in 0..ring_count {
            let r = 1.0 + ring as f32 * 0.6;
            let speed = 1.5 - ring as f32 * 0.3;
            let points = 12 + ring * 4;
            for i in 0..points {
                let angle = self.time * speed + i as f32 * std::f32::consts::TAU / points as f32;
                let wobble = (self.time * 3.0 + i as f32).sin() * 0.15;
                let px = portal_x + angle.cos() * (r + wobble);
                let py = portal_y + angle.sin() * (r + wobble) * 0.7; // elliptical
                let t = ring as f32 / ring_count as f32;
                let color = color_lerp(rgb(200, 40, 200), rgb(80, 40, 255), t);
                let alpha = 0.4 + (self.time * 2.0 + i as f32).sin() * 0.2;
                engine.spawn_glyph(Glyph {
                    character: '\u{00B7}',
                    position: Vec3::new(px, py, 0.1 - ring as f32 * 0.1),
                    scale: Vec2::splat(0.25),
                    color: Vec4::new(color.x, color.y, color.z, alpha),
                    emission: alpha * 0.6,
                    layer: RenderLayer::UI,
                    ..Default::default()
                });
            }
        }

        // Center of portal
        let core_pulse = (self.time * 4.0).sin() * 0.3 + 0.7;
        engine.spawn_glyph(Glyph {
            character: '?',
            position: Vec3::new(portal_x, portal_y, 0.5),
            scale: Vec2::splat(0.8),
            color: Vec4::new(1.0, 0.3, 1.0, core_pulse),
            emission: core_pulse,
            layer: RenderLayer::UI,
            ..Default::default()
        });

        // Random objects spawning (conceptual — icons appearing/fading)
        for i in 0..5 {
            let seed = i as f32 * 4.3;
            let spawn_cycle = (self.time * 0.7 + seed) % 3.0;
            if spawn_cycle < 1.5 {
                let t = spawn_cycle / 1.5;
                let sx = (seed * 5.0).sin() * 3.0;
                let sy = (seed * 3.0).cos() * 2.0 + portal_y;
                let chars = ['!', '?', '*', '#', '$'];
                let ch = chars[i % chars.len()];
                let alpha = (1.0 - (t - 0.5).abs() * 2.0).max(0.0);
                engine.spawn_glyph(Glyph {
                    character: ch,
                    position: Vec3::new(sx, sy, 0.0),
                    scale: Vec2::splat(0.35),
                    color: rgb_a(200, 100, 255, alpha),
                    emission: alpha * 0.4,
                    layer: RenderLayer::UI,
                    ..Default::default()
                });
            }
        }

        // Escalating danger indicator
        let danger_pulse = (self.time * 1.0).sin() * 0.5 + 0.5;
        let danger_level = (self.anim_phase / 10.0).min(1.0); // builds over time
        let danger_bar_width = danger_level * 8.0;
        ui_render::bar(engine, -4.0, -2.5, danger_bar_width, danger_level, theme.danger, theme.muted, 0.25);
        ui_render::small(engine, "CHAOS INTENSITY", -4.0, -2.1, dim(theme.danger, danger_pulse));

        ui_render::small(engine, "[Enter] Brave the rift  [Esc] Flee", -5.0, -3.5, theme.muted);
    }

    // ── Secret room ─────────────────────────────────────────────────────────

    fn render_secret(
        &self,
        engine: &mut ProofEngine,
        theme: &Theme,
    ) {
        if self.crumble_progress < 0.01 {
            // Pre-discovery: looks like a wall
            ui_render::text_centered(engine, "...something feels off...", 2.0, theme.dim, 0.35, 0.3);

            // Fake wall
            for i in 0..8 {
                let x = -3.5 + i as f32 * 1.0;
                engine.spawn_glyph(Glyph {
                    character: '\u{2588}',
                    position: Vec3::new(x, 0.0, 0.0),
                    scale: Vec2::splat(0.7),
                    color: rgb(80, 80, 90),
                    emission: 0.1,
                    layer: RenderLayer::UI,
                    ..Default::default()
                });
            }
        } else {
            // Wall crumble animation
            ui_render::text_centered(engine, "SECRET ROOM!", 3.5, theme.gold, 0.5, 0.8);

            let crumble = self.crumble_progress;
            for i in 0..8 {
                let x = -3.5 + i as f32 * 1.0;
                let fall = if (i as f32 / 8.0) < crumble {
                    let local_t = (crumble - i as f32 / 8.0).min(0.3) / 0.3;
                    local_t * 4.0
                } else {
                    0.0
                };
                let alpha = (1.0 - fall / 4.0).max(0.0);
                // Falling block
                engine.spawn_glyph(Glyph {
                    character: '\u{2593}',
                    position: Vec3::new(x + (i as f32 * 0.7).sin() * fall * 0.3, -fall, 0.0),
                    scale: Vec2::splat(0.6),
                    color: rgb_a(80, 80, 90, alpha),
                    emission: 0.05,
                    layer: RenderLayer::UI,
                    ..Default::default()
                });
                // Debris particles
                if fall > 0.0 {
                    for j in 0..3 {
                        let dx = (i as f32 * 3.0 + j as f32 * 7.0 + self.time * 2.0).sin() * 0.5;
                        let dy = -fall + (j as f32 * 0.3);
                        engine.spawn_glyph(Glyph {
                            character: '\u{00B7}',
                            position: Vec3::new(x + dx, dy, 0.1),
                            scale: Vec2::splat(0.15),
                            color: rgb_a(120, 120, 130, alpha * 0.5),
                            emission: 0.05,
                            layer: RenderLayer::UI,
                            ..Default::default()
                        });
                    }
                }
            }

            // Special loot reveal (after crumble completes)
            if crumble > 0.9 {
                let reveal_alpha = (crumble - 0.9) * 10.0;
                let glow = (self.time * 3.0).sin() * 0.2 + 0.8;
                engine.spawn_glyph(Glyph {
                    character: '\u{2605}',
                    position: Vec3::new(0.0, -0.5, 0.5),
                    scale: Vec2::splat(0.8),
                    color: rgb_a(255, 220, 80, reveal_alpha * glow),
                    emission: reveal_alpha * glow,
                    layer: RenderLayer::UI,
                    ..Default::default()
                });
            }
        }

        ui_render::small(engine, "[P] Pick up  [Enter] Continue", -5.0, -3.5, theme.muted);
    }

    // ── Generic room ────────────────────────────────────────────────────────

    fn render_generic(
        &self,
        engine: &mut ProofEngine,
        _theme: &Theme,
    ) {
        // Simple ambient particle field
        for i in 0..8 {
            let seed = i as f32 * 1.618;
            let x = (seed * 5.3 + self.time * 0.08).sin() * 4.0;
            let y = (seed * 3.1 + self.time * 0.12).cos() * 2.5;
            let alpha = (seed * 9.0 + self.time * 0.4).sin() * 0.1 + 0.08;
            engine.spawn_glyph(Glyph {
                character: '\u{00B7}',
                position: Vec3::new(x, y, -2.0),
                scale: Vec2::splat(0.2),
                color: rgb_a(180, 180, 200, alpha.max(0.0)),
                emission: 0.05,
                layer: RenderLayer::World,
                ..Default::default()
            });
        }
    }
}
