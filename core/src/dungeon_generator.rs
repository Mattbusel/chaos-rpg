//! Dungeon generation: BSP tree room placement + corridor carving

#[derive(Clone, Debug)]
pub struct Rect {
    pub x: u32, pub y: u32, pub w: u32, pub h: u32,
}

impl Rect {
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self { Rect { x, y, w, h } }
    pub fn center(&self) -> (u32, u32) { (self.x + self.w / 2, self.y + self.h / 2) }
    pub fn area(&self) -> u32 { self.w * self.h }
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.w && self.x + self.w > other.x &&
        self.y < other.y + other.h && self.y + self.h > other.y
    }
    pub fn shrink(&self, margin: u32) -> Option<Rect> {
        if self.w <= margin * 2 + 2 || self.h <= margin * 2 + 2 { return None; }
        Some(Rect::new(self.x + margin, self.y + margin, self.w - margin * 2, self.h - margin * 2))
    }
}

pub enum RoomType { Start, Boss, Treasure, Monster, Empty, Shop }

pub struct Room {
    pub rect: Rect,
    pub room_type: RoomType,
    pub connections: Vec<usize>,
}

pub struct DungeonMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<bool>, // true = floor
    pub rooms: Vec<Room>,
}

impl DungeonMap {
    pub fn new(width: u32, height: u32) -> Self {
        DungeonMap {
            width, height,
            tiles: vec![false; (width * height) as usize],
            rooms: Vec::new(),
        }
    }

    pub fn set_floor(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            self.tiles[(y * self.width + x) as usize] = true;
        }
    }

    pub fn is_floor(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height && self.tiles[(y * self.width + x) as usize]
    }

    fn carve_rect(&mut self, rect: &Rect) {
        for y in rect.y..rect.y + rect.h {
            for x in rect.x..rect.x + rect.w {
                self.set_floor(x, y);
            }
        }
    }

    fn carve_corridor(&mut self, (x1, y1): (u32, u32), (x2, y2): (u32, u32), seed: u64) {
        // L-shaped corridor (choice based on seed)
        let state = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        if (state >> 11) & 1 == 0 {
            // Horizontal then vertical
            let (lx, rx) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
            for x in lx..=rx { self.set_floor(x, y1); }
            let (ly, ry) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
            for y in ly..=ry { self.set_floor(x2, y); }
        } else {
            let (ly, ry) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
            for y in ly..=ry { self.set_floor(x1, y); }
            let (lx, rx) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
            for x in lx..=rx { self.set_floor(x, y2); }
        }
    }

    pub fn render_ascii(&self) -> String {
        let mut out = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                out.push(if self.is_floor(x, y) { '.' } else { '#' });
            }
            out.push('\n');
        }
        out
    }
}

struct BspNode { bounds: Rect, left: Option<Box<BspNode>>, right: Option<Box<BspNode>>, room: Option<Rect> }

impl BspNode {
    fn new(bounds: Rect) -> Self { BspNode { bounds, left: None, right: None, room: None } }

    fn split(&mut self, min_size: u32, depth: u32, state: &mut u64) -> bool {
        if depth == 0 || (self.bounds.w < min_size * 2 && self.bounds.h < min_size * 2) {
            return false;
        }
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let horizontal = if self.bounds.w * 4 > self.bounds.h * 5 { false }
                         else if self.bounds.h * 4 > self.bounds.w * 5 { true }
                         else { (*state >> 11) & 1 == 1 };

        let (split_min, split_max) = if horizontal {
            (self.bounds.y + min_size, self.bounds.y + self.bounds.h - min_size)
        } else {
            (self.bounds.x + min_size, self.bounds.x + self.bounds.w - min_size)
        };
        if split_min >= split_max { return false; }
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let split = split_min + (*state >> 11) as u32 % (split_max - split_min);

        let (left_bounds, right_bounds) = if horizontal {
            (Rect::new(self.bounds.x, self.bounds.y, self.bounds.w, split - self.bounds.y),
             Rect::new(self.bounds.x, split, self.bounds.w, self.bounds.y + self.bounds.h - split))
        } else {
            (Rect::new(self.bounds.x, self.bounds.y, split - self.bounds.x, self.bounds.h),
             Rect::new(split, self.bounds.y, self.bounds.x + self.bounds.w - split, self.bounds.h))
        };

        let mut l = Box::new(BspNode::new(left_bounds));
        let mut r = Box::new(BspNode::new(right_bounds));
        l.split(min_size, depth - 1, state);
        r.split(min_size, depth - 1, state);
        self.left = Some(l);
        self.right = Some(r);
        true
    }

    fn create_rooms(&mut self, min_size: u32, max_size: u32, state: &mut u64) {
        if self.left.is_none() {
            if let Some(inner) = self.bounds.shrink(2) {
                *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let w = min_size.max(min_size + (*state >> 11) as u32 % (inner.w.saturating_sub(min_size) + 1)).min(inner.w).min(max_size);
                *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let h = min_size.max(min_size + (*state >> 11) as u32 % (inner.h.saturating_sub(min_size) + 1)).min(inner.h).min(max_size);
                *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let x = inner.x + (*state >> 11) as u32 % (inner.w - w + 1);
                *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let y = inner.y + (*state >> 11) as u32 % (inner.h - h + 1);
                self.room = Some(Rect::new(x, y, w, h));
            }
        } else {
            if let Some(l) = &mut self.left { l.create_rooms(min_size, max_size, state); }
            if let Some(r) = &mut self.right { r.create_rooms(min_size, max_size, state); }
        }
    }

    fn collect_rooms(&self) -> Vec<Rect> {
        if let Some(ref room) = self.room { return vec![room.clone()]; }
        let mut rooms = Vec::new();
        if let Some(l) = &self.left { rooms.extend(l.collect_rooms()); }
        if let Some(r) = &self.right { rooms.extend(r.collect_rooms()); }
        rooms
    }

    fn leftmost_room(&self) -> Option<Rect> {
        if let Some(ref room) = self.room { return Some(room.clone()); }
        self.left.as_ref().and_then(|l| l.leftmost_room())
            .or_else(|| self.right.as_ref().and_then(|r| r.leftmost_room()))
    }

    fn rightmost_room(&self) -> Option<Rect> {
        if let Some(ref room) = self.room { return Some(room.clone()); }
        self.right.as_ref().and_then(|r| r.rightmost_room())
            .or_else(|| self.left.as_ref().and_then(|l| l.leftmost_room()))
    }
}

pub struct DungeonGenerator {
    pub width: u32,
    pub height: u32,
    pub min_room_size: u32,
    pub max_room_size: u32,
    pub bsp_depth: u32,
}

impl DungeonGenerator {
    pub fn new(width: u32, height: u32) -> Self {
        DungeonGenerator { width, height, min_room_size: 4, max_room_size: 12, bsp_depth: 5 }
    }

    pub fn generate(&self, seed: u64) -> DungeonMap {
        let mut state = seed;
        let mut root = BspNode::new(Rect::new(0, 0, self.width, self.height));
        root.split(self.min_room_size + 4, self.bsp_depth, &mut state);
        root.create_rooms(self.min_room_size, self.max_room_size, &mut state);

        let rects = root.collect_rooms();
        let mut map = DungeonMap::new(self.width, self.height);

        // Carve rooms
        for rect in &rects {
            map.carve_rect(rect);
        }

        // Connect adjacent rooms with corridors
        for i in 0..rects.len().saturating_sub(1) {
            let c1 = rects[i].center();
            let c2 = rects[i + 1].center();
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            map.carve_corridor(c1, c2, state);
        }

        map
    }
}
