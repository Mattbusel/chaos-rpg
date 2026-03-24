//! Bridge between proof-engine's DungeonManager and chaos-rpg's game state.
//!
//! Maps proof-engine's `FloorMap` / `Tile` / `DungeonRoom` system into
//! chaos-rpg's existing dungeon and room concepts so the graphical frontend
//! can render procedural floors without duplicating generation logic.

use glam::IVec2;

use proof_engine::game::dungeon::{
    DungeonManager, DungeonRoom, EnemySpawn, FloorBiome, FloorMap, FogOfWar,
    HazardType, Minimap, MinimapGlyph, RoomItemKind, RoomType as PeRoomType,
    Tile, Visibility,
};

use chaos_rpg_core::world::RoomType as CoreRoomType;

// ═══════════════════════════════════════════════════════════════════════════════
// Bridge-level room info
// ═══════════════════════════════════════════════════════════════════════════════

/// Lightweight snapshot of what lives in a room, suitable for the game
/// to decide how to handle entry (combat, shop, shrine, etc.).
#[derive(Debug, Clone)]
pub struct RoomInfo {
    /// Room identifier within the floor.
    pub room_id: usize,
    /// What kind of room this is (mapped to chaos-rpg terms).
    pub room_type: CoreRoomType,
    /// Enemies present (not yet killed).
    pub enemies: Vec<BridgeEnemy>,
    /// Interactable items / fixtures.
    pub items: Vec<BridgeItem>,
    /// Whether the player has already visited this room.
    pub visited: bool,
    /// Whether all enemies are dead.
    pub cleared: bool,
    /// Room bounding rectangle center (tile coords).
    pub center: (i32, i32),
    /// Room width in tiles.
    pub width: i32,
    /// Room height in tiles.
    pub height: i32,
}

/// Enemy data extracted from proof-engine's `EnemySpawn`.
#[derive(Debug, Clone)]
pub struct BridgeEnemy {
    pub name: String,
    pub hp: f32,
    pub attack: f32,
    pub is_elite: bool,
    pub abilities: Vec<String>,
    pub pos: (i32, i32),
}

/// Item / fixture data extracted from proof-engine's `RoomItem`.
#[derive(Debug, Clone)]
pub struct BridgeItem {
    pub kind: BridgeItemKind,
    pub pos: (i32, i32),
}

/// Simplified item categories for the chaos-rpg frontend.
#[derive(Debug, Clone, PartialEq)]
pub enum BridgeItemKind {
    Chest { trapped: bool, tier: u32 },
    HealingShrine,
    BuffShrine { name: String, floors: u32 },
    RiskShrine,
    Merchant { items: u32, price_mult: f32 },
    Campfire,
    Forge,
    LoreBook { id: u32 },
    SpellScroll { name: String },
    Puzzle,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Floor info returned on descent / ascent
// ═══════════════════════════════════════════════════════════════════════════════

/// Summary of a newly entered floor.
#[derive(Debug, Clone)]
pub struct FloorInfo {
    pub floor_number: u32,
    pub biome: String,
    pub biome_flavor: String,
    pub ambient_light: f32,
    pub music_vibe: String,
    pub hazard: String,
    pub room_count: usize,
    pub width: usize,
    pub height: usize,
    pub player_start: (i32, i32),
    pub exit_point: (i32, i32),
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tile grid snapshot for rendering
// ═══════════════════════════════════════════════════════════════════════════════

/// A single tile with visibility for the renderer.
#[derive(Debug, Clone, Copy)]
pub struct RenderTile {
    pub glyph: char,
    pub walkable: bool,
    pub blocks_sight: bool,
    pub visibility: TileVisibility,
    pub damage_on_step: Option<f32>,
    pub tile_type: TileBridge,
}

/// Visibility state for a tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileVisibility {
    Unseen,
    Explored,
    Visible,
}

/// Simplified tile type for rendering decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileBridge {
    Floor,
    Wall,
    Corridor,
    Door,
    StairsDown,
    StairsUp,
    Trap,
    Chest,
    Shrine,
    Shop,
    Void,
    Secret,
    Water,
    Lava,
    Ice,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tile grid (2D snapshot)
// ═══════════════════════════════════════════════════════════════════════════════

/// Full tile grid snapshot for the renderer.
#[derive(Debug, Clone)]
pub struct TileGrid {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<RenderTile>,
    pub player_start: (i32, i32),
    pub exit_point: (i32, i32),
}

impl TileGrid {
    /// Get tile at (x, y), returning a wall if out of bounds.
    pub fn get(&self, x: i32, y: i32) -> RenderTile {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return RenderTile {
                glyph: '#',
                walkable: false,
                blocks_sight: true,
                visibility: TileVisibility::Unseen,
                damage_on_step: None,
                tile_type: TileBridge::Wall,
            };
        }
        self.tiles[y as usize * self.width + x as usize]
    }

