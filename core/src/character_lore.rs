//! Player-authored character lore — stored per-character in the save file.
//!
//! Accessible from the Character Sheet via [L]. Appears in the graveyard,
//! run history, and auto-generated run narrative.

use serde::{Deserialize, Serialize};

/// Field length limits (in characters).
pub const MAX_ORIGIN: usize = 500;
pub const MAX_MOTIVATION: usize = 300;
pub const MAX_PERSONALITY: usize = 300;
pub const MAX_EPITAPH: usize = 200;
pub const MAX_NOTES: usize = 1000;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharacterLore {
    /// Where the character comes from / how they entered The Proof.
    #[serde(default)]
    pub origin: String,

    /// Why they are here.
    #[serde(default)]
    pub motivation: String,

    /// How they approach the chaos.
    #[serde(default)]
    pub personality: String,

    /// If set, replaces the procedurally generated graveyard epitaph.
    #[serde(default)]
    pub epitaph_override: String,

    /// Free-form notes.
    #[serde(default)]
    pub notes: String,
}

impl CharacterLore {
    pub fn is_empty(&self) -> bool {
        self.origin.is_empty()
            && self.motivation.is_empty()
            && self.personality.is_empty()
            && self.epitaph_override.is_empty()
            && self.notes.is_empty()
    }

    /// Returns the epitaph override if set, otherwise returns None.
    pub fn custom_epitaph(&self) -> Option<&str> {
        if self.epitaph_override.is_empty() {
            None
        } else {
            Some(&self.epitaph_override)
        }
    }

    /// Truncate all fields to their maximum lengths (call before saving).
    pub fn clamp_lengths(&mut self) {
        self.origin = truncate_chars(&self.origin, MAX_ORIGIN);
        self.motivation = truncate_chars(&self.motivation, MAX_MOTIVATION);
        self.personality = truncate_chars(&self.personality, MAX_PERSONALITY);
        self.epitaph_override = truncate_chars(&self.epitaph_override, MAX_EPITAPH);
        self.notes = truncate_chars(&self.notes, MAX_NOTES);
    }
}

fn truncate_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// The lore editor state — tracks which field is being edited and the cursor.
#[derive(Debug, Clone)]
pub struct LoreEditorState {
    pub lore: CharacterLore,
    pub active_field: LoreField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoreField {
    Origin,
    Motivation,
    Personality,
    EpitaphOverride,
    Notes,
}

impl LoreField {
    pub const ALL: &'static [LoreField] = &[
        LoreField::Origin,
        LoreField::Motivation,
        LoreField::Personality,
        LoreField::EpitaphOverride,
        LoreField::Notes,
    ];

    pub fn label(self) -> &'static str {
        match self {
            LoreField::Origin => "Origin",
            LoreField::Motivation => "Motivation",
            LoreField::Personality => "Personality",
            LoreField::EpitaphOverride => "Epitaph Override",
            LoreField::Notes => "Notes",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            LoreField::Origin => "Where you came from. How you entered The Proof. (500 chars)",
            LoreField::Motivation => "Why you're here. What you want. (300 chars)",
            LoreField::Personality => "How you approach the chaos. (300 chars)",
            LoreField::EpitaphOverride => {
                "Your epitaph on death. Leave blank for auto-generated. (200 chars)"
            }
            LoreField::Notes => "Anything else. The proof doesn't read this. (1000 chars)",
        }
    }

    pub fn max_len(self) -> usize {
        match self {
            LoreField::Origin => MAX_ORIGIN,
            LoreField::Motivation => MAX_MOTIVATION,
            LoreField::Personality => MAX_PERSONALITY,
            LoreField::EpitaphOverride => MAX_EPITAPH,
            LoreField::Notes => MAX_NOTES,
        }
    }

    pub fn get<'a>(self, lore: &'a CharacterLore) -> &'a str {
        match self {
            LoreField::Origin => &lore.origin,
            LoreField::Motivation => &lore.motivation,
            LoreField::Personality => &lore.personality,
            LoreField::EpitaphOverride => &lore.epitaph_override,
            LoreField::Notes => &lore.notes,
        }
    }

    pub fn get_mut<'a>(self, lore: &'a mut CharacterLore) -> &'a mut String {
        match self {
            LoreField::Origin => &mut lore.origin,
            LoreField::Motivation => &mut lore.motivation,
            LoreField::Personality => &mut lore.personality,
            LoreField::EpitaphOverride => &mut lore.epitaph_override,
            LoreField::Notes => &mut lore.notes,
        }
    }

    pub fn next(self) -> Self {
        match self {
            LoreField::Origin => LoreField::Motivation,
            LoreField::Motivation => LoreField::Personality,
            LoreField::Personality => LoreField::EpitaphOverride,
            LoreField::EpitaphOverride => LoreField::Notes,
            LoreField::Notes => LoreField::Origin,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            LoreField::Origin => LoreField::Notes,
            LoreField::Motivation => LoreField::Origin,
            LoreField::Personality => LoreField::Motivation,
            LoreField::EpitaphOverride => LoreField::Personality,
            LoreField::Notes => LoreField::EpitaphOverride,
        }
    }
}

impl LoreEditorState {
    pub fn new(lore: CharacterLore) -> Self {
        Self {
            lore,
            active_field: LoreField::Origin,
        }
    }

    pub fn active_text(&self) -> &str {
        self.active_field.get(&self.lore)
    }

    pub fn push_char(&mut self, ch: char) {
        let field = self.active_field;
        let max = field.max_len();
        let text = field.get_mut(&mut self.lore);
        if text.chars().count() < max {
            text.push(ch);
        }
    }

    pub fn pop_char(&mut self) {
        let field = self.active_field;
        field.get_mut(&mut self.lore).pop();
    }

    pub fn next_field(&mut self) {
        self.active_field = self.active_field.next();
    }

    pub fn prev_field(&mut self) {
        self.active_field = self.active_field.prev();
    }
}
