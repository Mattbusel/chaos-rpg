//! Voronoi-based dungeon generation (v2).

use std::collections::VecDeque;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Shape of a dungeon room.
#[derive(Debug, Clone, PartialEq)]
pub enum RoomShape {
    Rect,
    Circle,
    Polygon(usize), // number of sides
}

/// A single dungeon room.
#[derive(Debug, Clone)]
pub struct Room {
    pub id: u32,
    pub center: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub shape: RoomShape,
    pub connections: Vec<u32>,
}

/// A corridor connecting two rooms.
#[derive(Debug, Clone)]
pub struct Corridor {
    pub from_room: u32,
    pub to_room: u32,
    pub waypoints: Vec<(f32, f32)>,
}

/// A Voronoi-based dungeon.
#[derive(Debug, Clone)]
pub struct VoronoiDungeon {
    pub rooms: Vec<Room>,
    pub corridors: Vec<Corridor>,
    pub width: f32,
    pub height: f32,
}

// ---------------------------------------------------------------------------
// LCG random
// ---------------------------------------------------------------------------

/// Linear Congruential Generator — returns a value in [0, 1).
pub fn lcg_next(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (*state >> 33) as f64 / (1u64 << 31) as f64
}

// ---------------------------------------------------------------------------
// Site generation
// ---------------------------------------------------------------------------

