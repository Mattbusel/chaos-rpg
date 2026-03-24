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
pub fn particle_lifetime_normal() -> u32     { f(90) }
/// Critical hit particle lifetime (brighter, stays longer).
pub fn particle_lifetime_crit() -> u32       { f(120) }
/// Healing particle lifetime.
pub fn particle_lifetime_heal() -> u32       { f(80) }
/// Status applied / misc label lifetime.
pub fn particle_lifetime_status() -> u32     { f(70) }
/// Kill reward (+XP +gold) particle lifetime.
pub fn particle_lifetime_reward() -> u32     { f(110) }
/// Spell damage particle lifetime.
pub fn particle_lifetime_spell() -> u32      { f(100) }
/// Backfire particle lifetime.
pub fn particle_lifetime_backfire() -> u32   { f(110) }

/// Upward drift per frame (tile units). Slightly faster rise for visibility.
pub const PARTICLE_DRIFT: f32 = 0.22;

/// Fraction of lifetime at which fade-out begins (0.0–1.0).
pub const PARTICLE_FADE_START: f32 = 0.60;  // fade over last 40% for longer glow

/// Y-offset stagger between simultaneous hits (tiles).
pub const PARTICLE_STAGGER_Y: i32 = 1;

// ── Hit flash ─────────────────────────────────────────────────────────────────
/// Normal hit border-flash duration (frames).
pub fn flash_normal() -> u32  { f(18) }
/// Crit hit border-flash duration — much more visible now.
pub fn flash_crit() -> u32    { f(28) }

// ── Screen shake ─────────────────────────────────────────────────────────────
/// Shake frames on a player crit / normal big hit.
pub fn shake_crit() -> u32   { f(20) }
/// Shake frames on a backfire / catastrophe.
pub fn shake_heavy() -> u32  { f(28) }
/// Shake frames on a boss attack.
pub fn shake_boss() -> u32   { f(35) }

// ── Spell beam ───────────────────────────────────────────────────────────────
/// Frames the spell beam stays visible after damage.
pub fn beam_hold() -> u32    { f(55) }
/// Frames of "charge" animation before beam fires.
pub fn beam_charge() -> u32  { f(12) }

// ── Status pulse ─────────────────────────────────────────────────────────────
/// Status effect pulse period in frames (higher = slower blink).
pub const STATUS_PULSE_PERIOD: u64 = 12;

// ── Kill effect ───────────────────────────────────────────────────────────────
/// Frames of death-flash inversion after enemy kill.
pub fn kill_flash() -> u32   { f(22) }

// ── Kill linger ───────────────────────────────────────────────────────────────
/// Frames to hold the combat screen after enemy death (so effects resolve).
pub fn kill_linger() -> u32  { f(80) }
