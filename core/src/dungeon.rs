//! Procedural dungeon map generator for CHAOS RPG.
//!
//! Every dungeon floor is generated from scratch using the chaos math pipeline.
//! The map is a grid of cells carved into rooms and corridors. Room shapes,
//! sizes, connections, and special features are all determined by seeded
//! chaos rolls — no two floors are ever the same.
//!
//! ## Architecture
//!
//! 1. [`DungeonGenerator`] carves rooms into a blank grid via BSP splitting.
//! 2. Rooms are connected by L-shaped corridors.
//! 3. Special cells (traps, shrines, treasure, stairs) are placed using chaos.
//! 4. [`DungeonMap`] holds the final grid and exposes ASCII rendering.
//!
//! ## Usage
//!
//! ```rust
//! use chaos_rpg::dungeon::{DungeonGenerator, DungeonConfig};
//!
//! let cfg = DungeonConfig::for_floor(3);
//! let map = DungeonGenerator::new(cfg, 12345).generate();
//! println!("{}", map.render_ascii());
//! println!("Rooms: {}", map.rooms.len());
//! ```

use crate::chaos_pipeline::chaos_roll_verbose;
use serde::{Deserialize, Serialize};

// ─── CELL TYPES ──────────────────────────────────────────────────────────────

/// What occupies a single grid cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Wall,
    Floor,
    Corridor,
    Door,
    StairsDown,
    StairsUp,
    Trap,
    Shrine,
    Treasure,
    Shop,
    Spawn,   // player start
    BossArena,
}

impl Cell {
    /// ASCII glyph for terminal rendering.
    pub fn glyph(self) -> char {
        match self {
            Cell::Wall => '#',
            Cell::Floor => '.',
            Cell::Corridor => '+',
            Cell::Door => 'D',
            Cell::StairsDown => '>',
            Cell::StairsUp => '<',
            Cell::Trap => '^',
            Cell::Shrine => '☯',
            Cell::Treasure => '$',
            Cell::Shop => 'S',
            Cell::Spawn => '@',
            Cell::BossArena => 'B',
        }
    }

    /// True when the player can walk through this cell.
    pub fn is_walkable(self) -> bool {
        !matches!(self, Cell::Wall)
    }
}

// ─── ROOM ────────────────────────────────────────────────────────────────────

/// An axis-aligned rectangular room carved into the dungeon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub room_type: RoomKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomKind {
    Normal,
    Treasure,
    Shrine,
    Shop,
    Boss,
    Spawn,
}

impl Room {
    pub fn center(&self) -> (usize, usize) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    /// True if the two rooms overlap (with a 1-cell buffer).
    pub fn overlaps(&self, other: &Room) -> bool {
        self.x.saturating_sub(1) < other.x + other.width + 1
            && self.x + self.width + 1 > other.x.saturating_sub(1)
            && self.y.saturating_sub(1) < other.y + other.height + 1
            && self.y + self.height + 1 > other.y.saturating_sub(1)
    }
}

// ─── CONFIG ──────────────────────────────────────────────────────────────────

/// Configuration for dungeon generation.
#[derive(Debug, Clone)]
pub struct DungeonConfig {
    /// Grid width in cells.
    pub width: usize,
    /// Grid height in cells.
    pub height: usize,
    /// Maximum number of room placement attempts.
    pub max_rooms: usize,
    /// Minimum room dimension (width or height).
    pub min_room_size: usize,
    /// Maximum room dimension.
    pub max_room_size: usize,
    /// Floor depth — affects special room probability and enemy tiers.
    pub floor: u32,
    /// Probability (0..1) that any floor tile becomes a trap after placement.
    pub trap_density: f64,
}

impl DungeonConfig {
    /// Sensible defaults scaled to the given floor number.
    pub fn for_floor(floor: u32) -> Self {
        let scale = 1.0 + floor as f64 * 0.15;
        Self {
            width: (60.0 * scale.min(2.0)) as usize,
            height: (30.0 * scale.min(2.0)) as usize,
            max_rooms: 8 + floor as usize * 2,
            min_room_size: 4,
            max_room_size: 10 + floor as usize,
            floor,
            trap_density: (0.02 + floor as f64 * 0.005).min(0.12),
        }
    }
}

