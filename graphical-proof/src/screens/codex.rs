//! Codex — lore fragments and world knowledge.

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

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let up = engine.input.just_pressed(Key::Up);
    let down = engine.input.just_pressed(Key::Down);
    let esc = engine.input.just_pressed(Key::Escape) || engine.input.just_pressed(Key::Q);
    let enter = engine.input.just_pressed(Key::Enter) || engine.input.just_pressed(Key::Space);

    if up && state.codex_scroll > 0 { state.codex_scroll -= 1; }
    if down && state.codex_scroll < CODEX_ENTRIES.len().saturating_sub(1) { state.codex_scroll += 1; }
    if esc || enter { state.screen = AppScreen::Title; }
}

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];

    ui_render::heading_centered(engine, "CODEX", 5.0, theme.heading);
    ui_render::small(engine, &format!("{} entries", CODEX_ENTRIES.len()), -2.0, 4.2, theme.dim);

    // Left panel: entry list
    let start = state.codex_scroll.saturating_sub(5);
    let end = (start + 14).min(CODEX_ENTRIES.len());
    for (di, idx) in (start..end).enumerate() {
        let (title, _) = CODEX_ENTRIES[idx];
        let selected = idx == state.codex_scroll;
        let color = if selected { theme.selected } else { theme.primary };
        let prefix = if selected { "> " } else { "  " };
        let truncated: String = format!("{}{}", prefix, title).chars().take(25).collect();
        ui_render::text(engine, &truncated, -8.2, 3.2 - di as f32 * 0.48, color, 0.28,
            if selected { 0.7 } else { 0.35 });
    }

    // Right panel: selected entry content
    if let Some((title, body)) = CODEX_ENTRIES.get(state.codex_scroll) {
        let px = 1.0;
        ui_render::body(engine, title, px, 3.2, theme.heading);

        // Word-wrap the body text
        let max_width = 28;
        let mut y = 2.5;
        let mut line = String::new();
        for word in body.split_whitespace() {
            if line.len() + word.len() + 1 > max_width {
                ui_render::small(engine, &line, px, y, theme.primary);
                y -= 0.35;
                line = word.to_string();
                if y < -4.5 { break; }
            } else {
                if !line.is_empty() { line.push(' '); }
                line.push_str(word);
            }
        }
        if !line.is_empty() && y >= -4.5 {
            ui_render::small(engine, &line, px, y, theme.primary);
        }
    }

    ui_render::small(engine, "[Up/Down] Scroll  [Enter/Space/Esc] Back", -6.5, -5.2, theme.muted);
}
