//! Floor navigation screen — room map with status sidebar.

use proof_engine::prelude::*;
use chaos_rpg_core::world::RoomType;
use chaos_rpg_core::enemy::generate_enemy;
use chaos_rpg_core::combat::CombatState;
use crate::state::{AppScreen, GameState, RoomEvent};
use crate::theme::THEMES;
use crate::ui_render;

fn room_icon(rt: &RoomType) -> &'static str {
    match rt {
        RoomType::Combat => "[x]",
        RoomType::Boss => "[*]",
        RoomType::Treasure => "[$]",
        RoomType::Shop => "[S]",
        RoomType::Shrine => "[~]",
        RoomType::Trap => "[!]",
        RoomType::Portal => "[^]",
        RoomType::Empty => "[.]",
        RoomType::ChaosRift => "[?]",
        RoomType::CraftingBench => "[C]",
    }
}

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let key_e = engine.input.just_pressed(Key::E) || engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);
    let key_d = engine.input.just_pressed(Key::D);
    let key_c = engine.input.just_pressed(Key::C);
    let key_n = engine.input.just_pressed(Key::N);
    let key_q = engine.input.just_pressed(Key::Q) || engine.input.just_pressed(Key::Escape);
    let key_up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
    let key_down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);

    if let Some(ref mut floor) = state.floor {
        let room_count = floor.rooms.len();
        if key_up && floor.current_room > 0 { floor.current_room -= 1; }
        if key_down && floor.current_room + 1 < room_count { floor.current_room += 1; }
    }

    if key_e {
        if let Some(ref floor) = state.floor {
            let room = &floor.rooms[floor.current_room];
            match room.room_type {
                RoomType::Combat | RoomType::Boss => {
                    let enemy = generate_enemy(state.floor_num, state.seed.wrapping_add(floor.current_room as u64));
                    state.is_boss_fight = matches!(room.room_type, RoomType::Boss);
                    state.enemy = Some(enemy);
                    state.combat_state = Some(CombatState::new(state.seed));
                    state.combat_log.clear();
                    state.display_enemy_hp = 1.0;
                    state.screen = AppScreen::Combat;
                }
                RoomType::Shop => { state.screen = AppScreen::Shop; }
                RoomType::CraftingBench => { state.screen = AppScreen::Crafting; }
                _ => {
                    state.room_event = RoomEvent::empty();
                    state.room_event.title = format!("{:?}", room.room_type);
                    state.room_event.lines.push(room.description.clone());
                    state.room_event.resolved = false;
                    state.screen = AppScreen::RoomView;
                }
            }
        }
    }

    if key_d {
        state.floor_num += 1;
        let new_floor = chaos_rpg_core::world::generate_floor(state.floor_num, state.seed.wrapping_add(state.floor_num as u64));
        state.floor = Some(new_floor);
        if let Some(ref mut player) = state.player { player.floor = state.floor_num; }
    }

    if key_c { state.screen = AppScreen::CharacterSheet; }
    if key_n { state.screen = AppScreen::PassiveTree; }
    if key_q { state.screen = AppScreen::Title; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    // Header
    let header = if let Some(ref p) = state.player {
        format!("Floor {} | {} Lv.{} {}", state.floor_num, p.name, p.level, p.class.name())
    } else {
        format!("Floor {}", state.floor_num)
    };
    ui_render::heading_centered(engine, &header, 5.0, theme.heading);

    // Room map (left side)
    ui_render::text(engine, "ROOM MAP", -8.0, 4.0, theme.accent, 0.35, 0.6);

    if let Some(ref floor) = state.floor {
        let visible = 10usize;
        let start = floor.current_room.saturating_sub(visible / 2);
        let end = (start + visible).min(floor.rooms.len());

        for (di, idx) in (start..end).enumerate() {
            let room = &floor.rooms[idx];
            let is_current = idx == floor.current_room;
            let y = 3.0 - di as f32 * 0.55;

            let icon = room_icon(&room.room_type);
            let label = format!("{}{} {:>2}. {:?}", if is_current { ">" } else { " " }, icon, idx + 1, room.room_type);
            let color = if is_current { theme.selected } else {
                match room.room_type {
                    RoomType::Boss => theme.danger,
                    RoomType::Treasure => theme.gold,
                    RoomType::Shop => theme.success,
                    RoomType::Trap => theme.warn,
                    RoomType::ChaosRift => theme.accent,
                    _ => theme.primary,
                }
            };
            ui_render::text(engine, &label, -8.0, y, color, 0.3, if is_current { 0.7 } else { 0.3 });

            // Description for current room
            if is_current && !room.description.is_empty() {
                let desc: String = room.description.chars().take(35).collect();
                ui_render::small(engine, &desc, -7.5, y - 0.3, theme.dim);
            }
        }
    }

    // Status sidebar (right side)
    ui_render::text(engine, "STATUS", 2.0, 4.0, theme.accent, 0.35, 0.6);

    if let Some(ref p) = state.player {
        let hp_pct = if p.max_hp > 0 { p.current_hp as f32 / p.max_hp as f32 } else { 0.0 };
        ui_render::text(engine, &format!("HP {}/{}", p.current_hp, p.max_hp), 2.0, 3.0, theme.hp_color(hp_pct), 0.3, 0.5);
        ui_render::bar(engine, 2.0, 2.6, 4.0, hp_pct, theme.hp_color(hp_pct), theme.muted, 0.25);

        let max_mp = (p.stats.mana + 50).max(50);
        ui_render::text(engine, &format!("MP {}/{}", state.current_mana, max_mp), 2.0, 2.0, theme.mana, 0.3, 0.5);

        ui_render::text(engine, &format!("Gold: {}", p.gold), 2.0, 1.2, theme.gold, 0.3, 0.4);
        ui_render::text(engine, &format!("Kills: {}", p.kills), 2.0, 0.7, theme.primary, 0.3, 0.4);
        ui_render::text(engine, &format!("Tier: {:?}", p.power_tier()), 2.0, 0.2, theme.accent, 0.3, 0.4);
    }

    // Controls
    ui_render::small(engine, "[E/Enter] Enter room  [D] Descend", -8.0, -4.0, theme.muted);
    ui_render::small(engine, "[C] Character  [N] Passives  [Esc] Quit", -8.0, -4.5, theme.muted);
}