    /// Iterate over all visible tiles with their coordinates.
    pub fn visible_tiles(&self) -> impl Iterator<Item = (i32, i32, &RenderTile)> {
        self.tiles.iter().enumerate().filter_map(move |(idx, tile)| {
            if tile.visibility == TileVisibility::Unseen {
                return None;
            }
            let x = (idx % self.width) as i32;
            let y = (idx / self.width) as i32;
            Some((x, y, tile))
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Minimap data
// ═══════════════════════════════════════════════════════════════════════════════

/// A single glyph in the minimap.
#[derive(Debug, Clone)]
pub struct MinimapEntry {
    pub x: i32,
    pub y: i32,
    pub ch: char,
    pub color: (u8, u8, u8),
}

/// Complete minimap snapshot.
#[derive(Debug, Clone)]
pub struct MinimapData {
    pub entries: Vec<MinimapEntry>,
    pub player_pos: (i32, i32),
}

// ═══════════════════════════════════════════════════════════════════════════════
// Conversion helpers
// ═══════════════════════════════════════════════════════════════════════════════

fn tile_to_glyph(tile: Tile, biome: FloorBiome) -> char {
    let props = biome.properties();
    match tile {
        Tile::Floor => props.floor_char,
        Tile::Wall => props.wall_char,
        Tile::Corridor => '+',
        Tile::Door => 'D',
        Tile::StairsDown => '>',
        Tile::StairsUp => '<',
        Tile::Trap => '^',
        Tile::Chest => '$',
        Tile::Shrine => '*',
        Tile::ShopCounter => 'S',
        Tile::Void => ' ',
        Tile::SecretWall => '#',
        Tile::Water => '~',
        Tile::Lava => '=',
        Tile::Ice => '-',
    }
}

fn tile_to_bridge(tile: Tile) -> TileBridge {
    match tile {
        Tile::Floor => TileBridge::Floor,
        Tile::Wall => TileBridge::Wall,
        Tile::Corridor => TileBridge::Corridor,
        Tile::Door => TileBridge::Door,
        Tile::StairsDown => TileBridge::StairsDown,
        Tile::StairsUp => TileBridge::StairsUp,
        Tile::Trap => TileBridge::Trap,
        Tile::Chest => TileBridge::Chest,
        Tile::Shrine => TileBridge::Shrine,
        Tile::ShopCounter => TileBridge::Shop,
        Tile::Void => TileBridge::Void,
        Tile::SecretWall => TileBridge::Secret,
        Tile::Water => TileBridge::Water,
        Tile::Lava => TileBridge::Lava,
        Tile::Ice => TileBridge::Ice,
    }
}

fn visibility_to_bridge(vis: Visibility) -> TileVisibility {
    match vis {
        Visibility::Unseen => TileVisibility::Unseen,
        Visibility::Seen => TileVisibility::Explored,
        Visibility::Visible => TileVisibility::Visible,
    }
}

fn pe_room_to_core(pe: &PeRoomType) -> CoreRoomType {
    match pe {
        PeRoomType::Normal => CoreRoomType::Empty,
        PeRoomType::Combat => CoreRoomType::Combat,
        PeRoomType::Treasure => CoreRoomType::Treasure,
        PeRoomType::Shop => CoreRoomType::Shop,
        PeRoomType::Shrine => CoreRoomType::Shrine,
        PeRoomType::Trap => CoreRoomType::Trap,
        PeRoomType::Puzzle => CoreRoomType::Shrine,
        PeRoomType::MiniBoss => CoreRoomType::Combat,
        PeRoomType::Boss => CoreRoomType::Boss,
        PeRoomType::ChaosRift => CoreRoomType::Combat,
        PeRoomType::Rest => CoreRoomType::Shrine,
        PeRoomType::Secret => CoreRoomType::Treasure,
        PeRoomType::Library => CoreRoomType::Shrine,
        PeRoomType::Forge => CoreRoomType::Shop,
    }
}

fn item_kind_to_bridge(kind: &RoomItemKind) -> BridgeItemKind {
    match kind {
        RoomItemKind::Chest { trapped, loot_tier } => BridgeItemKind::Chest {
            trapped: *trapped,
            tier: *loot_tier,
        },
        RoomItemKind::HealingShrine => BridgeItemKind::HealingShrine,
        RoomItemKind::BuffShrine {
            buff_name,
            floors_remaining,
        } => BridgeItemKind::BuffShrine {
            name: buff_name.clone(),
            floors: *floors_remaining,
        },
        RoomItemKind::RiskShrine => BridgeItemKind::RiskShrine,
        RoomItemKind::Merchant {
            item_count,
            price_mult,
        } => BridgeItemKind::Merchant {
            items: *item_count,
            price_mult: *price_mult,
        },
        RoomItemKind::Campfire => BridgeItemKind::Campfire,
        RoomItemKind::ForgeAnvil => BridgeItemKind::Forge,
        RoomItemKind::LoreBook { entry_id } => BridgeItemKind::LoreBook { id: *entry_id },
        RoomItemKind::SpellScroll { spell_name } => BridgeItemKind::SpellScroll {
            name: spell_name.clone(),
        },
        RoomItemKind::PuzzleBlock { .. } => BridgeItemKind::Puzzle,
    }
}

fn enemy_spawn_to_bridge(e: &EnemySpawn) -> BridgeEnemy {
    BridgeEnemy {
        name: e.name.clone(),
        hp: e.stats.hp,
        attack: e.stats.damage,
        is_elite: e.is_elite,
        abilities: e.abilities.clone(),
        pos: (e.pos.x, e.pos.y),
    }
}

fn dungeon_room_to_info(room: &DungeonRoom) -> RoomInfo {
    let cx = room.rect.x + room.rect.w / 2;
    let cy = room.rect.y + room.rect.h / 2;
    RoomInfo {
        room_id: room.id,
        room_type: pe_room_to_core(&room.room_type),
        enemies: room.enemies.iter().map(enemy_spawn_to_bridge).collect(),
        items: room
            .items
            .iter()
            .map(|i| BridgeItem {
                kind: item_kind_to_bridge(&i.kind),
                pos: (i.pos.x, i.pos.y),
            })
            .collect(),
        visited: room.visited,
        cleared: room.cleared,
        center: (cx, cy),
        width: room.rect.w,
        height: room.rect.h,
    }
}

fn biome_name(biome: FloorBiome) -> &'static str {
    match biome {
        FloorBiome::Ruins => "Ruins",
        FloorBiome::Crypt => "Crypt",
        FloorBiome::Library => "Library",
        FloorBiome::Forge => "Forge",
        FloorBiome::Garden => "Garden",
        FloorBiome::Void => "Void",
        FloorBiome::Chaos => "Chaos",
        FloorBiome::Abyss => "Abyss",
        FloorBiome::Cathedral => "Cathedral",
        FloorBiome::Laboratory => "Laboratory",
    }
}

fn hazard_name(h: HazardType) -> &'static str {
    match h {
        HazardType::None => "None",
        HazardType::Crumble => "Crumble",
        HazardType::Poison => "Poison",
        HazardType::Fire => "Fire",
        HazardType::Ice => "Ice",
        HazardType::Thorns => "Thorns",
        HazardType::VoidRift => "Void Rift",
        HazardType::ChaosBurst => "Chaos Burst",
        HazardType::Darkness => "Darkness",
        HazardType::Acid => "Acid",
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DungeonBridge
// ═══════════════════════════════════════════════════════════════════════════════

/// Bridge between proof-engine's `DungeonManager` and the chaos-rpg frontend.
///
/// Owns the dungeon manager and provides higher-level queries that the
/// renderer and game screens need without exposing proof-engine internals.
pub struct DungeonBridge {
    manager: DungeonManager,
    /// Cached player position for fog/minimap queries.
    player_pos: IVec2,
    /// Default vision radius.
    vision_radius: i32,
}

impl DungeonBridge {
    // ── Construction ─────────────────────────────────────────────────────────

    /// Create a new bridge and generate floor 1.
    pub fn init(seed: u64) -> Self {
        let mut manager = DungeonManager::new(seed);
        manager.start();
        let start = manager
            .current_floor()
            .map(|f| f.map.player_start)
            .unwrap_or(IVec2::ZERO);
        let mut bridge = Self {
            manager,
            player_pos: start,
            vision_radius: 8,
        };
        // Initial fog reveal around spawn.
        bridge.update_fog_at(start);
        bridge
    }

    // ── Floor navigation ─────────────────────────────────────────────────────

    /// Descend to the next floor. Returns a summary of the new floor.
    pub fn descend(&mut self) -> FloorInfo {
        self.manager.descend();
        let info = self.build_floor_info();
        // Move player to new floor's spawn point.
        if let Some(floor) = self.manager.current_floor() {
            self.player_pos = floor.map.player_start;
        }
        self.update_fog_at(self.player_pos);
        info
    }

    /// Ascend to the previous floor. Returns a summary of the floor.
    pub fn ascend(&mut self) -> Option<FloorInfo> {
        if self.floor_number() <= 1 {
            return None;
        }
        self.manager.ascend();
        let info = self.build_floor_info();
        // Restore player position to stairs up location.
        if let Some(floor) = self.manager.current_floor() {
            self.player_pos = floor.map.exit_point;
        }
        Some(info)
    }

    /// Get the current floor number.
    pub fn floor_number(&self) -> u32 {
        self.manager.floor_number()
    }

    /// Maximum floor reached this run.
    pub fn max_floor(&self) -> u32 {
        self.manager.max_floor()
    }

    /// Dungeon seed.
    pub fn seed(&self) -> u64 {
        self.manager.seed()
    }

    // ── Map queries ──────────────────────────────────────────────────────────

    /// Build a full tile grid snapshot for rendering.
    pub fn get_current_map(&self) -> Option<TileGrid> {
        let floor = self.manager.current_floor()?;
        let map = &floor.map;
        let fog = &floor.fog;
        let biome = map.biome;

        let mut tiles = Vec::with_capacity(map.width * map.height);
        for y in 0..map.height as i32 {
            for x in 0..map.width as i32 {
                let tile = map.get_tile(x, y);
                let vis = fog.get(x, y);
                let props = tile.properties();
                tiles.push(RenderTile {
                    glyph: tile_to_glyph(tile, biome),
                    walkable: props.walkable,
                    blocks_sight: props.blocks_sight,
                    visibility: visibility_to_bridge(vis),
                    damage_on_step: props.damage_on_step,
                    tile_type: tile_to_bridge(tile),
                });
            }
        }

        Some(TileGrid {
            width: map.width,
            height: map.height,
            tiles,
            player_start: (map.player_start.x, map.player_start.y),
            exit_point: (map.exit_point.x, map.exit_point.y),
        })
    }

    /// Get information about the room at a given tile position.
    pub fn get_room_at(&self, x: i32, y: i32) -> Option<RoomInfo> {
        let room = self.manager.get_room_at(IVec2::new(x, y))?;
        Some(dungeon_room_to_info(room))
    }

    /// Get all rooms on the current floor.
    pub fn get_all_rooms(&self) -> Vec<RoomInfo> {
        self.manager
            .current_floor()
            .map(|f| f.map.rooms.iter().map(dungeon_room_to_info).collect())
            .unwrap_or_default()
    }

    /// Mark a room as visited.
    pub fn mark_room_visited(&mut self, room_id: usize) {
        if let Some(floor) = self.manager.current_floor_mut() {
            if let Some(room) = floor.map.rooms.iter_mut().find(|r| r.id == room_id) {
                room.visited = true;
            }
        }
    }

    /// Mark a room as cleared (all enemies defeated).
    pub fn mark_room_cleared(&mut self, room_id: usize) {
        if let Some(floor) = self.manager.current_floor_mut() {
            if let Some(room) = floor.map.rooms.iter_mut().find(|r| r.id == room_id) {
                room.cleared = true;
                room.enemies.clear();
            }
            floor.check_cleared();
        }
    }

    /// Check if the current floor is fully cleared.
    pub fn is_floor_cleared(&self) -> bool {
        self.manager
            .current_floor()
            .map(|f| f.cleared)
            .unwrap_or(false)
    }

    // ── Fog of war ───────────────────────────────────────────────────────────

    /// Reveal tiles around the player's current position.
    pub fn update_fog(&mut self, player_x: i32, player_y: i32) {
        self.player_pos = IVec2::new(player_x, player_y);
        self.update_fog_at(self.player_pos);
    }

    /// Set the vision radius (e.g., reduced by Null boss or darkness biome).
    pub fn set_vision_radius(&mut self, radius: i32) {
        self.vision_radius = radius.max(2);
    }

    /// Current explored fraction of the floor (0.0 - 1.0).
    pub fn explored_fraction(&self) -> f32 {
        self.manager
            .current_floor()
            .map(|f| f.fog.explored_fraction())
            .unwrap_or(0.0)
    }

    // ── Minimap ──────────────────────────────────────────────────────────────

    /// Build minimap data for the current floor.
    pub fn get_minimap_data(&self) -> Option<MinimapData> {
        let floor = self.manager.current_floor()?;
        let glyphs = Minimap::render_minimap(&floor.map, self.player_pos, &floor.fog);
        let entries = glyphs
            .into_iter()
            .map(|g| MinimapEntry {
                x: g.x,
                y: g.y,
                ch: g.ch,
                color: g.color,
            })
            .collect();
        Some(MinimapData {
            entries,
            player_pos: (self.player_pos.x, self.player_pos.y),
        })
    }

    // ── Tile queries ─────────────────────────────────────────────────────────

    /// Get the tile type at a position.
    pub fn get_tile(&self, x: i32, y: i32) -> Option<TileBridge> {
        let floor = self.manager.current_floor()?;
        Some(tile_to_bridge(floor.map.get_tile(x, y)))
    }

    /// Check if a position is walkable.
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        self.manager
            .current_floor()
            .map(|f| f.map.get_tile(x, y).is_walkable())
            .unwrap_or(false)
    }

    /// Get walkable neighbors of a position.
    pub fn walkable_neighbors(&self, x: i32, y: i32) -> Vec<(i32, i32)> {
        self.manager
            .current_floor()
            .map(|f| {
                f.map
                    .walkable_neighbors(IVec2::new(x, y))
                    .into_iter()
                    .map(|p| (p.x, p.y))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get the biome color accent for the current floor (r, g, b).
    pub fn biome_accent(&self) -> (u8, u8, u8) {
        self.manager
            .current_floor()
            .map(|f| f.map.biome.properties().accent_color)
            .unwrap_or((128, 128, 128))
    }

    /// Get current floor's biome name.
    pub fn biome(&self) -> &'static str {
        self.manager
            .current_floor()
            .map(|f| biome_name(f.map.biome))
            .unwrap_or("Unknown")
    }

    // ── Internal helpers ─────────────────────────────────────────────────────

    fn update_fog_at(&mut self, pos: IVec2) {
        self.manager.reveal_around(pos, self.vision_radius);
    }

    fn build_floor_info(&self) -> FloorInfo {
        if let Some(floor) = self.manager.current_floor() {
            let map = &floor.map;
            let props = map.biome.properties();
            FloorInfo {
                floor_number: map.floor_number,
                biome: biome_name(map.biome).to_string(),
                biome_flavor: props.flavor_text.to_string(),
                ambient_light: props.ambient_light,
                music_vibe: props.music_vibe.to_string(),
                hazard: hazard_name(props.hazard_type).to_string(),
                room_count: map.rooms.len(),
                width: map.width,
                height: map.height,
                player_start: (map.player_start.x, map.player_start.y),
                exit_point: (map.exit_point.x, map.exit_point.y),
            }
        } else {
            FloorInfo {
                floor_number: 0,
                biome: "Unknown".into(),
                biome_flavor: String::new(),
                ambient_light: 0.5,
                music_vibe: String::new(),
                hazard: "None".into(),
                room_count: 0,
                width: 0,
                height: 0,
                player_start: (0, 0),
                exit_point: (0, 0),
            }
        }
    }
}
