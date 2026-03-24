//! Crafting bench — recipe list, ingredient slots, result preview,
//! progress bar animation, particle effects on craft success.

use proof_engine::prelude::*;
use crate::state::{AppScreen, CraftPhase, GameState};
use crate::theme::THEMES;
use crate::ui_render;

const OPS: &[(&str, &str)] = &[
    ("Reforge",    "Chaos-roll all modifiers anew"),
    ("Augment",    "Add one new random modifier"),
    ("Annul",      "Remove one random modifier"),
    ("Corrupt",    "Risk-tiered chaos outcome"),
    ("Fuse",       "Double value, upgrade rarity"),
    ("EngineLock", "Embed chaos engine signature"),
    ("Shatter",    "Destroy item, scatter mods"),
    ("Imbue",      "Add 3 charges (max 9)"),
    ("Repair",     "Restore durability"),
];

const OP_RISK: &[&str] = &[
    "Medium", "Low", "Medium", "HIGH", "Medium", "Low", "EXTREME", "Low", "Safe",
];

// ── Update ──────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
    let down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);
    let esc = engine.input.just_pressed(Key::Escape);

    match state.craft_phase {
        CraftPhase::SelectItem => {
            let item_count = state.player.as_ref().map(|p| p.inventory.len()).unwrap_or(0);
            if item_count == 0 {
                if esc || enter { state.screen = AppScreen::FloorNav; }
                return;
            }
            if up && state.craft_item_cursor > 0 { state.craft_item_cursor -= 1; }
            if down && state.craft_item_cursor < item_count.saturating_sub(1) { state.craft_item_cursor += 1; }
            if enter { state.craft_phase = CraftPhase::SelectOp; state.craft_op_cursor = 0; }
            if esc { state.screen = AppScreen::FloorNav; }
        }
        CraftPhase::SelectOp => {
            if up && state.craft_op_cursor > 0 { state.craft_op_cursor -= 1; }
            if down && state.craft_op_cursor < OPS.len() - 1 { state.craft_op_cursor += 1; }
            if enter {
                state.craft_message = format!("Applied {} to item.", OPS[state.craft_op_cursor].0);
                state.craft_anim_timer = 1.0;
                state.craft_anim_type = (state.craft_op_cursor + 1) as u8;
                state.craft_phase = CraftPhase::SelectItem;
            }
            if esc { state.craft_phase = CraftPhase::SelectItem; }
        }
    }
}