// ─── MAP ─────────────────────────────────────────────────────────────────────

/// The fully-generated dungeon floor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonMap {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<Cell>>,
    pub rooms: Vec<Room>,
    pub floor: u32,
    /// Player starting position `(x, y)`.
    pub player_start: (usize, usize),
    /// Position of the downstairs exit.
    pub stairs_down: Option<(usize, usize)>,
}

impl DungeonMap {
    /// Render the dungeon as a multi-line ASCII string.
    pub fn render_ascii(&self) -> String {
        let mut out = String::with_capacity((self.width + 1) * self.height);
        for row in &self.grid {
            for cell in row {
                out.push(cell.glyph());
            }
            out.push('\n');
        }
        out
    }

    /// Return the cell at `(x, y)` or `Cell::Wall` if out of bounds.
    pub fn get(&self, x: usize, y: usize) -> Cell {
        if y < self.height && x < self.width {
            self.grid[y][x]
        } else {
            Cell::Wall
        }
    }

    /// Mutably set a cell, bounds-checked.
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        if y < self.height && x < self.width {
            self.grid[y][x] = cell;
        }
    }

    /// Count rooms of the given kind.
    pub fn room_count_of(&self, kind: RoomKind) -> usize {
        self.rooms.iter().filter(|r| r.room_type == kind).count()
    }

    /// Summary statistics for display.
    pub fn stats(&self) -> DungeonStats {
        let trap_count = self.grid.iter().flatten().filter(|&&c| c == Cell::Trap).count();
        let treasure_count = self.grid.iter().flatten().filter(|&&c| c == Cell::Treasure).count();
        let floor_count = self.grid.iter().flatten()
            .filter(|&&c| matches!(c, Cell::Floor | Cell::Corridor)).count();
        DungeonStats {
            floor: self.floor,
            room_count: self.rooms.len(),
            floor_tiles: floor_count,
            trap_count,
            treasure_count,
            has_boss: self.rooms.iter().any(|r| r.room_type == RoomKind::Boss),
            has_shop: self.rooms.iter().any(|r| r.room_type == RoomKind::Shop),
        }
    }
}

/// Summary info for a generated dungeon floor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonStats {
    pub floor: u32,
    pub room_count: usize,
    pub floor_tiles: usize,
    pub trap_count: usize,
    pub treasure_count: usize,
    pub has_boss: bool,
    pub has_shop: bool,
}

impl std::fmt::Display for DungeonStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Floor {} | {} rooms | {} open tiles | {} traps | {} treasure{}{}",
            self.floor,
            self.room_count,
            self.floor_tiles,
            self.trap_count,
            self.treasure_count,
            if self.has_boss { " | BOSS ROOM" } else { "" },
            if self.has_shop { " | Shop" } else { "" },
        )
    }
}

// ─── GENERATOR ───────────────────────────────────────────────────────────────

/// The procedural dungeon generator.
pub struct DungeonGenerator {
    cfg: DungeonConfig,
    seed: u64,
}

impl DungeonGenerator {
    pub fn new(cfg: DungeonConfig, seed: u64) -> Self {
        Self { cfg, seed }
    }

    /// Advance the internal LCG seed and return the next value.
    fn next_seed(&mut self) -> u64 {
        self.seed = self
            .seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.seed
    }

    /// Roll a value in `[lo, hi]` using the chaos pipeline.
    fn chaos_range(&mut self, lo: usize, hi: usize) -> usize {
        if lo >= hi {
            return lo;
        }
        let s = self.next_seed();
        let roll = chaos_roll_verbose(0.5, s);
        // Map [-1, 1] → [lo, hi]
        let norm = (roll.final_value + 1.0) / 2.0; // 0..1
        lo + (norm * (hi - lo + 1) as f64).floor() as usize % (hi - lo + 1)
    }

    /// Roll a float in `[0, 1)`.
    fn chaos_frac(&mut self) -> f64 {
        let s = self.next_seed();
        let roll = chaos_roll_verbose(0.3, s);
        (roll.final_value + 1.0) / 2.0
    }

