// CHAOS RPG — Run History (last 50 runs, persisted to JSON)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Summary of one finished run.
pub struct RunRecord {
    /// Date of the run.
    pub date:          String,
    /// Character name.
    pub name:          String,
    /// Class name.
    pub class:         String,
    /// Difficulty name.
    pub difficulty:    String,
    /// Game mode name.
    pub game_mode:     String,
    /// Deepest floor reached.
    pub floor:         u32,
    /// Final level.
    pub level:         u32,
    /// Enemies killed.
    pub kills:         u64,
    /// Final score.
    pub score:         u64,
    /// Total damage dealt.
    pub damage_dealt:  i64,
    /// Total damage taken.
    pub damage_taken:  i64,
    /// Biggest single hit.
    pub highest_hit:   i64,
    /// Spells cast.
    pub spells_cast:   u32,
    /// Items used.
    pub items_used:    u32,
    /// Gold at the end.
    pub gold:          i64,
    /// Misery index at the end.
    pub misery_index:  f64,
    /// Corruption stacks at the end.
    pub corruption:    u32,
    /// Final power tier name.
    pub power_tier:    String,
    /// What killed the character (empty if they won).
    pub cause_of_death:String,
    /// Seed of the run.
    pub seed:          u64,
    /// True if the run was won.
    pub won:           bool,
    /// Generated epitaph.
    pub epitaph:       String,
    /// Auto-generated narrative prose for this run (built on run end).
    #[serde(default)]
    pub auto_narrative: String,
    /// Player-authored character lore snapshot at run end.
    #[serde(default)]
    pub character_lore: Option<crate::character_lore::CharacterLore>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// The last runs, newest first, saved to `chaos_rpg_history.json`.
pub struct RunHistory {
    /// Past runs, newest first (at most 50).
    pub runs: Vec<RunRecord>,
}

impl RunHistory {
    const MAX_RUNS: usize = 50;

    /// Load the history from disk, or an empty history if there is none or it cannot be read.
    pub fn load() -> Self {
        let path = Self::path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(h) = serde_json::from_str::<RunHistory>(&data) {
                return h;
            }
        }
        Self::default()
    }

    /// Save the history to disk; write errors are ignored.
    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(Self::path(), json);
        }
    }

    /// Add a run at the front, keep the newest 50 and save.
    pub fn push(&mut self, record: RunRecord) {
        self.runs.insert(0, record); // newest first
        self.runs.truncate(Self::MAX_RUNS);
        self.save();
    }

    fn path() -> std::path::PathBuf {
        crate::paths::data_file("chaos_rpg_history.json")
    }
}
