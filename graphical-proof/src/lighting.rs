//! PBR Lighting System for CHAOS RPG
//!
//! Maps game state (room type, floor depth, combat events, boss phase)
//! to proof-engine's PBR lighting system: point lights, spot lights,
//! area lights, animated lights, ambient, and atmospheric scattering.

use proof_engine::prelude::*;
use proof_engine::render::lighting::{
    PointLight, SpotLight, DirectionalLight, AmbientLight, LightId,
    LightManager, LightAnimation, AnimatedPointLight, RectLight,
    Attenuation,
};
use crate::state::GameState;
use crate::theme::THEMES;

// ═══════════════════════════════════════════════════════════════════════════════
// ROOM LIGHTING PRESETS
// ═══════════════════════════════════════════════════════════════════════════════

/// Room type determines the base lighting setup.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoomLighting {
    Combat,
    Shop,
    Shrine,
    ChaosRift,
    Boss,
    Treasure,
    Crafting,
    Corridor,
    Empty,
}

/// Full lighting state for the current scene.
pub struct SceneLighting {
    pub manager: LightManager,
    pub animated: Vec<AnimatedPointLight>,
    /// Transient combat flash lights (auto-expire).
    pub flashes: Vec<LightFlash>,
    /// Per-entity follow-lights.
    pub player_light: Option<LightId>,
    pub enemy_light: Option<LightId>,
    /// Status effect lights on entities.
    pub status_lights: Vec<StatusLight>,
    /// Time accumulator for animations.
    pub time: f32,
}

/// A brief flash light (attack impact, crit, spell).
pub struct LightFlash {
    pub light: PointLight,
    pub start_intensity: f32,
    pub duration: f32,
    pub elapsed: f32,
}

/// A persistent status effect light on an entity.
pub struct StatusLight {
    pub light_id: LightId,
    pub animation: LightAnimation,
    pub base_intensity: f32,
    pub status_type: u8, // bitmask: 1=burn, 2=freeze, 4=poison, 16=stun
}

impl SceneLighting {
    pub fn new() -> Self {
        Self {
            manager: LightManager::new(),
            animated: Vec::new(),
            flashes: Vec::new(),
            player_light: None,
            enemy_light: None,
            status_lights: Vec::new(),
            time: 0.0,
        }
    }

    // ── Room presets ──────────────────────────────────────────────────────────

