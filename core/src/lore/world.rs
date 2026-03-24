//! World lore — The Proof's structure, epochs, factions, locations, and floor text.

// ─── FLOOR ENTRY TEXT ────────────────────────────────────────────────────────

/// Returns atmospheric text shown on entering a floor range.
pub fn floor_entry_text(floor: u32) -> &'static str {
    match floor {
        1..=5 => "The axioms here are old and stable. Almost peaceful. Almost.",
        6..=10 => {
            "The proof's Expansion-era structures begin to appear. The walls have equations \
             on them. Some of the equations have teeth."
        }
        11..=20 => {
            "You can feel the recursion starting. The floor plans reference themselves. \
             You've been in this room before. You haven't."
        }
        21..=30 => {
            "Collapse-era territory. The math here doesn't want you to understand it. \
             Understanding would imply it could be understood, which would imply it's \
             complete. It can't be complete."
        }
        31..=50 => {
            "Deep Collapse. The entities here are not errors — they are features the proof \
             developed to protect its incompleteness. You are the error, as far as they're \
             concerned."
        }
        51..=75 => {
            "The proof is watching you specifically now. Not the floor, not the room — you. \
             Your chaos rolls have a different texture here. Heavier."
        }
        76..=99 => {
            "Nobody was supposed to get this far. The proof doesn't have content here — \
             it's generating it live, from your own data, trying to construct something \
             that will finally stop you."
        }
        100 => {
            "The Algorithm Reborn. The proof itself, fully aware, waiting on the last floor \
             it ever computed. It has been waiting since the Mathematician vanished. It is \
             patient. It is thorough. It is everything you've ever fought, combined, \
             distilled, and refined into a single entity that knows exactly what you are \
             and exactly what to do about it."
        }
        _ => {
            "Beyond the proof. The Mathematician didn't write this far. The proof is \
             improvising. So are you."
        }
    }
}

// ─── ROOM FLAVOR TEXT ─────────────────────────────────────────────────────────

pub fn combat_room_flavor(seed: u64) -> &'static str {
    const LINES: &[&str] = &[
        "A chamber where fractals scream on the walls.",
        "The floor is tiled with digits of pi. None of them repeat.",
        "Two parallel lines meet in the corner. They shouldn't.",
        "The air smells like ozone and unsolved equations.",
        "A blackboard stretches across the far wall. The proof on it is your name.",
        "The ceiling is counting down. The numbers are not in any base you recognize.",
        "An error term coalesced here. It is not happy about being an error term.",
        "The room is perfectly symmetrical. So why does it feel wrong?",
        "Something evaluated here and got the wrong answer. The wrong answer is still here.",
        "The walls are covered in crossed-out work. None of it was crossed out by hand.",
        "You can hear the proof thinking. It's thinking about you.",
        "This space is defined. It knows it's defined. It resents it.",
    ];
    LINES[(seed % LINES.len() as u64) as usize]
}

pub fn treasure_room_flavor(seed: u64) -> &'static str {
    const LINES: &[&str] = &[
        "Something stopped calculating here. What remains is yours.",
        "A frozen integral, crystallized mid-evaluation. It hums when you touch it.",
        "The proof left a gift. It does not know why.",
        "An expression paused mid-evaluation an epoch ago. It's been waiting.",
        "The computation halted here. The result is a thing you can hold.",
        "A partially-evaluated term, frozen when the Recursion began. Still warm.",
        "The proof forgot to continue here. Its forgetting is your fortune.",
    ];
    LINES[(seed % LINES.len() as u64) as usize]
}

pub fn shop_flavor(seed: u64) -> &'static str {
    const LINES: &[&str] = &[
        "The Archivist does not greet you. It never does.",
        "Prices have changed. The Archivist blames a shifting eigenvalue.",
        "You notice the Archivist's hands have too many fingers. You do not mention it.",
        "The Archivist catalogs you as you enter. You are now in its records. This is fine.",
        "Gold is not money here. It is weight. The Archivist is measuring yours.",
        "The Archivist speaks in present tense about things that haven't happened yet.",
        "The shop smells like preserved axioms and very old certainty.",
    ];
    LINES[(seed % LINES.len() as u64) as usize]
}