    /// Full dungeon generation pipeline.
    pub fn generate(mut self) -> DungeonMap {
        let w = self.cfg.width;
        let h = self.cfg.height;

        // ── 1. Blank wall grid ───────────────────────────────────────────────
        let mut grid = vec![vec![Cell::Wall; w]; h];
        let mut rooms: Vec<Room> = Vec::new();

        // ── 2. Place rooms ───────────────────────────────────────────────────
        let max_rooms = self.cfg.max_rooms;
        let min_r = self.cfg.min_room_size;
        let max_r = self.cfg.max_room_size;

        for _ in 0..max_rooms * 5 {
            if rooms.len() >= max_rooms {
                break;
            }
            let rw = self.chaos_range(min_r, max_r);
            let rh = self.chaos_range(min_r, (max_r * 2 / 3).max(min_r));
            if rw + 2 > w || rh + 2 > h {
                continue;
            }
            let rx = self.chaos_range(1, w - rw - 1);
            let ry = self.chaos_range(1, h - rh - 1);

            let candidate = Room {
                x: rx,
                y: ry,
                width: rw,
                height: rh,
                room_type: RoomKind::Normal,
            };

            if rooms.iter().any(|r| r.overlaps(&candidate)) {
                continue;
            }
            // Carve floor tiles
            for row in ry..(ry + rh) {
                for col in rx..(rx + rw) {
                    grid[row][col] = Cell::Floor;
                }
            }
            rooms.push(candidate);
        }

        if rooms.is_empty() {
            // Fallback: carve one safe room
            let rx = 2;
            let ry = 2;
            let rw = 8.min(w.saturating_sub(4));
            let rh = 5.min(h.saturating_sub(4));
            for row in ry..(ry + rh) {
                for col in rx..(rx + rw) {
                    grid[row][col] = Cell::Floor;
                }
            }
            rooms.push(Room { x: rx, y: ry, width: rw, height: rh, room_type: RoomKind::Spawn });
        }

        // ── 3. Connect rooms with L-shaped corridors ─────────────────────────
        for i in 1..rooms.len() {
            let (ax, ay) = rooms[i - 1].center();
            let (bx, by) = rooms[i].center();
            // Randomly choose horizontal-first or vertical-first
            let h_first = self.chaos_frac() > 0.5;
            if h_first {
                Self::carve_h(&mut grid, ax, bx, ay);
                Self::carve_v(&mut grid, ay, by, bx);
            } else {
                Self::carve_v(&mut grid, ay, by, ax);
                Self::carve_h(&mut grid, ax, bx, by);
            }
            // Place a door at the corridor entry point
            let door_x = if h_first { bx } else { ax };
            let door_y = if h_first { ay } else { by };
            if grid[door_y][door_x] == Cell::Corridor {
                grid[door_y][door_x] = Cell::Door;
            }
        }

        // ── 4. Assign special room types ─────────────────────────────────────
        // First room = spawn, last room = boss if floor >= 3 or random
        let room_count = rooms.len();
        rooms[0].room_type = RoomKind::Spawn;
        let (sx, sy) = rooms[0].center();

        let boss_floor = self.cfg.floor >= 3 || self.chaos_frac() > 0.5;
        if boss_floor && room_count >= 2 {
            rooms[room_count - 1].room_type = RoomKind::Boss;
            let (bx, by) = rooms[room_count - 1].center();
            grid[by][bx] = Cell::BossArena;
        }

        // Treasure room: second room (if enough rooms exist)
        if room_count >= 4 {
            let treasure_idx = 1 + self.chaos_range(0, (room_count / 3).max(1) - 1);
            if rooms[treasure_idx].room_type == RoomKind::Normal {
                rooms[treasure_idx].room_type = RoomKind::Treasure;
                let (tx, ty) = rooms[treasure_idx].center();
                grid[ty][tx] = Cell::Treasure;
            }
        }

        // Shop room
        if room_count >= 5 {
            let shop_idx = room_count / 2 + self.chaos_range(0, 1);
            if rooms[shop_idx].room_type == RoomKind::Normal {
                rooms[shop_idx].room_type = RoomKind::Shop;
                let (shx, shy) = rooms[shop_idx].center();
                grid[shy][shx] = Cell::Shop;
            }
        }

        // Shrine in a random normal room
        for room in rooms.iter().filter(|r| r.room_type == RoomKind::Normal) {
            if self.chaos_frac() < 0.25 {
                let (shrx, shry) = room.center();
                if grid[shry][shrx] == Cell::Floor {
                    grid[shry][shrx] = Cell::Shrine;
                }
                break;
            }
        }

        // ── 5. Place stairs ──────────────────────────────────────────────────
        // Upstairs in spawn room, downstairs in boss or last room
        grid[sy][sx] = Cell::Spawn;
        let stairs_down_pos = if room_count >= 2 {
            let last_center = rooms[room_count - 1].center();
            let (dx, dy) = last_center;
            if grid[dy][dx] == Cell::Floor || grid[dy][dx] == Cell::BossArena {
                // Place stairs adjacent to center
                let sdx = (dx + 1).min(w - 1);
                grid[dy][sdx] = Cell::StairsDown;
                Some((sdx, dy))
            } else {
                None
            }
        } else {
            None
        };

        // ── 6. Scatter traps ─────────────────────────────────────────────────
        let trap_density = self.cfg.trap_density;
        for row in 0..h {
            for col in 0..w {
                if grid[row][col] == Cell::Floor && self.chaos_frac() < trap_density {
                    grid[row][col] = Cell::Trap;
                }
            }
        }

        DungeonMap {
            width: w,
            height: h,
            grid,
            rooms,
            floor: self.cfg.floor,
            player_start: (sx, sy),
            stairs_down: stairs_down_pos,
        }
    }

