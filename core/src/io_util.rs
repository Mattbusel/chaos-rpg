//! Minimal terminal I/O utilities for use inside core game logic.
//!
//! This module provides only what boss fights and other core systems
//! need to produce output without depending on a full UI crate.
//! Color constants are plain ANSI escape codes.

use std::io::{self, BufRead, Write};

// ─── ANSI COLOR CONSTANTS ────────────────────────────────────────────────────

/// ANSI code that resets all colours and styles.
pub const RESET: &str = "\x1b[0m";
/// ANSI code for bold text.
pub const BOLD: &str = "\x1b[1m";
/// ANSI code for dim text.
pub const DIM: &str = "\x1b[2m";
/// ANSI code for red text.
pub const RED: &str = "\x1b[31m";
/// ANSI code for green text.
pub const GREEN: &str = "\x1b[32m";
/// ANSI code for yellow text.
pub const YELLOW: &str = "\x1b[33m";
/// ANSI code for cyan text.
pub const CYAN: &str = "\x1b[36m";
/// ANSI code for magenta text.
pub const MAGENTA: &str = "\x1b[35m";
/// ANSI code for bright white text.
pub const WHITE: &str = "\x1b[97m";
/// ANSI code for bright red text.
pub const BRIGHT_RED: &str = "\x1b[91m";
/// ANSI code for bright green text.
pub const BRIGHT_GREEN: &str = "\x1b[92m";
/// ANSI code for bright cyan text.
pub const BRIGHT_CYAN: &str = "\x1b[96m";
/// ANSI code for bright magenta text.
pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
/// ANSI code for blue text.
pub const BLUE: &str = "\x1b[34m";

// ─── BASIC I/O ───────────────────────────────────────────────────────────────

/// Read a line of input, returning an empty string on error.
pub fn read_line() -> String {
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).ok();
    line.trim().to_string()
}

/// Print a prompt and read a line of input.
pub fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().ok();
    read_line()
}

/// Wait for the user to press Enter.
pub fn press_enter(msg: &str) {
    print!("{}", msg);
    io::stdout().flush().ok();
    read_line();
}

/// Clear the terminal screen.
pub fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    io::stdout().flush().ok();
}