    /// Configure lighting for a room type at the given floor depth.
    pub fn setup_room(&mut self, room: RoomLighting, floor: u32, center: Vec3) {
        self.manager = LightManager::new();
        self.animated.clear();
        self.flashes.clear();
        self.status_lights.clear();

        // Floor-depth ambient scaling
        let ambient_level = floor_ambient(floor);
        let ambient_color = floor_ambient_color(floor);

        match room {
            RoomLighting::Combat => {
                // Dim ambient + red-tinted overhead
                self.manager.ambient = AmbientLight::uniform(
                    ambient_color, ambient_level,
                );
                let overhead = PointLight::new(
                    center + Vec3::new(0.0, 8.0, 0.0),
                    Vec3::new(1.0, 0.2, 0.1), // red tint
                    3.0 * ambient_level.max(0.3),
                    20.0,
                ).with_shadow().with_tag("room");
                self.manager.add_point_light(overhead);
            }

            RoomLighting::Shop => {
                // Warm torch lights flanking NPC
                self.manager.ambient = AmbientLight::uniform(
                    Vec3::new(0.15, 0.12, 0.08), ambient_level * 1.5,
                );
                // Left torch
                let left = PointLight::new(
                    center + Vec3::new(-4.0, 3.0, 0.0),
                    Vec3::new(1.0, 0.7, 0.3),
                    4.0,
                    10.0,
                ).with_tag("torch");
                self.animated.push(AnimatedPointLight::new(
                    left, LightAnimation::Flicker { speed: 3.0, depth: 0.3 },
                ));
                // Right torch
                let right = PointLight::new(
                    center + Vec3::new(4.0, 3.0, 0.0),
                    Vec3::new(1.0, 0.7, 0.3),
                    4.0,
                    10.0,
                ).with_tag("torch");
                self.animated.push(AnimatedPointLight::new(
                    right, LightAnimation::Flicker { speed: 2.7, depth: 0.25 },
                ));
            }

            RoomLighting::Shrine => {
                // Bright blue-white overhead area light, SSAO shadows
                self.manager.ambient = AmbientLight::uniform(
                    Vec3::new(0.1, 0.15, 0.25), ambient_level * 2.0,
                );
                let shrine_light = PointLight::new(
                    center + Vec3::new(0.0, 6.0, 0.0),
                    Vec3::new(0.7, 0.85, 1.0), // cool blue-white
                    6.0,
                    15.0,
                ).with_shadow().with_tag("shrine");
                self.manager.add_point_light(shrine_light);
            }

            RoomLighting::ChaosRift => {
                // Multiple rapidly flickering random-color lights
                self.manager.ambient = AmbientLight::uniform(
                    Vec3::new(0.05, 0.02, 0.08), ambient_level * 0.5,
                );
                let rift_colors = [
                    Vec3::new(1.0, 0.0, 0.5),
                    Vec3::new(0.0, 1.0, 0.8),
                    Vec3::new(0.8, 0.0, 1.0),
                    Vec3::new(1.0, 0.5, 0.0),
                    Vec3::new(0.0, 0.5, 1.0),
                ];
                for (i, color) in rift_colors.iter().enumerate() {
                    let angle = (i as f32 / 5.0) * std::f32::consts::TAU;
                    let pos = center + Vec3::new(angle.cos() * 6.0, 2.0, angle.sin() * 6.0);
                    let light = PointLight::new(pos, *color, 3.0, 8.0).with_tag("rift");
                    self.animated.push(AnimatedPointLight::new(
                        light,
                        LightAnimation::Strobe { frequency: 3.0 + i as f32 * 0.7 },
                    ));
                }
            }

            RoomLighting::Boss => {
                // Dramatic single spotlight on boss, near-zero ambient
                self.manager.ambient = AmbientLight::uniform(
                    Vec3::new(0.01, 0.01, 0.02), ambient_level * 0.2,
                );
                let spotlight = SpotLight {
                    id: LightId::invalid(),
                    position: center + Vec3::new(0.0, 12.0, 0.0),
                    direction: Vec3::new(0.0, -1.0, 0.0),
                    color: Vec3::new(1.0, 0.95, 0.9),
                    intensity: 8.0,
                    range: 25.0,
                    inner_angle: 0.3,
                    outer_angle: 0.6,
                    attenuation: Attenuation::WindowedInverseSquare { range: 25.0 },
                    cast_shadow: true,
                    enabled: true,
                    tag: Some("boss_spot".to_string()),
                };
                self.manager.add_spot_light(spotlight);
            }

            RoomLighting::Treasure => {
                // Golden point light from treasure
                self.manager.ambient = AmbientLight::uniform(
                    Vec3::new(0.1, 0.08, 0.03), ambient_level,
                );
                let gold = PointLight::new(
                    center,
                    Vec3::new(1.0, 0.85, 0.3), // warm gold
                    5.0,
                    12.0,
                ).with_tag("treasure");
                self.animated.push(AnimatedPointLight::new(
                    gold,
                    LightAnimation::Pulse {
                        frequency: 0.5,
                        min_intensity: 3.0,
                        max_intensity: 6.0,
                    },
                ));
            }

            RoomLighting::Crafting => {
                // Amber work light above bench (rect area light approximation)
                self.manager.ambient = AmbientLight::uniform(
                    Vec3::new(0.12, 0.09, 0.04), ambient_level * 1.2,
                );
                let work_light = PointLight::new(
                    center + Vec3::new(0.0, 4.0, 0.0),
                    Vec3::new(1.0, 0.8, 0.4), // amber
                    5.0,
                    10.0,
                ).with_shadow().with_tag("bench");
                self.manager.add_point_light(work_light);
            }

            RoomLighting::Corridor | RoomLighting::Empty => {
                // Dim moonlight, long shadows
                self.manager.ambient = AmbientLight::uniform(
                    Vec3::new(0.04, 0.05, 0.08), ambient_level * 0.6,
                );
                self.manager.set_directional(DirectionalLight::sun(
                    Vec3::new(-0.2, -0.8, -0.5),
                    Vec3::new(0.5, 0.55, 0.8),
                    0.4 * ambient_level.max(0.1),
                ));
            }
        }
    }

