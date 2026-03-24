//! Dungeon exploration visual systems.
//!
//! Rich tile-based dungeon rendering with fog of war, lighting, biome palettes,
//! minimap overlay, world-map floor navigation, and room-event animations.
//! All rendering goes through `crate::ui_render` and `engine.spawn_glyph`.

pub mod dungeon_renderer;
pub mod world_map;
pub mod room_events;

// Re-export the primary entry points so callers can use short paths.
pub use dungeon_renderer::DungeonRenderer;
pub use world_map::WorldMap;
pub use room_events::RoomEventRenderer;

use proof_engine::prelude::*;

// ---- shared color helpers used by all sub-modules ----

/// Linearly interpolate between two Vec4 colors.
#[inline]
pub fn color_lerp(a: Vec4, b: Vec4, t: f32) -> Vec4 {
    let t = t.clamp(0.0, 1.0);
    Vec4::new(
        a.x + (b.x - a.x) * t,
        a.y + (b.y - a.y) * t,
        a.z + (b.z - a.z) * t,
        a.w + (b.w - a.w) * t,
    )
}

/// Build a Vec4 from (u8,u8,u8) with alpha.
#[inline]
pub fn rgb_a(r: u8, g: u8, b: u8, a: f32) -> Vec4 {
    Vec4::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
}

/// Build a Vec4 from (u8,u8,u8) with full alpha.
#[inline]
pub fn rgb(r: u8, g: u8, b: u8) -> Vec4 {
    rgb_a(r, g, b, 1.0)
}

/// Dim a color to a fraction of its brightness (keeps alpha).
#[inline]
pub fn dim(c: Vec4, factor: f32) -> Vec4 {
    Vec4::new(c.x * factor, c.y * factor, c.z * factor, c.w)
}
