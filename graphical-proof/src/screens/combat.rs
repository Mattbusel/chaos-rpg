//! Combat screen — the engine showcase.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

use chaos_rpg_core::combat::{resolve_action, CombatAction, CombatOutcome};

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let key_a = engine.input.just_pressed(Key::A) || engine.input.just_pressed(Key::Num1);
    let key_h = engine.input.just_pressed(Key::H) || engine.input.just_pressed(Key::Num2);
    let key_d = engine.input.just_pressed(Key::D) || engine.input.just_pressed(Key::Num3);
    let key_f = engine.input.just_pressed(Key::F);
    let key_t = engine.input.just_pressed(Key::T);
    let key_v = engine.input.just_pressed(Key::V);
    let key_l = engine.input.just_pressed(Key::L);

    if state.kill_linger > 0.0 { return; }

    if let (Some(ref mut player), Some(ref mut enemy), Some(ref mut combat)) =
        (&mut state.player, &mut state.enemy, &mut state.combat_state)
    {
        let action = if key_a { Some(CombatAction::Attack) }
            else if key_h { Some(CombatAction::HeavyAttack) }
            else if key_d { Some(CombatAction::Defend) }
            else if key_f { Some(CombatAction::Flee) }
            else if key_t { Some(CombatAction::Taunt) }
            else { None };

        if let Some(action) = action {
            let (events, outcome) = resolve_action(player, enemy, action, combat);
            for event in &events {
                state.combat_log.push(event.to_display_string());
            }
            match outcome {
                CombatOutcome::Ongoing => {}
                CombatOutcome::PlayerWon { xp: _, gold: _ } => {
                    state.kill_linger = 2.5;
                    state.post_combat_screen = Some(AppScreen::FloorNav);
                    engine.add_trauma(0.6);
                }
                CombatOutcome::PlayerDied => {
                    state.kill_linger = 1.0;
                    state.post_combat_screen = Some(AppScreen::GameOver);
                }
                CombatOutcome::PlayerFled => {
                    state.screen = AppScreen::FloorNav;
                }
            }
        }
    }

    if key_v { state.chaos_viz_open = !state.chaos_viz_open; }
    if key_l { state.combat_log_collapsed = !state.combat_log_collapsed; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    // Header
    let header = if state.is_boss_fight {
        format!("COMBAT - Floor {} - BOSS", state.floor_num)
    } else {
        format!("COMBAT - Floor {}", state.floor_num)
    };
    ui_render::heading_centered(engine, &header, 5.0, theme.heading);

    // Player panel (left)
    if let Some(ref player) = state.player {
        let px = -8.0;
        let py = 3.5;
        ui_render::body(engine, &format!("{} ({})", player.name, player.class.name()), px, py, theme.heading);
        let hp_pct = state.display_player_hp;
        ui_render::small(engine, "HP", px, py - 0.6, theme.muted);
        ui_render::bar(engine, px + 1.0, py - 0.6, 4.0, hp_pct, theme.hp_color(hp_pct), theme.muted, 0.3);
        ui_render::small(engine, &format!("{}/{}", player.current_hp, player.max_hp), px + 5.5, py - 0.6, theme.hp_color(hp_pct));
        let max_mp = state.max_mana();
        if max_mp > 0 {
            ui_render::small(engine, "MP", px, py - 1.2, theme.muted);
            ui_render::bar(engine, px + 1.0, py - 1.2, 4.0, state.display_mp, theme.mana, theme.muted, 0.3);
            ui_render::small(engine, &format!("{}/{}", state.current_mana, max_mp), px + 5.5, py - 1.2, theme.mana);
        }
        ui_render::small(engine, &format!("Lv.{} | {} gold | {} kills", player.level, player.gold, player.kills), px, py - 2.0, theme.dim);
    }

    // Enemy panel (right)
    if let Some(ref enemy) = state.enemy {
        let ex = 1.5;
        let ey = 3.5;
        ui_render::body(engine, &enemy.name, ex, ey, theme.danger);
        let hp_pct = state.display_enemy_hp;
        ui_render::small(engine, "HP", ex, ey - 0.6, theme.muted);
        ui_render::bar(engine, ex + 1.0, ey - 0.6, 4.0, hp_pct, theme.hp_color(hp_pct), theme.muted, 0.3);
        ui_render::small(engine, &format!("{}/{}", enemy.hp, enemy.max_hp), ex + 5.5, ey - 0.6, theme.hp_color(hp_pct));
        ui_render::small(engine, &format!("Tier: {:?}", enemy.tier), ex, ey - 1.2, theme.dim);
    }

    // Actions
    ui_render::small(engine, "[A]ttack [H]eavy [D]efend [F]lee [T]aunt [V]iz", -7.0, -3.5, theme.primary);

    // Combat log
    if !state.combat_log_collapsed {
        let start = state.combat_log.len().saturating_sub(4);
        for (i, msg) in state.combat_log[start..].iter().enumerate() {
            let truncated: String = msg.chars().take(50).collect();
            ui_render::small(engine, &truncated, -8.0, -4.3 - i as f32 * 0.45, theme.dim);
        }
    }

    crate::effects::boss_visuals::render_boss_overlay(state, engine);
}