    // ── Combat entity lights ─────────────────────────────────────────────────

    /// Add the player's carry-light (warm, follows entity).
    pub fn add_player_light(&mut self, position: Vec3) {
        let light = PointLight::new(
            position + Vec3::new(0.0, 1.5, 0.0),
            Vec3::new(1.0, 0.9, 0.7), // warm
            3.0,
            10.0,
        ).with_tag("player");
        let id = self.manager.add_point_light(light);
        self.player_light = Some(id);
    }

    /// Add the enemy's light (cold blue, or element-tinted).
    pub fn add_enemy_light(&mut self, position: Vec3, element_tint: Option<Vec3>) {
        let color = element_tint.unwrap_or(Vec3::new(0.3, 0.4, 1.0)); // default cold blue
        let light = PointLight::new(
            position + Vec3::new(0.0, 1.5, 0.0),
            color,
            2.5,
            8.0,
        ).with_tag("enemy");
        let id = self.manager.add_point_light(light);
        self.enemy_light = Some(id);
    }

    /// Update entity light positions (call each frame).
    pub fn update_entity_positions(&mut self, player_pos: Vec3, enemy_pos: Vec3) {
        if let Some(id) = self.player_light {
            if let Some(light) = self.manager.get_point_light_mut(id) {
                light.position = player_pos + Vec3::new(0.0, 1.5, 0.0);
            }
        }
        if let Some(id) = self.enemy_light {
            if let Some(light) = self.manager.get_point_light_mut(id) {
                light.position = enemy_pos + Vec3::new(0.0, 1.5, 0.0);
            }
        }
    }

    // ── Combat flash effects ─────────────────────────────────────────────────

    /// Brief bright flash at impact point (normal attack).
    pub fn flash_attack(&mut self, position: Vec3, intensity: f32) {
        self.flashes.push(LightFlash {
            light: PointLight::new(
                position,
                Vec3::new(1.0, 0.9, 0.7),
                intensity,
                6.0,
            ),
            start_intensity: intensity,
            duration: 0.15,
            elapsed: 0.0,
        });
    }

    /// Larger, brighter area flash (critical hit).
    pub fn flash_crit(&mut self, position: Vec3) {
        self.flashes.push(LightFlash {
            light: PointLight::new(
                position,
                Vec3::new(1.0, 0.8, 0.2),
                12.0,
                15.0,
            ),
            start_intensity: 12.0,
            duration: 0.4,
            elapsed: 0.0,
        });
    }

    /// Element-colored flash for spell impact.
    pub fn flash_spell(&mut self, position: Vec3, element_color: Vec3) {
        self.flashes.push(LightFlash {
            light: PointLight::new(
                position,
                element_color,
                8.0,
                12.0,
            ),
            start_intensity: 8.0,
            duration: 0.5,
            elapsed: 0.0,
        });
    }

    /// Lightning: screen-wide directional flash for 2 frames.
    pub fn flash_lightning(&mut self) {
        self.manager.set_directional(DirectionalLight::sun(
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::ONE,
            20.0,
        ));
        // This will be cleared on next room setup or after 2 frames
        self.flashes.push(LightFlash {
            light: PointLight::new(Vec3::ZERO, Vec3::ONE, 15.0, 100.0),
            start_intensity: 15.0,
            duration: 0.033, // ~2 frames at 60fps
            elapsed: 0.0,
        });
    }

    // ── Status effect lights ─────────────────────────────────────────────────

