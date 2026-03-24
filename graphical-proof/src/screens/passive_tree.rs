//! Passive tree browser — 820+ nodes across 8 class rings.

use proof_engine::prelude::*;
use chaos_rpg_core::passive_tree;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up);
    let down = engine.input.just_pressed(Key::Down);
    let pgup = engine.input.just_pressed(Key::PageUp);
    let pgdn = engine.input.just_pressed(Key::PageDown);
    let enter = engine.input.just_pressed(Key::Enter);
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
                // Find first unallocated node
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
    let tree = passive_tree::nodes();

    // Header
    let sp = state.player.as_ref().map(|p| p.skill_points).unwrap_or(0);
    let allocated = state.player.as_ref().map(|p| p.allocated_nodes.len()).unwrap_or(0);
    let header = format!("PASSIVE TREE — {}/{} allocated — {} points available", allocated, tree.len(), sp);
    render_text(engine, &header, -18.0, 9.0, theme.heading, 0.8);

    if sp > 0 {
        let pulse = ((state.frame as f32 * 0.1).sin() * 0.3 + 0.7).max(0.0);
        render_text(engine, &format!("★ {} SKILL POINTS AVAILABLE", sp), -10.0, 7.5,
            Vec4::new(theme.gold.x * pulse, theme.gold.y * pulse, theme.gold.z * pulse, pulse), 0.7);
    }

    // Column headers
    render_text(engine, "Status   Node Name                    Type        Effect", -18.0, 6.5, theme.muted, 0.3);
    render_text(engine, "─".repeat(60).as_str(), -18.0, 6.0, theme.muted, 0.15);

    // Node list
    let player_nodes: Vec<u32> = state.player.as_ref()
        .map(|p| p.allocated_nodes.clone())
        .unwrap_or_default();

    let display_start = state.passive_scroll.saturating_sub(8);
    let display_end = (display_start + 18).min(tree.len());

    for (di, idx) in (display_start..display_end).enumerate() {
        let node = &tree[idx];
        let is_selected = idx == state.passive_scroll;
        let is_allocated = player_nodes.contains(&(node.id as u32));

        let status = if is_allocated { "■" } else { "□" };
        let (color, emission) = if is_selected {
            (theme.selected, 0.8)
        } else if is_allocated {
            (theme.accent, 0.5)
        } else {
            (theme.dim, 0.25)
        };

        let node_type_str = format!("{:?}", node.node_type);
        let truncated: String = node_type_str.chars().take(45).collect();
        let line = format!("  {}  {:>4}  {}", status, node.id, truncated);
        let display: String = line.chars().take(65).collect();
        let prefix = if is_selected { "> " } else { "  " };

        render_text(engine, &format!("{}{}", prefix, display), -18.0, 5.0 - di as f32 * 0.9, color, emission);
    }

    // Scroll indicator
    let scroll_pct = if tree.is_empty() { 0.0 } else { state.passive_scroll as f32 / tree.len() as f32 };
    let bar_y = 5.0 - scroll_pct * 16.0;
    render_text(engine, "█", 18.0, bar_y, theme.accent, 0.4);

    // Selected node detail
    if state.passive_scroll < tree.len() {
        let node = &tree[state.passive_scroll];
        render_text(engine, &format!("Node #{} — {}", node.id, node.name),
            -18.0, -10.0, theme.heading, 0.5);
        let detail = format!("{} — {:?}", node.short_desc, node.node_type);
        let truncated: String = detail.chars().take(70).collect();
        render_text(engine, &truncated, -18.0, -11.0, theme.primary, 0.35);
    }

    render_text(engine, "[Up/Down] Navigate  [Enter] Allocate  [N] Auto  [PgUp/PgDn] Jump  [Esc] Back",
        -18.0, -13.0, theme.muted, 0.2);
}

fn render_text(engine: &mut ProofEngine, text: &str, x: f32, y: f32, color: Vec4, emission: f32) {
    for (i, ch) in text.chars().enumerate() {
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(x + i as f32 * 0.45, y, 0.0),
            color, emission,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}