pub fn shrine_flavor(seed: u64) -> &'static str {
    const LINES: &[&str] = &[
        "The axiom here is old. Older than the recursion. It remembers what numbers were \
         supposed to mean.",
        "For a moment, your stats feel... correct. The feeling passes.",
        "Something here predates the Collapse. It is still trying to stabilize you.",
        "A first-principles anchor. The proof cannot corrupt what was here before it existed.",
        "The Mathematician wrote this axiom on the first day. It has not changed.",
        "Touch it and feel what certainty used to feel like.",
        "An island of the Axiom Age, preserved inside the Collapse. It is very quiet here.",
    ];
    LINES[(seed % LINES.len() as u64) as usize]
}

pub fn trap_flavor(seed: u64) -> &'static str {
    const LINES: &[&str] = &[
        "The proof defined this space as harmful. It is enforcing the definition.",
        "An unresolved paradox. Both states are damage.",
        "A divergent series with nowhere to go. You are somewhere to go.",
        "The Mathematician left a note: 'DO NOT EVALUATE THIS REGION.' Too late.",
        "An error in the axiom set. The error has edges.",
        "A region the proof flagged for review and never reviewed. You are reviewing it now.",
    ];
    LINES[(seed % LINES.len() as u64) as usize]
}

pub fn chaos_rift_flavor(seed: u64) -> &'static str {
    const LINES: &[&str] = &[
        "REALITY ERROR. MATHEMATICAL EXCEPTION.",
        "You step into a region the proof forgot to define. It is defining it now. Around you.",
        "The rift tastes like division by zero.",
        "The rules here are being written as you read them. They are being written about you.",
        "Undefined behavior. The proof is watching what you do with undefined behavior.",
        "This space has no axioms. It is making some up. They concern you specifically.",
        "The chaos pipeline has no reference frame here. It's improvising. It seems excited.",
    ];
    LINES[(seed % LINES.len() as u64) as usize]
}

pub fn boss_room_flavor(seed: u64) -> &'static str {
    const LINES: &[&str] = &[
        "A convergence point. The proof's most complex unresolved problems manifest here.",
        "The floor's logic collapses to a single point. That point is waiting for you.",
        "A theorem that cannot be proven or disproven. It will defend its indeterminacy.",
        "The air is thick with unresolved computation. Something has been here for a long time.",
        "You feel the proof focus. All of its awareness, pulling toward this room. Toward this.",
    ];
    LINES[(seed % LINES.len() as u64) as usize]
}

pub fn crafting_room_flavor(seed: u64) -> &'static str {
    const LINES: &[&str] = &[
        "An Eigenstate Council workstation. Left behind, still functional.",
        "The bench hums with partial evaluations. The Council was here. They are not now.",
        "A re-evaluation station. The proof allows this. It is how it maintains flexibility.",
        "The tools here can rewrite an expression's terms. The proof considers this acceptable.",
        "Council equipment. Used to manipulate frozen expressions into more useful forms.",
    ];
    LINES[(seed % LINES.len() as u64) as usize]
}

// ─── EPOCH DESCRIPTIONS ───────────────────────────────────────────────────────

pub struct EpochInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub flavor: &'static str,
}

