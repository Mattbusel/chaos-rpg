//! Audio bridge — maps game events to proof-engine audio system.
//!
//! Translates combat actions, spell casts, UI interactions, and ambient
//! game state into engine audio events (procedural music vibes, SFX).

use proof_engine::prelude::*;
use proof_engine::audio::MusicVibe;
use crate::state::{AppScreen, GameState};

/// Set the music vibe based on current game state.
pub fn update_music_vibe(state: &GameState, engine: &mut ProofEngine) {
    let target_vibe = match state.screen {
        AppScreen::Title | AppScreen::ModeSelect |
        AppScreen::CharacterCreation | AppScreen::BoonSelect |
        AppScreen::Settings | AppScreen::Tutorial => MusicVibe::Title,

        AppScreen::FloorNav | AppScreen::RoomView |
        AppScreen::Shop | AppScreen::Crafting |
        AppScreen::CharacterSheet | AppScreen::BodyChart |
        AppScreen::PassiveTree => MusicVibe::Exploration,

        AppScreen::Combat => {
            if state.is_boss_fight {
                MusicVibe::BossFight
            } else {
                MusicVibe::Combat
            }
        }

        AppScreen::GameOver => MusicVibe::Death,
        AppScreen::Victory => MusicVibe::Victory,

        AppScreen::Scoreboard | AppScreen::Achievements |
        AppScreen::RunHistory | AppScreen::DailyLeaderboard |
        AppScreen::Bestiary | AppScreen::Codex => MusicVibe::Title,
    };

    engine.emit_audio(AudioEvent::SetMusicVibe(target_vibe));
}

/// Emit audio for a combat action result.
pub fn on_combat_action(engine: &mut ProofEngine, action_type: u8, damage: i64, is_crit: bool) {
    match action_type {
        1 | 2 => {
            // Attack / Heavy Attack
            let volume = (damage as f32 / 200.0).clamp(0.3, 1.0);
            let name = if is_crit { "crit_hit" } else { "hit" };
            engine.emit_audio(AudioEvent::PlaySfx {
                name: name.to_string(),
                position: Vec3::ZERO,
                volume,
            });
        }
        3 => {
            // Spell cast
            engine.emit_audio(AudioEvent::PlaySfx {
                name: "spell_cast".to_string(),
                position: Vec3::ZERO,
                volume: 0.7,
            });
        }
        4 => {
            // Defend
            engine.emit_audio(AudioEvent::PlaySfx {
                name: "defend".to_string(),
                position: Vec3::ZERO,
                volume: 0.5,
            });
        }
        _ => {}
    }
}

/// Emit audio for enemy death.
pub fn on_enemy_death(engine: &mut ProofEngine, is_boss: bool) {
    let name = if is_boss { "boss_death" } else { "enemy_death" };
    engine.emit_audio(AudioEvent::PlaySfx {
        name: name.to_string(),
        position: Vec3::ZERO,
        volume: if is_boss { 1.0 } else { 0.7 },
    });
}

/// Emit audio for player death.
pub fn on_player_death(engine: &mut ProofEngine) {
    engine.emit_audio(AudioEvent::PlaySfx {
        name: "player_death".to_string(),
        position: Vec3::ZERO,
        volume: 1.0,
    });
}

/// Emit audio for level up.
pub fn on_level_up(engine: &mut ProofEngine) {
    engine.emit_audio(AudioEvent::PlaySfx {
        name: "level_up".to_string(),
        position: Vec3::ZERO,
        volume: 0.8,
    });
}

/// Emit audio for item pickup.
pub fn on_item_pickup(engine: &mut ProofEngine) {
    engine.emit_audio(AudioEvent::PlaySfx {
        name: "item_pickup".to_string(),
        position: Vec3::ZERO,
        volume: 0.5,
    });
}

/// Emit audio for menu navigation.
pub fn on_menu_select(engine: &mut ProofEngine) {
    engine.emit_audio(AudioEvent::PlaySfx {
        name: "menu_select".to_string(),
        position: Vec3::ZERO,
        volume: 0.3,
    });
}

/// Emit audio for crafting operation.
pub fn on_craft(engine: &mut ProofEngine, op_type: u8) {
    let name = match op_type {
        1 => "craft_reforge",
        2 => "craft_corrupt",
        3 => "craft_shatter",
        _ => "craft_generic",
    };
    engine.emit_audio(AudioEvent::PlaySfx {
        name: name.to_string(),
        position: Vec3::ZERO,
        volume: 0.7,
    });
}
