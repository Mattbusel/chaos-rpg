//! Game state serialization — simple key=value text format, no external crates.
//!
//! ## Format
//!
//! ```text
//! [SLOT N]
//! player_name=Alice
//! level=5
//! gold=250
//! current_floor=3
//! completed_quests=tutorial,first_dungeon
//! inventory=Sword,Chaos Crystal
//! max_hp=100
//! current_hp=80
//! attack=15
//! defense=8
//! experience=4200
//! ```

// ─── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum SaveError {
    InvalidFormat(String),
    MissingField(String),
    InvalidSlot,
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat(msg) => write!(f, "Invalid save format: {msg}"),
            Self::MissingField(field) => write!(f, "Missing save field: {field}"),
            Self::InvalidSlot => write!(f, "Invalid save slot"),
        }
    }
}

// ─── Data structures ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerStats {
    pub max_hp: u32,
    pub current_hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub experience: u64,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            max_hp: 100,
            current_hp: 100,
            attack: 10,
            defense: 5,
            experience: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SaveData {
    pub player_name: String,
    pub level: u32,
    pub gold: u32,
    pub current_floor: u32,
    pub completed_quests: Vec<String>,
    pub inventory: Vec<String>,
    pub stats: PlayerStats,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            player_name: "Unknown".to_string(),
            level: 1,
            gold: 0,
            current_floor: 1,
            completed_quests: Vec::new(),
            inventory: Vec::new(),
            stats: PlayerStats::default(),
        }
    }
}

// ─── Serializer ───────────────────────────────────────────────────────────────

pub struct SaveSerializer;

impl SaveSerializer {
    /// Serialize `data` to a `key=value\n` string.
    pub fn serialize(data: &SaveData) -> String {
        let mut out = String::new();
        out.push_str(&format!("player_name={}\n", data.player_name));
        out.push_str(&format!("level={}\n", data.level));
        out.push_str(&format!("gold={}\n", data.gold));
        out.push_str(&format!("current_floor={}\n", data.current_floor));
        out.push_str(&format!(
            "completed_quests={}\n",
            data.completed_quests.join(",")
        ));
        out.push_str(&format!("inventory={}\n", data.inventory.join(",")));
        out.push_str(&format!("max_hp={}\n", data.stats.max_hp));
        out.push_str(&format!("current_hp={}\n", data.stats.current_hp));
        out.push_str(&format!("attack={}\n", data.stats.attack));
        out.push_str(&format!("defense={}\n", data.stats.defense));
        out.push_str(&format!("experience={}\n", data.stats.experience));
        out
    }

    /// Deserialize from a `key=value\n` string (ignores blank lines and slot headers).
    pub fn deserialize(s: &str) -> Result<SaveData, SaveError> {
        let mut map: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();

        for line in s.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('[') {
                continue;
            }
            let mut parts = trimmed.splitn(2, '=');
            let key = parts
                .next()
                .ok_or_else(|| SaveError::InvalidFormat(format!("bad line: {trimmed}")))?;
            let val = parts
                .next()
                .ok_or_else(|| SaveError::InvalidFormat(format!("no value for key: {key}")))?;
            map.insert(key.trim(), val.trim());
        }

        fn get_str<'a>(
            map: &'a std::collections::HashMap<&str, &str>,
            key: &str,
        ) -> Result<&'a str, SaveError> {
            map.get(key)
                .copied()
                .ok_or_else(|| SaveError::MissingField(key.to_string()))
        }

        fn get_u32(
            map: &std::collections::HashMap<&str, &str>,
            key: &str,
        ) -> Result<u32, SaveError> {
            let s = map
                .get(key)
                .copied()
                .ok_or_else(|| SaveError::MissingField(key.to_string()))?;
            s.parse::<u32>()
                .map_err(|_| SaveError::InvalidFormat(format!("field '{key}' is not u32: {s}")))
        }

        fn get_u64(
            map: &std::collections::HashMap<&str, &str>,
            key: &str,
        ) -> Result<u64, SaveError> {
            let s = map
                .get(key)
                .copied()
                .ok_or_else(|| SaveError::MissingField(key.to_string()))?;
            s.parse::<u64>()
                .map_err(|_| SaveError::InvalidFormat(format!("field '{key}' is not u64: {s}")))
        }

        fn split_list(val: &str) -> Vec<String> {
            if val.is_empty() {
                Vec::new()
            } else {
                val.split(',').map(|s| s.trim().to_string()).collect()
            }
        }

        let player_name = get_str(&map, "player_name")?.to_string();
        let level = get_u32(&map, "level")?;
        let gold = get_u32(&map, "gold")?;
        let current_floor = get_u32(&map, "current_floor")?;
        let completed_quests = split_list(get_str(&map, "completed_quests")?);
        let inventory = split_list(get_str(&map, "inventory")?);
        let stats = PlayerStats {
            max_hp: get_u32(&map, "max_hp")?,
            current_hp: get_u32(&map, "current_hp")?,
            attack: get_u32(&map, "attack")?,
            defense: get_u32(&map, "defense")?,
            experience: get_u64(&map, "experience")?,
        };

        Ok(SaveData {
            player_name,
            level,
            gold,
            current_floor,
            completed_quests,
            inventory,
            stats,
        })
    }
}

