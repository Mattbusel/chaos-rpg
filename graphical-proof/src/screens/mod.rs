//! Screen modules — one per game screen.
//!
//! Each screen module exposes `update()` and `render()` functions that operate
//! on `GameState` and `ProofEngine`.

pub mod title;
pub mod mode_select;
pub mod character_creation;
pub mod boon_select;
pub mod combat;
pub mod floor_nav;
pub mod room_view;
pub mod shop;
pub mod crafting;
pub mod character_sheet;
pub mod passive_tree;
pub mod game_over;
pub mod victory;
pub mod scoreboard;
pub mod achievements;
pub mod run_history;
pub mod daily_leaderboard;
pub mod bestiary;
pub mod codex;
pub mod settings;
pub mod tutorial;
