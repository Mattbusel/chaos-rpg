//! Game-specific particle and force field effects.
//!
//! Bridges game events (combat actions, spells, status effects) to
//! proof-engine visual effects (particles, fields, screen shake).

pub mod combat_fx;
pub mod spell_fx;
pub mod status_fx;
pub mod boss_visuals;
pub mod environmental;
pub mod transitions;
pub mod particle_presets;
pub mod screen_effects;
