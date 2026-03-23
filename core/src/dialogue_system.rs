//! Branching dialogue tree system with conditions and effects.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Conditions & Effects
// ---------------------------------------------------------------------------

/// A condition that must be true for a dialogue option to be available.
#[derive(Debug, Clone, PartialEq)]
pub enum DialogueCondition {
    HasItem(String),
    QuestComplete(String),
    ReputationAbove(i32),
    ReputationBelow(i32),
    Always,
}

/// An effect applied when a dialogue option is selected.
#[derive(Debug, Clone, PartialEq)]
pub enum DialogueEffect {
    GiveItem(String),
    SetFlag(String),
    ModifyReputation(i32),
    StartQuest(String),
    EndDialogue,
}

// ---------------------------------------------------------------------------
// Dialogue structures
// ---------------------------------------------------------------------------

/// A single dialogue option presented to the player.
#[derive(Debug, Clone)]
pub struct DialogueOption {
    pub text: String,
    pub conditions: Vec<DialogueCondition>,
    pub effects: Vec<DialogueEffect>,
    pub next_node_id: Option<u32>,
}

/// A single node in the dialogue tree.
#[derive(Debug, Clone)]
pub struct DialogueNode {
    pub id: u32,
    pub speaker: String,
    pub text: String,
    pub options: Vec<DialogueOption>,
}

/// A complete branching dialogue tree.
#[derive(Debug, Clone)]
pub struct DialogueTree {
    pub nodes: HashMap<u32, DialogueNode>,
    pub root_id: u32,
}

/// Runtime context for evaluating dialogue conditions.
#[derive(Debug, Clone, Default)]
pub struct DialogueContext {
    pub inventory: Vec<String>,
    pub completed_quests: Vec<String>,
    pub reputation: i32,
    pub active_flags: Vec<String>,
}

// ---------------------------------------------------------------------------
// DialogueTree implementation
// ---------------------------------------------------------------------------

impl DialogueTree {
    /// Create an empty dialogue tree with the given root node ID.
    pub fn new(root_id: u32) -> Self {
        Self {
            nodes: HashMap::new(),
            root_id,
        }
    }

    /// Add a node to the tree.
    pub fn add_node(&mut self, node: DialogueNode) {
        self.nodes.insert(node.id, node);
    }

    /// Evaluate a list of conditions against the provided context.
    pub fn evaluate_conditions(
        &self,
        conditions: &[DialogueCondition],
        ctx: &DialogueContext,
    ) -> bool {
        conditions.iter().all(|cond| match cond {
            DialogueCondition::Always => true,
            DialogueCondition::HasItem(item) => ctx.inventory.contains(item),
            DialogueCondition::QuestComplete(q) => ctx.completed_quests.contains(q),
            DialogueCondition::ReputationAbove(threshold) => ctx.reputation > *threshold,
            DialogueCondition::ReputationBelow(threshold) => ctx.reputation < *threshold,
        })
    }

