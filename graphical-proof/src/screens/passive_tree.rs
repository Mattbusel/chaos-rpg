//! Passive tree browser — 820+ nodes across 8 class rings.
//! Node graph visualization with connected nodes, highlight path, scroll.

use proof_engine::prelude::*;
use chaos_rpg_core::passive_tree;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

// ── Update ──────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
    let down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);
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

// ── Render ──────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let tree = passive_tree::nodes();
    let frame = state.frame;

    // ── Header ──
    let sp = state.player.as_ref().map(|p| p.skill_points).unwrap_or(0);
    let allocated = state.player.as_ref().map(|p| p.allocated_nodes.len()).unwrap_or(0);
    ui_render::heading_centered(engine, "PASSIVE TREE", 5.0, theme.heading);

    // Allocation summary
    let summary = format!("{}/{} allocated | {} points free", allocated, tree.len(), sp);
    ui_render::text_centered(engine, &summary, 4.2, theme.dim, 0.25, 0.3);

    // Skill points available banner (pulsing)
    if sp > 0 {
        let pulse = ((frame as f32 * 0.1).sin() * 0.3 + 0.7).max(0.0);
        let gold_pulse = Vec4::new(theme.gold.x * pulse, theme.gold.y * pulse, theme.gold.z * pulse, pulse);
        ui_render::text_centered(engine, &format!("{} SKILL POINTS AVAILABLE", sp), 3.6, gold_pulse, 0.3, 0.7);
    }

    // ── Mini node graph visualization (top right) ──
    render_mini_graph(state, engine, theme, tree, frame);

    // ── Column headers ──
    ui_render::text(engine, "  St  ID  Node", -8.2, 3.0, theme.muted, 0.22, 0.2);
    ui_render::text_centered(engine, "--------------------------------", 2.7, theme.border, 0.22, 0.12);

    // ── Node list ──
    let player_nodes: Vec<u32> = state.player.as_ref()
        .map(|p| p.allocated_nodes.clone())
        .unwrap_or_default();

    let display_start = state.passive_scroll.saturating_sub(7);
    let display_end = (display_start + 16).min(tree.len());

    for (di, idx) in (display_start..display_end).enumerate() {
        let node = &tree[idx];
        let is_selected = idx == state.passive_scroll;
        let is_allocated = player_nodes.contains(&(node.id as u32));

        let status = if is_allocated { "#" } else { "." };
        let (color, em) = if is_selected {
            (theme.selected, 0.8)
        } else if is_allocated {
            (theme.accent, 0.45)
        } else {
            (theme.dim, 0.2)
        };

        let node_type_str = format!("{:?}", node.node_type);
        let type_short: String = node_type_str.chars().take(30).collect();
        let prefix = if is_selected { "> " } else { "  " };
        let line = format!("{}{} {:>4} {}", prefix, status, node.id, type_short);
        let display: String = line.chars().take(45).collect();
        ui_render::text(engine, &display, -8.2, 2.2 - di as f32 * 0.45, color, 0.23, em);

        // Connection indicator for allocated nodes
        if is_allocated {
            let glow = ((frame as f32 * 0.06 + idx as f32 * 0.1).sin() * 0.2 + 0.5).max(0.0);
            engine.spawn_glyph(Glyph {
                character: '*',
                position: Vec3::new(-8.6, 2.2 - di as f32 * 0.45, 0.0),
                color: Vec4::new(theme.accent.x * glow, theme.accent.y * glow, theme.accent.z * glow, glow),
                emission: glow * 0.6,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }
    }

    // ── Scroll indicator ──
    let scroll_pct = if tree.is_empty() { 0.0 } else { state.passive_scroll as f32 / tree.len() as f32 };
    let bar_height = 7.0;
    let bar_y = 2.2 - scroll_pct * bar_height;
    ui_render::text(engine, "|", 8.2, bar_y, theme.accent, 0.25, 0.4);
    // Track
    for i in 0..16 {
        ui_render::text(engine, ":", 8.2, 2.2 - i as f32 * 0.45, theme.muted, 0.15, 0.05);
    }

    // ── Selected node detail ──
    if state.passive_scroll < tree.len() {
        let node = &tree[state.passive_scroll];
        let is_alloc = player_nodes.contains(&(node.id as u32));
        let status_text = if is_alloc { "[ALLOCATED]" } else { "[locked]" };
        let status_color = if is_alloc { theme.success } else { theme.muted };

        ui_render::text(engine, &format!("Node #{} - {}", node.id, node.name), -8.2, -5.0, theme.heading, 0.28, 0.5);
        ui_render::small(engine, &node.short_desc, -8.2, -5.4, theme.dim);
        ui_render::small(engine, &format!("{:?} {}", node.node_type, status_text), -8.2, -5.8, status_color);

        // Prerequisites
        if !node.requires.is_empty() {
            let req_str = format!("Requires: {:?}", node.requires);
            let truncated: String = req_str.chars().take(40).collect();
            ui_render::small(engine, &truncated, 2.0, -5.4, theme.warn);
        }
    }

    // ── Footer ──
    ui_render::small(engine, "[Up/Down] Nav  [Enter/Space] Allocate  [N] Auto  [PgUp/Dn] Jump  [Esc] Back", -8.0, -6.5, theme.muted);
}

// ── Mini node graph ─────────────────────────────────────────────────────────

fn render_mini_graph(
    state: &GameState,
    engine: &mut ProofEngine,
    theme: &crate::theme::Theme,
    tree: &[passive_tree::TreeNode],
    frame: u64,
) {
    let player_nodes: Vec<u32> = state.player.as_ref()
        .map(|p| p.allocated_nodes.clone())
        .unwrap_or_default();

    // Draw a small radial graph in the top-right
    let cx = 5.5;
    let cy = 4.0;
    let radius = 1.5;
    let node_sample = tree.len().min(24);

    for i in 0..node_sample {
        let angle = (i as f32 / node_sample as f32) * std::f32::consts::TAU;
        let ring = if i < node_sample / 3 { 0.5 } else if i < 2 * node_sample / 3 { 1.0 } else { 1.4 };
        let nx = cx + angle.cos() * radius * ring;
        let ny = cy + angle.sin() * radius * ring * 0.7;

        let node_id = tree.get(i).map(|n| n.id as u32).unwrap_or(0);
        let is_alloc = player_nodes.contains(&node_id);
        let is_sel = i == state.passive_scroll && state.passive_scroll < node_sample;

        let (ch, c, em) = if is_sel {
            ('@', theme.selected, 0.9)
        } else if is_alloc {
            ('#', theme.accent, 0.5)
        } else {
            ('.', theme.muted, 0.15)
        };

        let pulse = if is_alloc { ((frame as f32 * 0.05 + i as f32).sin() * 0.1 + 0.9).max(0.0) } else { 1.0 };
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(nx, ny, 0.0),
            color: Vec4::new(c.x * pulse, c.y * pulse, c.z * pulse, c.w),
            emission: em * pulse,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }
}
