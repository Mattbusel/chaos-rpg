//! CHAOS RPG Lore System — the narrative layer connecting every mechanic to The Proof.
//!
//! ## Structure
//!
//! - `world`    — floor text, room flavor, epoch/faction/engine lore
//! - `items`    — rarity flavor, material lore, suffix lore
//! - `bosses`   — boss entries with one-liners and strategy hints
//! - `enemies`  — enemy lore entries unlocked on encounter
//! - `events`   — rare event flavor text (zero rolls, negative damage, etc.)
//! - `fragments`— The Mathematician's fragments (rarest unlocks)
//! - `codex`    — ~130 Codex entries with unlock conditions
//! - `narrative`— NarrativeEvent enum and run narrative auto-generator

pub mod bosses;
pub mod codex;
pub mod enemies;
pub mod events;
pub mod fragments;
pub mod items;
pub mod narrative;
pub mod world;
