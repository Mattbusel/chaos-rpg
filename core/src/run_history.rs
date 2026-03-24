// CHAOS RPG — Run History (last 50 runs, persisted to JSON)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub date:          String,
    pub name:          String,
    pub class:         String,
    pub difficulty:    String,
    pub game_mode:     String,
    pub floor:         u32,
    pub level:         u32,
    pub kills:         u64,
    pub score:         u64,
    pub damage_dealt:  i64,
    pub damage_taken:  i64,
    pub highest_hit:   i64,
    pub spells_cast:   u32,
    pub items_used:    u32,
    pub gold:          i64,
    pub misery_index:  f64,
    pub corruption:    u32,
    pub power_tier:    String,
    pub cause_of_death:String,
    pub seed:          u64,
    pub won:           bool,
    pub epitaph:       String,
    /// Auto-generated narrative prose for this run (built on run end).
    #[serde(default)]
    pub auto_narrative: String,
    /// Player-authored character lore snapshot at run end.
    #[serde(default)]
    pub character_lore: Option<crate::character_lore::CharacterLore>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RunHistory {
    pub runs: Vec<RunRecord>,
}

impl RunHistory {
    const MAX_RUNS: usize = 50;

    pub fn load() -> Self {
        let path = Self::path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(h) = serde_json::from_str::<RunHistory>(&data) {
                return h;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(Self::path(), json);
        }
    }

    pub fn push(&mut self, record: RunRecord) {
        self.runs.insert(0, record); // newest first
        self.runs.truncate(Self::MAX_RUNS);
        self.save();
    }

    fn path() -> std::path::PathBuf {
        let mut p = std::env::current_exe().unwrap_or_default();
        p.pop();
        p.push("chaos_rpg_history.json");
        p
    }
}
