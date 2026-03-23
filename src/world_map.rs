//! Overworld map with rooms and connections.
//!
//! Provides a graph-based world map where rooms are connected and the player
//! can navigate between them. Supports BFS pathfinding and exploration tracking.

use std::collections::{HashMap, HashSet, VecDeque};

// ─── ROOM TYPE ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoomType {
    Dungeon,
    Town,
    Wilderness,
    Cave,
    Temple,
    Port,
}

impl RoomType {
    pub fn name(&self) -> &str {
        match self {
            RoomType::Dungeon => "Dungeon",
            RoomType::Town => "Town",
            RoomType::Wilderness => "Wilderness",
            RoomType::Cave => "Cave",
            RoomType::Temple => "Temple",
            RoomType::Port => "Port",
        }
    }
}

// ─── ROOM ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub description: String,
    pub room_type: RoomType,
    pub connections: Vec<String>,
    pub visited: bool,
    pub npcs: Vec<String>,
    pub items: Vec<String>,
}

impl Room {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        room_type: RoomType,
    ) -> Self {
        Room {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            room_type,
            connections: Vec::new(),
            visited: false,
            npcs: Vec::new(),
            items: Vec::new(),
        }
    }
}

// ─── MAP ERROR ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapError {
    RoomNotFound(String),
    AlreadyConnected(String, String),
    NoPath(String, String),
}

impl std::fmt::Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapError::RoomNotFound(id) => write!(f, "Room not found: {}", id),
            MapError::AlreadyConnected(a, b) => {
                write!(f, "Rooms {} and {} are already connected", a, b)
            }
            MapError::NoPath(from, to) => write!(f, "No path from {} to {}", from, to),
        }
    }
}

// ─── WORLD MAP ────────────────────────────────────────────────────────────────

pub struct WorldMap {
    pub rooms: HashMap<String, Room>,
    pub current_room: String,
}

impl WorldMap {
    /// Create an empty world map starting in the given room ID.
    pub fn new(starting_room_id: impl Into<String>) -> Self {
        WorldMap {
            rooms: HashMap::new(),
            current_room: starting_room_id.into(),
        }
    }

    /// Add a room to the map.
    pub fn add_room(&mut self, room: Room) {
        self.rooms.insert(room.id.clone(), room);
    }

    /// Connect two rooms. If `bidirectional`, add the link in both directions.
    pub fn connect(
        &mut self,
        from: &str,
        to: &str,
        bidirectional: bool,
    ) -> Result<(), MapError> {
        if !self.rooms.contains_key(from) {
            return Err(MapError::RoomNotFound(from.to_string()));
        }
        if !self.rooms.contains_key(to) {
            return Err(MapError::RoomNotFound(to.to_string()));
        }

        // Add forward connection
        {
            let room = self.rooms.get_mut(from).unwrap();
            if !room.connections.contains(&to.to_string()) {
                room.connections.push(to.to_string());
            }
        }

        if bidirectional {
            let room = self.rooms.get_mut(to).unwrap();
            if !room.connections.contains(&from.to_string()) {
                room.connections.push(from.to_string());
            }
        }

        Ok(())
    }

    /// Move the player to the given room, mark it visited, and return a reference to the room.
    pub fn move_to(&mut self, room_id: &str) -> Result<&Room, MapError> {
        if !self.rooms.contains_key(room_id) {
            return Err(MapError::RoomNotFound(room_id.to_string()));
        }
        self.current_room = room_id.to_string();
        let room = self.rooms.get_mut(room_id).unwrap();
        room.visited = true;
        Ok(room)
    }

    /// Return a reference to the current room.
    pub fn current(&self) -> &Room {
        self.rooms
            .get(&self.current_room)
            .expect("current_room must always point to a valid room")
    }

    /// Return references to all rooms connected from the current room.
    pub fn available_exits(&self) -> Vec<&Room> {
        let current = self.current();
        current
            .connections
            .iter()
            .filter_map(|id| self.rooms.get(id))
            .collect()
    }

    /// BFS to find the shortest path from `from` to `to`.
    /// Returns the list of room IDs including both endpoints, or None if no path exists.
    pub fn find_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        if !self.rooms.contains_key(from) || !self.rooms.contains_key(to) {
            return None;
        }
        if from == to {
            return Some(vec![from.to_string()]);
        }

        let mut queue: VecDeque<Vec<String>> = VecDeque::new();
        let mut visited: HashSet<String> = HashSet::new();

        queue.push_back(vec![from.to_string()]);
        visited.insert(from.to_string());

        while let Some(path) = queue.pop_front() {
            let current_id = path.last().unwrap();
            if let Some(room) = self.rooms.get(current_id) {
                for neighbor_id in &room.connections {
                    if neighbor_id == to {
                        let mut full_path = path.clone();
                        full_path.push(neighbor_id.clone());
                        return Some(full_path);
                    }
                    if !visited.contains(neighbor_id) {
                        visited.insert(neighbor_id.clone());
                        let mut new_path = path.clone();
                        new_path.push(neighbor_id.clone());
                        queue.push_back(new_path);
                    }
                }
            }
        }

