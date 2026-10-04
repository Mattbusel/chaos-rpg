//! Boss-specific combat visual treatments.
//!
//! Each of the 12 unique bosses gets a distinct visual signature
//! that uses different engine capabilities.

use proof_engine::prelude::*;
use crate::state::GameState;
use crate::theme::THEMES;

/// Render boss-specific visual overlay during combat.
pub fn render_boss_overlay(state: &GameState, engine: &mut ProofEngine) {
    let boss_id = match state.boss_id {
        Some(id) => id,
        None => return,
    };
    let theme = &THEMES[state.theme_idx % THEMES.len()];
    let frame = state.frame;
    let turn = state.boss_turn;

    match boss_id {
        // ── Boss 1: THE MIRROR ──
        // Vertical symmetry line bisects arena
        1 => {
            for y_i in 0..30 {
                let y = -15.0 + y_i as f32;
                let pulse = ((frame as f32 * 0.1 + y_i as f32 * 0.2).sin() * 0.3 + 0.7).max(0.0);
                engine.spawn_glyph(Glyph {
                    character: '│',
                    position: Vec3::new(0.0, y, 0.0),
                    color: Vec4::new(0.5 * pulse, 0.8 * pulse, 1.0 * pulse, 0.6),
                    emission: pulse * 0.5,
                    layer: RenderLayer::Overlay,
                    ..Default::default()
                });
            }
        }

        // ── Boss 2: THE ACCOUNTANT ──
        // Gold coin particles streaming from player to enemy
        2 => {
            let coins_per_turn = (turn as f32 * 0.5).min(8.0) as usize;
            for i in 0..coins_per_turn {
                let t = (frame as f32 * 0.03 + i as f32 * 0.4) % 1.0;
                let x = -8.0 + t * 16.0; // player side to enemy side
                let y = (t * std::f32::consts::PI).sin() * 2.0 + 5.0;
                engine.spawn_glyph(Glyph {
                    character: '◉',
                    position: Vec3::new(x, y, 0.0),
                    color: Vec4::new(1.0, 0.88, 0.0, 0.8),
                    emission: 0.8,
                    glow_color: Vec3::new(1.0, 0.8, 0.0),
                    glow_radius: 0.5,
                    layer: RenderLayer::Particle,
                    ..Default::default()
                });
            }
        }

        // ── Boss 3: FIBONACCI HYDRA ──
        // Golden spiral pattern on arena floor
        3 => {
            let phi: f32 = 1.618_034;
            for i in 0..40 {
                let angle = i as f32 * 2.399963; // golden angle in radians
                let r = (i as f32).sqrt() * 1.2;
                let x = angle.cos() * r;
                let y = angle.sin() * r * 0.5;
                let fade = 1.0 - (i as f32 / 40.0);
                engine.spawn_glyph(Glyph {
                    character: if i % 3 == 0 { 'φ' } else { '·' },
                    position: Vec3::new(x, y - 5.0, -1.0),
                    color: Vec4::new(1.0 * fade, 0.85 * fade, 0.2 * fade, fade * 0.5),
                    emission: fade * 0.3,
                    layer: RenderLayer::World,
                    ..Default::default()
                });
            }
        }

        // ── Boss 4: THE EIGENSTATE ──
        // Rapid alternation between two visual states
        4 => {
            let is_large = (frame / 8).is_multiple_of(2); // fast square wave
            let size_label = if is_large { "██ 10,000 HP ██" } else { "· 1 HP ·" };
            let color = if is_large {
                Vec4::new(1.0, 0.3, 0.3, 1.0)
            } else {
                Vec4::new(0.3, 0.3, 0.3, 0.5)
            };
            render_text(engine, size_label, 4.0, 4.0, color, if is_large { 1.0 } else { 0.2 });

            // Static noise particles around enemy
            for i in 0..12 {
                let seed_f = i as f32 * 31.7 + frame as f32 * 0.5;
                let x = 8.0 + seed_f.sin() * 3.0;
                let y = 2.0 + seed_f.cos() * 2.0;
                engine.spawn_glyph(Glyph {
                    character: if (frame + i as u64).is_multiple_of(3) { '░' } else { '▒' },
                    position: Vec3::new(x, y, 0.0),
                    color: Vec4::new(0.4, 0.4, 0.5, 0.4),
                    emission: 0.2,
                    layer: RenderLayer::Particle,
                    ..Default::default()
                });
            }
        }

        // ── Boss 5: THE TAXMAN ──
        // Gold particle drain, increasing speed
        5 => {
            let drain_speed = 1.0 + turn as f32 * 0.3;
            for i in 0..6 {
                let t = ((frame as f32 * 0.02 * drain_speed + i as f32 * 0.3) % 1.0);
                let x = -6.0 + t * 12.0;
                let y = 6.0 - t * 2.0;
                engine.spawn_glyph(Glyph {
                    character: '$',
                    position: Vec3::new(x, y, 0.0),
                    color: Vec4::new(1.0, 0.85, 0.0, 1.0 - t),
                    emission: (1.0 - t) * 0.8,
                    layer: RenderLayer::Particle,
                    ..Default::default()
                });
            }
        }

        // ── Boss 6: THE NULL ──
        // Progressive visual stripping — effects reduce each turn
        6 => {
            let null_progress = (turn as f32 / 10.0).min(1.0);
            // Void zone: glyphs cannot exist near enemy
            let void_radius = 3.0 + null_progress * 5.0;
            // Render absence — dark holes
            for i in 0..16 {
                let angle = (i as f32 / 16.0) * std::f32::consts::TAU;
                let r = void_radius * 0.8;
                let x = 8.0 + angle.cos() * r;
                let y = 2.0 + angle.sin() * r * 0.5;
                engine.spawn_glyph(Glyph {
                    character: ' ',
                    position: Vec3::new(x, y, 0.5),
                    color: Vec4::new(0.0, 0.0, 0.0, null_progress),
                    emission: 0.0,
                    layer: RenderLayer::Overlay,
                    ..Default::default()
                });
            }
            // Label showing what's been stripped
            if turn >= 3 {
                render_text(engine, "[ bloom disabled ]", 2.0, -6.0,
                    Vec4::new(0.3, 0.3, 0.3, 0.5), 0.15);
            }
            if turn >= 5 {
                render_text(engine, "[ particles stopped ]", 2.0, -7.0,
                    Vec4::new(0.25, 0.25, 0.25, 0.4), 0.1);
            }
            if turn >= 7 {
                render_text(engine, "[ emission: 0 ]", 2.0, -8.0,
                    Vec4::new(0.2, 0.2, 0.2, 0.3), 0.05);
            }
        }

        // ── Boss 7: OUROBOROS ──
        // Circular particle ring filling as cycle approaches heal
        7 => {
            let cycle_progress = (turn % 3) as f32 / 3.0;
            let ring_points = 24;
            for i in 0..ring_points {
                let frac = i as f32 / ring_points as f32;
                let angle = frac * std::f32::consts::TAU;
                let r = 4.0;
                let x = 8.0 + angle.cos() * r;
                let y = 2.0 + angle.sin() * r * 0.5;
                let filled = frac <= cycle_progress;
                let color = if filled {
                    Vec4::new(0.2, 0.9, 0.3, 0.8) // green = heal approaching
                } else {
                    Vec4::new(0.3, 0.3, 0.3, 0.3)
                };
                engine.spawn_glyph(Glyph {
                    character: if filled { '█' } else { '░' },
                    position: Vec3::new(x, y, 0.0),
                    color,
                    emission: if filled { 0.6 } else { 0.1 },
                    layer: RenderLayer::Overlay,
                    ..Default::default()
                });
            }
        }

        // ── Boss 8: COLLATZ TITAN ──
        // Live Collatz sequence display
        8 => {
            if let Some(ref enemy) = state.enemy {
                let hp = enemy.hp.max(1);
                let mut seq = Vec::new();
                let mut n = hp;
                for _ in 0..5 {
                    seq.push(n);
                    if n <= 1 { break; }
                    n = if n % 2 == 0 { n / 2 } else { n * 3 + 1 };
                }
                let seq_str: String = seq.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" → ");
                let truncated: String = seq_str.chars().take(50).collect();
                render_text(engine, &truncated, 1.0, -4.0, theme.accent, 0.5);

                // Flash warning on odd values
                if hp % 2 != 0 {
                    let flash = ((frame as f32 * 0.2).sin() * 0.5 + 0.5).max(0.0);
                    render_text(engine, "▲ ODD — value will INCREASE", 1.0, -5.5,
                        Vec4::new(1.0, flash * 0.5, 0.0, flash), flash * 0.7);
                }
            }
        }

        // ── Boss 9: THE COMMITTEE ──
        // 5 judge indicators
        9 => {
            let judges = 5;
            for j in 0..judges {
                let x = 3.0 + j as f32 * 2.5;
                let y = 6.0;
                // Alternate approved/denied based on frame seed
                let approved = !((frame / 15 + j as u64) * 7919).is_multiple_of(3);
                let (ch, color) = if approved {
                    ('✓', Vec4::new(0.2, 0.9, 0.3, 0.8))
                } else {
                    ('✗', Vec4::new(0.9, 0.2, 0.2, 0.8))
                };
                engine.spawn_glyph(Glyph {
                    character: ch,
                    position: Vec3::new(x, y, 0.0),
                    color,
                    emission: 0.6,
                    layer: RenderLayer::Overlay,
                    ..Default::default()
                });
            }
        }

        // ── Boss 10: THE RECURSION ──
        // Stack visualization bar on right edge
        10 => {
            let stack_height = turn.min(20) as usize;
            for i in 0..stack_height {
                let is_player = i % 2 == 0;
                let color = if is_player {
                    Vec4::new(0.3, 0.5, 1.0, 0.8)
                } else {
                    Vec4::new(1.0, 0.3, 0.2, 0.8)
                };
                engine.spawn_glyph(Glyph {
                    character: '█',
                    position: Vec3::new(18.0, -10.0 + i as f32 * 0.8, 0.0),
                    color,
                    emission: 0.4,
                    layer: RenderLayer::Overlay,
                    ..Default::default()
                });
            }
            render_text(engine, "STACK", 17.0, -11.5, theme.muted, 0.2);
        }

        // ── Boss 11: THE PARADOX ──
        // Full hue inversion effect — render inverted color label
        11 => {
            let invert_label = "[ REALITY INVERTED ]";
            let pulse = ((frame as f32 * 0.1).sin() * 0.3 + 0.7).max(0.0);
            // Invert theme colors for the label
            let inv_color = Vec4::new(
                1.0 - theme.heading.x,
                1.0 - theme.heading.y,
                1.0 - theme.heading.z,
                pulse,
            );
            render_text_centered(engine, invert_label, 9.0, inv_color, pulse * 0.8);
            render_text_centered(engine, "HP grows when you take damage", 8.0,
                Vec4::new(0.6, 0.2, 0.2, 0.5), 0.3);
        }

        // ── Boss 12: THE ALGORITHM REBORN ──
        // 3-phase visual escalation
        12 => {
            let phase = if turn < 5 { 1 } else if turn < 10 { 2 } else { 3 };

            match phase {
                1 => {
                    // Phase 1: Evaluation — player name spelled across background
                    if let Some(ref player) = state.player {
                        let name_chars: Vec<char> = player.name.chars().collect();
                        for (i, &ch) in name_chars.iter().enumerate() {
                            let t = (frame as f32 * 0.01 + i as f32 * 0.5) % 30.0;
                            let x = -15.0 + t;
                            let y = ((i as f32 * 2.7 + frame as f32 * 0.005).sin()) * 8.0;
                            engine.spawn_glyph(Glyph {
                                character: ch,
                                position: Vec3::new(x, y, -2.0),
                                color: Vec4::new(0.5, 0.3, 0.8, 0.4),
                                emission: 0.3,
                                layer: RenderLayer::Background,
                                ..Default::default()
                            });
                        }
                    }
                    render_text(engine, "EVALUATING...", 2.0, -5.0, theme.accent, 0.4);
                }
                2 => {
                    // Phase 2: Adaptation — mathematical symbols orbit
                    let symbols = ['∑', '∫', '∏', 'Ω', '∂', 'λ', 'π', 'φ'];
                    for (i, &sym) in symbols.iter().enumerate() {
                        let angle = (frame as f32 * 0.02 + i as f32 * std::f32::consts::TAU / 8.0);
                        let r = 12.0;
                        let x = angle.cos() * r;
                        let y = angle.sin() * r * 0.4;
                        engine.spawn_glyph(Glyph {
                            character: sym,
                            position: Vec3::new(x, y, 0.0),
                            color: Vec4::new(0.7, 0.4, 1.0, 0.7),
                            emission: 0.6,
                            glow_color: Vec3::new(0.5, 0.2, 0.8),
                            glow_radius: 1.5,
                            layer: RenderLayer::Overlay,
                            ..Default::default()
                        });
                    }
                    render_text(engine, "ADAPTING TO YOUR STRATEGY...", 0.0, -5.0,
                        Vec4::new(0.8, 0.4, 1.0, 0.8), 0.6);
                }
                _ => {
                    // Phase 3: Counter-specialization — "I SEE YOU" + full chaos
                    let see_you = "I  S E E  Y O U";
                    let pulse = ((frame as f32 * 0.15).sin() * 0.4 + 0.6).max(0.0);
                    render_text_centered(engine, see_you, 9.5,
                        Vec4::new(1.0, 0.2 * pulse, 0.8 * pulse, pulse), pulse * 1.5);

                    // Glitch static across screen
                    for i in 0..30 {
                        let seed_f = i as f32 * 97.3 + frame as f32 * 0.7;
                        let x = seed_f.sin() * 20.0;
                        let y = seed_f.cos() * 12.0;
                        let glitch_chars = ['█', '▓', '▒', '░', '#', '!', '?'];
                        engine.spawn_glyph(Glyph {
                            character: glitch_chars[(frame as usize + i) % glitch_chars.len()],
                            position: Vec3::new(x, y, 0.5),
                            color: Vec4::new(
                                (seed_f * 0.3).sin().abs(),
                                (seed_f * 0.7).cos().abs() * 0.3,
                                (seed_f * 1.1).sin().abs(),
                                0.3,
                            ),
                            emission: 0.4,
                            layer: RenderLayer::Overlay,
                            ..Default::default()
                        });
                    }
                }
            }
        }

        _ => {} // Unknown boss — no special overlay
    }
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

fn render_text_centered(engine: &mut ProofEngine, text: &str, y: f32, color: Vec4, emission: f32) {
    let x = -(text.len() as f32 * 0.225);
    render_text(engine, text, x, y, color, emission);
}
