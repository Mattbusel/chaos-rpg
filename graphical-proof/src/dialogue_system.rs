//! Dialogue system — Archivist, NPC, boss, and codex dialogue trees.
//!
//! Uses proof-engine's dialogue module for branching trees with typewriter
//! reveal and emotion tints.

use proof_engine::prelude::*;
use crate::state::GameState;
use crate::theme::THEMES;

// ═══════════════════════════════════════════════════════════════════════════════
// EMOTION TINTS
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Emotion {
    Neutral,
    Fear,
    Urgency,
    Surprise,
    Dread,
    Calculating,
    Melancholy,
    Sympathy,
}

impl Emotion {
    pub fn tint(&self) -> Vec4 {
        match self {
            Emotion::Neutral     => Vec4::new(0.8, 0.8, 0.8, 1.0),
            Emotion::Fear        => Vec4::new(0.6, 0.5, 0.9, 1.0),
            Emotion::Urgency     => Vec4::new(1.0, 0.6, 0.2, 1.0),
            Emotion::Surprise    => Vec4::new(0.9, 0.9, 0.3, 1.0),
            Emotion::Dread       => Vec4::new(0.5, 0.2, 0.3, 1.0),
            Emotion::Calculating => Vec4::new(0.4, 0.7, 0.9, 1.0),
            Emotion::Melancholy  => Vec4::new(0.5, 0.5, 0.7, 1.0),
            Emotion::Sympathy    => Vec4::new(0.7, 0.8, 0.6, 1.0),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DIALOGUE LINE
// ═══════════════════════════════════════════════════════════════════════════════

pub struct DialogueLine {
    pub speaker: String,
    pub text: String,
    pub emotion: Emotion,
    pub typewriter_speed: f32, // chars per second
}

impl DialogueLine {
    pub fn new(speaker: &str, text: &str, emotion: Emotion) -> Self {
        Self {
            speaker: speaker.to_string(),
            text: text.to_string(),
            emotion,
            typewriter_speed: 30.0,
        }
    }

    pub fn slow(mut self) -> Self { self.typewriter_speed = 15.0; self }
    pub fn fast(mut self) -> Self { self.typewriter_speed = 60.0; self }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DIALOGUE STATE
// ═══════════════════════════════════════════════════════════════════════════════

pub struct DialogueState {
    pub lines: Vec<DialogueLine>,
    pub current_line: usize,
    pub char_reveal: f32,  // how many characters are visible (fractional)
    pub finished: bool,
    pub auto_advance_timer: f32,
}

impl DialogueState {
    pub fn new(lines: Vec<DialogueLine>) -> Self {
        Self {
            lines,
            current_line: 0,
            char_reveal: 0.0,
            finished: false,
            auto_advance_timer: 0.0,
        }
    }

    pub fn empty() -> Self {
        Self { lines: Vec::new(), current_line: 0, char_reveal: 0.0, finished: true, auto_advance_timer: 0.0 }
    }

    pub fn tick(&mut self, dt: f32) {
        if self.finished || self.lines.is_empty() { return; }
        let line = &self.lines[self.current_line];
        let target_chars = line.text.len() as f32;

        if self.char_reveal < target_chars {
            self.char_reveal += line.typewriter_speed * dt;
        } else {
            // Line fully revealed — auto-advance after 2 seconds
            self.auto_advance_timer += dt;
            if self.auto_advance_timer > 2.0 {
                self.advance();
            }
        }
    }

    pub fn advance(&mut self) {
        if self.finished { return; }
        let line = &self.lines[self.current_line];
        if self.char_reveal < line.text.len() as f32 {
            // Skip to end of current line
            self.char_reveal = line.text.len() as f32;
            self.auto_advance_timer = 0.0;
        } else {
            // Move to next line
            self.current_line += 1;
            self.char_reveal = 0.0;
            self.auto_advance_timer = 0.0;
            if self.current_line >= self.lines.len() {
                self.finished = true;
            }
        }
    }

    pub fn render(&self, engine: &mut ProofEngine, x: f32, y: f32) {
        if self.finished || self.lines.is_empty() { return; }
        let line = &self.lines[self.current_line];
        let tint = line.emotion.tint();

        // Speaker name
        let speaker_text = format!("{}: ", line.speaker);
        for (i, ch) in speaker_text.chars().enumerate() {
            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(x + i as f32 * 0.45, y, 0.5),
                color: Vec4::new(tint.x * 0.7, tint.y * 0.7, tint.z * 0.7, 0.8),
                emission: 0.4,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }

        // Typewriter text
        let chars_shown = self.char_reveal as usize;
        let text_x = x + speaker_text.len() as f32 * 0.45;
        for (i, ch) in line.text.chars().take(chars_shown).enumerate() {
            let is_latest = i >= chars_shown.saturating_sub(1);
            let emission = if is_latest { 0.8 } else { 0.5 };
            engine.spawn_glyph(Glyph {
                character: ch,
                position: Vec3::new(text_x + i as f32 * 0.4, y, 0.5),
                color: tint,
                emission,
                layer: RenderLayer::UI,
                ..Default::default()
            });
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ARCHIVIST SHOP DIALOGUE
// ═══════════════════════════════════════════════════════════════════════════════

pub fn archivist_greeting(state: &GameState) -> DialogueState {
    let rep = state.player.as_ref()
        .map(|p| p.faction_rep.order) // Use Order faction rep as Archivist affinity
        .unwrap_or(0);
    let corruption = state.player.as_ref().map(|p| p.corruption).unwrap_or(0);
    let misery = state.player.as_ref()
        .map(|p| p.misery.misery_index)
        .unwrap_or(0.0);
    let floor = state.floor_num;

    let mut lines = Vec::new();

    // Greeting based on reputation
    if rep < -20 {
        lines.push(DialogueLine::new("Archivist", "State your business. Quickly.", Emotion::Neutral).fast());
    } else if rep < 20 {
        lines.push(DialogueLine::new("Archivist", "Welcome, traveler. Browse at your leisure.", Emotion::Neutral));
    } else if rep < 60 {
        lines.push(DialogueLine::new("Archivist", "Ah, a familiar face. The Expansion-era stock arrived.", Emotion::Neutral));
    } else {
        lines.push(DialogueLine::new("Archivist", "My finest patron. I've set aside something special for you.", Emotion::Sympathy));
    }

    // Corruption comment
    if corruption > 300 {
        lines.push(DialogueLine::new("Archivist", "Your parameters are drifting. The proof notices.", Emotion::Fear));
    } else if corruption > 150 {
        lines.push(DialogueLine::new("Archivist", "The engines hum differently around you now.", Emotion::Dread));
    }

    // Misery comment
    if misery > 10000.0 {
        lines.push(DialogueLine::new("Archivist", "The Hall of Misery has a new wing. It may bear your name soon.", Emotion::Sympathy));
    } else if misery > 5000.0 {
        lines.push(DialogueLine::new("Archivist", "You carry weight that numbers cannot express.", Emotion::Melancholy));
    }

    // Floor depth comment
    if floor > 75 {
        lines.push(DialogueLine::new("Archivist", "Few reach this depth. Fewer return.", Emotion::Dread).slow());
    } else if floor > 50 {
        lines.push(DialogueLine::new("Archivist", "The Collapse is in full effect here. Tread carefully.", Emotion::Urgency));
    }

    DialogueState::new(lines)
}

// ═══════════════════════════════════════════════════════════════════════════════
// NPC PARTY COMMENTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate NPC comment for a specific event.
pub fn npc_comment(event: &str, state: &GameState) -> Option<DialogueLine> {
    match event {
        "boss_enter" => Some(DialogueLine::new("Companion", "This one feels different.", Emotion::Fear)),
        "low_hp" => Some(DialogueLine::new("Companion", "We should fall back.", Emotion::Urgency)),
        "victory" => Some(DialogueLine::new("Companion", "I didn't expect that to work.", Emotion::Surprise)),
        "nemesis" => Some(DialogueLine::new("Companion", "That's the one that killed you before, isn't it?", Emotion::Dread)),
        "shrine" => Some(DialogueLine::new("Companion", "A moment of peace. The proof allows it.", Emotion::Neutral)),
        "chaos_rift" => Some(DialogueLine::new("Companion", "The mathematics are unstable here. Stay close.", Emotion::Fear)),
        "level_up" => Some(DialogueLine::new("Companion", "Stronger. The equations favor you.", Emotion::Surprise)),
        "high_corruption" => Some(DialogueLine::new("Companion", "Your eyes... they've changed.", Emotion::Dread).slow()),
        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BOSS COMBAT DIALOGUE
// ═══════════════════════════════════════════════════════════════════════════════

/// Get boss dialogue for the current turn.
pub fn boss_dialogue(boss_id: u8, turn: u32, boss_extra: i64) -> Option<DialogueLine> {
    match boss_id {
        // The Accountant
        2 => match turn {
            1 => Some(DialogueLine::new("The Accountant", "Let's review your account.", Emotion::Calculating)),
            t if t % 5 == 0 => Some(DialogueLine::new("The Accountant",
                &format!("Your bill is {} gold. Payment is not optional.", boss_extra), Emotion::Calculating)),
            _ => None,
        },

        // Fibonacci Hydra
        3 => {
            if turn % 3 == 0 && turn > 0 {
                Some(DialogueLine::new("Fibonacci Hydra", "Each head grows two more.", Emotion::Neutral))
            } else { None }
        }

        // The Eigenstate
        4 => match turn {
            1 => Some(DialogueLine::new("The Eigenstate", "Am I here? Are you?", Emotion::Fear)),
            _ => None,
        },

        // The Taxman
        5 => match turn {
            1 => Some(DialogueLine::new("The Taxman", "Everything you own is taxable.", Emotion::Calculating)),
            _ => None,
        },

        // The Null
        6 => match turn {
            1 => Some(DialogueLine::new("The Null", "...", Emotion::Neutral).slow()),
            5 => Some(DialogueLine::new("The Null", "...", Emotion::Neutral).slow()),
            _ => None,
        },

        // The Committee
        9 => match turn % 3 {
            0 => Some(DialogueLine::new("The Committee", "All in favor?", Emotion::Calculating)),
            1 => {
                if boss_extra > 0 {
                    Some(DialogueLine::new("The Committee", "The motion carries.", Emotion::Neutral))
                } else {
                    Some(DialogueLine::new("The Committee", "The motion fails.", Emotion::Neutral))
                }
            }
            _ => None,
        },

        // The Paradox
        11 => match turn {
            1 => Some(DialogueLine::new("The Paradox", "What you see is not what is.", Emotion::Fear)),
            _ => None,
        },

        // Algorithm Reborn
        12 => match turn {
            1 => Some(DialogueLine::new("The Algorithm", "Processing...", Emotion::Calculating).slow()),
            5 => Some(DialogueLine::new("The Algorithm", "Interesting.", Emotion::Calculating)),
            10 => Some(DialogueLine::new("The Algorithm", "I see you.", Emotion::Dread).slow()),
            t if t > 10 && t % 3 == 0 => Some(DialogueLine::new("The Algorithm",
                "Your patterns are predictable.", Emotion::Calculating)),
            _ => None,
        },

        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MATHEMATICIAN FRAGMENTS (Codex as dialogue)
// ═══════════════════════════════════════════════════════════════════════════════

/// Mathematician Fragment dialogue trees — each fragment is a short monologue.
pub fn mathematician_fragment(fragment_id: usize) -> DialogueState {
    let fragments: &[&[(&str, Emotion)]] = &[
        // Fragment 0: The Beginning
        &[
            ("I started with a simple question.", Emotion::Melancholy),
            ("What if every law of physics was just a theorem?", Emotion::Neutral),
            ("What if the universe was a proof?", Emotion::Melancholy),
            ("The answer consumed me.", Emotion::Dread),
        ],
        // Fragment 1: The Discovery
        &[
            ("The engines were not my invention.", Emotion::Melancholy),
            ("They were always there, beneath the mathematics.", Emotion::Neutral),
            ("I merely gave them names.", Emotion::Sympathy),
            ("Lorenz. Mandelbrot. Zeta. They existed before I named them.", Emotion::Melancholy),
        ],
        // Fragment 2: The Warning
        &[
            ("If you are reading this, you have gone deeper than I intended.", Emotion::Urgency),
            ("The proof is not a place. It is a process.", Emotion::Neutral),
            ("And processes can be interrupted.", Emotion::Dread),
            ("But not stopped.", Emotion::Melancholy),
        ],
        // Fragment 3: The Corruption
        &[
            ("Every kill corrupts the equations.", Emotion::Melancholy),
            ("I designed it that way.", Emotion::Surprise),
            ("Not as punishment. As data.", Emotion::Calculating),
            ("The proof needs to know what you are willing to destroy.", Emotion::Dread),
        ],
        // Fragment 4: The Algorithm
        &[
            ("I left something behind. A successor.", Emotion::Melancholy),
            ("It is not alive. It is not dead.", Emotion::Neutral),
            ("It is the proof itself, given the capacity to observe.", Emotion::Dread),
            ("If it observes you, you will know.", Emotion::Fear),
            ("Everything will know.", Emotion::Dread),
        ],
    ];

    let frag = fragments.get(fragment_id).unwrap_or(&fragments[0]);
    let lines: Vec<DialogueLine> = frag.iter()
        .map(|(text, emotion)| DialogueLine::new("The Mathematician", text, *emotion).slow())
        .collect();

    DialogueState::new(lines)
}