// ── Render ──────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let frame = state.frame;

    ui_render::heading_centered(engine, "CRAFTING BENCH", 4.8, theme.heading);

    // Animated anvil
    let anvil_pulse = ((frame as f32 * 0.05).sin() * 0.1 + 0.9).max(0.0);
    let ac = Vec4::new(theme.accent.x * anvil_pulse, theme.accent.y * anvil_pulse, theme.accent.z * anvil_pulse, 1.0);
    ui_render::small(engine, " _/\\_", 5.5, 4.2, ac);
    ui_render::small(engine, "|____|", 5.5, 3.9, ac);

    match state.craft_phase {
        CraftPhase::SelectItem => render_item_select(state, engine, theme, frame),
        CraftPhase::SelectOp => render_op_select(state, engine, theme, frame),
    }

    // Craft result message with animation
    if !state.craft_message.is_empty() && state.craft_anim_timer > 0.0 {
        let fade = state.craft_anim_timer.min(1.0);
        let c = Vec4::new(theme.success.x * fade, theme.success.y * fade, theme.success.z * fade, fade);
        ui_render::text_centered(engine, &state.craft_message, -3.8, c, 0.35, 0.6 * fade);

        let progress = 1.0 - state.craft_anim_timer;
        ui_render::bar(engine, -3.0, -4.3, 6.0, progress, theme.accent, theme.muted, 0.25);

        // Craft particles
        let particle_count = ((1.0 - state.craft_anim_timer) * 12.0) as usize;
        for i in 0..particle_count {
            let seed_f = i as f32 * 61.7 + frame as f32 * 0.12;
            let px = seed_f.sin() * 5.0;
            let py = -3.5 + seed_f.cos() * 1.5;
            let sparkle = fade * 0.7;
            let chars = ['+', '*', '.', '~'];
            engine.spawn_glyph(Glyph {
                character: chars[i % chars.len()],
                position: Vec3::new(px, py, 0.0),
                color: Vec4::new(theme.success.x * sparkle, theme.success.y * sparkle, theme.accent.z * sparkle, sparkle),
                emission: sparkle * 0.8,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }
    }
}

// ── Item selection phase ────────────────────────────────────────────────────

fn render_item_select(
    state: &GameState,
    engine: &mut ProofEngine,
    theme: &crate::theme::Theme,
    frame: u64,
) {
    ui_render::small(engine, "Select an item to modify:", -8.0, 3.5, theme.dim);

    if let Some(ref player) = state.player {
        if player.inventory.is_empty() {
            ui_render::body(engine, "Your inventory is empty.", -5.0, 1.5, theme.muted);
        } else {
            // Item list (left panel)
            let start = state.craft_item_cursor.saturating_sub(6);
            let end = (start + 14).min(player.inventory.len());
            for (di, idx) in (start..end).enumerate() {
                let item = &player.inventory[idx];
                let selected = idx == state.craft_item_cursor;
                let color = if selected { theme.selected } else { theme.primary };
                let em = if selected { 0.7 } else { 0.3 };
                let prefix = if selected { "> " } else { "  " };
                let rarity_tag: String = format!("{:?}", item.rarity).chars().take(5).collect();
                let line = format!("{}{} ({})", prefix, item.name, rarity_tag);
                let truncated: String = line.chars().take(30).collect();
                ui_render::text(engine, &truncated, -8.2, 2.5 - di as f32 * 0.5, color, 0.28, em);
            }

            // Item detail (right panel)
            if let Some(item) = player.inventory.get(state.craft_item_cursor) {
                let px = 1.5;
                let mut py = 3.2;

                let rarity_color = match format!("{:?}", item.rarity).as_str() {
                    "Legendary" => theme.gold,
                    "Rare" => theme.accent,
                    "Uncommon" => theme.success,
                    _ => theme.primary,
                };
                ui_render::text(engine, &item.name, px, py, rarity_color, 0.35, 0.7);
                py -= 0.5;
                ui_render::small(engine, &format!("Rarity: {:?}", item.rarity), px, py, theme.accent);
                py -= 0.38;
                ui_render::small(engine, &format!("Value: {}", item.value), px, py, theme.gold);
                py -= 0.38;
                ui_render::small(engine, &format!("Dur: {}/{}", item.durability, item.max_durability), px, py, theme.primary);

                let dur_pct = item.durability as f32 / item.max_durability.max(1) as f32;
                ui_render::bar(engine, px + 4.5, py, 2.5, dur_pct, theme.success, theme.muted, 0.22);
                py -= 0.55;

                if !item.stat_modifiers.is_empty() {
                    ui_render::small(engine, "-- Modifiers --", px, py, theme.accent);
                    py -= 0.38;
                    for m in item.stat_modifiers.iter().take(6) {
                        let sign = if m.value >= 0 { "+" } else { "" };
                        let c = if m.value >= 0 { theme.success } else { theme.danger };
                        ui_render::small(engine, &format!("{}{} {}", sign, m.value, m.stat), px, py, c);
                        py -= 0.35;
                    }
                }

                // Floating rarity glow
                let glow_pulse = ((frame as f32 * 0.07).sin() * 0.2 + 0.8).max(0.0);
                engine.spawn_glyph(Glyph {
                    character: '*',
                    position: Vec3::new(px + 3.0, 3.5, 0.0),
                    color: Vec4::new(rarity_color.x * glow_pulse, rarity_color.y * glow_pulse, rarity_color.z * glow_pulse, glow_pulse),
                    emission: glow_pulse,
                    layer: RenderLayer::UI,
                    ..Default::default()
                });
            }
        }
    }

    ui_render::small(engine, "[Up/Down] Select  [Enter/Space] Choose  [Esc] Leave", -8.0, -5.0, theme.muted);
}

// ── Operation selection phase ───────────────────────────────────────────────

fn render_op_select(
    state: &GameState,
    engine: &mut ProofEngine,
    theme: &crate::theme::Theme,
    frame: u64,
) {
    if let Some(ref player) = state.player {
        if let Some(item) = player.inventory.get(state.craft_item_cursor) {
            ui_render::text(engine, &format!("Crafting: {}", item.name), -8.0, 3.5, theme.heading, 0.35, 0.6);
        }
    }

    ui_render::small(engine, "Select operation:", -8.0, 2.8, theme.dim);

    for (i, (name, desc)) in OPS.iter().enumerate() {
        let selected = i == state.craft_op_cursor;
        let color = if selected { theme.selected } else { theme.primary };
        let em = if selected { 0.7 } else { 0.3 };
        let prefix = if selected { "> " } else { "  " };
        let line = format!("{}[{}] {} - {}", prefix, i + 1, name, desc);
        let truncated: String = line.chars().take(42).collect();
        ui_render::text(engine, &truncated, -8.2, 2.0 - i as f32 * 0.52, color, 0.27, em);

        if selected {
            let risk = OP_RISK.get(i).unwrap_or(&"?");
            let risk_color = match *risk {
                "EXTREME" => theme.danger,
                "HIGH" => theme.warn,
                "Medium" => theme.primary,
                _ => theme.success,
            };
            let pulse = ((frame as f32 * 0.1).sin() * 0.2 + 0.8).max(0.0);
            ui_render::small(engine, &format!("Risk: {}", risk),
                3.5, 2.0 - i as f32 * 0.52,
                Vec4::new(risk_color.x * pulse, risk_color.y * pulse, risk_color.z * pulse, 1.0));
        }
    }

    ui_render::small(engine, "[Up/Down] Select  [Enter/Space] Apply  [Esc] Back", -8.0, -5.0, theme.muted);
}
