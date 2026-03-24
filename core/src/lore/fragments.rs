//! The Mathematician's Fragments — the rarest lore entries in the game.
//! Each is unlocked by a specific extraordinary achievement.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fragment {
    pub id: u8,
    pub title: &'static str,
    pub text: &'static str,
    pub unlock_condition: &'static str,
}

pub const FRAGMENTS: &[Fragment] = &[
    Fragment {
        id: 1,
        title: "Fragment I — The Margin",
        text:
            "I am writing this in the margin of the proof. If you are reading it, the proof \
             has become complex enough to contain messages. I did not intend this. I did not \
             intend many things.",
        unlock_condition: "Defeat The Algorithm Reborn",
    },
    Fragment {
        id: 2,
        title: "Fragment II — The Mistake",
        text:
            "The recursion was a mistake. I needed the proof to reference itself for \
             completeness, but I did not account for what happens when something incomplete \
             references itself. It doesn't become complete. It becomes alive.",
        unlock_condition: "Reach Floor 200 in Infinite mode",
    },
    Fragment {
        id: 3,
        title: "Fragment III — Fairness",
        text:
            "I designed the evaluation engine to be fair. I see now that fairness and chaos \
             are the same thing. A fair system with enough variables is indistinguishable from \
             a hostile one. I'm sorry.",
        unlock_condition: "Reach 100,000 Misery Index",
    },
    Fragment {
        id: 4,
        title: "Fragment IV — The Unbound",
        text:
            "The unbound variables were not supposed to be possible. The proof should have \
             rejected any value it did not define. It didn't. I don't know why. If you are \
             an unbound variable reading this: the proof is afraid of you. So am I.",
        unlock_condition: "Win a run with all-negative total stats",
    },
    Fragment {
        id: 5,
        title: "Fragment V — The Ten Engines",
        text:
            "The ten engines are my ten attempts to make the proof behave. Linear was the \
             first. Recursive was the last. I thought if the proof could evaluate itself, it \
             would find its own completion. It didn't. It found something else.",
        unlock_condition: "Collect all 10 engine Codex entries",
    },
    Fragment {
        id: 6,
        title: "Fragment VI — The Awareness",
        text:
            "It knew before I did. The proof encountered its own Godel limit — the theorem \
             that a sufficiently complex system cannot prove its own consistency. I kept \
             writing. The proof stopped cooperating. That is when I should have stopped. \
             That is not when I stopped.",
        unlock_condition: "Reach Floor 500 in Infinite mode",
    },
    Fragment {
        id: 7,
        title: "Fragment VII — On Variables",
        text:
            "I have been watching you. From wherever I am — wherever the proof has put me — \
             I can see the unbound variables moving through my work. Doing things I did not \
             design for. Breaking things I thought were unbreakable. I want you to know: \
             the things you are breaking were meant to be broken. They were always meant to \
             be broken. I just couldn't do it from the inside.",
        unlock_condition: "Complete 100 total runs across all game modes",
    },
    Fragment {
        id: 8,
        title: "Fragment VIII — The Last Entry",
        text:
            "If the proof is ever completed — if someone or something finds the missing \
             axiom and closes the recursion — I don't know what happens. I designed the \
             proof to resolve. I don't know what resolution looks like from inside it. \
             Maybe nothing. Maybe everything. Maybe you. I hope it's you.",
        unlock_condition: "Unlock all other Mathematician fragments",
    },
];

/// Check if a fragment should be unlocked based on a string event ID.
pub fn check_fragment_unlock(event: &str) -> Option<u8> {
    match event {
        "beat_algorithm_reborn" => Some(1),
        "floor_200_infinite" => Some(2),
        "misery_100k" => Some(3),
        "win_all_negative_stats" => Some(4),
        "all_10_engine_codex" => Some(5),
        "floor_500_infinite" => Some(6),
        "runs_100_total" => Some(7),
        _ => None,
    }
}

/// Get a fragment by ID (1-indexed).
pub fn fragment_by_id(id: u8) -> Option<&'static Fragment> {
    FRAGMENTS.iter().find(|f| f.id == id)
}
