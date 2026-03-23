//! Quest and objective tracking system for chaos-rpg.
//!
//! Tracks player quests through their full lifecycle: from `NotStarted` through
//! `InProgress` to `Completed`, `Failed`, or `Abandoned`.

use std::collections::HashMap;

// ─── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum QuestError {
    /// The quest ID does not exist in the log.
    QuestNotFound(String),
    /// Attempt to start a quest that is already in progress or done.
    AlreadyStarted(String),
    /// Prerequisites for the quest have not been completed.
    PrerequisitesNotMet(String),
    /// The objective ID does not exist on the given quest.
    ObjectiveNotFound(String),
}

impl std::fmt::Display for QuestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QuestNotFound(id) => write!(f, "Quest not found: {id}"),
            Self::AlreadyStarted(id) => write!(f, "Quest already started: {id}"),
            Self::PrerequisitesNotMet(id) => write!(f, "Prerequisites not met for: {id}"),
            Self::ObjectiveNotFound(id) => write!(f, "Objective not found: {id}"),
        }
    }
}

// ─── Status ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
    Abandoned,
}

// ─── Objective types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectiveType {
    KillEnemies {
        enemy_type: String,
        count: u32,
        killed: u32,
    },
    CollectItems {
        item_name: String,
        count: u32,
        collected: u32,
    },
    ReachLocation {
        location: String,
        reached: bool,
    },
    TalkToNpc {
        npc_name: String,
        talked: bool,
    },
    SurviveWaves {
        waves: u32,
        survived: u32,
    },
}

impl ObjectiveType {
    /// Returns true when the objective's completion condition is satisfied.
    pub fn is_complete(&self) -> bool {
        match self {
            Self::KillEnemies { count, killed, .. } => killed >= count,
            Self::CollectItems { count, collected, .. } => collected >= count,
            Self::ReachLocation { reached, .. } => *reached,
            Self::TalkToNpc { talked, .. } => *talked,
            Self::SurviveWaves { waves, survived, .. } => survived >= waves,
        }
    }

    /// Advance numeric progress by `amount`; boolean objectives are set to true when amount > 0.
    pub fn advance(&mut self, amount: u32) {
        match self {
            Self::KillEnemies { count, killed, .. } => {
                *killed = (*killed + amount).min(*count);
            }
            Self::CollectItems { count, collected, .. } => {
                *collected = (*collected + amount).min(*count);
            }
            Self::ReachLocation { reached, .. } => {
                if amount > 0 {
                    *reached = true;
                }
            }
            Self::TalkToNpc { talked, .. } => {
                if amount > 0 {
                    *talked = true;
                }
            }
            Self::SurviveWaves { waves, survived, .. } => {
                *survived = (*survived + amount).min(*waves);
            }
        }
    }
}

// ─── Objective ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Objective {
    pub id: String,
    pub description: String,
    pub objective_type: ObjectiveType,
    /// If true, this objective is not required to complete the quest.
    pub optional: bool,
}

impl Objective {
    pub fn new(id: impl Into<String>, description: impl Into<String>, objective_type: ObjectiveType) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            objective_type,
            optional: false,
        }
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub fn is_complete(&self) -> bool {
        self.objective_type.is_complete()
    }
}

// ─── Reward ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct QuestReward {
    pub gold: u32,
    pub xp: u32,
    pub items: Vec<String>,
}

// ─── Quest ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Quest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub objectives: Vec<Objective>,
    pub status: QuestStatus,
    pub reward_gold: u32,
    pub reward_xp: u32,
    pub reward_items: Vec<String>,
    pub prerequisites: Vec<String>,
}

