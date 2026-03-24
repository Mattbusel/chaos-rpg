//! Combat screen — the engine showcase.
//!
//! 3D perspective arena with player/enemy AmorphousEntities,
//! attack/spell animations, status effects, HP bars.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

use chaos_rpg_core::combat::{resolve_action, CombatAction, CombatOutcome};

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    // Read input into locals to avoid borrow conflicts
    let key_a = engine.input.just_pressed(Key::A) || engine.input.just_pressed(Key::Num1);
    let key_h = engine.input.just_pressed(Key::H) || engine.input.just_pressed(Key::Num2);
    let key_d = engine.input.just_pressed(Key::D) || engine.input.just_pressed(Key::Num3);
    let key_f = engine.input.just_pressed(Key::F);
    let key_t = engine.input.just_pressed(Key::T);
    let key_v = engine.input.just_pressed(Key::V);
    let key_l = engine.input.just_pressed(Key::L);

    // Kill linger: don't accept input while death animation plays
    if state.kill_linger > 0.0 {
        return;
    }

    // Combat input
    if let (Some(ref mut player), Some(ref mut enemy), Some(ref mut combat)) =
        (&mut state.player, &mut state.enemy, &mut state.combat_state)
    {
        let action = if key_a {
            Some(CombatAction::Attack)
        } else if key_h {
            Some(CombatAction::HeavyAttack)
        } else if key_d {
            Some(CombatAction::Defend)
        } else if key_f {
            Some(CombatAction::Flee)
        } else if key_t {
            Some(CombatAction::Taunt)
        } else {
            None
        };

        if let Some(action) = action {
            let (events, outcome) = resolve_action(player, enemy, action, combat);

            // Process events for visual effects
            for event in &events {
                let msg = event.to_display_string();
                state.combat_log.push(msg);
            }

            // Check outcome
            match outcome {
                CombatOutcome::Ongoing => {}
                CombatOutcome::PlayerWon { xp, gold } => {
                    state.kill_linger = 2.5; // seconds
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

    // Toggle chaos engine visualizer
    if key_v {
        state.chaos_viz_open = !state.chaos_viz_open;
    }

    // Toggle combat log
    if key_l {
        state.combat_log_collapsed = !state.combat_log_collapsed;
    }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    // ── Player info panel (left side) ──
    if let Some(ref player) = state.player {
        let panel_x = -18.0;
        let panel_y = 8.0;

        // Name and class
        let name_line = format!("{} ({})", player.name, player.class.name());
        render_text(engine, &name_line, panel_x, panel_y, theme.heading, 0.8);

        // HP bar
        let hp_pct = state.display_player_hp;
        let hp_color = theme.hp_color(hp_pct);
        render_text(engine, "HP", panel_x, panel_y - 1.2, theme.muted, 0.3);
        render_bar(engine, panel_x + 2.0, panel_y - 1.2, 10.0, hp_pct, hp_color);
        let hp_text = format!("{}/{}", player.current_hp, player.max_hp);
        render_text(engine, &hp_text, panel_x + 13.0, panel_y - 1.2, hp_color, 0.5);

        // MP bar
        let max_mp = state.max_mana();
        if max_mp > 0 {
            render_text(engine, "MP", panel_x, panel_y - 2.4, theme.muted, 0.3);
            render_bar(engine, panel_x + 2.0, panel_y - 2.4, 10.0, state.display_mp, theme.mana);
            let mp_text = format!("{}/{}", state.current_mana, max_mp);
            render_text(engine, &mp_text, panel_x + 13.0, panel_y - 2.4, theme.mana, 0.5);
        }

        // Level / Gold / Kills
        let info = format!("Lv.{} | {} gold | {} kills", player.level, player.gold, player.kills);
        render_text(engine, &info, panel_x, panel_y - 3.8, theme.dim, 0.3);
    }

    // ── Enemy info panel (right side) ──
    if let Some(ref enemy) = state.enemy {
        let panel_x = 4.0;
        let panel_y = 8.0;

        render_text(engine, &enemy.name, panel_x, panel_y, theme.danger, 0.8);

        let hp_pct = state.display_enemy_hp;
        let hp_color = theme.hp_color(hp_pct);
        render_text(engine, "HP", panel_x, panel_y - 1.2, theme.muted, 0.3);
        render_bar(engine, panel_x + 2.0, panel_y - 1.2, 10.0, hp_pct, hp_color);
        let hp_text = format!("{}/{}", enemy.hp, enemy.max_hp);
        render_text(engine, &hp_text, panel_x + 13.0, panel_y - 1.2, hp_color, 0.5);

        render_text(engine, &format!("Tier: {:?}", enemy.tier), panel_x, panel_y - 2.4, theme.dim, 0.3);
    }

    // ── Action bar (bottom) ──
    let actions = ["[A]ttack", "[H]eavy", "[D]efend", "[F]lee", "[T]aunt", "[V]iz"];
    let bar_y = -12.0;
    let mut x = -16.0;
    for label in &actions {
        render_text(engine, label, x, bar_y, theme.primary, 0.5);
        x += label.len() as f32 * 0.45 + 1.0;
    }

    // ── Combat log (bottom panel) ──
    if !state.combat_log_collapsed {
        let log_start = state.combat_log.len().saturating_sub(5);
        for (i, msg) in state.combat_log[log_start..].iter().enumerate() {
            let truncated: String = msg.chars().take(60).collect();
            render_text(engine, &truncated, -18.0, -8.0 - i as f32 * 0.8, theme.dim, 0.25);
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn render_text(engine: &mut ProofEngine, text: &str, x: f32, y: f32, color: Vec4, emission: f32) {
    for (i, ch) in text.chars().enumerate() {
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x + i as f32 * 0.45, y, 0.0),
            color,
            emission,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}

fn render_bar(engine: &mut ProofEngine, x: f32, y: f32, width: f32, pct: f32, color: Vec4) {
    let filled = (width * pct.clamp(0.0, 1.0)) as usize;
    let empty = width as usize - filled;

    for i in 0..filled {
        engine.spawn_glyph(Glyph {
            character: '█',
            position: Vec3::new(x + i as f32 * 0.45, y, 0.0),
            color,
            emission: 0.6,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
    for i in 0..empty {
        engine.spawn_glyph(Glyph {
            character: '░',
            position: Vec3::new(x + (filled + i) as f32 * 0.45, y, 0.0),
            color: Vec4::new(color.x * 0.2, color.y * 0.2, color.z * 0.2, 0.5),
            emission: 0.1,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}
