// Visual timing constants for CHAOS RPG — Graphical Edition.
// All values in frames (game runs at ~60 fps).
// Tweak these to taste without touching logic code.
// Set FAST_MODE=1 as an env var to halve all durations (for recording / speed players).

fn fast() -> bool {
    std::env::var("FAST_MODE").map(|v| v == "1").unwrap_or(false)
}

fn f(v: u32) -> u32 { if fast() { (v / 2).max(1) } else { v } }

// ── Particles ─────────────────────────────────────────────────────────────────
/// Normal floating damage number lifetime (frames).
pub fn particle_lifetime_normal() -> u32     { f(52) }
/// Critical hit particle lifetime (brighter, stays longer).
pub fn particle_lifetime_crit() -> u32       { f(60) }
/// Healing particle lifetime.
pub fn particle_lifetime_heal() -> u32       { f(48) }
/// Status applied / misc label lifetime.
pub fn particle_lifetime_status() -> u32     { f(36) }
/// Kill reward (+XP +gold) particle lifetime.
pub fn particle_lifetime_reward() -> u32     { f(55) }
/// Spell damage particle lifetime.
pub fn particle_lifetime_spell() -> u32      { f(52) }
/// Backfire particle lifetime.
pub fn particle_lifetime_backfire() -> u32   { f(55) }

/// Upward drift per frame (tile units). Spec: ~2px/frame at 12px/tile = 0.167.
pub const PARTICLE_DRIFT: f32 = 0.16;

/// Fraction of lifetime at which fade-out begins (0.0–1.0).
pub const PARTICLE_FADE_START: f32 = 0.70;  // fade over last 30%

/// Y-offset stagger between simultaneous hits (tiles).
pub const PARTICLE_STAGGER_Y: i32 = 1;

// ── Hit flash ─────────────────────────────────────────────────────────────────
/// Normal hit border-flash duration (frames).
pub fn flash_normal() -> u32  { f(7) }
/// Crit hit border-flash duration.
pub fn flash_crit() -> u32    { f(9) }

// ── Screen shake ─────────────────────────────────────────────────────────────
/// Shake frames on a player crit / normal big hit.
pub fn shake_crit() -> u32   { f(10) }
/// Shake frames on a backfire / catastrophe.
pub fn shake_heavy() -> u32  { f(12) }
/// Shake frames on a boss attack.
pub fn shake_boss() -> u32   { f(15) }

// ── Spell beam ───────────────────────────────────────────────────────────────
/// Frames the spell beam stays visible after damage.
pub fn beam_hold() -> u32    { f(25) }
/// Frames of "charge" animation before beam fires.
pub fn beam_charge() -> u32  { f(5) }

// ── Status pulse ─────────────────────────────────────────────────────────────
/// Status effect pulse period in frames (higher = slower blink).
pub const STATUS_PULSE_PERIOD: u64 = 16;

// ── Kill effect ───────────────────────────────────────────────────────────────
/// Frames of death-flash inversion after enemy kill.
pub fn kill_flash() -> u32   { f(10) }

// ── Kill linger ───────────────────────────────────────────────────────────────
/// Frames to hold the combat screen after enemy death (so effects resolve).
pub fn kill_linger() -> u32  { f(45) }
