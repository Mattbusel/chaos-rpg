//! Screen modules — one per game screen.
//!
//! Each screen module exposes `update()` and `render()` functions that operate
//! on `GameState` and `ProofEngine`.

pub mod title;
pub mod combat;
pub mod floor_nav;
pub mod character_creation;
pub mod character_sheet;
