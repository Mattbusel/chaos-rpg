//! Full dungeon tile visualization with fog of war, lighting, biome palettes,
//! animated tiles, entity rendering, minimap overlay, and room transitions.
//!
//! Camera at (0,0,-10), visible area ~17.4 x ~10.8 at z=0.

use proof_engine::prelude::*;
use crate::state::GameState;
use crate::theme::{Theme, THEMES};
use crate::dungeon_bridge::{
    DungeonBridge, RenderTile, TileBridge, TileGrid, TileVisibility,
    BridgeItemKind,
};
use crate::ui_render;
use super::{color_lerp, rgb, rgb_a, dim};

// ═══════════════════════════════════════════════════════════════════════════════
// BIOME COLOR PALETTES
// ═══════════════════════════════════════════════════════════════════════════════

/// A biome-specific tile palette.
struct BiomePalette {
    floor_fg: Vec4,
    floor_chars: &'static [char],
    wall_fg: Vec4,
    wall_chars: &'static [char],
    wall_bg: Vec4,
    accent: Vec4,
    ambient: f32,
}

fn palette_for_biome(biome: &str) -> BiomePalette {
    match biome {
        "Ruins" => BiomePalette {
            floor_fg: rgb(140, 120, 100),
            floor_chars: &['.', ',', '\'', '.'],
            wall_fg: rgb(100, 90, 75),
            wall_chars: &['\u{2588}', '\u{2593}', '\u{2592}'],
            wall_bg: rgb(50, 45, 38),
            accent: rgb(180, 150, 100),
            ambient: 0.35,
        },
        "Crypt" => BiomePalette {
            floor_fg: rgb(90, 90, 100),
            floor_chars: &['.', '\u{00B7}', ','],
            wall_fg: rgb(70, 70, 80),
            wall_chars: &['\u{2588}', '\u{2593}'],
            wall_bg: rgb(30, 30, 38),
            accent: rgb(200, 200, 220),
            ambient: 0.2,
        },
        "Library" => BiomePalette {
            floor_fg: rgb(160, 130, 90),
            floor_chars: &['.', '\u{00B7}'],
            wall_fg: rgb(140, 100, 60),
            wall_chars: &['\u{2261}', '\u{2588}', '\u{2593}'],
            wall_bg: rgb(70, 50, 30),
            accent: rgb(220, 180, 100),
            ambient: 0.4,
        },
        "Forge" => BiomePalette {
            floor_fg: rgb(160, 100, 60),
            floor_chars: &['.', '\u{00B7}', ','],
            wall_fg: rgb(180, 80, 40),
            wall_chars: &['\u{2588}', '\u{2593}', '\u{2592}'],
            wall_bg: rgb(80, 30, 10),
            accent: rgb(255, 140, 30),
            ambient: 0.45,
        },
        "Garden" => BiomePalette {
            floor_fg: rgb(80, 150, 60),
            floor_chars: &['.', ',', '\'', '\u{00B7}'],
            wall_fg: rgb(60, 120, 40),
            wall_chars: &['\u{2663}', '\u{2660}', '\u{2588}'],
            wall_bg: rgb(30, 60, 20),
            accent: rgb(100, 200, 80),
            ambient: 0.5,
        },
        "Void" => BiomePalette {
            floor_fg: rgb(100, 50, 160),
            floor_chars: &['\u{00B7}', ' ', '.'],
            wall_fg: rgb(80, 30, 140),
            wall_chars: &['\u{2588}', '\u{2593}'],
            wall_bg: rgb(20, 5, 40),
            accent: rgb(160, 80, 255),
            ambient: 0.15,
        },
        "Cathedral" => BiomePalette {
            floor_fg: rgb(200, 180, 120),
            floor_chars: &['.', '\u{00B7}'],
            wall_fg: rgb(220, 200, 140),
            wall_chars: &['\u{2588}', '\u{2593}', '\u{2592}'],
            wall_bg: rgb(100, 90, 60),
            accent: rgb(255, 220, 100),
            ambient: 0.55,
        },
        "Chaos" => BiomePalette {
            floor_fg: rgb(200, 60, 60),
            floor_chars: &['.', '\u{00B7}', ',', '!'],
            wall_fg: rgb(180, 40, 80),
            wall_chars: &['\u{2588}', '\u{2593}', '\u{2592}'],
            wall_bg: rgb(60, 10, 30),
            accent: rgb(255, 60, 120),
            ambient: 0.3,
        },
        "Abyss" => BiomePalette {
            floor_fg: rgb(40, 40, 60),
            floor_chars: &['.', ' ', '\u{00B7}'],
            wall_fg: rgb(30, 30, 50),
            wall_chars: &['\u{2588}'],
            wall_bg: rgb(10, 10, 20),
            accent: rgb(60, 60, 120),
            ambient: 0.1,
        },
        "Laboratory" => BiomePalette {
            floor_fg: rgb(120, 180, 160),
            floor_chars: &['.', '\u{00B7}', '+'],
            wall_fg: rgb(100, 160, 140),
            wall_chars: &['\u{2588}', '\u{2593}'],
            wall_bg: rgb(40, 70, 60),
            accent: rgb(80, 220, 200),
            ambient: 0.45,
        },
        _ => BiomePalette {
            floor_fg: rgb(128, 128, 128),
            floor_chars: &['.', '\u{00B7}'],
            wall_fg: rgb(100, 100, 100),
            wall_chars: &['\u{2588}', '\u{2593}'],
            wall_bg: rgb(40, 40, 40),
            accent: rgb(180, 180, 180),
            ambient: 0.3,
        },
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// LIGHTING MODEL
// ═══════════════════════════════════════════════════════════════════════════════

/// Compute light intensity at a tile given player position and optional torches.
fn light_at(
    tx: i32,
    ty: i32,
    player_x: i32,
    player_y: i32,
    torch_positions: &[(i32, i32)],
    ambient: f32,
) -> f32 {
    let dx = (tx - player_x) as f32;
    let dy = (ty - player_y) as f32;
    let dist = (dx * dx + dy * dy).sqrt();
    // Player light: radius 8, smooth falloff
    let player_light = (1.0 - dist / 8.0).max(0.0);
    let player_light = player_light * player_light; // quadratic falloff

    // Torch light: radius 4
    let mut torch_light = 0.0_f32;
    for &(tx2, ty2) in torch_positions {
        let dx2 = (tx - tx2) as f32;
        let dy2 = (ty - ty2) as f32;
        let d2 = (dx2 * dx2 + dy2 * dy2).sqrt();
        let tl = (1.0 - d2 / 4.0).max(0.0);
        torch_light = torch_light.max(tl * tl * 0.7);
    }

    (ambient + player_light + torch_light).clamp(0.0, 1.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// TILE GLYPH + COLOR SELECTION
// ═══════════════════════════════════════════════════════════════════════════════

/// Tile visual: character, foreground color, emission, z-layer.
struct TileVisual {
    ch: char,
    color: Vec4,
    emission: f32,
    z: f32,
}

fn tile_visual(
    tile: &RenderTile,
    tx: i32,
    ty: i32,
    time: f32,
    palette: &BiomePalette,
    light: f32,
    seed_hash: u64,
) -> TileVisual {
    // Simple tile-stable hash for variation
    let h = ((tx as u64).wrapping_mul(73856093) ^ (ty as u64).wrapping_mul(19349663) ^ seed_hash) as f32
        / u64::MAX as f32;

    match tile.tile_type {
        TileBridge::Floor | TileBridge::Corridor => {
            let idx = (h * palette.floor_chars.len() as f32) as usize % palette.floor_chars.len();
            let ch = palette.floor_chars[idx];
            // Subtle color variation
            let variation = 0.9 + h * 0.2;
            let c = Vec4::new(
                palette.floor_fg.x * variation * light,
                palette.floor_fg.y * variation * light,
                palette.floor_fg.z * variation * light,
                palette.floor_fg.w,
            );
            TileVisual { ch, color: c, emission: light * 0.2, z: -2.0 }
        }
        TileBridge::Wall => {
            let idx = (h * palette.wall_chars.len() as f32) as usize % palette.wall_chars.len();
            let ch = palette.wall_chars[idx];
            // Walls are darker further from light source
            let shade = (light * 0.6 + 0.1).clamp(0.1, 0.8);
            let c = Vec4::new(
                palette.wall_fg.x * shade,
                palette.wall_fg.y * shade,
                palette.wall_fg.z * shade,
                1.0,
            );
            TileVisual { ch, color: c, emission: shade * 0.1, z: -1.5 }
        }
        TileBridge::Door => {
            // Animated open/close based on time
            let open_phase = ((time * 0.5).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
            let ch = if open_phase > 0.5 { '\u{2590}' } else { '\u{258C}' };
            let brown = rgb(160, 100, 40);
            let c = Vec4::new(
                brown.x * light,
                brown.y * light,
                brown.z * light,
                1.0,
            );
            TileVisual { ch, color: c, emission: light * 0.3, z: -1.0 }
        }
        TileBridge::StairsDown => {
            // Bright pulsing glow
            let pulse = (time * 2.0).sin() * 0.3 + 0.7;
            let c = rgb_a(100, 200, 255, 1.0);
            let c = Vec4::new(c.x * pulse, c.y * pulse, c.z * pulse, 1.0);
            TileVisual { ch: '>', color: c, emission: pulse, z: -0.5 }
        }
        TileBridge::StairsUp => {
            let c = rgb(180, 180, 255);
            TileVisual { ch: '<', color: dim(c, light), emission: light * 0.5, z: -0.5 }
        }
        TileBridge::Chest => {
            // Gold sparkle
            let sparkle = ((time * 4.0 + h * 20.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
            let gold = rgb_a(255, 200, 50, 1.0);
            let c = color_lerp(dim(gold, 0.6), gold, sparkle);
            TileVisual { ch: '\u{25A0}', color: c, emission: 0.5 + sparkle * 0.3, z: -0.5 }
        }
        TileBridge::Shrine => {
            // Element-colored glow
            let glow = (time * 1.5).sin() * 0.2 + 0.8;
            let c = Vec4::new(
                palette.accent.x * glow,
                palette.accent.y * glow,
                palette.accent.z * glow,
                1.0,
            );
            TileVisual { ch: '\u{2726}', color: c, emission: glow, z: -0.5 }
        }
        TileBridge::Shop => {
            let c = rgb(80, 200, 80);
            TileVisual { ch: '$', color: dim(c, light), emission: light * 0.4, z: -0.5 }
        }
        TileBridge::Water => {
            // Animated wave using sine
            let wave = ((time * 2.0 + tx as f32 * 0.5 + ty as f32 * 0.3).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
            let blue1 = rgb(40, 80, 200);
            let blue2 = rgb(80, 140, 255);
            let c = color_lerp(blue1, blue2, wave);
            let c = Vec4::new(c.x * light, c.y * light, c.z * light, 0.85);
            TileVisual { ch: '~', color: c, emission: light * 0.3 + wave * 0.1, z: -1.8 }
        }
        TileBridge::Lava => {
            // Orange/red wave + ember particles conceptually
            let wave = ((time * 3.0 + tx as f32 * 0.4).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
            let orange = rgb(255, 120, 20);
            let red = rgb(220, 40, 10);
            let c = color_lerp(red, orange, wave);
            TileVisual { ch: '~', color: c, emission: 0.7 + wave * 0.3, z: -1.8 }
        }
        TileBridge::Ice => {
            let c = rgb(180, 220, 255);
            TileVisual { ch: '-', color: dim(c, light), emission: light * 0.35, z: -1.8 }
        }
        TileBridge::Trap => {
            // Hidden-ish: red, flickers
            let flicker = ((time * 6.0 + h * 30.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
            let red = rgb_a(200, 40, 40, 0.7 + flicker * 0.3);
            let ch = if flicker > 0.7 { '!' } else { '\u{00D7}' };
            TileVisual { ch, color: dim(red, light), emission: light * 0.2, z: -1.0 }
        }
        TileBridge::Secret => {
            // Looks like a wall unless triggered
            let idx = (h * palette.wall_chars.len() as f32) as usize % palette.wall_chars.len();
            let ch = palette.wall_chars[idx];
            let c = dim(palette.wall_fg, light * 0.5);
            TileVisual { ch, color: c, emission: 0.05, z: -1.5 }
        }
        TileBridge::Void => {
            // Edge particles: sparse dots at tile edges
            let edge_dot = (time * 0.5 + h * 10.0).sin() > 0.85;
            if edge_dot {
                let c = rgb_a(80, 40, 120, 0.3);
                TileVisual { ch: '\u{00B7}', color: c, emission: 0.1, z: -3.0 }
            } else {
                TileVisual { ch: ' ', color: Vec4::ZERO, emission: 0.0, z: -3.0 }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENTITY ICONS
// ═══════════════════════════════════════════════════════════════════════════════

struct EntityGlyph {
    ch: char,
    color: Vec4,
    emission: f32,
}

fn player_glyph() -> EntityGlyph {
    EntityGlyph {
        ch: '@',
        color: rgb(255, 255, 255),
        emission: 0.9,
    }
}

fn enemy_glyph(name: &str, is_elite: bool) -> EntityGlyph {
    let base_color = if is_elite { rgb(255, 60, 60) } else { rgb(220, 100, 100) };
    let ch = name.chars().next().unwrap_or('?');
    EntityGlyph {
        ch,
        color: base_color,
        emission: if is_elite { 0.7 } else { 0.4 },
    }
}

fn item_glyph(kind: &BridgeItemKind) -> EntityGlyph {
    match kind {
        BridgeItemKind::Chest { .. } => EntityGlyph { ch: '\u{25A0}', color: rgb(255, 200, 50), emission: 0.5 },
        BridgeItemKind::HealingShrine => EntityGlyph { ch: '+', color: rgb(80, 255, 80), emission: 0.6 },
        BridgeItemKind::BuffShrine { .. } => EntityGlyph { ch: '\u{2726}', color: rgb(100, 150, 255), emission: 0.6 },
        BridgeItemKind::RiskShrine => EntityGlyph { ch: '\u{2726}', color: rgb(255, 80, 80), emission: 0.6 },
        BridgeItemKind::Merchant { .. } => EntityGlyph { ch: '$', color: rgb(80, 200, 80), emission: 0.5 },
        BridgeItemKind::Campfire => EntityGlyph { ch: '*', color: rgb(255, 160, 40), emission: 0.7 },
        BridgeItemKind::Forge => EntityGlyph { ch: 'A', color: rgb(255, 120, 30), emission: 0.5 },
        BridgeItemKind::LoreBook { .. } => EntityGlyph { ch: '\u{2261}', color: rgb(200, 180, 120), emission: 0.4 },
        BridgeItemKind::SpellScroll { .. } => EntityGlyph { ch: '?', color: rgb(160, 100, 255), emission: 0.5 },
        BridgeItemKind::Puzzle => EntityGlyph { ch: '#', color: rgb(200, 200, 80), emission: 0.4 },
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DUNGEON RENDERER
// ═══════════════════════════════════════════════════════════════════════════════

/// Persistent state for dungeon rendering (camera offset, transition anim, etc.).
pub struct DungeonRenderer {
    /// Camera offset in tile coords, smoothly tracks the player.
    pub camera_x: f32,
    pub camera_y: f32,
    /// Target camera position (for smooth pan).
    pub target_camera_x: f32,
    pub target_camera_y: f32,
    /// Accumulated time for animations.
    pub time: f32,
    /// Whether the minimap overlay is visible.
    pub minimap_visible: bool,
    /// Room transition progress (0.0 = no transition, >0 = transitioning).
    pub transition_timer: f32,
    /// Previous camera position for transition lerp.
    pub prev_camera_x: f32,
    pub prev_camera_y: f32,
}

impl DungeonRenderer {
    pub fn new() -> Self {
        Self {
            camera_x: 0.0,
            camera_y: 0.0,
            target_camera_x: 0.0,
            target_camera_y: 0.0,
            time: 0.0,
            minimap_visible: true,
            transition_timer: 0.0,
            prev_camera_x: 0.0,
            prev_camera_y: 0.0,
        }
    }

    /// Update camera tracking and timers. Call once per frame.
    pub fn update(&mut self, dt: f32, player_tile_x: f32, player_tile_y: f32) {
        self.time += dt;

        // Set target to player position
        self.target_camera_x = player_tile_x;
        self.target_camera_y = player_tile_y;

        // Smooth camera follow
        let lerp_speed = 1.0 - (1.0 - 0.12_f32).powf(dt * 60.0);
        if self.transition_timer > 0.0 {
            // During room transition: use transition lerp
            self.transition_timer = (self.transition_timer - dt).max(0.0);
            let t = 1.0 - self.transition_timer / 0.5; // 0.5s transition
            let t = t.clamp(0.0, 1.0);
            let t = t * t * (3.0 - 2.0 * t); // smoothstep
            self.camera_x = self.prev_camera_x + (self.target_camera_x - self.prev_camera_x) * t;
            self.camera_y = self.prev_camera_y + (self.target_camera_y - self.prev_camera_y) * t;
        } else {
            self.camera_x += (self.target_camera_x - self.camera_x) * lerp_speed;
            self.camera_y += (self.target_camera_y - self.camera_y) * lerp_speed;
        }
    }

    /// Start a room transition animation.
    pub fn start_transition(&mut self, new_target_x: f32, new_target_y: f32) {
        self.prev_camera_x = self.camera_x;
        self.prev_camera_y = self.camera_y;
        self.target_camera_x = new_target_x;
        self.target_camera_y = new_target_y;
        self.transition_timer = 0.5;
    }

    /// Main render call. Draws the full dungeon map with fog, lighting, entities.
    pub fn render(
        &self,
        state: &GameState,
        engine: &mut ProofEngine,
    ) {
        let theme = &THEMES[state.theme_idx % THEMES.len()];
        let bridge = match &state.dungeon_bridge {
            Some(b) => b,
            None => return,
        };

        let grid = match bridge.get_current_map() {
            Some(g) => g,
            None => return,
        };

        let biome = bridge.biome();
        let palette = palette_for_biome(biome);

        // Player position from bridge
        let (px, py) = grid.player_start;
        let player_x = px;
        let player_y = py;

        // Collect torch positions (walls adjacent to visible floors can have torches)
        let torch_positions = self.find_torches(&grid, bridge.seed());

        // Tile rendering scale: how many screen units per tile
        let tile_scale = 0.45_f32;

        // Viewport: how many tiles fit on screen
        let view_w = (17.4 / tile_scale) as i32 / 2 + 1;
        let view_h = (10.8 / tile_scale) as i32 / 2 + 1;

        let cam_tx = self.camera_x as i32;
        let cam_ty = self.camera_y as i32;

        // ── Render tiles ────────────────────────────────────────────────────
        for ty in (cam_ty - view_h)..=(cam_ty + view_h) {
            for tx in (cam_tx - view_w)..=(cam_tx + view_w) {
                let tile = grid.get(tx, ty);

                // Fog of war
                match tile.visibility {
                    TileVisibility::Unseen => continue, // invisible
                    TileVisibility::Explored => {
                        // Seen but not currently visible: dark gray, 30% brightness
                        let vis = tile_visual(&tile, tx, ty, self.time, &palette, 0.3, bridge.seed());
                        if vis.ch == ' ' { continue; }
                        let dark_color = dim(vis.color, 0.3);
                        let screen_x = (tx as f32 - self.camera_x) * tile_scale;
                        let screen_y = -(ty as f32 - self.camera_y) * tile_scale; // flip Y
                        engine.spawn_glyph(Glyph {
                            character: vis.ch,
                            position: Vec3::new(screen_x, screen_y, vis.z),
                            scale: Vec2::splat(tile_scale * 0.9),
                            color: dark_color,
                            emission: 0.05,
                            layer: RenderLayer::World,
                            ..Default::default()
                        });
                    }
                    TileVisibility::Visible => {
                        let light = light_at(tx, ty, player_x, player_y, &torch_positions, palette.ambient);
                        let vis = tile_visual(&tile, tx, ty, self.time, &palette, light, bridge.seed());
                        if vis.ch == ' ' { continue; }
                        let screen_x = (tx as f32 - self.camera_x) * tile_scale;
                        let screen_y = -(ty as f32 - self.camera_y) * tile_scale;
                        engine.spawn_glyph(Glyph {
                            character: vis.ch,
                            position: Vec3::new(screen_x, screen_y, vis.z),
                            scale: Vec2::splat(tile_scale * 0.9),
                            color: vis.color,
                            emission: vis.emission,
                            layer: RenderLayer::World,
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // ── Render entities ─────────────────────────────────────────────────
        self.render_entities(state, engine, bridge, &grid, tile_scale);

        // ── Render player ───────────────────────────────────────────────────
        {
            let pg = player_glyph();
            let screen_x = (player_x as f32 - self.camera_x) * tile_scale;
            let screen_y = -(player_y as f32 - self.camera_y) * tile_scale;
            // Breathing animation
            let breathe = (self.time * 2.0).sin() * 0.02;
            engine.spawn_glyph(Glyph {
                character: pg.ch,
                position: Vec3::new(screen_x, screen_y + breathe, 0.0),
                scale: Vec2::splat(tile_scale * 1.1),
                color: pg.color,
                emission: pg.emission,
                layer: RenderLayer::World,
                ..Default::default()
            });
        }

        // ── Particle effects for special tiles ──────────────────────────────
        self.render_particles(engine, &grid, tile_scale, &palette);

        // ── Minimap overlay ─────────────────────────────────────────────────
        if self.minimap_visible {
            self.render_minimap(engine, bridge, theme);
        }

        // ── HUD: biome label and floor number ───────────────────────────────
        let floor_label = format!("F{} - {}", bridge.floor_number(), biome);
        ui_render::small(engine, &floor_label, -8.5, 5.0, theme.muted);

        // Explored percentage
        let explored = (bridge.explored_fraction() * 100.0) as u32;
        let explored_label = format!("Explored: {}%", explored);
        ui_render::small(engine, &explored_label, -8.5, 4.6, theme.dim);
    }

    // ── Torch placement ─────────────────────────────────────────────────────

    fn find_torches(&self, grid: &TileGrid, seed: u64) -> Vec<(i32, i32)> {
        let mut torches = Vec::new();
        // Place torches on wall tiles that are adjacent to floor tiles, sparsely
        for y in 0..grid.height as i32 {
            for x in 0..grid.width as i32 {
                let tile = grid.get(x, y);
                if tile.tile_type != TileBridge::Wall { continue; }
                // Check adjacency to floor
                let has_floor_neighbor =
                    grid.get(x + 1, y).tile_type == TileBridge::Floor
                    || grid.get(x - 1, y).tile_type == TileBridge::Floor
                    || grid.get(x, y + 1).tile_type == TileBridge::Floor
                    || grid.get(x, y - 1).tile_type == TileBridge::Floor;
                if !has_floor_neighbor { continue; }
                // Sparse placement: hash-based
                let h = (x as u64).wrapping_mul(73856093) ^ (y as u64).wrapping_mul(19349663) ^ seed;
                if h % 7 == 0 {
                    torches.push((x, y));
                }
            }
        }
        torches
    }

    // ── Entity rendering ────────────────────────────────────────────────────

    fn render_entities(
        &self,
        _state: &GameState,
        engine: &mut ProofEngine,
        bridge: &DungeonBridge,
        _grid: &TileGrid,
        tile_scale: f32,
    ) {
        let rooms = bridge.get_all_rooms();
        for room in &rooms {
            // Enemies
            for enemy in &room.enemies {
                let eg = enemy_glyph(&enemy.name, enemy.is_elite);
                let screen_x = (enemy.pos.0 as f32 - self.camera_x) * tile_scale;
                let screen_y = -(enemy.pos.1 as f32 - self.camera_y) * tile_scale;
                // Hover animation for elites
                let hover = if enemy.is_elite {
                    (self.time * 3.0 + enemy.pos.0 as f32).sin() * 0.04
                } else {
                    0.0
                };
                engine.spawn_glyph(Glyph {
                    character: eg.ch,
                    position: Vec3::new(screen_x, screen_y + hover, -0.2),
                    scale: Vec2::splat(tile_scale),
                    color: eg.color,
                    emission: eg.emission,
                    layer: RenderLayer::World,
                    ..Default::default()
                });
            }
            // Items
            for item in &room.items {
                let ig = item_glyph(&item.kind);
                let screen_x = (item.pos.0 as f32 - self.camera_x) * tile_scale;
                let screen_y = -(item.pos.1 as f32 - self.camera_y) * tile_scale;
                engine.spawn_glyph(Glyph {
                    character: ig.ch,
                    position: Vec3::new(screen_x, screen_y, -0.3),
                    scale: Vec2::splat(tile_scale * 0.9),
                    color: ig.color,
                    emission: ig.emission,
                    layer: RenderLayer::World,
                    ..Default::default()
                });
            }
        }
    }

    // ── Particle effects ────────────────────────────────────────────────────

    fn render_particles(
        &self,
        engine: &mut ProofEngine,
        grid: &TileGrid,
        tile_scale: f32,
        palette: &BiomePalette,
    ) {
        // Sparse iteration: only emit particles for special tiles near camera
        let cam_tx = self.camera_x as i32;
        let cam_ty = self.camera_y as i32;
        let radius = 10;

        for ty in (cam_ty - radius)..=(cam_ty + radius) {
            for tx in (cam_tx - radius)..=(cam_tx + radius) {
                let tile = grid.get(tx, ty);
                if tile.visibility != TileVisibility::Visible { continue; }

                match tile.tile_type {
                    TileBridge::Lava => {
                        // Ember particles rising
                        let h = ((tx as u64).wrapping_mul(48271) ^ (ty as u64).wrapping_mul(12345)) as f32
                            / u64::MAX as f32;
                        if h > 0.85 {
                            let ember_y = (self.time * 0.8 + h * 5.0) % 1.5;
                            let screen_x = (tx as f32 - self.camera_x) * tile_scale + h * 0.2 - 0.1;
                            let screen_y = -(ty as f32 - self.camera_y) * tile_scale + ember_y * tile_scale;
                            let fade = 1.0 - ember_y / 1.5;
                            engine.spawn_glyph(Glyph {
                                character: '\u{00B7}',
                                position: Vec3::new(screen_x, screen_y, -0.5),
                                scale: Vec2::splat(tile_scale * 0.4),
                                color: rgb_a(255, 160, 40, fade),
                                emission: fade * 0.6,
                                layer: RenderLayer::World,
                                ..Default::default()
                            });
                        }
                    }
                    TileBridge::Chest => {
                        // Sparkle particles
                        let phase = (self.time * 5.0 + tx as f32 * 3.0 + ty as f32 * 7.0).sin();
                        if phase > 0.7 {
                            let sparkle_x = (tx as f32 - self.camera_x) * tile_scale + phase * 0.15;
                            let sparkle_y = -(ty as f32 - self.camera_y) * tile_scale + 0.1;
                            engine.spawn_glyph(Glyph {
                                character: '*',
                                position: Vec3::new(sparkle_x, sparkle_y, -0.1),
                                scale: Vec2::splat(tile_scale * 0.3),
                                color: rgb_a(255, 220, 80, 0.8),
                                emission: 0.8,
                                layer: RenderLayer::World,
                                ..Default::default()
                            });
                        }
                    }
                    TileBridge::Shrine => {
                        // Halo particles orbiting
                        let angle = self.time * 1.5 + tx as f32;
                        let orbit_r = 0.2;
                        let ox = angle.cos() * orbit_r;
                        let oy = angle.sin() * orbit_r;
                        let screen_x = (tx as f32 - self.camera_x) * tile_scale + ox;
                        let screen_y = -(ty as f32 - self.camera_y) * tile_scale + oy;
                        engine.spawn_glyph(Glyph {
                            character: '\u{00B7}',
                            position: Vec3::new(screen_x, screen_y, -0.1),
                            scale: Vec2::splat(tile_scale * 0.3),
                            color: Vec4::new(palette.accent.x, palette.accent.y, palette.accent.z, 0.6),
                            emission: 0.6,
                            layer: RenderLayer::World,
                            ..Default::default()
                        });
                    }
                    TileBridge::StairsDown => {
                        // Concentric ring pulse
                        let ring_phase = (self.time * 1.0) % 2.0;
                        let ring_r = ring_phase * 0.3;
                        let ring_alpha = 1.0 - ring_phase / 2.0;
                        for i in 0..4 {
                            let a = i as f32 * std::f32::consts::FRAC_PI_2;
                            let rx = a.cos() * ring_r;
                            let ry = a.sin() * ring_r;
                            let screen_x = (tx as f32 - self.camera_x) * tile_scale + rx;
                            let screen_y = -(ty as f32 - self.camera_y) * tile_scale + ry;
                            engine.spawn_glyph(Glyph {
                                character: '\u{00B7}',
                                position: Vec3::new(screen_x, screen_y, -0.1),
                                scale: Vec2::splat(tile_scale * 0.2),
                                color: rgb_a(100, 200, 255, ring_alpha),
                                emission: ring_alpha * 0.5,
                                layer: RenderLayer::World,
                                ..Default::default()
                            });
                        }
                    }
                    TileBridge::Water => {
                        // Occasional ripple dot
                        let ripple = (self.time * 1.2 + tx as f32 * 2.7 + ty as f32 * 1.3).sin();
                        if ripple > 0.92 {
                            let screen_x = (tx as f32 - self.camera_x) * tile_scale;
                            let screen_y = -(ty as f32 - self.camera_y) * tile_scale;
                            engine.spawn_glyph(Glyph {
                                character: 'o',
                                position: Vec3::new(screen_x, screen_y, -1.0),
                                scale: Vec2::splat(tile_scale * 0.25),
                                color: rgb_a(120, 180, 255, 0.4),
                                emission: 0.2,
                                layer: RenderLayer::World,
                                ..Default::default()
                            });
                        }
                    }
                    TileBridge::Void => {
                        // Edge particles: wispy purple dots drifting
                        let h = ((tx as u64).wrapping_mul(91831) ^ (ty as u64).wrapping_mul(77773)) as f32
                            / u64::MAX as f32;
                        if h > 0.92 {
                            let drift = (self.time * 0.3 + h * 10.0).sin() * 0.2;
                            let screen_x = (tx as f32 - self.camera_x) * tile_scale + drift;
                            let screen_y = -(ty as f32 - self.camera_y) * tile_scale;
                            engine.spawn_glyph(Glyph {
                                character: '\u{00B7}',
                                position: Vec3::new(screen_x, screen_y, -2.5),
                                scale: Vec2::splat(tile_scale * 0.3),
                                color: rgb_a(100, 50, 160, 0.4),
                                emission: 0.15,
                                layer: RenderLayer::World,
                                ..Default::default()
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // ── Minimap overlay ─────────────────────────────────────────────────────

    fn render_minimap(
        &self,
        engine: &mut ProofEngine,
        bridge: &DungeonBridge,
        theme: &Theme,
    ) {
        let minimap = match bridge.get_minimap_data() {
            Some(m) => m,
            None => return,
        };

        // Position minimap in top-right corner
        let offset_x = 6.0_f32;
        let offset_y = 4.5_f32;
        let scale = 0.12_f32;

        // Background panel
        engine.spawn_glyph(Glyph {
            character: '\u{2588}',
            position: Vec3::new(offset_x - 0.5, offset_y, 1.0),
            scale: Vec2::new(3.0, 2.5),
            color: Vec4::new(0.0, 0.0, 0.0, 0.6),
            emission: 0.0,
            layer: RenderLayer::UI,
            ..Default::default()
        });

        for entry in &minimap.entries {
            let px = offset_x + entry.x as f32 * scale;
            let py = offset_y - entry.y as f32 * scale;
            let (r, g, b) = entry.color;
            let color = Vec4::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 0.85);
            let is_player = entry.ch == '@';
            engine.spawn_glyph(Glyph {
                character: if is_player { '\u{25CF}' } else { entry.ch },
                position: Vec3::new(px, py, 2.0),
                scale: Vec2::splat(scale * 0.8),
                color: if is_player { rgb(255, 255, 255) } else { color },
                emission: if is_player { 0.9 } else { 0.3 },
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }

        // Minimap border label
        ui_render::text(engine, "MAP", offset_x - 0.3, offset_y + 1.5, theme.dim, 0.2, 0.3);
    }

    // ── Interactive object highlighting ─────────────────────────────────────

    /// Render a highlight around objects near the player.
    pub fn render_highlights(
        &self,
        engine: &mut ProofEngine,
        bridge: &DungeonBridge,
        player_x: i32,
        player_y: i32,
        tile_scale: f32,
    ) {
        let rooms = bridge.get_all_rooms();
        for room in &rooms {
            for item in &room.items {
                let dx = (item.pos.0 - player_x).abs();
                let dy = (item.pos.1 - player_y).abs();
                if dx <= 2 && dy <= 2 {
                    // Glow outline
                    let pulse = (self.time * 3.0).sin() * 0.3 + 0.7;
                    let screen_x = (item.pos.0 as f32 - self.camera_x) * tile_scale;
                    let screen_y = -(item.pos.1 as f32 - self.camera_y) * tile_scale;
                    engine.spawn_glyph(Glyph {
                        character: '\u{25CB}',
                        position: Vec3::new(screen_x, screen_y, -0.05),
                        scale: Vec2::splat(tile_scale * 1.3),
                        color: rgb_a(255, 255, 200, pulse * 0.5),
                        emission: pulse * 0.4,
                        layer: RenderLayer::World,
                        ..Default::default()
                    });
                }
            }
        }
    }
}