    /// Add a persistent status effect light on an entity.
    pub fn add_status_light(&mut self, position: Vec3, status_type: u8) {
        let (color, animation) = match status_type {
            1 => (
                Vec3::new(1.0, 0.5, 0.1), // burn: orange
                LightAnimation::Flicker { speed: 4.0, depth: 0.4 },
            ),
            2 => (
                Vec3::new(0.3, 0.6, 1.0), // freeze: blue, steady
                LightAnimation::Constant,
            ),
            4 => (
                Vec3::new(0.2, 0.8, 0.3), // poison: green
                LightAnimation::Pulse { frequency: 0.8, min_intensity: 1.0, max_intensity: 3.0 },
            ),
            16 => (
                Vec3::new(1.0, 0.9, 0.2), // stun: yellow strobe
                LightAnimation::Strobe { frequency: 6.0 },
            ),
            _ => (Vec3::ONE, LightAnimation::Constant),
        };

        let light = PointLight::new(position, color, 2.5, 5.0).with_tag("status");
        let id = self.manager.add_point_light(light);
        self.status_lights.push(StatusLight {
            light_id: id,
            animation,
            base_intensity: 2.5,
            status_type,
        });
    }

    // ── Fire spell persistent path lights ────────────────────────────────────

    /// Leave a fading warm light along a fire spell's path.
    pub fn add_fire_trail_light(&mut self, position: Vec3) {
        self.flashes.push(LightFlash {
            light: PointLight::new(
                position,
                Vec3::new(1.0, 0.5, 0.1),
                3.0,
                5.0,
            ),
            start_intensity: 3.0,
            duration: 2.0, // fades over 2 seconds
            elapsed: 0.0,
        });
    }

    // ── Tick ──────────────────────────────────────────────────────────────────

    /// Advance all animated lights and expire flash lights.
    pub fn tick(&mut self, dt: f32) {
        self.time += dt;

        // Animate persistent lights
        for (i, anim_light) in self.animated.iter().enumerate() {
            let factor = anim_light.animation.intensity_factor(self.time, i as u32);
            let color = anim_light.animation.color_at(self.time, anim_light.base_color);
            // We can't easily update the manager's lights by index here without IDs,
            // so animated lights are rendered as transient glyphs with glow
        }

        // Animate status lights
        for sl in &self.status_lights {
            let factor = sl.animation.intensity_factor(self.time, sl.light_id.0);
            if let Some(light) = self.manager.get_point_light_mut(sl.light_id) {
                light.intensity = sl.base_intensity * factor;
            }
        }

        // Expire flash lights
        for flash in &mut self.flashes {
            flash.elapsed += dt;
            let t = (flash.elapsed / flash.duration).clamp(0.0, 1.0);
            flash.light.intensity = flash.start_intensity * (1.0 - t * t); // quadratic falloff
        }
        self.flashes.retain(|f| f.elapsed < f.duration);
    }

    // ── Render transient light glyphs ────────────────────────────────────────