impl Quest {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        reward_gold: u32,
        reward_xp: u32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            objectives: Vec::new(),
            status: QuestStatus::NotStarted,
            reward_gold,
            reward_xp,
            reward_items: Vec::new(),
            prerequisites: Vec::new(),
        }
    }

    pub fn with_objective(mut self, obj: Objective) -> Self {
        self.objectives.push(obj);
        self
    }

    pub fn with_prerequisite(mut self, prereq: impl Into<String>) -> Self {
        self.prerequisites.push(prereq.into());
        self
    }

    pub fn with_reward_item(mut self, item: impl Into<String>) -> Self {
        self.reward_items.push(item.into());
        self
    }

    /// Returns true when every non-optional objective is complete.
    pub fn all_required_objectives_complete(&self) -> bool {
        self.objectives
            .iter()
            .filter(|o| !o.optional)
            .all(|o| o.is_complete())
    }

    // ── Built-in quests ──────────────────────────────────────────────────────

    pub fn tutorial() -> Self {
        Self::new(
            "tutorial",
            "Welcome to Chaos",
            "Learn the basics: kill something, grab an item, and talk to the weird NPC.",
            50,
            100,
        )
        .with_objective(Objective::new(
            "kill_rat",
            "Kill 1 Rat",
            ObjectiveType::KillEnemies {
                enemy_type: "Rat".to_string(),
                count: 1,
                killed: 0,
            },
        ))
        .with_objective(Objective::new(
            "collect_herb",
            "Collect 1 Chaos Herb",
            ObjectiveType::CollectItems {
                item_name: "Chaos Herb".to_string(),
                count: 1,
                collected: 0,
            },
        ))
        .with_objective(
            Objective::new(
                "talk_to_herald",
                "Talk to the Herald of Broken Things",
                ObjectiveType::TalkToNpc {
                    npc_name: "Herald of Broken Things".to_string(),
                    talked: false,
                },
            )
            .optional(),
        )
    }

    pub fn first_dungeon() -> Self {
        Self::new(
            "first_dungeon",
            "Into the Glitch",
            "Reach Floor 5 of the dungeon and slay 10 enemies along the way.",
            200,
            500,
        )
        .with_prerequisite("tutorial")
        .with_objective(Objective::new(
            "reach_floor5",
            "Reach Dungeon Floor 5",
            ObjectiveType::ReachLocation {
                location: "Dungeon Floor 5".to_string(),
                reached: false,
            },
        ))
        .with_objective(Objective::new(
            "slay_10",
            "Slay 10 dungeon enemies",
            ObjectiveType::KillEnemies {
                enemy_type: "any".to_string(),
                count: 10,
                killed: 0,
            },
        ))
        .with_objective(
            Objective::new(
                "survive_3_waves",
                "Survive 3 ambush waves",
                ObjectiveType::SurviveWaves {
                    waves: 3,
                    survived: 0,
                },
            )
            .optional(),
        )
        .with_reward_item("Chaos Crystal")
    }

    pub fn slay_the_boss() -> Self {
        Self::new(
            "slay_the_boss",
            "The Equation Must Be Solved",
            "Find and defeat the Mathematical Abomination on Floor 10.",
            1000,
            2500,
        )
        .with_prerequisite("first_dungeon")
        .with_objective(Objective::new(
            "reach_floor10",
            "Reach Dungeon Floor 10",
            ObjectiveType::ReachLocation {
                location: "Dungeon Floor 10".to_string(),
                reached: false,
            },
        ))
        .with_objective(Objective::new(
            "kill_abomination",
            "Slay the Mathematical Abomination",
            ObjectiveType::KillEnemies {
                enemy_type: "Mathematical Abomination".to_string(),
                count: 1,
                killed: 0,
            },
        ))
        .with_reward_item("Prime Shard")
        .with_reward_item("Singularity")
    }
}

// ─── Quest Log ────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct QuestLog {
    quests: HashMap<String, Quest>,
}

