use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Biome {
    Ocean,
    Desert,
    Forest,
    Grassland,
    Mountain,
    Tundra,
    Swamp,
    Volcano,
}

#[derive(Debug, Clone)]
pub struct TerrainTile {
    pub x: u32,
    pub y: u32,
    pub elevation: f32,
    pub moisture: f32,
    pub biome: Biome,
    pub passable: bool,
}

pub struct WorldMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<TerrainTile>,
    pub seed: u64,
}

pub fn classify_biome(elevation: f32, moisture: f32) -> Biome {
    if elevation > 0.8 {
        if moisture < 0.2 {
            return Biome::Volcano;
        }
        return Biome::Mountain;
    }
    if elevation < 0.2 {
        return Biome::Ocean;
    }
    if elevation < 0.3 && moisture > 0.6 {
        return Biome::Swamp;
    }
    if elevation < 0.3 {
        return Biome::Grassland;
    }
    if moisture < 0.2 {
        return Biome::Desert;
    }
    if moisture > 0.6 {
        return Biome::Forest;
    }
    if moisture > 0.4 && elevation < 0.5 {
        return Biome::Grassland;
    }
    if elevation > 0.6 {
        return Biome::Tundra;
    }
    Biome::Grassland
}

pub fn lcg_next(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (*state >> 33) as f64 / (u32::MAX as f64 + 1.0)
}

pub fn integer_hash(x: u32, y: u32, seed: u64) -> f64 {
    let h = (x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263) ^ seed as u32)
        .wrapping_mul(2246822519);
    h as f64 / (u32::MAX as f64 + 1.0)
}

pub fn generate_noise(x: u32, y: u32, width: u32, height: u32, seed: u64, octaves: u32) -> f32 {
    let _ = (width, height); // unused params kept for API compatibility
    let mut value = 0.0f64;
    let mut amplitude = 1.0f64;
    let mut total_amplitude = 0.0f64;
    let mut freq = 1u32;

    for octave in 0..octaves {
        let sx = x.wrapping_mul(freq);
        let sy = y.wrapping_mul(freq);
        let h = integer_hash(sx, sy, seed.wrapping_add(octave as u64));
        value += h * amplitude;
        total_amplitude += amplitude;
        amplitude *= 0.5;
        freq = freq.wrapping_mul(2);
    }

    if total_amplitude > 0.0 {
        (value / total_amplitude) as f32
    } else {
        0.0
    }
}

impl WorldMap {
    pub fn generate(width: u32, height: u32, seed: u64) -> Self {
        let mut tiles = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let elevation = generate_noise(x, y, width, height, seed, 4);
                let moisture = generate_noise(x, y, width, height, seed.wrapping_add(99999), 4);
                let biome = classify_biome(elevation, moisture);
                let passable = !matches!(biome, Biome::Ocean | Biome::Volcano);
                tiles.push(TerrainTile {
                    x,
                    y,
                    elevation,
                    moisture,
                    biome,
                    passable,
                });
            }
        }

        WorldMap { width, height, tiles, seed }
    }

    pub fn tile_at(&self, x: u32, y: u32) -> Option<&TerrainTile> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.tiles.get((y * self.width + x) as usize)
    }

    pub fn biome_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for tile in &self.tiles {
            let name = format!("{:?}", tile.biome);
            *counts.entry(name).or_insert(0) += 1;
        }
        counts
    }

    pub fn passable_neighbors(&self, x: u32, y: u32) -> Vec<(u32, u32)> {
        let mut result = Vec::new();
        let dirs: &[(i64, i64)] = &[(0, -1), (0, 1), (-1, 0), (1, 0)];
        for &(dx, dy) in dirs {
            let nx = x as i64 + dx;
            let ny = y as i64 + dy;
            if nx < 0 || ny < 0 {
                continue;
            }
            let nx = nx as u32;
            let ny = ny as u32;
            if let Some(tile) = self.tile_at(nx, ny) {
                if tile.passable {
                    result.push((nx, ny));
                }
            }
        }
        result
    }

    pub fn find_path(&self, start: (u32, u32), end: (u32, u32)) -> Option<Vec<(u32, u32)>> {
        if start == end {
            return Some(vec![start]);
        }
        let start_tile = self.tile_at(start.0, start.1)?;
        if !start_tile.passable {
            return None;
        }
        let end_tile = self.tile_at(end.0, end.1)?;
        if !end_tile.passable {
            return None;
        }

        let mut visited: HashMap<(u32, u32), (u32, u32)> = HashMap::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited.insert(start, start);

        while let Some(current) = queue.pop_front() {
            if current == end {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = end;
                while node != start {
                    path.push(node);
                    node = visited[&node];
                }
                path.push(start);
                path.reverse();
                return Some(path);
            }
            for neighbor in self.passable_neighbors(current.0, current.1) {
                if !visited.contains_key(&neighbor) {
                    visited.insert(neighbor, current);
                    queue.push_back(neighbor);
                }
            }
        }
        None
    }

    pub fn to_ascii(&self) -> Vec<Vec<char>> {
        let mut grid = vec![vec![' '; self.width as usize]; self.height as usize];
        for tile in &self.tiles {
            let ch = match tile.biome {
                Biome::Ocean => 'O',
                Biome::Grassland => '.',
                Biome::Mountain => '^',
                Biome::Desert => 'D',
                Biome::Forest => 'F',
                Biome::Swamp => 'S',
                Biome::Tundra => 'T',
                Biome::Volcano => 'V',
            };
            grid[tile.y as usize][tile.x as usize] = ch;
        }
        grid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_dimensions() {
        let map = WorldMap::generate(32, 16, 42);
        assert_eq!(map.width, 32);
        assert_eq!(map.height, 16);
        assert_eq!(map.tiles.len(), 32 * 16);
    }

    #[test]
    fn test_biome_counts_positive() {
        let map = WorldMap::generate(32, 32, 7);
        let counts = map.biome_counts();
        assert!(!counts.is_empty());
        for (_biome, count) in &counts {
            assert!(*count > 0);
        }
    }

    #[test]
    fn test_ocean_percentage_under_50() {
        let map = WorldMap::generate(64, 64, 123);
        let counts = map.biome_counts();
        let ocean = counts.get("Ocean").copied().unwrap_or(0);
        let total = map.tiles.len();
        assert!((ocean as f64 / total as f64) < 0.5);
    }

    #[test]
    fn test_ascii_grid_correct_size() {
        let map = WorldMap::generate(20, 10, 55);
        let grid = map.to_ascii();
        assert_eq!(grid.len(), 10);
        for row in &grid {
            assert_eq!(row.len(), 20);
        }
    }

    #[test]
    fn test_pathfinding() {
        // Generate a small map and find two passable tiles to path between
        let map = WorldMap::generate(30, 30, 999);
        let passable: Vec<_> = map.tiles.iter().filter(|t| t.passable).collect();
        if passable.len() >= 2 {
            let start = (passable[0].x, passable[0].y);
            let end = (passable[passable.len() - 1].x, passable[passable.len() - 1].y);
            // Path may or may not exist; if it exists it should be valid
            if let Some(path) = map.find_path(start, end) {
                assert_eq!(path[0], start);
                assert_eq!(*path.last().unwrap(), end);
            }
        }
    }
}
