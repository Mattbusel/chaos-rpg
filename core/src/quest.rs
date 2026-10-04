//! Quest and objective tracking system for chaos-rpg.
//!
//! Tracks player quests through their full lifecycle: from `NotStarted` through
//! `InProgress` to `Completed`, `Failed`, or `Abandoned`.

use std::collections::HashMap;

// ─── Error ────────────────────────────────────────────────────────────────────

/// Why a quest log operation failed.
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

/// Where a quest stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestStatus {
    /// Registered but not started.
    NotStarted,
    /// Started and not yet finished.
    InProgress,
    /// All required objectives done.
    Completed,
    /// Failed and can no longer be completed.
    Failed,
    /// Given up by the player.
    Abandoned,
}

// ─── Objective types ─────────────────────────────────────────────────────────

/// What an objective asks for, with its progress so far.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectiveType {
    /// Kill a number of enemies of one type.
    KillEnemies {
        /// Enemy type to kill.
        enemy_type: String,
        /// How many kills are needed.
        count: u32,
        /// Kills so far.
        killed: u32,
    },
    /// Collect a number of one item.
    CollectItems {
        /// Item to collect.
        item_name: String,
        /// How many are needed.
        count: u32,
        /// How many have been collected.
        collected: u32,
    },
    /// Reach a named place.
    ReachLocation {
        /// Place to reach.
        location: String,
        /// Whether it has been reached.
        reached: bool,
    },
    /// Talk to a named NPC.
    TalkToNpc {
        /// NPC to talk to.
        npc_name: String,
        /// Whether the conversation has happened.
        talked: bool,
    },
    /// Survive a number of enemy waves.
    SurviveWaves {
        /// Waves to survive.
        waves: u32,
        /// Waves survived so far.
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

/// One step of a quest.
#[derive(Debug, Clone)]
pub struct Objective {
    /// Identifier, unique within its quest.
    pub id: String,
    /// Text shown to the player.
    pub description: String,
    /// What the objective asks for and its progress.
    pub objective_type: ObjectiveType,
    /// If true, this objective is not required to complete the quest.
    pub optional: bool,
}

impl Objective {
    /// A required objective.
    pub fn new(id: impl Into<String>, description: impl Into<String>, objective_type: ObjectiveType) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            objective_type,
            optional: false,
        }
    }

    /// Mark the objective as optional.
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Whether the objective's target has been met.
    pub fn is_complete(&self) -> bool {
        self.objective_type.is_complete()
    }
}

// ─── Reward ───────────────────────────────────────────────────────────────────

/// What completing a quest pays out.
#[derive(Debug, Clone)]
pub struct QuestReward {
    /// Gold awarded.
    pub gold: u32,
    /// Experience awarded.
    pub xp: u32,
    /// Item names awarded.
    pub items: Vec<String>,
}

// ─── Quest ────────────────────────────────────────────────────────────────────

/// A quest: objectives, status, rewards and the quests it depends on.
#[derive(Debug, Clone)]
pub struct Quest {
    /// Unique quest identifier.
    pub id: String,
    /// Name shown to the player.
    pub name: String,
    /// Description shown to the player.
    pub description: String,
    /// Steps of the quest.
    pub objectives: Vec<Objective>,
    /// Current status.
    pub status: QuestStatus,
    /// Gold paid on completion.
    pub reward_gold: u32,
    /// Experience paid on completion.
    pub reward_xp: u32,
    /// Item names given on completion.
    pub reward_items: Vec<String>,
    /// Quest ids that must be completed before this one can start.
    pub prerequisites: Vec<String>,
}

impl Quest {
    /// A new, not started quest with no objectives.
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

    /// Add an objective.
    pub fn with_objective(mut self, obj: Objective) -> Self {
        self.objectives.push(obj);
        self
    }

    /// Add a prerequisite quest id.
    pub fn with_prerequisite(mut self, prereq: impl Into<String>) -> Self {
        self.prerequisites.push(prereq.into());
        self
    }

    /// Add an item to the reward.
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

    /// The built-in tutorial quest: kill an enemy, pick up an item, talk to an NPC.
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

    /// The built-in "Into the Glitch" quest: reach floor 5 and kill 10 enemies.
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

    /// The built-in "The Equation Must Be Solved" quest: defeat the boss on floor 10.
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

/// All quests the player knows about, keyed by id.
#[derive(Debug, Default)]
pub struct QuestLog {
    quests: HashMap<String, Quest>,
}

impl QuestLog {
    /// An empty quest log.
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

    /// Quests in progress.
    pub fn active_quests(&self) -> Vec<&Quest> {
        self.quests
            .values()
            .filter(|q| q.status == QuestStatus::InProgress)
            .collect()
    }

    /// Quests completed.
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

    /// Look up a quest by id.
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
