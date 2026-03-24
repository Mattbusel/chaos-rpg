//! Floor navigation screen — room map with status sidebar.

use proof_engine::prelude::*;
use chaos_rpg_core::world::{Floor, RoomType};
use chaos_rpg_core::enemy::generate_enemy;
use chaos_rpg_core::combat::CombatState;
use crate::state::{AppScreen, GameState, RoomEvent};
use crate::theme::THEMES;

// ── Helpers ─────────────────────────────────────────────────────────────────

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

fn render_bar(
    engine: &mut ProofEngine,
    x: f32, y: f32,
    current: i64, max: i64,
    width: usize,
    fill_color: Vec4, empty_color: Vec4,
    emission: f32,
) {
    let ratio = if max > 0 { (current as f32 / max as f32).clamp(0.0, 1.0) } else { 0.0 };
    let filled = (ratio * width as f32).round() as usize;
    for i in 0..width {
        let ch = if i < filled { '\u{2588}' } else { '\u{2591}' }; // █ and ░
        let color = if i < filled { fill_color } else { empty_color };
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

fn room_icon(rt: &RoomType) -> char {
    match rt {
        RoomType::Combat => '\u{00D7}',       // ×
        RoomType::Boss => '\u{2605}',          // ★
        RoomType::Treasure => '\u{2666}',      // ♦
        RoomType::Shop => '$',
        RoomType::Shrine => '~',
        RoomType::Trap => '!',
        RoomType::Portal => '^',
        RoomType::Empty => '\u{00B7}',         // ·
        RoomType::ChaosRift => '\u{26A1}',     // ⚡
        RoomType::CraftingBench => '\u{2692}', // ⚒
    }
}

fn room_label(rt: &RoomType) -> &'static str {
    match rt {
        RoomType::Combat => "Combat",
        RoomType::Boss => "BOSS",
        RoomType::Treasure => "Treasure",
        RoomType::Shop => "Shop",
        RoomType::Shrine => "Shrine",
        RoomType::Trap => "Trap",
        RoomType::Portal => "Portal",
        RoomType::Empty => "Empty",
        RoomType::ChaosRift => "Chaos Rift",
        RoomType::CraftingBench => "Crafting",
    }
}

// ── Update ──────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    // Read inputs before borrowing engine mutably
    let key_e = engine.input.just_pressed(Key::E) || engine.input.just_pressed(Key::Enter);
    let key_d = engine.input.just_pressed(Key::D);
    let key_c = engine.input.just_pressed(Key::C);
    let key_n = engine.input.just_pressed(Key::N);
    let key_q = engine.input.just_pressed(Key::Q) || engine.input.just_pressed(Key::Escape);
    let key_up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
    let key_down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);

    // Navigate room selection
    if let Some(ref mut floor) = state.floor {
        let room_count = floor.rooms.len();
        if key_up && floor.current_room > 0 {
            floor.current_room -= 1;
        }
        if key_down && floor.current_room + 1 < room_count {
            floor.current_room += 1;
        }
    }

    // Enter current room
    if key_e {
        if let Some(ref floor) = state.floor {
            let room = &floor.rooms[floor.current_room];
            match room.room_type {
                RoomType::Combat | RoomType::Boss => {
                    let enemy = generate_enemy(
                        state.floor_num,
                        state.seed.wrapping_add(floor.current_room as u64),
                    );
                    state.is_boss_fight = matches!(room.room_type, RoomType::Boss);
                    state.enemy = Some(enemy);
                    state.combat_state = Some(CombatState::new(state.seed));
                    state.combat_log.clear();
                    state.display_enemy_hp = 1.0;
                    state.screen = AppScreen::Combat;
                }
                RoomType::Shop => {
                    state.screen = AppScreen::Shop;
                }
                RoomType::CraftingBench => {
                    state.screen = AppScreen::Crafting;
                }
                RoomType::Shrine | RoomType::Treasure | RoomType::Trap
                | RoomType::Portal | RoomType::Empty | RoomType::ChaosRift => {
                    state.room_event = RoomEvent::empty();
                    state.room_event.title = format!("{:?}", room.room_type);
                    state.room_event.lines.push(room.description.clone());
                    state.room_event.resolved = false;
                    state.screen = AppScreen::RoomView;
                }
            }
        }
    }

    // Descend to next floor
    if key_d {
        state.floor_num += 1;
        let new_floor = chaos_rpg_core::world::generate_floor(
            state.floor_num,
            state.seed.wrapping_add(state.floor_num as u64),
        );
        state.floor = Some(new_floor);
        if let Some(ref mut player) = state.player {
            player.floor = state.floor_num;
        }
    }

    if key_c {
        state.screen = AppScreen::CharacterSheet;
    }
    if key_n {
        state.screen = AppScreen::PassiveTree;
    }
    if key_q {
        state.screen = AppScreen::Title;
    }
}

