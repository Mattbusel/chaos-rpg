//! Auto-play system — automatic game progression for testing and demos.
//!
//! When `state.auto_mode` is true (toggled by Z key on FloorNav):
//! - Every 30 frames (~0.5s at 60fps), automatically performs the logical action
//! - FloorNav: enter the next room
//! - Combat: Attack (or Defend if HP < 30%)
//! - RoomView: accept/continue
//! - Shop: leave
//! - Stops on: GameOver, Victory, BoonSelect, CharacterCreation

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

/// Frames between auto-play actions (~0.5s at 60fps).
const AUTO_INTERVAL: u64 = 30;

/// Main auto-play tick. Call BEFORE the screen match in main.rs update.
/// Returns true if an action was taken this frame.
pub fn tick(state: &mut GameState, engine: &mut ProofEngine) -> bool {
    if !state.auto_mode {
        return false;
    }

    // Stop auto-play on screens that require human decision
    match state.screen {
        AppScreen::GameOver
        | AppScreen::Victory
        | AppScreen::BoonSelect
        | AppScreen::CharacterCreation
        | AppScreen::Title
        | AppScreen::ModeSelect
        | AppScreen::Tutorial
        | AppScreen::Settings => {
            state.auto_mode = false;
            return false;
        }
        _ => {}
    }

    // Rate limit: only act every AUTO_INTERVAL frames
    if state.frame.saturating_sub(state.auto_last_action) < AUTO_INTERVAL {
        return false;
    }

    // Don't act during kill linger animation
    if state.kill_linger > 0.0 {
        return false;
    }

    let acted = match state.screen {
        AppScreen::FloorNav => auto_floor_nav(state, engine),
        AppScreen::Combat => auto_combat(state, engine),
        AppScreen::RoomView => auto_room_view(state, engine),
        AppScreen::Shop => auto_shop(state, engine),
        AppScreen::Crafting => auto_crafting(state, engine),
        AppScreen::CharacterSheet | AppScreen::BodyChart => auto_back_to_floor(state),
        AppScreen::PassiveTree => auto_back_to_floor(state),
        AppScreen::Scoreboard
        | AppScreen::Achievements
        | AppScreen::RunHistory
        | AppScreen::DailyLeaderboard
        | AppScreen::Bestiary
        | AppScreen::Codex => auto_back_to_floor(state),
        _ => false,
    };

    if acted {
        state.auto_last_action = state.frame;
    }

    acted
}

/// FloorNav: enter the current room.
fn auto_floor_nav(state: &mut GameState, _engine: &mut ProofEngine) -> bool {
    // If we can descend (all rooms visited), do that
    if crate::game_logic::can_descend(state) {
        crate::game_logic::descend(state);
        return true;
    }
    // Otherwise enter the current room
    crate::game_logic::enter_room(state);
    true
}

/// Combat: Attack normally, or Defend if HP is low.
/// We set a synthetic input flag that the combat update will pick up.
fn auto_combat(state: &mut GameState, engine: &mut ProofEngine) -> bool {
    use chaos_rpg_core::combat::{resolve_action, CombatAction, CombatOutcome};

    if state.kill_linger > 0.0 {
        return false;
    }

    let hp_frac = state
        .player
        .as_ref()
        .map(|p| p.current_hp as f32 / p.max_hp.max(1) as f32)
        .unwrap_or(1.0);

    let action = if hp_frac < 0.30 {
        CombatAction::Defend
    } else {
        CombatAction::Attack
    };

    if let (Some(ref mut player), Some(ref mut enemy), Some(ref mut combat)) =
        (&mut state.player, &mut state.enemy, &mut state.combat_state)
    {
        state.last_action_type = match action {
            CombatAction::Defend => 4,
            _ => 1,
        };
        state.spell_beam_timer = 0.5;

        let prev_enemy_hp = enemy.hp as f32 / enemy.max_hp.max(1) as f32;
        let prev_player_hp = player.current_hp as f32 / player.max_hp.max(1) as f32;

        let (events, outcome) = resolve_action(player, enemy, action, combat);

        for event in &events {
            state.combat_log.push(event.to_display_string());
            match event {
                chaos_rpg_core::combat::CombatEvent::PlayerAttack { damage, .. } => {
                    state.enemy_flash = 0.4;
                    let shake = (*damage as f32 / 50.0).clamp(0.05, 0.4);
                    engine.add_trauma(shake);
                    state.ghost_enemy_hp = prev_enemy_hp;
                    state.ghost_enemy_timer = 1.5;
                }
                chaos_rpg_core::combat::CombatEvent::EnemyAttack { damage, .. } => {
                    state.player_flash = 0.4;
                    let shake = (*damage as f32 / 40.0).clamp(0.05, 0.5);
                    engine.add_trauma(shake);
                    state.ghost_player_hp = prev_player_hp;
                    state.ghost_player_timer = 1.5;
                }
                _ => {}
            }
        }

        // Chaos trace log
        if let Some(ref roll) = state.last_roll {
            for step in &roll.chain {
                let delta = step.output - step.input;
                state.combat_log.push(format!(
                    "[{}] {:.2} -> {:.2} ({:+.2})",
                    step.engine_name, step.input, step.output, delta,
                ));
            }
            let verdict = if roll.is_critical() { "CRITICAL" }
                else if roll.is_catastrophe() { "CATASTROPHE" }
                else if roll.final_value > 0.3 { "CLEAN HIT" }
                else if roll.final_value > -0.3 { "WEAK" }
                else { "MISS" };
            state.combat_log.push(format!("Final: {:+.3} {}", roll.final_value, verdict));
        }

        match outcome {
            CombatOutcome::Ongoing => {}
            CombatOutcome::PlayerWon { xp, gold } => {
                crate::audio_bridge::on_enemy_death(engine, state.is_boss_fight);
                crate::game_logic::on_combat_victory(state, xp, gold);
                engine.add_trauma(0.6);
            }
            CombatOutcome::PlayerDied => {
                crate::audio_bridge::on_player_death(engine);
                crate::game_logic::on_player_death(state);
                engine.add_trauma(0.8);
            }
            CombatOutcome::PlayerFled => {
                crate::game_logic::on_player_fled(state);
            }
        }
        return true;
    }
    false
}

