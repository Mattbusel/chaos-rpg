//! Achievement and trophy tracking for chaos-rpg.
//!
//! Provides a fully self-contained achievement system with JSON
//! serialisation/deserialisation implemented manually (no serde dependency
//! for this module, though serde is available in the crate).

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Achievement
// ---------------------------------------------------------------------------

/// A single achievement definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Achievement {
    pub id:          String,
    pub name:        String,
    pub description: String,
    pub points:      u32,
    /// Hidden achievements are not shown until unlocked.
    pub hidden:      bool,
}

impl Achievement {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        points: u32,
        hidden: bool,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            points,
            hidden,
        }
    }
}

// ---------------------------------------------------------------------------
// AchievementTrigger
// ---------------------------------------------------------------------------

/// The event type that can unlock an achievement.
#[derive(Debug, Clone, PartialEq)]
pub enum AchievementTrigger {
    KillCount(u32),
    LevelReached(u32),
    GoldEarned(u64),
    QuestCompleted(String),
    ItemCrafted(String),
}

// ---------------------------------------------------------------------------
// AchievementProgress
// ---------------------------------------------------------------------------

/// Progress record for one achievement, per player.
#[derive(Debug, Clone, PartialEq)]
pub struct AchievementProgress {
    pub achievement_id: String,
    pub current:        u64,
    pub required:       u64,
    pub unlocked:       bool,
    /// Unix-style timestamp (seconds) when the achievement was unlocked.
    pub unlocked_at:    Option<u64>,
}

impl AchievementProgress {
    pub fn new(achievement_id: impl Into<String>, required: u64) -> Self {
        Self {
            achievement_id: achievement_id.into(),
            current:        0,
            required,
            unlocked:       false,
            unlocked_at:    None,
        }
    }

    /// Fraction complete, clamped to `[0.0, 1.0]`.
    pub fn fraction(&self) -> f64 {
        if self.required == 0 { return 1.0; }
        (self.current as f64 / self.required as f64).min(1.0)
    }
}

// ---------------------------------------------------------------------------
// AchievementManager
// ---------------------------------------------------------------------------

/// Manages achievement definitions and their required trigger thresholds.
pub struct AchievementManager {
    /// All registered achievement definitions, keyed by achievement id.
    definitions: HashMap<String, Achievement>,
    /// Mapping from achievement id to the trigger that unlocks it and its threshold.
    triggers:    Vec<(String, AchievementTrigger)>,
}