// ─── Save Manager ─────────────────────────────────────────────────────────────

pub struct SaveManager {
    pub slot_count: usize,
}

impl SaveManager {
    pub fn new(slot_count: usize) -> Self {
        Self { slot_count }
    }

    /// Serialize `data` wrapped in a `[SLOT N]` header.
    pub fn save_to_string(&self, data: &SaveData, slot: usize) -> String {
        if slot >= self.slot_count {
            return format!("[ERROR: invalid slot {slot}]\n");
        }
        let mut out = format!("[SLOT {slot}]\n");
        out.push_str(&SaveSerializer::serialize(data));
        out
    }

    /// Find the `[SLOT N]` section in `s` and deserialize it.
    pub fn load_from_string(&self, s: &str, slot: usize) -> Result<SaveData, SaveError> {
        if slot >= self.slot_count {
            return Err(SaveError::InvalidSlot);
        }
        let header = format!("[SLOT {slot}]");
        let mut in_slot = false;
        let mut slot_lines: Vec<&str> = Vec::new();

        for line in s.lines() {
            if line.trim() == header {
                in_slot = true;
                continue;
            }
            if in_slot {
                // Stop at the next slot header.
                if line.trim().starts_with("[SLOT ") {
                    break;
                }
                slot_lines.push(line);
            }
        }

        if slot_lines.is_empty() {
            return Err(SaveError::InvalidFormat(format!("Slot {slot} not found")));
        }

        SaveSerializer::deserialize(&slot_lines.join("\n"))
    }

    /// Return which slot numbers are populated in the combined string.
    pub fn list_slots(&self, s: &str) -> Vec<usize> {
        let mut slots = Vec::new();
        for line in s.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("[SLOT ") && trimmed.ends_with(']') {
                let inner = &trimmed[6..trimmed.len() - 1];
                if let Ok(n) = inner.parse::<usize>() {
                    slots.push(n);
                }
            }
        }
        slots
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> SaveData {
        SaveData {
            player_name: "Alice".to_string(),
            level: 7,
            gold: 350,
            current_floor: 4,
            completed_quests: vec!["tutorial".to_string(), "first_dungeon".to_string()],
            inventory: vec!["Sword".to_string(), "Chaos Crystal".to_string()],
            stats: PlayerStats {
                max_hp: 150,
                current_hp: 120,
                attack: 20,
                defense: 12,
                experience: 8500,
            },
        }
    }

    #[test]
    fn roundtrip_serialize_deserialize() {
        let data = sample_data();
        let s = SaveSerializer::serialize(&data);
        let recovered = SaveSerializer::deserialize(&s).unwrap();
        assert_eq!(data, recovered);
    }

    #[test]
    fn empty_lists_roundtrip() {
        let data = SaveData {
            player_name: "Bob".to_string(),
            completed_quests: Vec::new(),
            inventory: Vec::new(),
            ..Default::default()
        };
        let s = SaveSerializer::serialize(&data);
        let recovered = SaveSerializer::deserialize(&s).unwrap();
        assert!(recovered.completed_quests.is_empty());
        assert!(recovered.inventory.is_empty());
    }

    #[test]
    fn missing_field_error() {
        let s = "player_name=Alice\nlevel=1\ngold=0\ncurrent_floor=1\n";
        let err = SaveSerializer::deserialize(s).unwrap_err();
        assert!(matches!(err, SaveError::MissingField(_)));
    }

    #[test]
    fn slot_manager_roundtrip() {
        let mgr = SaveManager::new(3);
        let data = sample_data();
        let s = mgr.save_to_string(&data, 0);
        let recovered = mgr.load_from_string(&s, 0).unwrap();
        assert_eq!(data, recovered);
    }

    #[test]
    fn multiple_slots() {
        let mgr = SaveManager::new(3);
        let d0 = sample_data();
        let mut d1 = sample_data();
        d1.player_name = "Bob".to_string();
        d1.level = 3;

        let mut combined = mgr.save_to_string(&d0, 0);
        combined.push_str(&mgr.save_to_string(&d1, 1));

        let slots = mgr.list_slots(&combined);
        assert!(slots.contains(&0));
        assert!(slots.contains(&1));

        let recovered0 = mgr.load_from_string(&combined, 0).unwrap();
        let recovered1 = mgr.load_from_string(&combined, 1).unwrap();
        assert_eq!(recovered0.player_name, "Alice");
        assert_eq!(recovered1.player_name, "Bob");
        assert_eq!(recovered1.level, 3);
    }

    #[test]
    fn invalid_slot_returns_error() {
        let mgr = SaveManager::new(2);
        let data = sample_data();
        let s = mgr.save_to_string(&data, 5); // slot 5 >= slot_count 2
        // save_to_string with invalid slot returns error message string, not valid data.
        let err = mgr.load_from_string(&s, 5).unwrap_err();
        assert_eq!(err, SaveError::InvalidSlot);
    }

    #[test]
    fn slot_not_found_error() {
        let mgr = SaveManager::new(3);
        let data = sample_data();
        let s = mgr.save_to_string(&data, 0);
        // Trying to load slot 1 which was never written.
        let err = mgr.load_from_string(&s, 1).unwrap_err();
        assert!(matches!(err, SaveError::InvalidFormat(_)));
    }
}
