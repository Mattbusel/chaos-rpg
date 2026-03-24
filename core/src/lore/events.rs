//! Rare event flavor text — triggered by specific extreme game states.

/// Flavor text for a zero chaos roll (exactly 0.0 final value).
pub const ZERO_ROLL: &str =
    "The pipeline returned zero. Perfect zero. The proof pauses. For the first time since \
     the Mathematician vanished, nothing is happening. Savor it. It won't last.";

/// Flavor text when the player's attack heals the enemy (negative damage).
pub const NEGATIVE_DAMAGE: &str =
    "Your attack... nourishes the enemy. The chaos pipeline evaluated your strike as a net \
     positive for your target. Somewhere in the proof, a function is laughing. Functions \
     cannot laugh. This one is.";

/// Flavor text when a character rolls all negative stats.
pub const ALL_NEGATIVE_STATS: &str =
    "The proof looked at you. The proof looked away. Your stats are a suicide note written \
     in integers. The Misery System is already warming up. You'll need it.";

/// Flavor text when a character rolls all stats above 100.
pub const ALL_STATS_ABOVE_100: &str =
    "The proof made a mistake. It defined you with too much value. It will spend the rest \
     of this run trying to correct its error. The correction will be violent.";

/// Flavor text for reaching OMEGA power tier.
pub const OMEGA_TIER: &str =
    "You are the largest value in The Proof. Every entity on every floor can feel you. \
     The Algorithm Reborn has started paying attention. It hasn't done that in a long time.";

/// Flavor text for reaching THE VOID power tier.
pub const VOID_TIER: &str =
    "You are the smallest value in The Proof. You are so small the proof has difficulty \
     proving you exist. This is, paradoxically, a kind of freedom.";

/// Flavor text for dying on Floor 1.
pub const DIED_FLOOR_1: &str =
    "The proof evaluated you in one floor and determined you were unnecessary. It was \
     efficient, if nothing else. The graveyard gains a new entry. The entry is very short.";

/// Flavor text for killing a Nemesis.
pub const NEMESIS_KILLED: &str =
    "It came back stronger, titled with your failure, armed with the memory of how it \
     beat you. And you killed it anyway. The proof is revising its opinion of you. \
     This is the first time it has revised anything.";

/// Flavor text when a party member flees due to low morale.
pub fn party_member_fled(name: &str) -> String {
    format!(
        "{name} has done the math. Not the chaos kind — the simple, honest kind. And the \
         answer was: leave. You can't blame a variable for seeking a more stable equation."
    )
}

/// Flavor text for hitting 100,000 Misery Index.
pub const MISERY_100K: &str =
    "Published Failure. The proof has allocated a permanent entry in the Hall of Misery \
     with your name on it. You have suffered more than any variable before you. The proof \
     does not feel guilt. But if it did, it might feel it now. It doesn't. But it might.";

/// Flavor text for corruption stack 400+.
pub const CORRUPTION_400: &str =
    "The chaos pipeline you started with no longer exists. Parameter drift has rewritten \
     it completely. You are being evaluated by a system the Mathematician did not design, \
     would not recognize, and cannot control. This is either the most dangerous thing that \
     has ever happened inside The Proof, or the most beautiful. The distinction is academic.";

/// Flavor text for finding an Artifact-tier item.
pub const ARTIFACT_FOUND: &str =
    "Stop. Look at what you're holding. The proof defined this once, at the very edge of \
     what it could compute, and it has not been able to define anything like it since. \
     This is not an item. It is a boundary condition. Treat it accordingly.";

/// Flavor text when the chaos pipeline produces NaN (clamped to usable).
pub const NAN_ROLL: &str =
    "NaN. Not a Number. The pipeline tried to evaluate your action and produced a result \
     that is not a number. This should be impossible. The proof has no axiom that permits \
     non-numeric output. And yet, here it is. The game clamped it to something usable. \
     The proof is pretending this didn't happen. You should too.";

/// Flavor text when a character wins with all-negative stats.
pub const WIN_NEGATIVE_STATS: &str =
    "The proof could not evaluate you. You were smaller than its threshold for existence. \
     And you won anyway. This information has been forwarded to The Mathematician's last \
     known location. There has been no reply. There is now a reply. It says: 'I see.'";

/// Flavor text for getting a kill count of exactly 0 on death.
pub const DIED_ZERO_KILLS: &str =
    "Not a single kill. The proof generated you, evaluated you, and found you incapable \
     of modifying any other entity. Your existence had no mathematical effect. Your absence \
     has no mathematical effect either. The proof has already moved on.";

/// Floor-range flavor variants for entering a new floor (called each floor, not each range).
pub fn floor_transition_flavor(floor: u32, seed: u64) -> Option<&'static str> {
    // Only show on milestone floors or first of a range
    if matches!(floor, 1 | 5 | 10 | 11 | 20 | 21 | 30 | 31 | 50 | 51 | 75 | 76 | 99 | 100 | 101) {
        Some(crate::lore::world::floor_entry_text(floor))
    } else if floor > 100 && seed % 5 == 0 {
        // Beyond floor 100: occasional reminders
        const BEYOND: &[&str] = &[
            "The proof is generating this floor around you as you walk through it.",
            "There is nothing here. The proof is making something. Quickly.",
            "You have outlasted the proof's content. It is improvising. It is not good at improvising.",
            "The Mathematician did not write floor numbers this high. The proof is counting \
             anyway. It can't stop counting.",
        ];
        Some(BEYOND[(seed % BEYOND.len() as u64) as usize])
    } else {
        None
    }
}

/// Misery milestone flavor text.
pub fn misery_milestone_flavor(milestone: u64) -> &'static str {
    match milestone {
        5_000 => {
            "The proof has noted your suffering. It has allocated Spite as a resource. \
             Even the proof believes in some form of compensation."
        }
        10_000 => {
            "Defiance activates. The proof's model of you includes a variable it cannot \
             evaluate: the tendency to refuse. It is recalculating."
        }
        25_000 => {
            "Cosmic Joke. The proof has generated a sense of humor from your data. It is \
             not a good sense of humor. It is yours."
        }
        50_000 => {
            "Transcendent Misery. Suffering has become a power source. The proof did not \
             intend this. The proof acknowledges it is happening anyway."
        }
        100_000 => MISERY_100K,
        _ => "The proof has updated its models based on your continued survival.",
    }
}