impl AchievementManager {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            triggers:    Vec::new(),
        }
    }

    /// Register an achievement and the trigger that unlocks it.
    pub fn register(&mut self, achievement: Achievement, trigger: AchievementTrigger) {
        self.triggers.push((achievement.id.clone(), trigger));
        self.definitions.insert(achievement.id.clone(), achievement);
    }

    /// Return a reference to an achievement definition.
    pub fn get_definition(&self, id: &str) -> Option<&Achievement> {
        self.definitions.get(id)
    }

    /// Check an incoming trigger event against all registered achievements and
    /// return the ids of any achievements that have just been unlocked.
    ///
    /// The caller is responsible for updating `PlayerAchievements` with the
    /// returned progress updates.
    pub fn check_trigger(
        &self,
        trigger: &AchievementTrigger,
        player: &mut PlayerAchievements,
        now_secs: u64,
    ) -> Vec<String> {
        let mut newly_unlocked = Vec::new();

        for (achievement_id, registered_trigger) in &self.triggers {
            // Skip already-unlocked achievements.
            if let Some(prog) = player.progress.get(achievement_id) {
                if prog.unlocked {
                    continue;
                }
            }

            let maybe_unlock = match (trigger, registered_trigger) {
                (AchievementTrigger::KillCount(n), AchievementTrigger::KillCount(req)) => {
                    let prog = player.progress
                        .entry(achievement_id.clone())
                        .or_insert_with(|| AchievementProgress::new(achievement_id, *req as u64));
                    prog.current = (*n).max(prog.current as u32) as u64;
                    prog.current >= prog.required
                }
                (AchievementTrigger::LevelReached(n), AchievementTrigger::LevelReached(req)) => {
                    let prog = player.progress
                        .entry(achievement_id.clone())
                        .or_insert_with(|| AchievementProgress::new(achievement_id, *req as u64));
                    prog.current = (*n).max(prog.current as u32) as u64;
                    prog.current >= prog.required
                }
                (AchievementTrigger::GoldEarned(n), AchievementTrigger::GoldEarned(req)) => {
                    let prog = player.progress
                        .entry(achievement_id.clone())
                        .or_insert_with(|| AchievementProgress::new(achievement_id, *req));
                    prog.current = (*n).max(prog.current);
                    prog.current >= prog.required
                }
                (AchievementTrigger::QuestCompleted(q), AchievementTrigger::QuestCompleted(req)) => {
                    if q == req {
                        let prog = player.progress
                            .entry(achievement_id.clone())
                            .or_insert_with(|| AchievementProgress::new(achievement_id, 1));
                        prog.current = 1;
                        true
                    } else {
                        false
                    }
                }
                (AchievementTrigger::ItemCrafted(item), AchievementTrigger::ItemCrafted(req)) => {
                    if item == req {
                        let prog = player.progress
                            .entry(achievement_id.clone())
                            .or_insert_with(|| AchievementProgress::new(achievement_id, 1));
                        prog.current = 1;
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if maybe_unlock {
                if let Some(prog) = player.progress.get_mut(achievement_id) {
                    if !prog.unlocked {
                        prog.unlocked    = true;
                        prog.unlocked_at = Some(now_secs);
                        newly_unlocked.push(achievement_id.clone());
                    }
                }
            }
        }

        newly_unlocked
    }
}

impl Default for AchievementManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// PlayerAchievements
// ---------------------------------------------------------------------------

/// All achievement progress for a single player.
#[derive(Debug, Clone, Default)]
pub struct PlayerAchievements {
    pub player_id: String,
    /// Progress keyed by achievement id.
    pub progress:  HashMap<String, AchievementProgress>,
}

impl PlayerAchievements {
    pub fn new(player_id: impl Into<String>) -> Self {
        Self {
            player_id: player_id.into(),
            progress:  HashMap::new(),
        }
    }

    /// Total achievement points earned.
    pub fn total_points(&self, manager: &AchievementManager) -> u32 {
        self.progress.values()
            .filter(|p| p.unlocked)
            .filter_map(|p| manager.get_definition(&p.achievement_id))
            .map(|a| a.points)
            .sum()
    }

    /// Number of unlocked achievements.
    pub fn unlocked_count(&self) -> usize {
        self.progress.values().filter(|p| p.unlocked).count()
    }

    // -----------------------------------------------------------------------
    // Manual JSON serialisation / deserialisation
    // -----------------------------------------------------------------------

    /// Serialise this player's achievements to a compact JSON string.
    ///
    /// Output format:
    /// ```json
    /// {"player_id":"p1","achievements":[{"id":"...","current":0,"required":10,"unlocked":false,"unlocked_at":null},...]}
    /// ```
    pub fn to_json(&self) -> String {
        let mut entries: Vec<String> = self.progress.values().map(|p| {
            let unlocked_at = match p.unlocked_at {
                Some(t) => t.to_string(),
                None    => "null".to_string(),
            };
            format!(
                r#"{{"id":{},"current":{},"required":{},"unlocked":{},"unlocked_at":{}}}"#,
                json_string(&p.achievement_id),
                p.current,
                p.required,
                p.unlocked,
                unlocked_at,
            )
        }).collect();
        entries.sort(); // deterministic output
        format!(
            r#"{{"player_id":{},"achievements":[{}]}}"#,
            json_string(&self.player_id),
            entries.join(","),
        )
    }

    /// Deserialise a [`PlayerAchievements`] from a JSON string produced by
    /// [`Self::to_json`].  Returns `None` on any parse error.
    pub fn from_json(json: &str) -> Option<Self> {
        let player_id = extract_json_string(json, "player_id")?;
        let achievements_start = json.find(r#""achievements":["#)? + r#""achievements":["#.len();
        let achievements_block = &json[achievements_start..];
        // Find the matching ']'
        let achievements_end = achievements_block.rfind(']')?;
        let achievements_json = &achievements_block[..achievements_end];

        let mut progress = HashMap::new();

        // Split on "},{"
        let raw_entries = split_json_objects(achievements_json);
        for entry in raw_entries {
            let id         = extract_json_string(entry, "id")?;
            let current    = extract_json_u64(entry, "current")?;
            let required   = extract_json_u64(entry, "required")?;
            let unlocked   = extract_json_bool(entry, "unlocked")?;
            let unlocked_at = extract_json_optional_u64(entry, "unlocked_at");

            progress.insert(id.clone(), AchievementProgress {
                achievement_id: id,
                current,
                required,
                unlocked,
                unlocked_at,
            });
        }

        Some(Self { player_id, progress })
    }
}

// ---------------------------------------------------------------------------
// JSON helpers (manual, no serde)
// ---------------------------------------------------------------------------

fn json_string(s: &str) -> String {
    // Minimal escaping: backslash and double-quote.
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

fn extract_json_string<'a>(json: &'a str, key: &str) -> Option<String> {
    let search = format!("\"{}\":\"", key);
    let start = json.find(&search)? + search.len();
    let rest  = &json[start..];
    let end   = rest.find('"')?;
    Some(rest[..end].replace("\\\"", "\"").replace("\\\\", "\\"))
}

fn extract_json_u64(json: &str, key: &str) -> Option<u64> {
    let search = format!("\"{}\":", key);
    let start  = json.find(&search)? + search.len();
    let rest   = json[start..].trim_start();
    // Read digits.
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

fn extract_json_bool(json: &str, key: &str) -> Option<bool> {
    let search = format!("\"{}\":", key);
    let start  = json.find(&search)? + search.len();
    let rest   = json[start..].trim_start();
    if rest.starts_with("true")  { return Some(true);  }
    if rest.starts_with("false") { return Some(false); }
    None
}

fn extract_json_optional_u64(json: &str, key: &str) -> Option<u64> {
    let search = format!("\"{}\":", key);
    let start  = json.find(&search)? + search.len();
    let rest   = json[start..].trim_start();
    if rest.starts_with("null") { return None; }
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// Split a JSON array body (without the outer `[` `]`) into individual object
/// strings by tracking brace depth.
fn split_json_objects(input: &str) -> Vec<&str> {
    let mut objects = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let bytes = input.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'{' => {
                if depth == 0 { start = i; }
                depth += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    objects.push(&input[start..=i]);
                }
            }
            _ => {}
        }
    }
    objects
}

// ---------------------------------------------------------------------------
// Predefined achievement catalogue builder
// ---------------------------------------------------------------------------

/// Populate a manager with a standard set of game achievements.
pub fn build_default_achievements(manager: &mut AchievementManager) {
    manager.register(
        Achievement::new("first_blood",    "First Blood",    "Kill your first enemy.",          10,  false),
        AchievementTrigger::KillCount(1),
    );
    manager.register(
        Achievement::new("serial_killer",  "Serial Killer",  "Kill 100 enemies.",               50,  false),
        AchievementTrigger::KillCount(100),
    );
    manager.register(
        Achievement::new("genocidal",      "Genocidal",      "Kill 1000 enemies.",              200, false),
        AchievementTrigger::KillCount(1000),
    );
    manager.register(
        Achievement::new("level_5",        "Novice Adventurer", "Reach level 5.",               20,  false),
        AchievementTrigger::LevelReached(5),
    );
    manager.register(
        Achievement::new("level_20",       "Seasoned Hero",  "Reach level 20.",                 75,  false),
        AchievementTrigger::LevelReached(20),
    );
    manager.register(
        Achievement::new("level_50",       "Legend",         "Reach level 50.",                300,  true),
        AchievementTrigger::LevelReached(50),
    );
    manager.register(
        Achievement::new("gold_1000",      "Coin Hoarder",   "Earn 1 000 gold.",                25,  false),
        AchievementTrigger::GoldEarned(1_000),
    );
    manager.register(
        Achievement::new("gold_1m",        "Millionaire",    "Earn 1 000 000 gold.",           500,  true),
        AchievementTrigger::GoldEarned(1_000_000),
    );
    manager.register(
        Achievement::new("first_quest",    "Quest Starter",  "Complete your first quest.",      15,  false),
        AchievementTrigger::QuestCompleted("intro_quest".into()),
    );
    manager.register(
        Achievement::new("master_crafter", "Master Crafter", "Craft a legendary item.",        100,  false),
        AchievementTrigger::ItemCrafted("legendary_sword".into()),
    );
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn build_manager() -> AchievementManager {
        let mut mgr = AchievementManager::new();
        build_default_achievements(&mut mgr);
        mgr
    }

    // --- Basic trigger tests ---

    #[test]
    fn kill_count_trigger_unlocks() {
        let mgr = build_manager();
        let mut player = PlayerAchievements::new("hero");
        let unlocked = mgr.check_trigger(&AchievementTrigger::KillCount(1), &mut player, 1000);
        assert!(unlocked.contains(&"first_blood".to_string()));
    }

    #[test]
    fn kill_count_does_not_unlock_before_threshold() {
        let mgr = build_manager();
        let mut player = PlayerAchievements::new("hero");
        let unlocked = mgr.check_trigger(&AchievementTrigger::KillCount(50), &mut player, 1000);
        assert!(!unlocked.contains(&"serial_killer".to_string()));
        assert!(!unlocked.contains(&"genocidal".to_string()));
    }

    #[test]
    fn level_trigger_unlocks() {
        let mgr = build_manager();
        let mut player = PlayerAchievements::new("hero");
        let unlocked = mgr.check_trigger(&AchievementTrigger::LevelReached(5), &mut player, 2000);
        assert!(unlocked.contains(&"level_5".to_string()));
    }

    #[test]
    fn gold_trigger_unlocks() {
        let mgr = build_manager();
        let mut player = PlayerAchievements::new("rich");
        let unlocked = mgr.check_trigger(&AchievementTrigger::GoldEarned(1_000), &mut player, 9999);
        assert!(unlocked.contains(&"gold_1000".to_string()));
    }

    #[test]
    fn quest_trigger_unlocks() {
        let mgr = build_manager();
        let mut player = PlayerAchievements::new("adventurer");
        let unlocked = mgr.check_trigger(
            &AchievementTrigger::QuestCompleted("intro_quest".into()),
            &mut player,
            100,
        );
        assert!(unlocked.contains(&"first_quest".to_string()));
    }

    #[test]
    fn wrong_quest_does_not_unlock() {
        let mgr = build_manager();
        let mut player = PlayerAchievements::new("adventurer");
        let unlocked = mgr.check_trigger(
            &AchievementTrigger::QuestCompleted("side_quest_1".into()),
            &mut player,
            100,
        );
        assert!(!unlocked.contains(&"first_quest".to_string()));
    }

    #[test]
    fn item_crafted_trigger_unlocks() {
        let mgr = build_manager();
        let mut player = PlayerAchievements::new("smith");
        let unlocked = mgr.check_trigger(
            &AchievementTrigger::ItemCrafted("legendary_sword".into()),
            &mut player,
            500,
        );
        assert!(unlocked.contains(&"master_crafter".to_string()));
    }

    #[test]
    fn achievement_does_not_unlock_twice() {
        let mgr = build_manager();
        let mut player = PlayerAchievements::new("hero");
        mgr.check_trigger(&AchievementTrigger::KillCount(1), &mut player, 100);
        let second = mgr.check_trigger(&AchievementTrigger::KillCount(1), &mut player, 200);
        assert!(!second.contains(&"first_blood".to_string()));
    }

    // --- Points ---

    #[test]
    fn total_points_accumulates() {
        let mgr = build_manager();
        let mut player = PlayerAchievements::new("hero");
        mgr.check_trigger(&AchievementTrigger::KillCount(1),   &mut player, 10);
        mgr.check_trigger(&AchievementTrigger::LevelReached(5), &mut player, 20);
        let pts = player.total_points(&mgr);
        assert_eq!(pts, 30); // 10 (first_blood) + 20 (level_5)
    }

    #[test]
    fn unlocked_count_increments() {
        let mgr = build_manager();
        let mut player = PlayerAchievements::new("hero");
        assert_eq!(player.unlocked_count(), 0);
        mgr.check_trigger(&AchievementTrigger::KillCount(1), &mut player, 1);
        assert_eq!(player.unlocked_count(), 1);
    }

    // --- JSON round-trip ---

    #[test]
    fn json_roundtrip_empty_player() {
        let player = PlayerAchievements::new("empty_player");
        let json   = player.to_json();
        let restored = PlayerAchievements::from_json(&json).expect("parse failed");
        assert_eq!(restored.player_id, "empty_player");
        assert!(restored.progress.is_empty());
    }

    #[test]
    fn json_roundtrip_with_unlocked_achievement() {
        let mgr = build_manager();
        let mut player = PlayerAchievements::new("tester");
        mgr.check_trigger(&AchievementTrigger::KillCount(1), &mut player, 42);

        let json     = player.to_json();
        let restored = PlayerAchievements::from_json(&json).expect("parse failed");

        assert_eq!(restored.player_id, "tester");
        let prog = restored.progress.get("first_blood").expect("missing progress");
        assert!(prog.unlocked);
        assert_eq!(prog.unlocked_at, Some(42));
    }

    #[test]
    fn json_roundtrip_with_multiple_achievements() {
        let mgr = build_manager();
        let mut player = PlayerAchievements::new("multi");
        mgr.check_trigger(&AchievementTrigger::KillCount(1),   &mut player, 1);
        mgr.check_trigger(&AchievementTrigger::LevelReached(5), &mut player, 2);
        mgr.check_trigger(&AchievementTrigger::GoldEarned(500), &mut player, 3);

        let json     = player.to_json();
        let restored = PlayerAchievements::from_json(&json).expect("parse");
        assert_eq!(restored.unlocked_count(), 2); // first_blood + level_5
    }

    // --- AchievementProgress helpers ---

    #[test]
    fn progress_fraction() {
        let mut p = AchievementProgress::new("test", 10);
        p.current = 5;
        assert!((p.fraction() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn progress_fraction_clamped_above_one() {
        let mut p = AchievementProgress::new("test", 10);
        p.current = 100;
        assert_eq!(p.fraction(), 1.0);
    }

    #[test]
    fn progress_fraction_zero_required() {
        let p = AchievementProgress::new("test", 0);
        assert_eq!(p.fraction(), 1.0);
    }
}