    /// Spawn glow glyphs for all active lights (since engine renders glyphs, not raw lights).
    pub fn render_light_glyphs(&self, engine: &mut ProofEngine) {
        // Flash lights as bright glow points
        for flash in &self.flashes {
            let t = (flash.elapsed / flash.duration).clamp(0.0, 1.0);
            let intensity = flash.light.intensity;
            if intensity < 0.01 { continue; }
            engine.spawn_glyph(Glyph {
                character: '·',
                position: flash.light.position,
                color: Vec4::new(
                    flash.light.color.x,
                    flash.light.color.y,
                    flash.light.color.z,
                    (1.0 - t).max(0.0),
                ),
                emission: intensity * 0.5,
                glow_color: flash.light.color,
                glow_radius: flash.light.range * 0.3 * (1.0 - t),
                layer: RenderLayer::Particle,
                ..Default::default()
            });
        }

        // Animated lights as visible glow sources
        for (i, anim) in self.animated.iter().enumerate() {
            let factor = anim.animation.intensity_factor(self.time, i as u32);
            let color = anim.animation.color_at(self.time, anim.base_color);
            let intensity = anim.base_intensity * factor;
            if intensity < 0.01 { continue; }
            engine.spawn_glyph(Glyph {
                character: '◆',
                position: anim.light.position,
                color: Vec4::new(color.x, color.y, color.z, 0.8),
                emission: intensity * 0.4,
                glow_color: color,
                glow_radius: anim.light.range * 0.2,
                layer: RenderLayer::World,
                ..Default::default()
            });
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BOSS-SPECIFIC LIGHTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Apply boss-specific lighting modifications.
pub fn apply_boss_lighting(lighting: &mut SceneLighting, boss_id: u8, turn: u32, center: Vec3) {
    match boss_id {
        // The Mirror: symmetric lighting — add a mirrored copy of the boss spotlight
        1 => {
            let mirror_light = PointLight::new(
                center + Vec3::new(0.0, 8.0, 0.0),
                Vec3::new(0.5, 0.8, 1.0),
                4.0,
                15.0,
            ).with_tag("mirror");
            lighting.manager.add_point_light(mirror_light);
            // Add symmetric rim lights
            let left = PointLight::new(center + Vec3::new(-5.0, 3.0, 0.0), Vec3::new(0.3, 0.3, 1.0), 2.0, 10.0).with_tag("mirror");
            let right = PointLight::new(center + Vec3::new(5.0, 3.0, 0.0), Vec3::new(0.3, 0.3, 1.0), 2.0, 10.0).with_tag("mirror");
            lighting.manager.add_point_light(left);
            lighting.manager.add_point_light(right);
        }

        // The Null: progressive blackout — ambient dies, lights dim
        6 => {
            let lights_alive = 10_u32.saturating_sub(turn);
            let ambient_factor = (lights_alive as f32 / 10.0).max(0.0);
            lighting.manager.ambient = AmbientLight::uniform(
                Vec3::new(0.01, 0.01, 0.02) * ambient_factor,
                0.1 * ambient_factor,
            );
            // Dim the player and enemy lights if they exist
            if let Some(id) = lighting.player_light {
                if let Some(light) = lighting.manager.get_point_light_mut(id) {
                    light.intensity *= ambient_factor;
                }
            }
            if let Some(id) = lighting.enemy_light {
                if let Some(light) = lighting.manager.get_point_light_mut(id) {
                    light.intensity *= ambient_factor;
                }
            }
        }

        // The Algorithm Reborn Phase 3: interrogation spotlight
        12 => {
            if turn >= 10 {
                let spot = SpotLight {
                    id: LightId::invalid(),
                    position: Vec3::new(0.0, 15.0, 0.0),
                    direction: Vec3::new(0.0, -1.0, 0.0),
                    color: Vec3::ONE,
                    intensity: 15.0,
                    range: 30.0,
                    inner_angle: 0.15,
                    outer_angle: 0.3,
                    attenuation: Attenuation::WindowedInverseSquare { range: 30.0 },
                    cast_shadow: true,
                    enabled: true,
                    tag: Some("interrogation".to_string()),
                };
                lighting.manager.add_spot_light(spot);
                // Kill ambient
                lighting.manager.ambient = AmbientLight::uniform(Vec3::ZERO, 0.0);
            }
        }

        // The Accountant: cold fluorescent
        2 => {
            lighting.manager.ambient = AmbientLight::uniform(
                Vec3::new(0.8, 0.85, 0.9), 0.4,
            );
            let fluorescent = PointLight::new(
                center + Vec3::new(0.0, 5.0, 0.0),
                Vec3::new(0.9, 0.95, 1.0),
                6.0,
                20.0,
            ).with_tag("fluorescent");
            lighting.animated.push(AnimatedPointLight::new(
                fluorescent,
                LightAnimation::Flicker { speed: 8.0, depth: 0.05 }, // subtle institutional flicker
            ));
        }

        // Fibonacci Hydra: each split gets its own light
        3 => {
            let splits = (turn / 3 + 1).min(8);
            for i in 0..splits {
                let angle = (i as f32 / splits as f32) * std::f32::consts::TAU;
                let r = 3.0 + i as f32 * 0.5;
                let pos = center + Vec3::new(angle.cos() * r, 2.0, angle.sin() * r);
                let light = PointLight::new(
                    pos,
                    Vec3::new(1.0, 0.85, 0.3),
                    2.0,
                    6.0,
                ).with_tag("hydra");
                lighting.manager.add_point_light(light);
            }
        }

        _ => {} // No special lighting for other bosses
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FLOOR DEPTH HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Ambient intensity by floor depth.
fn floor_ambient(floor: u32) -> f32 {
    match floor {
        0..=10  => 0.30,
        11..=25 => 0.20,
        26..=50 => 0.10,
        51..=75 => 0.05,
        76..=99 => 0.02,
        _       => 0.005, // floor 100+: near void
    }
}

/// Ambient color tint by floor depth (warm → cold → void).
fn floor_ambient_color(floor: u32) -> Vec3 {
    match floor {
        0..=10  => Vec3::new(0.12, 0.10, 0.08),  // warm
        11..=25 => Vec3::new(0.08, 0.09, 0.12),  // cooler
        26..=50 => Vec3::new(0.05, 0.06, 0.10),  // cold
        51..=75 => Vec3::new(0.03, 0.03, 0.06),  // deep cold
        76..=99 => Vec3::new(0.01, 0.01, 0.03),  // near dark
        _       => Vec3::new(0.005, 0.003, 0.01), // void
    }
}

/// Atmospheric scattering parameters for floor 100+.
pub fn void_fog_params(floor: u32) -> Option<(f32, Vec3)> {
    if floor >= 100 {
        let depth_factor = ((floor - 100) as f32 / 50.0).min(1.0);
        let density = 0.02 + depth_factor * 0.08;
        let color = Vec3::new(
            0.02 - depth_factor * 0.015,
            0.01,
            0.03 + depth_factor * 0.02,
        );
        Some((density, color))
    } else {
        None
    }
}

/// Get the element tint color for an enemy based on name/type.
pub fn enemy_element_tint(enemy_name: &str) -> Option<Vec3> {
    let name = enemy_name.to_lowercase();
    if name.contains("fire") || name.contains("flame") || name.contains("infernal") {
        Some(Vec3::new(1.0, 0.3, 0.1)) // red-orange
    } else if name.contains("ice") || name.contains("frost") || name.contains("cryo") {
        Some(Vec3::new(0.3, 0.6, 1.0)) // ice blue
    } else if name.contains("chaos") || name.contains("void") || name.contains("entropy") {
        Some(Vec3::new(0.6, 0.1, 0.9)) // purple
    } else if name.contains("lightning") || name.contains("storm") || name.contains("thunder") {
        Some(Vec3::new(1.0, 0.9, 0.3)) // yellow
    } else if name.contains("necro") || name.contains("undead") || name.contains("death") {
        Some(Vec3::new(0.2, 0.7, 0.3)) // sickly green
    } else {
        None // default cold blue
    }
}

/// Map a spell name to its element light color.
pub fn spell_element_color(spell_name: &str) -> Vec3 {
    let name = spell_name.to_lowercase();
    if name.contains("fire") || name.contains("burn") || name.contains("blaze") || name.contains("magma") {
        Vec3::new(1.0, 0.5, 0.1)
    } else if name.contains("ice") || name.contains("frost") || name.contains("freeze") || name.contains("cryo") {
        Vec3::new(0.3, 0.7, 1.0)
    } else if name.contains("lightning") || name.contains("shock") || name.contains("thunder") || name.contains("volt") {
        Vec3::new(1.0, 0.95, 0.5)
    } else if name.contains("necro") || name.contains("death") || name.contains("drain") || name.contains("soul") {
        Vec3::new(0.3, 0.8, 0.3)
    } else if name.contains("arcane") || name.contains("chaos") || name.contains("void") {
        Vec3::new(0.6, 0.2, 1.0)
    } else if name.contains("heal") || name.contains("restore") || name.contains("divine") || name.contains("holy") {
        Vec3::new(1.0, 0.9, 0.4)
    } else {
        Vec3::new(0.8, 0.8, 1.0) // default arcane white-blue
    }
}