/// RoomView: accept/continue (press Enter equivalent).
fn auto_room_view(state: &mut GameState, _engine: &mut ProofEngine) -> bool {
    // Pick up items if available
    if state.room_event.pending_item.is_some() {
        if let Some(item) = state.room_event.pending_item.take() {
            if let Some(ref mut player) = state.player {
                player.inventory.push(item);
            }
        }
    }
    // Learn spells if available
    if state.room_event.pending_spell.is_some() {
        if let Some(spell) = state.room_event.pending_spell.take() {
            if let Some(ref mut player) = state.player {
                player.known_spells.push(spell);
            }
        }
    }
    // Apply room event and continue
    if !state.room_event.resolved {
        crate::game_logic::apply_room_event(state);
        state.room_event.resolved = true;
    } else {
        if let Some(ref mut floor) = state.floor {
            floor.advance();
        }
        state.screen = AppScreen::FloorNav;
    }
    true
}

/// Shop: leave immediately.
fn auto_shop(state: &mut GameState, _engine: &mut ProofEngine) -> bool {
    state.screen = AppScreen::FloorNav;
    true
}

/// Crafting: leave immediately.
fn auto_crafting(state: &mut GameState, _engine: &mut ProofEngine) -> bool {
    state.screen = AppScreen::FloorNav;
    true
}

/// Any meta screen: return to floor nav.
fn auto_back_to_floor(state: &mut GameState) -> bool {
    state.screen = AppScreen::FloorNav;
    true
}

/// Render the AUTO-PILOT indicator overlay.
/// Call this at the end of ANY screen's render when auto_mode is active.
pub fn render_indicator(state: &GameState, engine: &mut ProofEngine) {
    if !state.auto_mode {
        return;
    }

    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let t = state.frame as f32 * 0.016;

    // Pulsing "AUTO-PILOT" text in top-right corner
    let pulse = (t * 3.0).sin() * 0.15 + 0.85;
    let color = Vec4::new(
        1.0 * pulse,
        0.8 * pulse,
        0.0,
        1.0,
    );

    // Background panel for visibility
    ui_render::panel_bg(engine, 5.0, 5.2, 3.5, 0.5, Vec4::new(0.0, 0.0, 0.0, 0.8), 0.3);
    ui_render::text_z(engine, "AUTO-PILOT", 5.2, 5.0, ui_render::Z_TOP, color, 0.35, 0.8);

    // Small blinking dot
    let blink = if (state.frame / 15) % 2 == 0 { '*' } else { ' ' };
    if blink != ' ' {
        engine.spawn_glyph(Glyph {
            character: blink,
            position: Vec3::new(4.7, -5.0, ui_render::Z_TOP),
            scale: Vec2::splat(0.3),
            color: Vec4::new(1.0, 0.3, 0.0, 1.0),
            emission: 0.8,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }

    // Show [Z] to toggle off
    ui_render::text_z(engine, "[Z] Stop", 5.4, 4.6, ui_render::Z_TOP, theme.dim, 0.22, 0.3);
}