    // ── corridor carvers ─────────────────────────────────────────────────────

    fn carve_h(grid: &mut Vec<Vec<Cell>>, x0: usize, x1: usize, y: usize) {
        let (lo, hi) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
        let h = grid.len();
        let w = if h > 0 { grid[0].len() } else { 0 };
        for x in lo..=hi {
            if y < h && x < w && grid[y][x] == Cell::Wall {
                grid[y][x] = Cell::Corridor;
            }
        }
    }

    fn carve_v(grid: &mut Vec<Vec<Cell>>, y0: usize, y1: usize, x: usize) {
        let (lo, hi) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
        let h = grid.len();
        let w = if h > 0 { grid[0].len() } else { 0 };
        for y in lo..=hi {
            if y < h && x < w && grid[y][x] == Cell::Wall {
                grid[y][x] = Cell::Corridor;
            }
        }
    }
}

// ─── TESTS ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn gen(floor: u32, seed: u64) -> DungeonMap {
        let cfg = DungeonConfig::for_floor(floor);
        DungeonGenerator::new(cfg, seed).generate()
    }

    #[test]
    fn generates_at_least_one_room() {
        let map = gen(1, 42);
        assert!(!map.rooms.is_empty(), "must generate at least one room");
    }

    #[test]
    fn player_start_is_walkable() {
        let map = gen(1, 99);
        let (sx, sy) = map.player_start;
        assert!(
            map.get(sx, sy).is_walkable(),
            "player start must be walkable"
        );
    }

    #[test]
    fn no_out_of_bounds_writes() {
        let map = gen(2, 1234);
        assert_eq!(map.grid.len(), map.height);
        for row in &map.grid {
            assert_eq!(row.len(), map.width);
        }
    }

    #[test]
    fn ascii_render_has_correct_line_count() {
        let map = gen(1, 7);
        let rendered = map.render_ascii();
        let line_count = rendered.lines().count();
        assert_eq!(line_count, map.height);
    }

    #[test]
    fn higher_floors_produce_bigger_grids() {
        let m1 = gen(1, 0);
        let m5 = gen(5, 0);
        assert!(m5.width >= m1.width || m5.height >= m1.height);
    }

    #[test]
    fn stats_display_does_not_panic() {
        let map = gen(3, 555);
        let stats = map.stats();
        let _s = stats.to_string();
    }

    #[test]
    fn deterministic_for_same_seed() {
        let m1 = gen(2, 9999);
        let m2 = gen(2, 9999);
        assert_eq!(m1.rooms.len(), m2.rooms.len());
        assert_eq!(m1.player_start, m2.player_start);
    }
}
