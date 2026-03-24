//! Tutorial — 5 step-by-step panels explaining the chaos math,
//! progress dots, animated examples, themed presentation.

use proof_engine::prelude::*;
use crate::state::{AppScreen, GameState};
use crate::theme::THEMES;
use crate::ui_render;

const SLIDES: &[(&str, &[&str])] = &[
    ("WHAT IS CHAOS RPG?", &[
        "A roguelike where every outcome is determined by",
        "chaining mathematical algorithms. No dice. Pure chaos.",
        "",
        "Your stats feed into 10 math engines. Their combined",
        "output determines damage, crits, dodges, and events.",
        "",
        "Higher stats mean better inputs. Better inputs mean",
        "better outputs. But chaos is unpredictable by definition.",
    ]),
    ("THE CHAOS PIPELINE", &[
        "Each action chains 4-8 of these 10 engines:",
        "",
        "  Linear    - proportional scaling (stable)",
        "  Lorenz    - chaotic attractor (wild swings)",
        "  Mandelbrot - fractal boundary (crit or bust)",
        "  Zeta      - Riemann zeta (oscillating extremes)",
        "  Collatz   - 3n+1 conjecture (up then down)",
        "  Fibonacci - golden ratio (moderate, reliable)",
        "  SharpEdge - step function (all or nothing)",
        "  Orbit     - elliptical (periodic outcomes)",
        "  Recursive - self-referencing (amplifies trends)",
        "  Euler     - exponential growth/decay",
    ]),
    ("COMBAT", &[
        "[A] Attack - force-based melee damage",
        "[H] Heavy  - 2x damage, accuracy roll",
        "[D] Defend - reduce incoming damage next turn",
        "[T] Taunt  - provoke enemy, may stun",
        "[F] Flee   - chaos-rolled escape attempt",
        "[1-8] Cast spells (costs mana)",
        "[Q/W/E/R/Y/U/I/O] Use inventory items",
        "",
        "Crits happen when pipeline output > 0.8",
        "Catastrophes happen when output < -0.8",
    ]),
    ("CORRUPTION & HUNGER", &[
        "Every kill corrupts the chaos engines.",
        "At 100 kills: engines begin to mutate.",
        "At 400 kills: mathematics barely resembles itself.",
        "",
        "On floor 50+, The Hunger activates:",
        "5 rooms without a kill = lose 5% max HP.",
        "The equations demand blood.",
        "",
        "Corruption is permanent. It makes the game",
        "progressively stranger. Embrace it.",
    ]),
    ("TIPS", &[
        "Press [V] in combat to see the chaos engine trace.",
        "Press [C] for your character sheet anytime.",
        "Press [T] on the title screen to change themes.",
        "Press [N] on the floor map for the passive tree.",
        "",
        "Crafting can destroy your items. Corrupt wisely.",
        "The Nemesis system remembers how you died.",
        "Misery is a resource, not just a statistic.",
        "",
        "Good luck. The mathematics are not on your side.",
    ]),
];

// ── Update ──────────────────────────────────────────────────────────────────

pub fn update(state: &mut GameState, engine: &mut ProofEngine, _dt: f32) {
    let left = engine.input.just_pressed(Key::Left);
    let right = engine.input.just_pressed(Key::Right) || engine.input.just_pressed(Key::Space) || engine.input.just_pressed(Key::Enter);
    let esc = engine.input.just_pressed(Key::Escape);

    if right && state.tutorial_slide < SLIDES.len() - 1 { state.tutorial_slide += 1; }
    if left && state.tutorial_slide > 0 { state.tutorial_slide -= 1; }
    if esc { state.tutorial_slide = 0; state.screen = AppScreen::Title; }
}

// ── Render ──────────────────────────────────────────────────────────────────

