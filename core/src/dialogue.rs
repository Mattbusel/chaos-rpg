//! NPC dialogue tree system.
//!
//! Provides a data-driven dialogue engine with conditional branching,
//! action effects, and quest/inventory integration.

use std::collections::{HashMap, HashSet};

// ─── CONDITIONS ───────────────────────────────────────────────────────────────

/// A condition that gates a dialogue node or choice.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DialogueCondition {
    /// Player must have at least one of this item.
    HasItem(String),
    /// Named quest must be in the completed set.
    QuestComplete(String),
    /// Faction reputation must exceed the threshold.
    ReputationAbove(String, i32),
    /// Always satisfied.
    Always,
    /// Never satisfied.
    Never,
}

impl DialogueCondition {
    /// Evaluate this condition against the current game context.
    pub fn evaluate(&self, ctx: &GameContext) -> bool {
        match self {
            DialogueCondition::HasItem(item) => {
                ctx.inventory.get(item).copied().unwrap_or(0) > 0
            }
            DialogueCondition::QuestComplete(quest) => ctx.completed_quests.contains(quest),
            DialogueCondition::ReputationAbove(faction, threshold) => {
                ctx.reputations.get(faction).copied().unwrap_or(0) > *threshold
            }
            DialogueCondition::Always => true,
            DialogueCondition::Never => false,
        }
    }
}

// ─── ACTIONS ──────────────────────────────────────────────────────────────────

/// An effect applied when a dialogue choice is selected.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DialogueAction {
    /// Give the player an item.
    GiveItem(String),
    /// Remove an item from the player's inventory.
    TakeItem(String),
    /// Begin a quest.
    StartQuest(String),
    /// Set a faction reputation to the given value.
    SetReputation(String, i32),
    /// Unlock a named feature, door, or area.
    Unlock(String),
}

// ─── DIALOGUE CHOICE ──────────────────────────────────────────────────────────

/// A player-selectable response option within a dialogue node.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DialogueChoice {
    /// Unique identifier within the tree.
    pub id: String,
    /// Text shown to the player.
    pub text: String,
    /// Target node to move to after selecting this choice, if any.
    pub next_node_id: Option<String>,
    /// Optional gate — choice is hidden/unavailable if condition is not met.
    pub condition: Option<DialogueCondition>,
}

// ─── DIALOGUE NODE ────────────────────────────────────────────────────────────

/// A single node (line of NPC speech + player options) in a dialogue tree.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DialogueNode {
    /// Unique identifier within the tree.
    pub id: String,
    /// Name of the speaking NPC or character.
    pub speaker: String,
    /// The spoken text displayed to the player.
    pub text: String,
    /// Player choices branching from this node.
    pub choices: Vec<DialogueChoice>,
    /// Conditions that must all be true for this node to be reachable.
    pub conditions: Vec<DialogueCondition>,
    /// Actions applied when entering this node.
    pub actions: Vec<DialogueAction>,
}

// ─── DIALOGUE TREE ────────────────────────────────────────────────────────────

/// A complete NPC conversation graph.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DialogueTree {
    /// Unique identifier for this conversation.
    pub id: String,
    /// Display name of the NPC owning this tree.
    pub npc_name: String,
    /// All nodes, keyed by their id.
    pub nodes: HashMap<String, DialogueNode>,
    /// The id of the first node to display.
    pub start_node: String,
}

