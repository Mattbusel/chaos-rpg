//! Quest chain system for CHAOS RPG.

use std::collections::HashMap;

/// Current status of a quest.
#[derive(Debug, Clone, PartialEq)]
pub enum QuestStatus {
    Available,
    Active,
    Completed,
    Failed,
    Hidden,
}

/// What a quest objective requires the player to do.
#[derive(Debug, Clone)]
pub enum ObjectiveType {
    KillCreature { creature_id: String, count: u32 },
    CollectItem { item_name: String, count: u32 },
    VisitLocation { location_id: String },
    TalkToNpc { npc_id: String },
    EscortNpc { npc_id: String, destination: String },
    CraftItem { item_name: String },
}

/// A single objective within a quest.
#[derive(Debug, Clone)]
pub struct QuestObjective {
    pub id: String,
    pub objective_type: ObjectiveType,
    pub description: String,
    pub progress: u32,
    pub required: u32,
    pub completed: bool,
}

impl QuestObjective {
    /// Advance progress by `amount`. Returns true if newly completed.
    pub fn update_progress(&mut self, amount: u32) -> bool {
        if self.completed {
            return false;
        }
        self.progress = self.progress.saturating_add(amount);
        if self.progress >= self.required {
            self.progress = self.required;
            self.completed = true;
            true
        } else {
            false
        }
    }
}

/// Rewards granted upon completing a quest.
#[derive(Debug, Clone, Default)]
pub struct QuestReward {
    pub xp: u32,
    pub gold: u32,
    pub items: Vec<(String, u32)>,
    pub reputation_gains: Vec<(String, i32)>,
}

/// A quest with its objectives, rewards, and chain metadata.
#[derive(Debug, Clone)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: QuestStatus,
    pub objectives: Vec<QuestObjective>,
    pub rewards: QuestReward,
    pub prerequisite_quests: Vec<String>,
    pub time_limit_minutes: Option<u32>,
    /// Unix timestamp (seconds) when the quest was started.
    pub started_at: Option<u64>,
    pub chain_id: Option<String>,
}

impl Quest {
    /// Returns true when every objective is completed.
    pub fn is_completable(&self) -> bool {
        !self.objectives.is_empty() && self.objectives.iter().all(|o| o.completed)
    }

    /// Fraction of objectives completed, in [0.0, 1.0].
    pub fn progress_pct(&self) -> f64 {
        if self.objectives.is_empty() {
            return 0.0;
        }
        let done = self.objectives.iter().filter(|o| o.completed).count();
        done as f64 / self.objectives.len() as f64
    }

    /// Returns true if the time limit has been exceeded.
    pub fn has_time_expired(&self, current_time: u64) -> bool {
        match (self.time_limit_minutes, self.started_at) {
            (Some(limit), Some(start)) => {
                current_time >= start + (limit as u64) * 60
            }
            _ => false,
        }
    }
}

/// An ordered chain of quests that unlock sequentially.
#[derive(Debug, Clone)]
pub struct QuestChain {
    pub id: String,
    pub name: String,
    /// Ordered list of quest ids.
    pub quests: Vec<String>,
    pub current_idx: usize,
}

impl QuestChain {
    /// Returns the id of the current quest in the chain, if any.
    pub fn current_quest_id(&self) -> Option<&str> {
        self.quests.get(self.current_idx).map(|s| s.as_str())
    }

    /// Advance to the next quest. Returns true if advanced, false if already at end.
    pub fn advance(&mut self) -> bool {
        if self.current_idx + 1 < self.quests.len() {
            self.current_idx += 1;
            true
        } else {
            false
        }
    }
}

/// Manages the full quest system: quests and chains.
#[derive(Debug, Default)]
pub struct QuestManager {
    quests: HashMap<String, Quest>,
    chains: HashMap<String, QuestChain>,
}

impl QuestManager {
    /// Create a new empty quest manager.
    pub fn new() -> Self {
        Self { quests: HashMap::new(), chains: HashMap::new() }
    }

    /// Register a quest.
    pub fn add_quest(&mut self, quest: Quest) {
        self.quests.insert(quest.id.clone(), quest);
    }

    /// Register a quest chain.
    pub fn add_chain(&mut self, chain: QuestChain) {
        self.chains.insert(chain.id.clone(), chain);
    }

    /// Start a quest, setting status to Active and recording start time.
    pub fn start_quest(&mut self, quest_id: &str, current_time: u64) -> Result<(), String> {
        let quest = self
            .quests
            .get_mut(quest_id)
            .ok_or_else(|| format!("Quest '{}' not found", quest_id))?;

        if quest.status == QuestStatus::Active {
            return Err(format!("Quest '{}' is already active", quest_id));
        }
        if quest.status == QuestStatus::Completed {
            return Err(format!("Quest '{}' is already completed", quest_id));
        }
        if quest.status == QuestStatus::Failed {
            return Err(format!("Quest '{}' has already failed", quest_id));
        }
        if quest.status == QuestStatus::Hidden {
            return Err(format!("Quest '{}' is hidden", quest_id));
        }

        quest.status = QuestStatus::Active;
        quest.started_at = Some(current_time);
        Ok(())
    }

    /// Update an objective's progress. Returns true if the objective was newly completed.
    pub fn update_objective(
        &mut self,
        quest_id: &str,
        objective_id: &str,
        amount: u32,
    ) -> bool {
        let quest = match self.quests.get_mut(quest_id) {
            Some(q) => q,
            None => return false,
        };
        if quest.status != QuestStatus::Active {
            return false;
        }
        for obj in &mut quest.objectives {
            if obj.id == objective_id {
                return obj.update_progress(amount);
            }
        }
        false
    }

    /// Complete a quest if completable, returning the reward.
    pub fn complete_quest(&mut self, quest_id: &str) -> Option<QuestReward> {
        let quest = self.quests.get_mut(quest_id)?;
        if !quest.is_completable() || quest.status != QuestStatus::Active {
            return None;
        }
        quest.status = QuestStatus::Completed;
        let reward = quest.rewards.clone();

        // Advance any chain this quest belongs to
        let chain_id = quest.chain_id.clone();
        if let Some(cid) = chain_id {
            if let Some(chain) = self.chains.get_mut(&cid) {
                chain.advance();
                // Make the next quest available
                if let Some(next_id) = chain.current_quest_id().map(|s| s.to_string()) {
                    if let Some(next_quest) = self.quests.get_mut(&next_id) {
                        if next_quest.status == QuestStatus::Hidden {
                            next_quest.status = QuestStatus::Available;
                        }
                    }
                }
            }
        }

        Some(reward)
    }

    /// Mark a quest as failed.
    pub fn fail_quest(&mut self, quest_id: &str) {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            if quest.status == QuestStatus::Active {
                quest.status = QuestStatus::Failed;
            }
        }
    }

    /// Return all quests that are Available and whose prerequisites have been met.
    pub fn available_quests(&self, completed_quests: &[String]) -> Vec<&Quest> {
        self.quests
            .values()
            .filter(|q| {
                q.status == QuestStatus::Available
                    && q.prerequisite_quests
                        .iter()
                        .all(|prereq| completed_quests.contains(prereq))
            })
            .collect()
    }

    /// Return all currently active quests.
    pub fn active_quests(&self) -> Vec<&Quest> {
        self.quests
            .values()
            .filter(|q| q.status == QuestStatus::Active)
            .collect()
    }

    /// Return (current_index, total) for a chain, or None if not found.
    pub fn chain_progress(&self, chain_id: &str) -> Option<(usize, usize)> {
        self.chains.get(chain_id).map(|c| (c.current_idx, c.quests.len()))
    }
}