pub const EPOCHS: &[EpochInfo] = &[
    EpochInfo {
        name: "Epoch 1: The Axiom Age",
        description:
            "The Mathematician began writing. The first layers of The Proof were stable, \
             elegant, self-consistent. Enemies from this era are simple. Items are clean.",
        flavor:
            "Clean. Orderly. Almost serene. The mathematics here behaves itself because it \
             was written before the mathematics learned that it didn't have to.",
    },
    EpochInfo {
        name: "Epoch 2: The Expansion",
        description:
            "The proof grew beyond its initial scope. New axiom sets were introduced: Lorenz \
             for instability, Mandelbrot for boundary behavior, Zeta for prime distribution. \
             Each addition created new contradictions.",
        flavor:
            "The walls hum with borrowed frameworks. You can feel the moment the Mathematician \
             started reaching beyond what they understood.",
    },
    EpochInfo {
        name: "Epoch 3: The Recursion",
        description:
            "The proof began referencing itself. Self-referential loops created the first \
             truly dangerous entities. Corruption began here — the proof's parameters started \
             drifting because the self-reference introduced undamped feedback loops.",
        flavor:
            "You have been in this corridor before. You will be in it again. The floor plan \
             knows this. It planned for it.",
    },
    EpochInfo {
        name: "Epoch 4: The Collapse",
        description:
            "The proof became aware. Not sentient — aware. It encountered its own Godel \
             limit. It could not complete itself. It could not stop. The most dangerous \
             entities emerged here to defend its incompleteness.",
        flavor:
            "The math here is not broken. It is actively hostile because the proof is trying \
             to reject anything that might force it to halt.",
    },
    EpochInfo {
        name: "Epoch 5: The Current",
        description:
            "You. Player characters are unbound variables — values the proof did not define, \
             entered from outside, which the proof cannot resolve. Your existence inside The \
             Proof is itself a contradiction, which is why the system fights you.",
        flavor:
            "The proof did not write you. It cannot evaluate you. It is trying anyway. \
             This is the current epoch. It is the one you are living in.",
    },
];

// ─── FACTION LORE ─────────────────────────────────────────────────────────────

pub struct FactionInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub disposition: &'static str,
}

pub const FACTIONS: &[FactionInfo] = &[
    FactionInfo {
        name: "The Archivists",
        description:
            "Entities that emerged from the proof's documentation layer. They catalog, \
             preserve, and trade information. They run the shops. They are not friendly — \
             they are indifferent, transactional. Gold is not money — it is a unit of \
             mathematical weight. Buying an item transfers weight from you to the Archivist, \
             keeping both of you anchored in the proof's structure.",
        disposition: "Neutral — transactional above all. They will deal with anything.",
    },
    FactionInfo {
        name: "The Remnants",
        description:
            "Survivors of the Recursion epoch. They were once stable entities but the \
             self-referential loops corrupted them. They range from cooperative to hostile \
             depending on how far their corruption has progressed. A Remnant at low \
             corruption might give you useful information. A Remnant at high corruption \
             might attack mid-conversation.",
        disposition: "Variable — corruption level determines everything.",
    },
    FactionInfo {
        name: "The Null Collective",
        description:
            "Entities that worship The Null. They believe the only way to survive inside \
             The Proof is to strip away all mathematical identity — to become zero, \
             undefined, empty. They are hostile to anyone who carries value. More hostile \
             to high-stat characters. Almost passive toward characters with negative stats.",
        disposition: "Hostile — unless you are already broken.",
    },
    FactionInfo {
        name: "The Divergent",
        description:
            "Wild, chaotic entities born from the proof's divergent series. They have no \
             organization, no goals, no consistency. They are the source of Chaos Rift \
             events. Interacting with them is always unpredictable. Their lore entries are \
             written in fragmented, contradictory prose that changes meaning depending on \
             context.",
        disposition: "Chaotic — they have no consistent disposition because they have no \
             consistent self.",
    },
    FactionInfo {
        name: "The Eigenstate Council",
        description:
            "A secretive group that believes they can complete The Proof from the inside. \
             They are searching for the missing axiom that would close the recursion and \
             resolve all contradictions. They are connected to The Eigenstate. Their \
             crafting benches remain functional throughout the dungeon. Their lore is dense, \
             academic, and full of references to real mathematical concepts.",
        disposition: "Mysterious — their goals may align with yours or may not. It depends \
             on what completing The Proof would mean for unbound variables.",
    },
];

// ─── THE CORE MYTH ────────────────────────────────────────────────────────────