impl QuestLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register quests in the log (does not start them).
    pub fn register(&mut self, quest: Quest) {
        self.quests.insert(quest.id.clone(), quest);
    }

    /// Register all built-in quests.
    pub fn register_defaults(&mut self) {
        self.register(Quest::tutorial());
        self.register(Quest::first_dungeon());
        self.register(Quest::slay_the_boss());
    }

    /// Start a quest. Fails if not found, already started, or prerequisites unmet.
    pub fn start_quest(&mut self, quest_id: &str) -> Result<(), QuestError> {
        // Check prerequisites first (need immutable borrow of the whole map).
        {
            let quest = self
                .quests
                .get(quest_id)
                .ok_or_else(|| QuestError::QuestNotFound(quest_id.to_string()))?;

            if quest.status != QuestStatus::NotStarted {
                return Err(QuestError::AlreadyStarted(quest_id.to_string()));
            }

            if !self.prerequisites_met(quest) {
                return Err(QuestError::PrerequisitesNotMet(quest_id.to_string()));
            }
        }

        let quest = self.quests.get_mut(quest_id).unwrap();
        quest.status = QuestStatus::InProgress;
        Ok(())
    }

    /// Advance an objective by `progress` units. Auto-completes the quest when all required objectives are done.
    pub fn update_objective(&mut self, quest_id: &str, objective_id: &str, progress: u32) {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            if quest.status != QuestStatus::InProgress {
                return;
            }
            if let Some(obj) = quest.objectives.iter_mut().find(|o| o.id == objective_id) {
                obj.objective_type.advance(progress);
            }
            // Auto-complete check.
            if quest.all_required_objectives_complete() {
                quest.status = QuestStatus::Completed;
            }
        }
    }

    /// Manually complete a quest and claim rewards. Returns None if not completable.
    pub fn complete_quest(&mut self, quest_id: &str) -> Option<QuestReward> {
        let quest = self.quests.get_mut(quest_id)?;
        if quest.status != QuestStatus::InProgress && quest.status != QuestStatus::Completed {
            return None;
        }
        quest.status = QuestStatus::Completed;
        Some(QuestReward {
            gold: quest.reward_gold,
            xp: quest.reward_xp,
            items: quest.reward_items.clone(),
        })
    }

    /// Abandon a quest in progress.
    pub fn abandon_quest(&mut self, quest_id: &str) {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            if quest.status == QuestStatus::InProgress {
                quest.status = QuestStatus::Abandoned;
            }
        }
    }

    pub fn active_quests(&self) -> Vec<&Quest> {
        self.quests
            .values()
            .filter(|q| q.status == QuestStatus::InProgress)
            .collect()
    }

    pub fn completed_quests(&self) -> Vec<&Quest> {
        self.quests
            .values()
            .filter(|q| q.status == QuestStatus::Completed)
            .collect()
    }

    /// Returns true if every prerequisite quest is completed.
    pub fn prerequisites_met(&self, quest: &Quest) -> bool {
        quest.prerequisites.iter().all(|prereq_id| {
            self.quests
                .get(prereq_id)
                .map(|q| q.status == QuestStatus::Completed)
                .unwrap_or(false)
        })
    }

    pub fn get(&self, quest_id: &str) -> Option<&Quest> {
        self.quests.get(quest_id)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_log() -> QuestLog {
        let mut log = QuestLog::new();
        log.register_defaults();
        log
    }

    #[test]
    fn start_quest_ok() {
        let mut log = make_log();
        assert!(log.start_quest("tutorial").is_ok());
        assert_eq!(log.get("tutorial").unwrap().status, QuestStatus::InProgress);
    }

    #[test]
    fn start_quest_already_started() {
        let mut log = make_log();
        log.start_quest("tutorial").unwrap();
        let err = log.start_quest("tutorial").unwrap_err();
        assert_eq!(err, QuestError::AlreadyStarted("tutorial".to_string()));
    }

    #[test]
    fn prerequisite_blocks_start() {
        let mut log = make_log();
        // first_dungeon requires tutorial to be completed.
        let err = log.start_quest("first_dungeon").unwrap_err();
        assert_eq!(err, QuestError::PrerequisitesNotMet("first_dungeon".to_string()));
    }

    #[test]
    fn prerequisite_met_after_completion() {
        let mut log = make_log();
        log.start_quest("tutorial").unwrap();
        // Complete tutorial by satisfying all required objectives.
        log.update_objective("tutorial", "kill_rat", 1);
        log.update_objective("tutorial", "collect_herb", 1);
        assert_eq!(log.get("tutorial").unwrap().status, QuestStatus::Completed);

        // Now first_dungeon can be started.
        assert!(log.start_quest("first_dungeon").is_ok());
    }

    #[test]
    fn objective_progress_and_auto_complete() {
        let mut log = make_log();
        log.start_quest("tutorial").unwrap();
        log.update_objective("tutorial", "kill_rat", 1);
        log.update_objective("tutorial", "collect_herb", 1);
        // Both required objectives done → auto-completed.
        assert_eq!(log.get("tutorial").unwrap().status, QuestStatus::Completed);
    }

    #[test]
    fn complete_quest_returns_reward() {
        let mut log = make_log();
        log.start_quest("tutorial").unwrap();
        log.update_objective("tutorial", "kill_rat", 1);
        log.update_objective("tutorial", "collect_herb", 1);
        let reward = log.complete_quest("tutorial").unwrap();
        assert_eq!(reward.gold, 50);
        assert_eq!(reward.xp, 100);
    }

    #[test]
    fn active_and_completed_lists() {
        let mut log = make_log();
        log.start_quest("tutorial").unwrap();
        assert_eq!(log.active_quests().len(), 1);
        assert_eq!(log.completed_quests().len(), 0);
        log.update_objective("tutorial", "kill_rat", 1);
        log.update_objective("tutorial", "collect_herb", 1);
        assert_eq!(log.active_quests().len(), 0);
        assert_eq!(log.completed_quests().len(), 1);
    }

    #[test]
    fn objective_capped_at_required_count() {
        let mut log = make_log();
        log.start_quest("tutorial").unwrap();
        // Over-advance kill count.
        log.update_objective("tutorial", "kill_rat", 999);
        let q = log.get("tutorial").unwrap();
        let obj = q.objectives.iter().find(|o| o.id == "kill_rat").unwrap();
        if let ObjectiveType::KillEnemies { count, killed, .. } = &obj.objective_type {
            assert_eq!(killed, count);
        } else {
            panic!("wrong type");
        }
    }

    #[test]
    fn quest_not_found_error() {
        let mut log = make_log();
        let err = log.start_quest("does_not_exist").unwrap_err();
        assert_eq!(err, QuestError::QuestNotFound("does_not_exist".to_string()));
    }
}
