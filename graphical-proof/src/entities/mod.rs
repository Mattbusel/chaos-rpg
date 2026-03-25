//! Entity rendering for proof-engine glyph formations.
//!
//! Maps game characters (player classes, enemies) to visually distinct
//! glyph formations with per-class/per-tier patterns, idle animations,
//! HP-linked cohesion, and glow effects.
//!
//! # Player classes
//!
//! All 12 `CharacterClass` variants get unique formations:
//! - **Mage** — loose diamond of arcane symbols, blue-purple, orbiting glyphs
//! - **Berserker** — tight aggressive cluster, red, rage glow below 30% HP
//! - **Ranger** — arrow chevron pointing right, green, geometric
//! - **Thief** — small compact blob, gray, dim stealth emission
//! - **Necromancer** — ring with dark center, green-purple, soul wisps
//! - **Alchemist** — bubbling flask shape, purple-gold
//! - **Paladin** — cross/shield, golden, steady warm glow
//! - **VoidWalker** — fractured ring with phase-in/out visibility
//! - **Warlord** — military grid, steel gray, disciplined
//! - **Trickster** — shifting positions each frame, multi-colored
//! - **Runesmith** — runic circle, orange-amber, counter-rotating rings
//! - **Chronomancer** — clock ring, blue-white, phase-staggered breathing
//!
//! # Enemy tiers
//!
//! Five tiers with escalating complexity:
//! - **Minion** (10 glyphs) — simple polar cluster
//! - **Elite** (20 glyphs) — rotating ring
//! - **Champion** (30 glyphs) — counter-rotating double ring
//! - **Boss** (55 glyphs) — star core + double helix + crown
//! - **Abomination** (85 glyphs) — pulsing core + triple ring + tendrils

pub mod formations;
pub mod player;
pub mod enemies;
pub mod soft_entity;

// Re-export primary entry points for convenience.
pub use player::render_player;
pub use enemies::{render_enemy, EnemyTier};
pub use soft_entity::{SoftEntity, SoftEntityManager, SoftEntityId, SoftEntityEvent};