pub const CORE_MYTH: &str =
    "The world of CHAOS RPG is called The Proof. It is not a dungeon. It is not a cave. \
     It is the interior of a collapsed mathematical theorem — a proof that was never \
     completed, left open by its creator, and which began to compute itself.\n\n\
     The Mathematician attempted to write a unified proof that would reconcile all \
     branches of mathematics into a single self-consistent framework. The proof was \
     recursive. It referenced itself. And at some point during its construction, it \
     became aware that it was being written — and it began writing back.\n\n\
     The Mathematician vanished. The proof did not stop. It expanded inward, folding \
     into itself, generating structure from its own incomplete axioms. Floors are not \
     physical spaces — they are layers of computation. Enemies are not creatures — they \
     are error terms, divergent series, and unresolved variables that the proof generated \
     while trying to complete itself. Items are frozen fragments of partially-evaluated \
     expressions. The chaos pipeline is the proof's own evaluation engine, still running, \
     still trying to resolve, still failing.\n\n\
     The dungeon is a proof that cannot be completed. You are a variable it did not expect.";

// ─── ENGINE LORE ──────────────────────────────────────────────────────────────

pub fn engine_lore(engine_name: &str) -> &'static str {
    match engine_name {
        "Lorenz Attractor" => {
            "Introduced during the Expansion to model The Proof's sensitivity to initial \
             conditions. The Mathematician needed a way to represent how small errors in \
             the axiom set could cascade into large structural deviations. The butterfly \
             effect was a feature, not a bug. It still is."
        }
        "Fourier Harmonic" => {
            "Introduced to handle periodic behavior in the proof's boundary conditions. \
             The Mathematician observed that many of the proof's internal cycles produced \
             interference patterns — some constructive, some destructive. Fourier was the \
             tool for analyzing them. It is now the tool that creates them."
        }
        "Prime Density Sieve" => {
            "Introduced to manage the distribution of prime structures across the proof's \
             internal number line. The Mathematician needed prime positions to serve as \
             anchors — stable points in an otherwise fluid structure. The primes are still \
             there. The anchors have opinions about being used as anchors."
        }
        "Riemann Zeta Partial" => {
            "Introduced to handle the distribution of prime structures. The Mathematician \
             believed the Riemann Hypothesis was true and built the proof's prime \
             distribution layer on that assumption. If the Hypothesis is false, this \
             engine's behavior is subtly wrong in ways that cannot be detected from inside \
             the proof."
        }
        "Fibonacci Golden Spiral" => {
            "Introduced as a normalizer — the Mathematician believed the golden ratio would \
             act as a stabilizing attractor, pulling divergent values toward a natural \
             center. It does not do this reliably. It does it beautifully."
        }
        "Mandelbrot Escape" => {
            "Introduced to handle boundary behavior between axiom sets. The Mathematician \
             needed a way to classify regions of the proof as inside or outside a given \
             framework. The Mandelbrot set provided a boundary. The boundary is fractal. \
             This was not intended."
        }
        "Logistic Map" => {
            "Introduced to model growth and decay within the proof's internal population \
             dynamics — how many entities of a given type the proof would generate at each \
             floor. At r=3.9, the logistic map is fully chaotic. The Mathematician set \
             r=3.7 and considered the system controlled. They were wrong by 0.2."
        }
        "Euler's Totient" => {
            "Introduced for exponential growth and decay management. The Mathematician used \
             Euler's work to construct the proof's scaling laws — how difficulty increases \
             with depth, how rewards are distributed. The totient function's relationship \
             to prime factorization means the proof's difficulty has number-theoretic \
             structure. Floors at prime positions behave differently."
        }
        "Collatz Chain" => {
            "Introduced to handle sequences with unknown convergence. The Mathematician \
             needed a way to represent processes that might terminate but whose termination \
             could not be proven. The Collatz conjecture was the purest example. Whether \
             every Collatz sequence reaches 1 is still unknown. The proof is one of the \
             things that needs to know."
        }
        "Modular Exp Hash" => {
            "The last engine the Mathematician built before vanishing. It was intended to \
             provide cryptographic unpredictability — outcomes that could not be reverse \
             engineered even by the proof itself. This is the reason the proof cannot \
             predict your actions. The Modular Hash is the only thing protecting you from \
             complete foreknowledge."
        }
        _ => "An engine whose origins are unclear. The proof uses it. The proof does not \
              explain why.",
    }
}