pub fn render(state: &GameState, engine: &mut ProofEngine) {
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let frame = state.frame;
    let slide_idx = state.tutorial_slide.min(SLIDES.len() - 1);
    let (title, lines) = SLIDES[slide_idx];

    // ── Header ──
    let header = format!("TUTORIAL - {}/{}", slide_idx + 1, SLIDES.len());
    ui_render::heading_centered(engine, &header, 4.8, theme.heading);

    // ── Progress dots ──
    let dot_start_x = -(SLIDES.len() as f32 * 0.4 * 0.5);
    for i in 0..SLIDES.len() {
        let active = i == slide_idx;
        let c = if active { theme.selected } else { theme.muted };
        let em = if active { 0.8 } else { 0.15 };
        let ch = if active { '#' } else { 'o' };

        let pulse = if active { ((frame as f32 * 0.08).sin() * 0.15 + 0.85).max(0.0) } else { 1.0 };
        engine.spawn_glyph(Glyph {
            character: ch,
            position: Vec3::new(dot_start_x + i as f32 * 0.5, 3.9, 0.0),
            color: Vec4::new(c.x * pulse, c.y * pulse, c.z * pulse, c.w),
            emission: em * pulse,
            layer: RenderLayer::UI,
            ..Default::default()
        });
    }

    // ── Slide title ──
    let title_trunc: String = title.chars().take(35).collect();
    ui_render::text_centered(engine, &title_trunc, 3.2, theme.selected, 0.4, 0.7);

    ui_render::text_centered(engine, "================================", 2.7, theme.border, 0.22, 0.12);

    // ── Content lines with staggered fade-in ──
    for (i, line) in lines.iter().enumerate() {
        let y = 2.2 - i as f32 * 0.48;
        if y < -3.5 { break; }

        // Staggered appearance based on frame count (animated entry)
        let line_delay = i as f32 * 3.0;
        let visible_frames = frame as f32 - line_delay;
        let alpha = (visible_frames * 0.1).clamp(0.0, 1.0);

        if line.is_empty() { continue; }

        let color = if line.starts_with("  ") {
            // Indented items (pipeline names, keybinds)
            Vec4::new(theme.accent.x * alpha, theme.accent.y * alpha, theme.accent.z * alpha, alpha)
        } else if line.starts_with('[') {
            // Keybind lines
            Vec4::new(theme.primary.x * alpha, theme.primary.y * alpha, theme.primary.z * alpha, alpha)
        } else {
            Vec4::new(theme.dim.x * alpha, theme.dim.y * alpha, theme.dim.z * alpha, alpha)
        };

        let truncated: String = line.chars().take(48).collect();
        ui_render::text(engine, &truncated, -7.5, y, color, 0.25, 0.35 * alpha);
    }

    // ── Animated example visualization (slide-specific) ──
    match slide_idx {
        1 => {
            // Pipeline visualization: animated chain
            let chain_y = -2.5;
            let engines = ["LN", "LZ", "MB", "ZT", "CZ", "FB", "SE", "OR", "RC", "EU"];
            for (i, name) in engines.iter().enumerate() {
                let x = -6.0 + i as f32 * 1.5;
                let active = ((frame / 20) % engines.len() as u64) == i as u64;
                let c = if active { theme.selected } else { theme.dim };
                let em = if active { 0.7 } else { 0.15 };
                ui_render::text(engine, name, x, chain_y, c, 0.22, em);
                // Connection arrow
                if i < engines.len() - 1 {
                    ui_render::text(engine, ">", x + 0.6, chain_y, theme.muted, 0.18, 0.1);
                }
            }
        }
        2 => {
            // Combat animation: oscillating damage number
            let dmg_val = ((frame as f32 * 0.08).sin() * 50.0 + 50.0) as i32;
            let dmg_text = format!("{}", dmg_val);
            let dmg_alpha = ((frame as f32 * 0.06).sin() * 0.3 + 0.7).max(0.0);
            ui_render::text_centered(engine, &dmg_text, -3.0,
                Vec4::new(theme.danger.x * dmg_alpha, theme.danger.y * dmg_alpha, 0.0, dmg_alpha),
                0.4, dmg_alpha * 0.6);
        }
        _ => {}
    }

    // ── Navigation hints ──
    let nav = if slide_idx == 0 {
        "[Right/Space/Enter] Next  [Esc] Back"
    } else if slide_idx == SLIDES.len() - 1 {
        "[Left] Previous  [Esc] Done"
    } else {
        "[Left/Right/Space/Enter] Navigate  [Esc] Back"
    };
    ui_render::small(engine, nav, -6.0, -5.2, theme.muted);
}
