//! CHAOS RPG Core — all game logic, zero rendering.
//!
//! Every system lives here: math engines, combat, characters, items, spells,
//! enemies, world generation, scoreboards, and the nemesis system.
//! Rendering is the job of the frontend crates (terminal, graphical, web).

pub mod audio_events;
pub mod audio_synth;
pub mod achievement_system;
pub mod io_util;
pub mod atlas;
pub mod body;
pub mod bosses;
pub mod chaos_pipeline;
pub mod character;
pub mod combat;
pub mod crafting;
pub mod dialogue;
pub mod dungeon;
pub mod enemy;
pub mod enemy_ai;
pub mod faction_system;
pub mod factions;
pub mod items;
pub mod loot_system;
pub mod magic;
pub mod math_engines;
pub mod nemesis;
pub mod npcs;
pub mod passive_tree;
pub mod quest;
pub mod recipes;
pub mod save_system;
pub mod scoreboard;
pub mod skill_checks;
pub mod skill_tree;
pub mod spells;
pub mod status_effects;
pub mod world;
pub mod world_map;
pub mod weather_system;
pub mod economy;
pub mod skill_tree_v2;
pub mod inventory_system;
pub mod combat_simulator;
pub mod npc_generator;
pub mod party_system;
pub mod class_system;
