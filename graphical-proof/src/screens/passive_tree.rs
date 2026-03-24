//! Passive tree browser — 820+ nodes across 8 class rings.

use proof_engine::prelude::*;
use chaos_rpg_core::passive_tree;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up);
    let down = engine.input.just_pressed(Key::Down);
    let pgup = engine.input.just_pressed(Key::PageUp);
    let pgdn = engine.input.just_pressed(Key::PageDown);
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::C);
    let auto = engine.input.just_pressed(Key::N);

    let tree = passive_tree::nodes();
    let node_count = tree.len();

    if up && state.passive_scroll > 0 { state.passive_scroll -= 1; }
    if down && state.passive_scroll < node_count.saturating_sub(1) { state.passive_scroll += 1; }
    if pgup { state.passive_scroll = state.passive_scroll.saturating_sub(15); }
    if pgdn { state.passive_scroll = (state.passive_scroll + 15).min(node_count.saturating_sub(1)); }

    // Allocate selected node
    if enter {
        if let Some(ref mut player) = state.player {
            if player.skill_points > 0 && state.passive_scroll < node_count {
                let node = &tree[state.passive_scroll];
                if !player.allocated_nodes.contains(&(node.id as u32)) {
                    player.allocated_nodes.push(node.id as u32);
                    player.skill_points -= 1;
                }
            }
        }
    }

    // Auto-allocate
    if auto {
        if let Some(ref mut player) = state.player {
            while player.skill_points > 0 {
                let next = tree.iter().find(|n| !player.allocated_nodes.contains(&(n.id as u32)));
                if let Some(node) = next {
                    player.allocated_nodes.push(node.id as u32);
                    player.skill_points -= 1;
                } else {
                    break;
                }
            }
        }
    }

    if esc { state.screen = AppScreen::FloorNav; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    ui_render::screen_backing(engine, 0.6);
    let tree = passive_tree::nodes();

    // Header
    let sp = state.player.as_ref().map(|p| p.skill_points).unwrap_or(0);
    let allocated = state.player.as_ref().map(|p| p.allocated_nodes.len()).unwrap_or(0);
    ui_render::heading_centered(engine, "PASSIVE TREE", 5.0, theme.heading);
    ui_render::small(engine, &format!("{}/{} allocated - {} pts", allocated, tree.len(), sp), -4.0, 4.2, theme.dim);

    if sp > 0 {
        let pulse = ((state.frame as f32 * 0.1).sin() * 0.3 + 0.7).max(0.0);
        let c = Vec4::new(theme.gold.x * pulse, theme.gold.y * pulse, theme.gold.z * pulse, pulse);
        ui_render::text_centered(engine, &format!("{} POINTS AVAILABLE", sp), 3.6, c, 0.35, 0.7);
    }

    // Column headers
    ui_render::small(engine, "  St  ID   Node Name", -8.0, 3.0, theme.muted);

    // Node list
    let player_nodes: Vec<u32> = state.player.as_ref()
        .map(|p| p.allocated_nodes.clone())
        .unwrap_or_default();

    let display_start = state.passive_scroll.saturating_sub(6);
    let display_end = (display_start + 16).min(tree.len());

    for (di, idx) in (display_start..display_end).enumerate() {
        let node = &tree[idx];
        let is_selected = idx == state.passive_scroll;
        let is_allocated = player_nodes.contains(&(node.id as u32));

        let status = if is_allocated { "[x]" } else { "[ ]" };
        let (color, emission) = if is_selected {
            (theme.selected, 0.8)
        } else if is_allocated {
            (theme.accent, 0.5)
        } else {
            (theme.dim, 0.25)
        };

        let prefix = if is_selected { "> " } else { "  " };
        let name_trunc: String = node.name.chars().take(22).collect();
        let line = format!("{}{} {:>4} {}", prefix, status, node.id, name_trunc);
        ui_render::text(engine, &line, -8.2, 2.3 - di as f32 * 0.45, color, 0.28, emission);
    }

    // Selected node detail panel (right side)
    if state.passive_scroll < tree.len() {
        let node = &tree[state.passive_scroll];
        let px = 2.5;
        ui_render::body(engine, &format!("#{} {}", node.id, node.name), px, 2.5, theme.heading);
        let desc_trunc: String = node.short_desc.chars().take(30).collect();
        ui_render::small(engine, &desc_trunc, px, 1.9, theme.primary);
        ui_render::small(engine, &format!("Type: {:?}", node.node_type), px, 1.4, theme.accent);
        let is_alloc = player_nodes.contains(&(node.id as u32));
        let status_text = if is_alloc { "ALLOCATED" } else if sp > 0 { "AVAILABLE" } else { "LOCKED" };
        let sc = if is_alloc { theme.success } else if sp > 0 { theme.gold } else { theme.dim };
        ui_render::small(engine, status_text, px, 0.9, sc);
    }

    // Scroll indicator
    if !tree.is_empty() {
        let scroll_pct = state.passive_scroll as f32 / tree.len().max(1) as f32;
        let bar_y = 2.3 - scroll_pct * 7.0;
        ui_render::text(engine, "|", 8.2, bar_y, theme.accent, 0.3, 0.4);
    }

    ui_render::small(engine, "[Up/Dn] Nav [Enter/Space] Alloc [N] Auto [PgUp/Dn] Jump [Esc] Back", -8.5, -5.2, theme.muted);
}
