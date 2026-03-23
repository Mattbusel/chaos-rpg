//! Minimal terminal I/O utilities for use inside core game logic.
//!
//! This module provides only what boss fights and other core systems
//! need to produce output without depending on a full UI crate.
//! Color constants are plain ANSI escape codes.

use std::io::{self, BufRead, Write};

// ─── ANSI COLOR CONSTANTS ────────────────────────────────────────────────────

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const CYAN: &str = "\x1b[36m";
pub const MAGENTA: &str = "\x1b[35m";
pub const WHITE: &str = "\x1b[97m";
pub const BRIGHT_RED: &str = "\x1b[91m";
pub const BRIGHT_GREEN: &str = "\x1b[92m";
pub const BRIGHT_CYAN: &str = "\x1b[96m";
pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
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