impl DialogueTree {
    /// Built-in example: a blacksmith's introductory conversation.
    ///
    /// Tree structure:
    /// ```text
    /// greeting
    ///   ├─[Always]     → "Ask about weapons"  → sells_info
    ///   └─[QuestComplete("find_iron")]  → "Mention the iron"  → quest_reward
    /// sells_info  (terminal)
    /// quest_reward (terminal)
    /// ```
    pub fn blacksmith_intro() -> Self {
        let mut nodes = HashMap::new();

        // Node 1: greeting
        nodes.insert(
            "greeting".into(),
            DialogueNode {
                id: "greeting".into(),
                speaker: "Aldric the Smith".into(),
                text: "Welcome, traveller! My forge burns hot today. \
                       What brings you to my humble smithy?"
                    .into(),
                choices: vec![
                    DialogueChoice {
                        id: "ask_weapons".into(),
                        text: "What weapons do you sell?".into(),
                        next_node_id: Some("sells_info".into()),
                        condition: Some(DialogueCondition::Always),
                    },
                    DialogueChoice {
                        id: "mention_iron".into(),
                        text: "I found the iron ore you needed.".into(),
                        next_node_id: Some("quest_reward".into()),
                        condition: Some(DialogueCondition::QuestComplete(
                            "find_iron".into(),
                        )),
                    },
                ],
                conditions: vec![],
                actions: vec![],
            },
        );

        // Node 2: sells_info
        nodes.insert(
            "sells_info".into(),
            DialogueNode {
                id: "sells_info".into(),
                speaker: "Aldric the Smith".into(),
                text: "I craft swords, shields, and the occasional battle-axe. \
                       If you bring me iron ore I can make you something special."
                    .into(),
                choices: vec![],
                conditions: vec![],
                actions: vec![DialogueAction::StartQuest("find_iron".into())],
            },
        );

        // Node 3: quest_reward
        nodes.insert(
            "quest_reward".into(),
            DialogueNode {
                id: "quest_reward".into(),
                speaker: "Aldric the Smith".into(),
                text: "Magnificent! Here, take this blade as payment. \
                       You have my gratitude, friend."
                    .into(),
                choices: vec![],
                conditions: vec![DialogueCondition::QuestComplete("find_iron".into())],
                actions: vec![
                    DialogueAction::GiveItem("iron_sword".into()),
                    DialogueAction::SetReputation("blacksmiths_guild".into(), 20),
                ],
            },
        );

        Self {
            id: "blacksmith_intro".into(),
            npc_name: "Aldric the Smith".into(),
            nodes,
            start_node: "greeting".into(),
        }
    }
}

// ─── GAME CONTEXT ─────────────────────────────────────────────────────────────

/// Snapshot of player state used to evaluate dialogue conditions.
#[derive(Debug, Clone, Default)]
pub struct GameContext {
    /// Items the player is carrying and their quantities.
    pub inventory: HashMap<String, u32>,
    /// Set of quest ids that have been completed.
    pub completed_quests: HashSet<String>,
    /// Faction reputation scores.
    pub reputations: HashMap<String, i32>,
    /// Unlocked areas, doors, or features.
    pub unlocked: HashSet<String>,
}

// ─── DIALOGUE STATE ───────────────────────────────────────────────────────────

/// Mutable runtime state for an active conversation.
#[derive(Debug, Clone)]
pub struct DialogueState {
    /// The id of the node currently being displayed.
    pub current_node_id: String,
    /// All node ids that have been visited in this session.
    pub visited_nodes: HashSet<String>,
    /// Arbitrary string key-value bag for scripted variables.
    pub variables: HashMap<String, String>,
}

// ─── DIALOGUE ENGINE ──────────────────────────────────────────────────────────

/// Manages one or more dialogue trees and drives conversation logic.
#[derive(Debug, Default)]
pub struct DialogueEngine {
    trees: HashMap<String, DialogueTree>,
}

impl DialogueEngine {
    /// Create a new, empty engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tree with the engine.
    pub fn load_tree(&mut self, tree: DialogueTree) {
        self.trees.insert(tree.id.clone(), tree);
    }

    /// Start a new conversation for the given tree id.
    ///
    /// # Panics
    /// Panics if `tree_id` is not loaded.
    pub fn start(&self, tree_id: &str) -> DialogueState {
        let tree = self
            .trees
            .get(tree_id)
            .unwrap_or_else(|| panic!("DialogueTree '{}' not loaded", tree_id));
        let mut visited = HashSet::new();
        visited.insert(tree.start_node.clone());
        DialogueState {
            current_node_id: tree.start_node.clone(),
            visited_nodes: visited,
            variables: HashMap::new(),
        }
    }

    /// Return a reference to the current node, or `None` if the id is invalid.
    pub fn current_node<'a>(
        &'a self,
        tree_id: &str,
        state: &DialogueState,
    ) -> Option<&'a DialogueNode> {
        self.trees
            .get(tree_id)?
            .nodes
            .get(&state.current_node_id)
    }

    /// Return the choices available given the current game state.
    ///
    /// Choices whose `condition` evaluates to `false` are excluded.
    pub fn available_choices<'a>(
        &'a self,
        tree_id: &str,
        state: &DialogueState,
        game_state: &GameContext,
    ) -> Vec<&'a DialogueChoice> {
        let Some(node) = self.current_node(tree_id, state) else {
            return vec![];
        };
        node.choices
            .iter()
            .filter(|c| {
                c.condition
                    .as_ref()
                    .map(|cond| cond.evaluate(game_state))
                    .unwrap_or(true)
            })
            .collect()
    }

    /// Select a choice by id, advance the state, and return triggered actions.
    ///
    /// Returns an empty vec if the choice id is not found in the current node.
    pub fn choose(
        &self,
        tree_id: &str,
        state: &mut DialogueState,
        choice_id: &str,
    ) -> Vec<DialogueAction> {
        let Some(node) = self.current_node(tree_id, state) else {
            return vec![];
        };
        let Some(choice) = node.choices.iter().find(|c| c.id == choice_id) else {
            return vec![];
        };

        // Advance to next node if specified.
        if let Some(next) = &choice.next_node_id {
            state.current_node_id = next.clone();
            state.visited_nodes.insert(next.clone());
        }

        // Return the actions of the *new* node (if any).
        self.trees
            .get(tree_id)
            .and_then(|t| t.nodes.get(&state.current_node_id))
            .map(|n| n.actions.clone())
            .unwrap_or_default()
    }
}