/// Generate `n` random sites inside the rectangle [0,width] × [0,height].
pub fn generate_sites(n: usize, width: f32, height: f32, seed: u64) -> Vec<(f32, f32)> {
    let mut state = seed;
    (0..n)
        .map(|_| {
            let x = lcg_next(&mut state) as f32 * width;
            let y = lcg_next(&mut state) as f32 * height;
            (x, y)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Lloyd relaxation (nearest-neighbour approximation)
// ---------------------------------------------------------------------------

/// Move each site toward the centroid of its Voronoi cell using a grid of
/// sample points.
pub fn lloyd_relaxation(
    sites: &mut Vec<(f32, f32)>,
    width: f32,
    height: f32,
    iterations: usize,
) {
    let grid_w: usize = 64;
    let grid_h: usize = 64;

    for _ in 0..iterations {
        let mut sum_x = vec![0.0f64; sites.len()];
        let mut sum_y = vec![0.0f64; sites.len()];
        let mut count = vec![0u32; sites.len()];

        for gy in 0..grid_h {
            for gx in 0..grid_w {
                let sx = (gx as f32 + 0.5) / grid_w as f32 * width;
                let sy = (gy as f32 + 0.5) / grid_h as f32 * height;

                // Find nearest site
                let mut best_idx = 0;
                let mut best_dist = f32::MAX;
                for (i, &(cx, cy)) in sites.iter().enumerate() {
                    let d = (cx - sx) * (cx - sx) + (cy - sy) * (cy - sy);
                    if d < best_dist {
                        best_dist = d;
                        best_idx = i;
                    }
                }
                sum_x[best_idx] += sx as f64;
                sum_y[best_idx] += sy as f64;
                count[best_idx] += 1;
            }
        }

        for (i, site) in sites.iter_mut().enumerate() {
            if count[i] > 0 {
                site.0 = (sum_x[i] / count[i] as f64) as f32;
                site.1 = (sum_y[i] / count[i] as f64) as f32;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Union-Find for Kruskal's MST
// ---------------------------------------------------------------------------

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, a: usize, b: usize) -> bool {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return false;
        }
        if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb;
        } else if self.rank[ra] > self.rank[rb] {
            self.parent[rb] = ra;
        } else {
            self.parent[rb] = ra;
            self.rank[ra] += 1;
        }
        true
    }
}

// ---------------------------------------------------------------------------
// VoronoiDungeon implementation
// ---------------------------------------------------------------------------

impl VoronoiDungeon {
    /// Generate a dungeon with `num_rooms` rooms.
    pub fn generate(num_rooms: usize, width: f32, height: f32, seed: u64) -> Self {
        let mut sites = generate_sites(num_rooms, width, height, seed);
        lloyd_relaxation(&mut sites, width, height, 3);

        // Create rooms at sites
        let mut state = seed.wrapping_add(1);
        let rooms: Vec<Room> = sites
            .iter()
            .enumerate()
            .map(|(i, &(cx, cy))| {
                let w = 3.0 + lcg_next(&mut state) as f32 * 5.0;
                let h = 3.0 + lcg_next(&mut state) as f32 * 5.0;
                let shape = match i % 3 {
                    0 => RoomShape::Rect,
                    1 => RoomShape::Circle,
                    _ => RoomShape::Polygon(6),
                };
                Room {
                    id: i as u32,
                    center: (cx, cy),
                    width: w,
                    height: h,
                    shape,
                    connections: Vec::new(),
                }
            })
            .collect();

        // Build all edges sorted by distance (Kruskal's MST)
        let n = rooms.len();
        let mut edges: Vec<(f32, usize, usize)> = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                let (x1, y1) = rooms[i].center;
                let (x2, y2) = rooms[j].center;
                let dist = ((x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1)).sqrt();
                edges.push((dist, i, j));
            }
        }
        edges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut uf = UnionFind::new(n);
        let mut mst_edges: Vec<(usize, usize)> = Vec::new();
        for (_, i, j) in edges {
            if uf.union(i, j) {
                mst_edges.push((i, j));
            }
        }

        // Build rooms with connections
        let mut rooms_with_conn = rooms;
        for &(i, j) in &mst_edges {
            rooms_with_conn[i].connections.push(j as u32);
            rooms_with_conn[j].connections.push(i as u32);
        }

        // Create corridors with L-shaped waypoints
        let corridors: Vec<Corridor> = mst_edges
            .iter()
            .map(|&(i, j)| {
                let (x1, y1) = rooms_with_conn[i].center;
                let (x2, y2) = rooms_with_conn[j].center;
                // L-shaped: go horizontally then vertically
                let waypoints = vec![(x1, y1), (x2, y1), (x2, y2)];
                Corridor {
                    from_room: i as u32,
                    to_room: j as u32,
                    waypoints,
                }
            })
            .collect();

        VoronoiDungeon {
            rooms: rooms_with_conn,
            corridors,
            width,
            height,
        }
    }

    /// Find the room containing point (x, y) — nearest center.
    pub fn room_at_point(&self, x: f32, y: f32) -> Option<&Room> {
        self.rooms.iter().min_by(|a, b| {
            let da = (a.center.0 - x).powi(2) + (a.center.1 - y).powi(2);
            let db = (b.center.0 - x).powi(2) + (b.center.1 - y).powi(2);
            da.partial_cmp(&db).unwrap()
        })
    }

    /// Entrance room: closest to (width*0.1, height*0.5).
    pub fn entrance_room(&self) -> &Room {
        let ex = self.width * 0.1;
        let ey = self.height * 0.5;
        self.rooms
            .iter()
            .min_by(|a, b| {
                let da = (a.center.0 - ex).powi(2) + (a.center.1 - ey).powi(2);
                let db = (b.center.0 - ex).powi(2) + (b.center.1 - ey).powi(2);
                da.partial_cmp(&db).unwrap()
            })
            .unwrap()
    }

    /// Boss room: farthest from the entrance room.
    pub fn boss_room(&self) -> &Room {
        let entrance = self.entrance_room();
        let (ex, ey) = entrance.center;
        self.rooms
            .iter()
            .max_by(|a, b| {
                let da = (a.center.0 - ex).powi(2) + (a.center.1 - ey).powi(2);
                let db = (b.center.0 - ex).powi(2) + (b.center.1 - ey).powi(2);
                da.partial_cmp(&db).unwrap()
            })
            .unwrap()
    }

    /// Render the dungeon as ASCII art.
    /// Rooms are marked 'R', corridor paths '.', walls '#'.
    pub fn to_ascii(&self, grid_w: usize, grid_h: usize) -> Vec<Vec<char>> {
        let mut grid = vec![vec!['#'; grid_w]; grid_h];

        let scale_x = |x: f32| -> usize {
            ((x / self.width) * (grid_w - 1) as f32).round() as usize
        };
        let scale_y = |y: f32| -> usize {
            ((y / self.height) * (grid_h - 1) as f32).round() as usize
        };

        // Draw corridors
        for corridor in &self.corridors {
            let pts = &corridor.waypoints;
            for seg in pts.windows(2) {
                let (x1, y1) = seg[0];
                let (x2, y2) = seg[1];
                let gx1 = scale_x(x1);
                let gy1 = scale_y(y1);
                let gx2 = scale_x(x2);
                let gy2 = scale_y(y2);
                // Rasterize segment
                let dx = (gx2 as i32 - gx1 as i32).abs();
                let dy = (gy2 as i32 - gy1 as i32).abs();
                let steps = dx.max(dy).max(1);
                for s in 0..=steps {
                    let t = s as f32 / steps as f32;
                    let px = (gx1 as f32 + t * (gx2 as f32 - gx1 as f32)).round() as usize;
                    let py = (gy1 as f32 + t * (gy2 as f32 - gy1 as f32)).round() as usize;
                    if px < grid_w && py < grid_h && grid[py][px] == '#' {
                        grid[py][px] = '.';
                    }
                }
            }
        }

        // Draw rooms
        for room in &self.rooms {
            let (cx, cy) = room.center;
            let half_w = (room.width / 2.0 / self.width * (grid_w - 1) as f32).round() as i32;
            let half_h = (room.height / 2.0 / self.height * (grid_h - 1) as f32).round() as i32;
            let gcx = scale_x(cx) as i32;
            let gcy = scale_y(cy) as i32;
            for dy in -half_h..=half_h {
                for dx in -half_w..=half_w {
                    let px = gcx + dx;
                    let py = gcy + dy;
                    if px >= 0 && py >= 0 && (px as usize) < grid_w && (py as usize) < grid_h {
                        grid[py as usize][px as usize] = 'R';
                    }
                }
            }
        }

        grid
    }
}

// ---------------------------------------------------------------------------
// BFS connectivity check (used in tests)
// ---------------------------------------------------------------------------

/// Returns true if all rooms are reachable from `start_id` via connections.
pub fn all_connected(rooms: &[Room], start_id: u32) -> bool {
    if rooms.is_empty() {
        return true;
    }
    let mut visited = std::collections::HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(start_id);
    visited.insert(start_id);
    while let Some(id) = queue.pop_front() {
        if let Some(room) = rooms.iter().find(|r| r.id == id) {
            for &next in &room.connections {
                if visited.insert(next) {
                    queue.push_back(next);
                }
            }
        }
    }
    visited.len() == rooms.len()
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_room_count() {
        let dungeon = VoronoiDungeon::generate(10, 100.0, 100.0, 42);
        assert_eq!(dungeon.rooms.len(), 10);
    }

    #[test]
    fn test_all_rooms_connected() {
        let dungeon = VoronoiDungeon::generate(8, 100.0, 100.0, 99);
        let entrance_id = dungeon.entrance_room().id;
        assert!(
            all_connected(&dungeon.rooms, entrance_id),
            "BFS from entrance should reach all rooms"
        );
    }

    #[test]
    fn test_ascii_grid_dimensions() {
        let dungeon = VoronoiDungeon::generate(5, 50.0, 50.0, 7);
        let grid = dungeon.to_ascii(40, 20);
        assert_eq!(grid.len(), 20);
        for row in &grid {
            assert_eq!(row.len(), 40);
        }
    }

    #[test]
    fn test_entrance_and_boss_different() {
        let dungeon = VoronoiDungeon::generate(6, 80.0, 80.0, 13);
        let eid = dungeon.entrance_room().id;
        let bid = dungeon.boss_room().id;
        // With 6 rooms the entrance and boss should usually differ
        // (they'd only be the same if there is 1 room)
        assert!(dungeon.rooms.len() <= 1 || eid != bid);
    }
}