// ── Render ──────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    // ── Header ──────────────────────────────────────────────────────────────
    let header = if let Some(ref p) = state.player {
        format!(
            "Floor {}  |  {} — Lv.{} {:?}",
            state.floor_num, p.name, p.level, p.class
        )
    } else {
        format!("Floor {}", state.floor_num)
    };
    render_text(engine, &header, -14.0, 8.5, theme.heading, 0.9);

    // Divider
    let divider = "─".repeat(70);
    render_text(engine, &divider, -14.0, 7.8, theme.border, 0.2);

    // ── Room Map (left side) ────────────────────────────────────────────────
    render_text(engine, "ROOM MAP", -14.0, 7.0, theme.accent, 0.7);

    if let Some(ref floor) = state.floor {
        let visible_slots = 12usize;
        let total = floor.rooms.len();
        // Scroll window so current room stays visible
        let scroll_start = if floor.current_room >= visible_slots {
            floor.current_room - visible_slots + 1
        } else {
            0
        };
        let scroll_end = (scroll_start + visible_slots).min(total);

        for (draw_idx, room_idx) in (scroll_start..scroll_end).enumerate() {
            let room = &floor.rooms[room_idx];
            let y = 5.8 - draw_idx as f32 * 1.0;
            let is_current = room_idx == floor.current_room;

            // Cursor indicator
            if is_current {
                render_text(engine, ">", -14.0, y, theme.selected, 1.0);
            }

            // Room icon
            let icon = room_icon(&room.room_type);
            let icon_color = match room.room_type {
                RoomType::Boss => theme.danger,
                RoomType::Treasure => theme.warn,
                RoomType::Shop => theme.success,
                RoomType::Trap => theme.danger,
                RoomType::ChaosRift => theme.accent,
                _ => theme.primary,
            };
            let icon_str = format!("[{}]", icon);
            render_text(engine, &icon_str, -13.0, y, icon_color, if is_current { 1.0 } else { 0.5 });

            // Room label and index
            let label = format!(
                "{:>2}. {}",
                room_idx + 1,
                room_label(&room.room_type)
            );
            let label_color = if is_current { theme.selected } else { theme.primary };
            render_text(engine, &label, -11.2, y, label_color, if is_current { 0.8 } else { 0.4 });

            // Description snippet for current room
            if is_current && !room.description.is_empty() {
                let desc: String = room.description.chars().take(40).collect();
                render_text(engine, &desc, -13.0, y - 0.45, theme.dim, 0.3);
            }
        }

        // Scroll indicators
        if scroll_start > 0 {
            render_text(engine, "  ... more above ...", -13.0, 6.3, theme.muted, 0.3);
        }
        if scroll_end < total {
            let bot_y = 5.8 - visible_slots as f32 * 1.0;
            render_text(engine, "  ... more below ...", -13.0, bot_y, theme.muted, 0.3);
        }
    } else {
        render_text(engine, "No floor data.", -14.0, 5.0, theme.dim, 0.4);
    }

    // ── Status Sidebar (right side) ─────────────────────────────────────────
    let sx = 4.0;
    render_text(engine, "STATUS", sx, 7.0, theme.accent, 0.7);

    if let Some(ref p) = state.player {
        // HP
        let hp_label = format!("HP {}/{}", p.current_hp, p.max_hp);
        render_text(engine, &hp_label, sx, 5.8, theme.primary, 0.5);
        render_bar(engine, sx, 5.3, p.current_hp, p.max_hp, 20, theme.danger, theme.muted, 0.6);

        // MP
        let mp_label = format!("MP {}/{}", state.current_mana, (p.stats.mana + 50).max(50));
        render_text(engine, &mp_label, sx, 4.4, theme.primary, 0.5);
        render_bar(engine, sx, 3.9, state.current_mana, (p.stats.mana + 50).max(50), 20, theme.accent, theme.muted, 0.6);

        // Gold
        let gold_line = format!("Gold: {}", p.gold);
        render_text(engine, &gold_line, sx, 2.8, theme.warn, 0.5);

        // Kills
        let kills_line = format!("Kills: {}", p.kills);
        render_text(engine, &kills_line, sx, 2.0, theme.primary, 0.4);

        // Power tier
        let tier = (p.level / 5).min(10);
        let tier_line = format!("Power Tier: {}", tier);
        render_text(engine, &tier_line, sx, 1.2, theme.success, 0.5);

        // Floor progress
        if let Some(ref floor) = state.floor {
            let progress = format!(
                "Room {}/{}",
                floor.current_room + 1,
                floor.rooms.len()
            );
            render_text(engine, &progress, sx, 0.2, theme.dim, 0.4);
        }
    }

    // ── Controls ────────────────────────────────────────────────────────────
    let cy = -6.5;
    render_text(engine, "CONTROLS", -14.0, cy, theme.accent, 0.6);
    render_text(engine, "[W/Up] [S/Down] Select room", -14.0, cy - 0.8, theme.dim, 0.35);
    render_text(engine, "[E/Enter] Enter room", -14.0, cy - 1.5, theme.dim, 0.35);
    render_text(engine, "[D] Descend to next floor", -14.0, cy - 2.2, theme.dim, 0.35);
    render_text(engine, "[C] Character  [N] Passives", -14.0, cy - 2.9, theme.dim, 0.35);
    render_text(engine, "[Esc/Q] Quit to title", -14.0, cy - 3.6, theme.dim, 0.35);
}
