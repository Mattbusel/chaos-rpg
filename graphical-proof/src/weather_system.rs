//! Weather system — per-floor atmospheric effects.
//!
//! Maps floor depth to weather type: digital rain, compute pulses, static,
//! ash storms, electrical storms, void snow. Boss room overrides.

use proof_engine::prelude::*;
use crate::state::GameState;
use crate::theme::THEMES;

// ═══════════════════════════════════════════════════════════════════════════════
// WEATHER TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeatherType {
    Clear,
    DigitalRain,      // Floors 1-10: subtle green particles falling
    ComputePulse,     // Floors 11-25: bright horizontal particle sweep
    StaticNoise,      // Floors 26-50: random white particles flashing
    AshStorm,         // Floors 51-75: ash falling diagonally, fog
    ElectricalStorm,  // Floors 76-99: lightning flashes, sparks
    VoidSnow,         // Floor 100+: sparse slow white particles
    BossNull,         // The Null: absolute stillness, no weather
    BossAlgorithm,    // Algorithm Reborn: phase-dependent
    GoldenRain,       // Fibonacci Hydra: gold particles
}

/// Get the weather type for a floor, optionally overridden by boss.
pub fn floor_weather(floor: u32, boss_id: Option<u8>, boss_turn: u32) -> WeatherType {
    // Boss overrides
    if let Some(id) = boss_id {
        match id {
            6 => return WeatherType::BossNull,
            12 => {
                if boss_turn < 5 { return WeatherType::VoidSnow; }
                if boss_turn < 10 { return WeatherType::ElectricalStorm; }
                return WeatherType::BossAlgorithm;
            }
            3 => return WeatherType::GoldenRain,
            _ => {} // Other bosses use floor weather
        }
    }

    match floor {
        0..=10  => WeatherType::DigitalRain,
        11..=25 => WeatherType::ComputePulse,
        26..=50 => WeatherType::StaticNoise,
        51..=75 => WeatherType::AshStorm,
        76..=99 => WeatherType::ElectricalStorm,
        _       => WeatherType::VoidSnow,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEATHER STATE
// ═══════════════════════════════════════════════════════════════════════════════

pub struct WeatherState {
    pub weather_type: WeatherType,
    pub time: f32,
    pub lightning_timer: f32,    // countdown to next lightning flash
    pub lightning_active: f32,   // remaining flash duration
    pub wind_angle: f32,         // current wind direction
    pub fog_density: f32,
}

impl WeatherState {
    pub fn new() -> Self {
        Self {
            weather_type: WeatherType::Clear,
            time: 0.0,
            lightning_timer: 8.0,
            lightning_active: 0.0,
            wind_angle: 0.3,
            fog_density: 0.0,
        }
    }

    /// Update weather state and set type based on floor.
    pub fn update(&mut self, state: &GameState, dt: f32) {
        self.time += dt;
        self.weather_type = floor_weather(state.floor_num, state.boss_id, state.boss_turn);

        // Wind slowly shifts
        self.wind_angle += dt * 0.05;

        // Fog density by floor
        self.fog_density = match state.floor_num {
            0..=25 => 0.0,
            26..=50 => 0.1,
            51..=75 => 0.3,
            76..=99 => 0.5,
            _ => 0.2, // Void: less fog, more emptiness
        };

        // Lightning timer for electrical storms
        if self.weather_type == WeatherType::ElectricalStorm || self.weather_type == WeatherType::BossAlgorithm {
            if self.lightning_active > 0.0 {
                self.lightning_active -= dt;
            } else {
                self.lightning_timer -= dt;
                if self.lightning_timer <= 0.0 {
                    self.lightning_active = 0.05; // 3 frames at 60fps
                    // Randomize next lightning (5-15 seconds)
                    let hash = (self.time * 12345.6).sin().abs();
                    self.lightning_timer = 5.0 + hash * 10.0;
                }
            }
        }
    }

    /// Render weather particles.
    pub fn render(&self, engine: &mut ProofEngine, state: &GameState) {
        let theme = &THEMES[state.theme_idx % THEMES.len()];
        let frame = state.frame;

        match self.weather_type {
            WeatherType::Clear | WeatherType::BossNull => {}

            WeatherType::DigitalRain => {
                // Subtle green particles falling
                let count = 30;
                for i in 0..count {
                    let seed_f = i as f32 * 47.3;
                    let x = (seed_f.sin() * 20.0 + self.time * 0.5) % 40.0 - 20.0;
                    let y = (10.0 - (self.time * 2.0 + i as f32 * 1.3) % 24.0);
                    let fade = 0.15 + (seed_f * 0.7).sin().abs() * 0.1;
                    let chars = ['0', '1', '0', '1', '+', '-', '='];
                    engine.spawn_glyph(Glyph {
                        character: chars[i % chars.len()],
                        position: Vec3::new(x, y, -2.0),
                        color: Vec4::new(0.0, fade, 0.0, fade * 0.6),
                        emission: fade * 0.3,
                        layer: RenderLayer::Background,
                        ..Default::default()
                    });
                }
            }

            WeatherType::ComputePulse => {
                // Bright horizontal sweep every few seconds
                let pulse_period = 4.0;
                let pulse_phase = (self.time % pulse_period) / pulse_period;
                if pulse_phase < 0.1 {
                    let sweep_x = -20.0 + pulse_phase * 400.0;
                    for i in 0..5 {
                        let y = (i as f32 - 2.0) * 3.0;
                        let fade = (0.1 - pulse_phase) * 10.0;
                        engine.spawn_glyph(Glyph {
                            character: '─',
                            position: Vec3::new(sweep_x, y, -1.0),
                            color: Vec4::new(0.3 * fade, 0.8 * fade, 1.0 * fade, fade * 0.5),
                            emission: fade * 0.8,
                            layer: RenderLayer::Background,
                            ..Default::default()
                        });
                    }
                }
            }

            WeatherType::StaticNoise => {
                // Random white particles flashing
                let count = 15;
                for i in 0..count {
                    let hash = (frame.wrapping_add(i * 7919) as f32 * 0.001).sin().abs();
                    if hash > 0.7 { continue; } // Only some flash each frame
                    let x = (hash * 54321.0).sin() * 18.0;
                    let y = (hash * 12345.0).cos() * 10.0;
                    let brightness = hash * 0.3;
                    engine.spawn_glyph(Glyph {
                        character: '·',
                        position: Vec3::new(x, y, -1.5),
                        color: Vec4::new(brightness, brightness, brightness, brightness),
                        emission: brightness * 0.5,
                        layer: RenderLayer::Background,
                        ..Default::default()
                    });
                }
            }

            WeatherType::AshStorm => {
                // Ash particles falling diagonally with wind
                let wind_dx = self.wind_angle.cos() * 0.3;
                let count = 40;
                for i in 0..count {
                    let seed_f = i as f32 * 83.7;
                    let base_x = (seed_f.sin() * 22.0);
                    let x = base_x + (self.time * wind_dx * 3.0 + i as f32 * 0.5) % 44.0 - 22.0;
                    let y = 12.0 - (self.time * 1.5 + i as f32 * 0.8) % 26.0;
                    let fade = 0.15;
                    let chars = ['·', ',', '.', '\''];
                    engine.spawn_glyph(Glyph {
                        character: chars[i % chars.len()],
                        position: Vec3::new(x, y, -1.0),
                        color: Vec4::new(fade, fade * 0.9, fade * 0.8, fade * 0.5),
                        emission: fade * 0.1,
                        layer: RenderLayer::Background,
                        ..Default::default()
                    });
                }
                // Fog overlay
                if self.fog_density > 0.0 {
                    for i in 0..8 {
                        let x = (i as f32 - 4.0) * 5.0;
                        let fog_alpha = self.fog_density * 0.15;
                        engine.spawn_glyph(Glyph {
                            character: '░',
                            position: Vec3::new(x, -5.0, -0.5),
                            color: Vec4::new(fog_alpha, fog_alpha, fog_alpha, fog_alpha),
                            emission: 0.0,
                            layer: RenderLayer::Background,
                            ..Default::default()
                        });
                    }
                }
            }

            WeatherType::ElectricalStorm => {
                // Spark particles + lightning flashes
                let count = 10;
                for i in 0..count {
                    let seed_f = i as f32 * 31.3 + frame as f32 * 0.3;
                    let x = seed_f.sin() * 16.0;
                    let y = seed_f.cos() * 9.0;
                    let spark = (seed_f * 7.1 + self.time).sin().abs() * 0.3;
                    engine.spawn_glyph(Glyph {
                        character: if (frame + i) % 4 == 0 { '⚡' } else { '·' },
                        position: Vec3::new(x, y, -1.0),
                        color: Vec4::new(1.0 * spark, 0.9 * spark, 0.3 * spark, spark),
                        emission: spark,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
                // Lightning flash
                if self.lightning_active > 0.0 {
                    let flash = (self.lightning_active / 0.05).min(1.0);
                    engine.spawn_glyph(Glyph {
                        character: '█',
                        position: Vec3::new(0.0, 0.0, 5.0),
                        color: Vec4::new(flash, flash, flash * 0.9, flash * 0.3),
                        emission: flash * 3.0,
                        glow_radius: 50.0,
                        glow_color: Vec3::ONE,
                        layer: RenderLayer::Overlay,
                        ..Default::default()
                    });
                    engine.add_trauma(flash * 0.15);
                }
            }

            WeatherType::VoidSnow => {
                // Extremely sparse, slow white particles
                let count = 8;
                for i in 0..count {
                    let seed_f = i as f32 * 97.1;
                    let x = (seed_f.sin() * 18.0 + self.time * 0.1) % 36.0 - 18.0;
                    let y = 10.0 - (self.time * 0.3 + i as f32 * 3.0) % 22.0;
                    engine.spawn_glyph(Glyph {
                        character: '·',
                        position: Vec3::new(x, y, -1.0),
                        color: Vec4::new(0.2, 0.2, 0.25, 0.3),
                        emission: 0.05,
                        layer: RenderLayer::Background,
                        ..Default::default()
                    });
                }
            }

            WeatherType::BossAlgorithm => {
                // Phase 3: all weather simultaneously
                // Digital rain
                for i in 0..10 {
                    let x = (i as f32 * 47.3).sin() * 18.0;
                    let y = 10.0 - (self.time * 3.0 + i as f32 * 2.0) % 22.0;
                    engine.spawn_glyph(Glyph {
                        character: ['0', '1', '∑', '∂'][i % 4],
                        position: Vec3::new(x, y, -2.0),
                        color: Vec4::new(0.0, 0.3, 0.0, 0.2),
                        emission: 0.1,
                        layer: RenderLayer::Background, ..Default::default()
                    });
                }
                // Sparks
                for i in 0..8 {
                    let sf = i as f32 * 31.3 + frame as f32 * 0.5;
                    let x = sf.sin() * 14.0;
                    let y = sf.cos() * 8.0;
                    engine.spawn_glyph(Glyph {
                        character: '⚡',
                        position: Vec3::new(x, y, -1.0),
                        color: Vec4::new(0.8, 0.7, 0.2, 0.4),
                        emission: 0.5,
                        layer: RenderLayer::Particle, ..Default::default()
                    });
                }
                // Lightning
                if self.lightning_active > 0.0 {
                    engine.add_trauma(0.2);
                }
            }

            WeatherType::GoldenRain => {
                // Gold particles falling — intensity increases (simulated by count)
                let count = 25;
                for i in 0..count {
                    let seed_f = i as f32 * 61.7;
                    let x = (seed_f.sin() * 20.0 + self.time * 0.2) % 40.0 - 20.0;
                    let y = 12.0 - (self.time * 1.8 + i as f32 * 1.1) % 26.0;
                    let gold = 0.2 + (seed_f * 0.3).sin().abs() * 0.1;
                    engine.spawn_glyph(Glyph {
                        character: ['·', '◆', '★', '✦'][i % 4],
                        position: Vec3::new(x, y, -1.0),
                        color: Vec4::new(gold, gold * 0.85, 0.0, gold * 0.6),
                        emission: gold * 0.4,
                        layer: RenderLayer::Particle,
                        ..Default::default()
                    });
                }
            }
        }
    }
}