    /// Return all options of `node_id` whose conditions are satisfied.
    pub fn available_options<'a>(
        &'a self,
        node_id: u32,
        ctx: &DialogueContext,
    ) -> Vec<&'a DialogueOption> {
        match self.nodes.get(&node_id) {
            None => Vec::new(),
            Some(node) => node
                .options
                .iter()
                .filter(|opt| self.evaluate_conditions(&opt.conditions, ctx))
                .collect(),
        }
    }

    /// Select option `option_index` from `node_id`, apply its effects, and
    /// return the next node ID (or `None` if the dialogue ends).
    pub fn select_option(
        &self,
        node_id: u32,
        option_index: usize,
        ctx: &mut DialogueContext,
    ) -> Option<u32> {
        let node = self.nodes.get(&node_id)?;
        let available = self.available_options(node_id, ctx);
        let option = available.get(option_index)?;

        // Apply effects
        for effect in &option.effects {
            match effect {
                DialogueEffect::GiveItem(item) => {
                    ctx.inventory.push(item.clone());
                }
                DialogueEffect::SetFlag(flag) => {
                    if !ctx.active_flags.contains(flag) {
                        ctx.active_flags.push(flag.clone());
                    }
                }
                DialogueEffect::ModifyReputation(delta) => {
                    ctx.reputation += delta;
                }
                DialogueEffect::StartQuest(quest) => {
                    // In a real game this would trigger quest logic; here we
                    // just track the flag.
                    let flag = format!("quest_started:{}", quest);
                    if !ctx.active_flags.contains(&flag) {
                        ctx.active_flags.push(flag);
                    }
                }
                DialogueEffect::EndDialogue => {
                    return None;
                }
            }
        }

        // Return next node, if any
        // We need to re-borrow to avoid borrow checker issues
        let _ = node; // drop immutable borrow
        let node2 = self.nodes.get(&node_id)?;
        let avail2 = node2
            .options
            .iter()
            .filter(|opt| {
                // re-evaluate with potentially modified ctx
                opt.conditions.iter().all(|c| match c {
                    DialogueCondition::Always => true,
                    DialogueCondition::HasItem(i) => ctx.inventory.contains(i),
                    DialogueCondition::QuestComplete(q) => ctx.completed_quests.contains(q),
                    DialogueCondition::ReputationAbove(t) => ctx.reputation > *t,
                    DialogueCondition::ReputationBelow(t) => ctx.reputation < *t,
                })
            })
            .collect::<Vec<_>>();
        avail2.get(option_index).and_then(|o| o.next_node_id)
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Fluent builder for `DialogueTree`.
pub struct DialogueTreeBuilder {
    tree: DialogueTree,
    next_node_id: u32,
}

impl DialogueTreeBuilder {
    /// Create a builder with a root node.
    pub fn new(root_speaker: impl Into<String>, root_text: impl Into<String>) -> Self {
        let root_id = 0;
        let root_node = DialogueNode {
            id: root_id,
            speaker: root_speaker.into(),
            text: root_text.into(),
            options: Vec::new(),
        };
        let mut tree = DialogueTree::new(root_id);
        tree.add_node(root_node);
        Self {
            tree,
            next_node_id: 1,
        }
    }

    /// Add a new node and return its ID.
    pub fn add_node(
        &mut self,
        speaker: impl Into<String>,
        text: impl Into<String>,
    ) -> u32 {
        let id = self.next_node_id;
        self.next_node_id += 1;
        self.tree.add_node(DialogueNode {
            id,
            speaker: speaker.into(),
            text: text.into(),
            options: Vec::new(),
        });
        id
    }

    /// Add an option to `from_node_id` that leads to `to_node_id`.
    pub fn connect_option(
        &mut self,
        from_node_id: u32,
        text: impl Into<String>,
        conditions: Vec<DialogueCondition>,
        effects: Vec<DialogueEffect>,
        to_node_id: Option<u32>,
    ) -> &mut Self {
        if let Some(node) = self.tree.nodes.get_mut(&from_node_id) {
            node.options.push(DialogueOption {
                text: text.into(),
                conditions,
                effects,
                next_node_id: to_node_id,
            });
        }
        self
    }

    /// Consume the builder and return the completed tree.
    pub fn build(self) -> DialogueTree {
        self.tree
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_tree() -> DialogueTree {
        let mut builder =
            DialogueTreeBuilder::new("Merchant", "Welcome, traveller. What do you seek?");

        let node_have_sword = builder.add_node("Merchant", "Ah, a fine blade you carry.");
        let node_no_sword = builder.add_node("Merchant", "You should arm yourself first.");
        let node_quest_done = builder.add_node("Merchant", "You've proven yourself!");

        builder
            .connect_option(
                0,
                "Show my sword",
                vec![DialogueCondition::HasItem("sword".to_string())],
                vec![DialogueEffect::ModifyReputation(5)],
                Some(node_have_sword),
            )
            .connect_option(
                0,
                "I need a sword",
                vec![DialogueCondition::Always],
                vec![],
                Some(node_no_sword),
            )
            .connect_option(
                0,
                "Quest complete!",
                vec![DialogueCondition::QuestComplete("main_quest".to_string())],
                vec![DialogueEffect::GiveItem("reward_gem".to_string())],
                Some(node_quest_done),
            );

        builder.build()
    }

    #[test]
    fn test_has_item_condition_false() {
        let tree = make_test_tree();
        let ctx = DialogueContext::default();
        let opts = tree.available_options(0, &ctx);
        // Only "I need a sword" should be visible (Always condition)
        // "Show my sword" and "Quest complete!" should be filtered
        let texts: Vec<&str> = opts.iter().map(|o| o.text.as_str()).collect();
        assert!(!texts.contains(&"Show my sword"));
        assert!(texts.contains(&"I need a sword"));
    }

    #[test]
    fn test_has_item_condition_true() {
        let tree = make_test_tree();
        let ctx = DialogueContext {
            inventory: vec!["sword".to_string()],
            ..Default::default()
        };
        let opts = tree.available_options(0, &ctx);
        let texts: Vec<&str> = opts.iter().map(|o| o.text.as_str()).collect();
        assert!(texts.contains(&"Show my sword"));
    }

    #[test]
    fn test_reputation_effect() {
        let tree = make_test_tree();
        let mut ctx = DialogueContext {
            inventory: vec!["sword".to_string()],
            ..Default::default()
        };
        assert_eq!(ctx.reputation, 0);
        // Select "Show my sword" (index 0 when sword present)
        tree.select_option(0, 0, &mut ctx);
        assert_eq!(ctx.reputation, 5);
    }

    #[test]
    fn test_tree_traversal() {
        let tree = make_test_tree();
        let ctx = DialogueContext::default();
        // Only option with Always condition
        let opts = tree.available_options(0, &ctx);
        assert!(!opts.is_empty());
        // The "Always" option leads to node_no_sword
        let next = opts[0].next_node_id;
        assert!(next.is_some());
        let next_node = tree.nodes.get(&next.unwrap()).unwrap();
        assert!(next_node.text.contains("sword"));
    }

    #[test]
    fn test_unavailable_options_filtered() {
        let tree = make_test_tree();
        let ctx = DialogueContext::default();
        let opts = tree.available_options(0, &ctx);
        // "Quest complete!" requires quest completion
        for opt in &opts {
            assert_ne!(opt.text, "Quest complete!");
        }
    }

    #[test]
    fn test_give_item_effect() {
        let tree = make_test_tree();
        let mut ctx = DialogueContext {
            completed_quests: vec!["main_quest".to_string()],
            ..Default::default()
        };
        // Find "Quest complete!" option
        let opts = tree.available_options(0, &ctx);
        let idx = opts
            .iter()
            .position(|o| o.text == "Quest complete!")
            .unwrap();
        tree.select_option(0, idx, &mut ctx);
        assert!(ctx.inventory.contains(&"reward_gem".to_string()));
    }
}