// ─── TESTS ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> (DialogueEngine, String) {
        let mut engine = DialogueEngine::new();
        let tree = DialogueTree::blacksmith_intro();
        let tree_id = tree.id.clone();
        engine.load_tree(tree);
        (engine, tree_id)
    }

    #[test]
    fn start_returns_greeting_node() {
        let (engine, tid) = make_engine();
        let state = engine.start(&tid);
        assert_eq!(state.current_node_id, "greeting");
        assert!(state.visited_nodes.contains("greeting"));
    }

    #[test]
    fn current_node_matches_state() {
        let (engine, tid) = make_engine();
        let state = engine.start(&tid);
        let node = engine.current_node(&tid, &state).unwrap();
        assert_eq!(node.id, "greeting");
        assert_eq!(node.speaker, "Aldric the Smith");
    }

    #[test]
    fn condition_gates_choice_when_quest_incomplete() {
        let (engine, tid) = make_engine();
        let state = engine.start(&tid);
        let ctx = GameContext::default(); // no completed quests
        let choices = engine.available_choices(&tid, &state, &ctx);
        // Only "ask_weapons" (Always) should be visible
        assert_eq!(choices.len(), 1);
        assert_eq!(choices[0].id, "ask_weapons");
    }

    #[test]
    fn condition_unlocks_choice_when_quest_complete() {
        let (engine, tid) = make_engine();
        let state = engine.start(&tid);
        let mut ctx = GameContext::default();
        ctx.completed_quests.insert("find_iron".into());
        let choices = engine.available_choices(&tid, &state, &ctx);
        // Both choices should now be visible
        assert_eq!(choices.len(), 2);
    }

    #[test]
    fn choose_advances_state_and_returns_actions() {
        let (engine, tid) = make_engine();
        let mut state = engine.start(&tid);
        let actions = engine.choose(&tid, &mut state, "ask_weapons");
        assert_eq!(state.current_node_id, "sells_info");
        assert!(state.visited_nodes.contains("sells_info"));
        // sells_info triggers StartQuest("find_iron")
        assert!(actions.contains(&DialogueAction::StartQuest("find_iron".into())));
    }

    #[test]
    fn choose_quest_reward_returns_give_item() {
        let (engine, tid) = make_engine();
        let mut state = engine.start(&tid);
        let actions = engine.choose(&tid, &mut state, "mention_iron");
        assert_eq!(state.current_node_id, "quest_reward");
        assert!(actions.contains(&DialogueAction::GiveItem("iron_sword".into())));
    }

    #[test]
    fn visited_tracking_accumulates() {
        let (engine, tid) = make_engine();
        let mut state = engine.start(&tid);
        engine.choose(&tid, &mut state, "ask_weapons");
        assert!(state.visited_nodes.contains("greeting"));
        assert!(state.visited_nodes.contains("sells_info"));
    }

    #[test]
    fn invalid_choice_returns_empty_actions() {
        let (engine, tid) = make_engine();
        let mut state = engine.start(&tid);
        let actions = engine.choose(&tid, &mut state, "nonexistent_choice");
        assert!(actions.is_empty());
        // State should not have changed
        assert_eq!(state.current_node_id, "greeting");
    }

    #[test]
    fn never_condition_hides_choice() {
        let cond = DialogueCondition::Never;
        let ctx = GameContext::default();
        assert!(!cond.evaluate(&ctx));
    }

    #[test]
    fn has_item_condition_requires_nonzero_quantity() {
        let mut ctx = GameContext::default();
        let cond = DialogueCondition::HasItem("sword".into());
        assert!(!cond.evaluate(&ctx));
        ctx.inventory.insert("sword".into(), 1);
        assert!(cond.evaluate(&ctx));
    }
}