        None
    }

    /// Return all rooms that have not yet been visited.
    pub fn unexplored_rooms(&self) -> Vec<&Room> {
        self.rooms.values().filter(|r| !r.visited).collect()
    }

    /// Build a starter world with 6 interconnected rooms.
    pub fn starter_world() -> Self {
        let mut map = WorldMap::new("starting_town");

        let mut starting_town = Room::new(
            "starting_town",
            "Thornwall",
            "A modest walled town at the edge of civilization. The smell of bread and fear fills the air.",
            RoomType::Town,
        );
        starting_town.npcs = vec!["Elder Brynn".to_string(), "Merchant Holt".to_string()];
        starting_town.items = vec!["Rusty Sword".to_string()];
        starting_town.visited = true;

        let forest = Room::new(
            "forest",
            "Mirewood Forest",
            "Ancient trees loom overhead. Strange lights flicker between the trunks at dusk.",
            RoomType::Wilderness,
        );

        let cave = Room::new(
            "cave",
            "Gloomhaven Cave",
            "A damp cavern carved into the hillside. Echoes suggest it goes much deeper than it looks.",
            RoomType::Cave,
        );

        let mut dungeon_entrance = Room::new(
            "dungeon_entrance",
            "The Sunken Gate",
            "Crumbling stone stairs descend into darkness. Carved runes warn of the math-cursed horrors below.",
            RoomType::Dungeon,
        );
        dungeon_entrance.items = vec!["Torch Bundle".to_string()];

        let mut market_town = Room::new(
            "market_town",
            "Crossroads Market",
            "A bustling trade hub where merchants from all directions converge. Gold flows freely here.",
            RoomType::Town,
        );
        market_town.npcs = vec![
            "Arms Dealer".to_string(),
            "Alchemist Selva".to_string(),
            "Mysterious Hooded Figure".to_string(),
        ];

        let port = Room::new(
            "port",
            "Saltbreak Port",
            "A seaside port town where ships come and go with the tide. The harbor smells of fish and opportunity.",
            RoomType::Port,
        );

        map.add_room(starting_town);
        map.add_room(forest);
        map.add_room(cave);
        map.add_room(dungeon_entrance);
        map.add_room(market_town);
        map.add_room(port);

        // Connections
        map.connect("starting_town", "forest", true).unwrap();
        map.connect("starting_town", "market_town", true).unwrap();
        map.connect("forest", "cave", true).unwrap();
        map.connect("cave", "dungeon_entrance", true).unwrap();
        map.connect("market_town", "port", true).unwrap();
        map.connect("forest", "dungeon_entrance", false).unwrap();

        map
    }
}

// ─── TESTS ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_world() -> WorldMap {
        WorldMap::starter_world()
    }

    #[test]
    fn move_to_updates_current_room() {
        let mut map = make_world();
        assert_eq!(map.current_room, "starting_town");

        let result = map.move_to("forest");
        assert!(result.is_ok());
        assert_eq!(map.current_room, "forest");
    }

    #[test]
    fn move_to_marks_room_visited() {
        let mut map = make_world();
        let cave_visited_before = map.rooms.get("cave").unwrap().visited;
        assert!(!cave_visited_before);

        map.move_to("forest").unwrap();
        map.move_to("cave").unwrap();
        assert!(map.rooms.get("cave").unwrap().visited);
    }

    #[test]
    fn move_to_unknown_room_returns_error() {
        let mut map = make_world();
        let result = map.move_to("nonexistent");
        assert_eq!(result, Err(MapError::RoomNotFound("nonexistent".to_string())));
    }

    #[test]
    fn find_path_finds_shortest() {
        let map = make_world();
        // starting_town -> forest -> cave -> dungeon_entrance
        let path = map.find_path("starting_town", "dungeon_entrance");
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.first().unwrap(), "starting_town");
        assert_eq!(path.last().unwrap(), "dungeon_entrance");
        // Direct path via forest -> dungeon is shorter than via market_town
        assert!(path.len() <= 4);
    }

    #[test]
    fn find_path_same_room() {
        let map = make_world();
        let path = map.find_path("starting_town", "starting_town");
        assert_eq!(path, Some(vec!["starting_town".to_string()]));
    }

    #[test]
    fn find_path_no_path_returns_none() {
        let mut map = WorldMap::new("island_a");
        let island_a = Room::new("island_a", "Island A", "Isolated.", RoomType::Wilderness);
        let island_b = Room::new("island_b", "Island B", "Also isolated.", RoomType::Wilderness);
        map.add_room(island_a);
        map.add_room(island_b);
        // No connection between them
        let path = map.find_path("island_a", "island_b");
        assert!(path.is_none());
    }

    #[test]
    fn connect_creates_bidirectional_link() {
        let mut map = WorldMap::new("a");
        let a = Room::new("a", "A", ".", RoomType::Wilderness);
        let b = Room::new("b", "B", ".", RoomType::Wilderness);
        map.add_room(a);
        map.add_room(b);

        map.connect("a", "b", true).unwrap();

        assert!(map.rooms.get("a").unwrap().connections.contains(&"b".to_string()));
        assert!(map.rooms.get("b").unwrap().connections.contains(&"a".to_string()));
    }

    #[test]
    fn connect_unidirectional() {
        let mut map = WorldMap::new("a");
        let a = Room::new("a", "A", ".", RoomType::Wilderness);
        let b = Room::new("b", "B", ".", RoomType::Wilderness);
        map.add_room(a);
        map.add_room(b);

        map.connect("a", "b", false).unwrap();

        assert!(map.rooms.get("a").unwrap().connections.contains(&"b".to_string()));
        assert!(!map.rooms.get("b").unwrap().connections.contains(&"a".to_string()));
    }

    #[test]
    fn unexplored_rooms_excludes_visited() {
        let map = make_world();
        let unexplored = map.unexplored_rooms();
        // starting_town is visited by default
        assert!(!unexplored.iter().any(|r| r.id == "starting_town"));
        // Other rooms are unexplored
        assert!(unexplored.iter().any(|r| r.id == "forest"));
    }

    #[test]
    fn available_exits_returns_connected_rooms() {
        let map = make_world();
        let exits = map.available_exits();
        let exit_ids: Vec<&str> = exits.iter().map(|r| r.id.as_str()).collect();
        assert!(exit_ids.contains(&"forest"));
        assert!(exit_ids.contains(&"market_town"));
    }
}
