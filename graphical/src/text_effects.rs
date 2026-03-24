// Text rendering effects — CHAOS RPG Visual Push
//
// Helpers that produce modified strings or draw text with special effects.
// Designed to be called in draw functions with minimal integration cost.

use bracket_lib::prelude::*;

// ── Glitch text ───────────────────────────────────────────────────────────────

const GLITCH_CHARS: &[char] = &[
    '0','1','2','3','4','5','6','7','8','9',
    '!','@','#','$','%','^','&','*','?','~',
    '░','▒','▓','╬','╪','╫','╩','╦','║','═',
    'π','φ','∞','Δ','λ','ε','∑','∂','μ','σ',
];

/// Return a version of `text` with some characters replaced by glitch chars.
/// `frame` drives the randomness; `intensity` 0.0-1.0 controls glitch density.
pub fn glitch_string(text: &str, frame: u64, intensity: f32) -> String {
    text.chars().enumerate().map(|(i, ch)| {
        let roll = ((i as u64).wrapping_mul(6364136223846793005)
            .wrapping_add(frame.wrapping_mul(1442695040888963407))) % 1000;
        if roll < (intensity * 150.0) as u64 {
            let gi = ((i as u64 * 31 + frame * 7) % GLITCH_CHARS.len() as u64) as usize;
            GLITCH_CHARS[gi]
        } else {
            ch
        }
    }).collect()
}

/// Stabilising glitch: `settle` goes 0.0→1.0 as text resolves.
/// At settle=0.0, heavily glitched; at 1.0, final text.
pub fn glitch_settle(text: &str, frame: u64, settle: f32) -> String {
    let intensity = (1.0 - settle).max(0.0) * 0.8;
    glitch_string(text, frame, intensity)
}

// ── Typewriter ────────────────────────────────────────────────────────────────

/// How many characters of `text` to show given a frame counter and chars-per-frame rate.
pub fn typewriter_len(text: &str, elapsed_frames: u32, chars_per_frame: f32) -> usize {
    ((elapsed_frames as f32 * chars_per_frame) as usize).min(text.len())
}

/// Draw text typewriter-style. Returns true when fully revealed.
pub fn draw_typewriter(
    ctx: &mut BTerm, x: i32, y: i32,
    text: &str, fg: RGB, bg: RGB,
    elapsed_frames: u32, chars_per_frame: f32,
) -> bool {
    let n = typewriter_len(text, elapsed_frames, chars_per_frame);
    let shown: String = text.chars().take(n).collect();
    ctx.print_color(x, y, fg, bg, &shown);
    n >= text.len()
}

// ── Shake text ────────────────────────────────────────────────────────────────

/// Draw text with per-character random offset (simulated via char substitution).
/// Since we can't do sub-tile offsets, we shift the draw position ±1 tile.
pub fn draw_shake_text(
    ctx: &mut BTerm, x: i32, y: i32,
    text: &str, fg: RGB, bg: RGB,
    frame: u64, amplitude: f32,
) {
    for (i, ch) in text.chars().enumerate() {
        let seed = (i as u64).wrapping_mul(1231) + frame.wrapping_mul(7);
        let ox = if amplitude > 0.5 && seed % 4 == 0 { (seed % 3) as i32 - 1 } else { 0 };
        let oy = if amplitude > 0.5 && seed % 5 == 1 { (seed % 3) as i32 - 1 } else { 0 };
        let cx = x + i as i32 + ox;
        let cy = y + oy;
        if cx >= 0 && cx < 160 && cy >= 0 && cy < 80 {
            ctx.print_color(cx, cy, fg, bg, &ch.to_string());
        }
    }
}

// ── Wave text ─────────────────────────────────────────────────────────────────

/// Draw text with a traveling vertical sine wave.
/// Each character is drawn at y + sin(phase + i * 0.35) * amplitude (rounded to ±1).
pub fn draw_wave_text(
    ctx: &mut BTerm, x: i32, y: i32,
    text: &str, fg: RGB, bg: RGB,
    frame: u64, amplitude: f32,
) {
    let phase = frame as f32 * 0.08;
    for (i, ch) in text.chars().enumerate() {
        let wave = (phase + i as f32 * 0.4).sin() * amplitude;
        let cy = y + wave.round() as i32;
        let cx = x + i as i32;
        if cx >= 0 && cx < 160 && cy >= 0 && cy < 80 {
            ctx.print_color(cx, cy, fg, bg, &ch.to_string());
        }
    }
}

// ── Breathing text ────────────────────────────────────────────────────────────

/// Return a brightness-pulsed color for breathing text.
pub fn breathing_color(base: (u8, u8, u8), frame: u64, amplitude: f32) -> (u8, u8, u8) {
    let pulse = (frame as f32 * 0.05).sin() * amplitude + 1.0;
    (
        (base.0 as f32 * pulse).clamp(0.0, 255.0) as u8,
        (base.1 as f32 * pulse).clamp(0.0, 255.0) as u8,
        (base.2 as f32 * pulse).clamp(0.0, 255.0) as u8,
    )
}

// ── Scramble reveal ───────────────────────────────────────────────────────────

/// Like typewriter but reveals by replacing random chars with final chars progressively.
/// `progress` 0.0-1.0: 0 = all random, 1 = all correct.
pub fn scramble_reveal(text: &str, frame: u64, progress: f32) -> String {
    let n = text.len();
    let revealed = (progress * n as f32) as usize;
    text.chars().enumerate().map(|(i, ch)| {
        if i < revealed {
            ch
        } else {
            let gi = ((i as u64).wrapping_mul(997) + frame.wrapping_mul(31)) % GLITCH_CHARS.len() as u64;
            GLITCH_CHARS[gi as usize]
        }
    }).collect()
}

// ── Drip text ─────────────────────────────────────────────────────────────────

/// Returns a y-offset for "drip" effect: characters slowly fall downward.
/// `phase` is per-character phase; offset increases with time.
pub fn drip_offset(char_idx: usize, frame: u64) -> i32 {
    let drift = ((frame / 4 + char_idx as u64 * 7) % 6) as i32;
    drift
}

// ── Rainbow text ──────────────────────────────────────────────────────────────

/// Draw text with each character cycling through rainbow colors.
pub fn draw_rainbow(ctx: &mut BTerm, x: i32, y: i32, text: &str, bg: RGB, frame: u64) {
    let phase = frame as f32 * 0.06;
    for (i, ch) in text.chars().enumerate() {
        let hue = (phase + i as f32 * 0.5).rem_euclid(std::f32::consts::TAU);
        let r = ((hue.cos() + 1.0) * 0.5 * 220.0 + 35.0) as u8;
        let g = (((hue + 2.09).cos() + 1.0) * 0.5 * 220.0 + 35.0) as u8;
        let b = (((hue + 4.19).cos() + 1.0) * 0.5 * 220.0 + 35.0) as u8;
        ctx.print_color(x + i as i32, y, RGB::from_u8(r, g, b), bg, &ch.to_string());
    }
}
