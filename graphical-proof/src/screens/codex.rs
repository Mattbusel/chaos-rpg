//! Codex — lore fragments and world knowledge. Categorized entries,
//! reading pane with word-wrapped text, animated scroll indicator.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

/// Codex entries — static lore content.
const CODEX_ENTRIES: &[(&str, &str)] = &[
    ("The Proof", "The universe is a mathematical proof. Every law of physics is a theorem. Every constant is an axiom. What we call reality is just the output of an equation that nobody wrote."),
    ("The Mathematician", "They say one person understood The Proof. Not just the parts — all of it. Every nested function, every recursive loop, every edge case. They didn't go mad. They went somewhere else. The Proof remembers them."),
    ("Chaos Engines", "Ten algorithms chain together to determine every outcome. Linear. Lorenz. Mandelbrot. Zeta. Collatz. Fibonacci. SharpEdge. Orbit. Recursive. Euler. Your fate is their output."),
    ("Corruption", "Every kill corrupts the engines. At first the corruption is invisible — small parameter drifts. By 100 kills the engines begin to mutate. By 400 kills the mathematics barely resembles what it was."),
    ("The Hunger", "Deep in The Proof, the equations demand blood. Five rooms without a kill and they begin consuming you. Your max HP erodes. The mathematics are not patient."),
    ("Misery", "Suffering has a number. Every damage taken, every backfire, every failed flee — it all accumulates. At 5,000 misery, Spite awakens. At 10,000, Defiance. The universe rewards persistence in failure."),
    ("Spite", "A weapon forged from pure suffering. Spend accumulated misery to deal guaranteed damage, survive killing blows, or curse enemies with your worst stat."),
    ("Defiance", "The statistical impossibility of your continued existence becomes its own power source. The more you defy death, the stronger your passive defenses become."),
    ("The Nemesis", "When you die, your killer remembers. It grows stronger, gains your abilities. It waits for your next incarnation. The mathematics have given it a name: your name, inverted."),
    ("Power Tiers", "Your combined stats determine your tier. Mortal. Awakened. Legendary. Godlike. Beyond Math. Axiom. Theorem. Cardinal. Aleph. Omega. Each tier is a declaration of mathematical significance."),
    ("The Mirror", "A boss that mirrors your formation. It built a function that returned its own input. Fighting it is fighting yourself, delayed by one frame."),
    ("The Null", "A void where mathematics cannot exist. As you fight it, the visual effects die. Bloom fades. Color drains. Particles stop. By the end you're fighting in nothing."),
    ("The Algorithm Reborn", "The final boss. The Proof itself, given will. It adapts to your strategy, counters your strengths, and in Phase 3, becomes the chaos field. Fighting it means fighting the engine itself."),
];

// ── Update ──────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up) || engine.input.just_pressed(Key::W);
    let down = engine.input.just_pressed(Key::Down) || engine.input.just_pressed(Key::S);
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Space);

    if up && state.codex_scroll > 0 { state.codex_scroll -= 1; }
    if down && state.codex_scroll < CODEX_ENTRIES.len().saturating_sub(1) { state.codex_scroll += 1; }
    if esc { state.screen = AppScreen::Title; }
}

// ── Render ──────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let frame = state.frame;

    // ── Header ──
    ui_render::heading_centered(engine, "CODEX", 4.8, theme.heading);
    ui_render::text_centered(engine, &format!("{} entries", CODEX_ENTRIES.len()), 4.0, theme.dim, 0.25, 0.3);

    ui_render::text_centered(engine, "================================", 3.3, theme.border, 0.22, 0.12);

    // ── Left panel: entry list ──
    let start = state.codex_scroll.saturating_sub(5);
    let end = (start + 12).min(CODEX_ENTRIES.len());
    for (di, idx) in (start..end).enumerate() {
        let (title, _) = CODEX_ENTRIES[idx];
        let selected = idx == state.codex_scroll;
        let color = if selected { theme.selected } else { theme.primary };
        let em = if selected { 0.7 } else { 0.3 };
        let prefix = if selected { "> " } else { "  " };

        // Entry number
        let line = format!("{}{:>2}. {}", prefix, idx + 1, title);
        ui_render::text(engine, &line, -8.2, 2.8 - di as f32 * 0.5, color, 0.27, em);

        // Active indicator
        if selected {
            let pulse = ((frame as f32 * 0.08).sin() * 0.2 + 0.8).max(0.0);
            engine.spawn_glyph(Glyph {
                character: '>',
                position: Vec3::new(-8.6, 2.8 - di as f32 * 0.5, 0.0),
                color: Vec4::new(theme.accent.x * pulse, theme.accent.y * pulse, theme.accent.z * pulse, pulse),
                emission: pulse * 0.6,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }
    }

    // Scroll indicator track
    let scroll_pct = if CODEX_ENTRIES.is_empty() { 0.0 } else { state.codex_scroll as f32 / CODEX_ENTRIES.len() as f32 };
    for i in 0..12 {
        ui_render::text(engine, ":", -2.0, 2.8 - i as f32 * 0.5, theme.muted, 0.15, 0.05);
    }
    let indicator_y = 2.8 - scroll_pct * 5.5;
    ui_render::text(engine, "#", -2.0, indicator_y, theme.accent, 0.2, 0.4);

    // ── Right panel: selected entry content ──
    if let Some((title, body)) = CODEX_ENTRIES.get(state.codex_scroll) {
        let px = -0.5;
        let mut py = 2.8;

        // Title with decorative border
        ui_render::text(engine, "+--Reading Pane--+", px - 0.2, py + 0.35, theme.border, 0.2, 0.2);
        ui_render::text(engine, title, px, py, theme.heading, 0.35, 0.7);
        py -= 0.55;

        // Separator
        ui_render::text(engine, "----------------", px, py, theme.muted, 0.2, 0.1);
        py -= 0.45;

        // Word-wrap the body text
        let max_width = 32;
        let mut line_buf = String::new();
        for word in body.split_whitespace() {
            if line_buf.len() + word.len() + 1 > max_width {
                ui_render::text(engine, &line_buf, px, py, theme.dim, 0.23, 0.3);
                py -= 0.38;
                line_buf = word.to_string();
                if py < -4.5 { break; }
            } else {
                if !line_buf.is_empty() { line_buf.push(' '); }
                line_buf.push_str(word);
            }
        }
        if !line_buf.is_empty() && py >= -4.5 {
            ui_render::text(engine, &line_buf, px, py, theme.dim, 0.23, 0.3);
            py -= 0.38;
        }

        // Bottom border
        ui_render::text(engine, "+----------------+", px - 0.2, py - 0.1, theme.border, 0.2, 0.2);
    }

    // ── Ambient particles ──
    for i in 0..3u32 {
        let seed_f = i as f32 * 53.7 + frame as f32 * 0.015;
        let px = -5.0 + seed_f.sin() * 6.0;
        let py = -4.5 + (seed_f * 0.5).cos() * 0.6;
        let glow = ((frame as f32 * 0.05 + i as f32 * 2.0).sin() * 0.3 + 0.4).max(0.0);
        engine.spawn_glyph(Glyph {
            character: '~',
            position: Vec3::new(px, py, 0.0),
            color: Vec4::new(theme.accent.x * glow, theme.accent.y * glow, theme.accent.z * glow, glow * 0.5),
            emission: glow * 0.2,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }

    // ── Footer ──
    ui_render::small(engine, "[Up/Down] Scroll  [Esc/Space] Back", -5.5, -5.2, theme.muted);
}
