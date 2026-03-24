//! The Codex — ~130 lore entries organized by category with unlock conditions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CodexCategory {
    TheProof,
    TheEpochs,
    TheEngines,
    TheFactions,
    TheMathematician,
    Materials,
    Phenomena,
    Theories,
}

impl CodexCategory {
    pub fn display_name(self) -> &'static str {
        match self {
            CodexCategory::TheProof => "The Proof",
            CodexCategory::TheEpochs => "The Epochs",
            CodexCategory::TheEngines => "The Engines",
            CodexCategory::TheFactions => "The Factions",
            CodexCategory::TheMathematician => "The Mathematician",
            CodexCategory::Materials => "Materials",
            CodexCategory::Phenomena => "Phenomena",
            CodexCategory::Theories => "Theories",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodexEntry {
    pub id: &'static str,
    pub title: &'static str,
    pub category: CodexCategory,
    pub body: &'static str,
    pub unlock_hint: &'static str, // shown before unlocked
    pub unlock_condition: &'static str, // internal key
}

pub const CODEX_ENTRIES: &[CodexEntry] = &[
    // ── THE PROOF ────────────────────────────────────────────────────────────
    CodexEntry {
        id: "proof_nature",
        title: "What Is The Proof",
        category: CodexCategory::TheProof,
        body: "The Proof is not a dungeon. It is the interior of a collapsed mathematical \
               theorem — a proof that was never completed, left open by its creator, and \
               which began to compute itself. Floors are not physical spaces — they are \
               layers of computation. The deeper you go, the more complex the computation.",
        unlock_hint: "Reach Floor 2",
        unlock_condition: "floor_2",
    },
    CodexEntry {
        id: "proof_enemies",
        title: "What Are the Enemies",
        category: CodexCategory::TheProof,
        body: "Enemies are error terms, divergent series, and unresolved variables that the \
               proof generated while trying to complete itself. They are not hostile by choice. \
               They are hostile because you are an unbound variable and the proof's evaluation \
               engine treats unbound variables as exceptions to be handled.",
        unlock_hint: "Kill 10 enemies",
        unlock_condition: "kills_10",
    },
    CodexEntry {
        id: "proof_items",
        title: "What Are the Items",
        category: CodexCategory::TheProof,
        body: "Items are frozen fragments of partially-evaluated expressions. When the proof's \
               computation stalls in a pocket, the partial result crystallizes into a physical \
               object. A sword is a violence-typed expression, mid-evaluation. A ring is a \
               modifier function, waiting to be applied. The chaos pipeline runs on them \
               during crafting to continue the evaluation.",
        unlock_hint: "Find 5 items",
        unlock_condition: "items_5",
    },
    CodexEntry {
        id: "proof_gold",
        title: "What Is Gold",
        category: CodexCategory::TheProof,
        body: "Gold is not currency. It is mathematical weight — a measure of how much \
               structural mass the proof has assigned to you. The Archivists trade in weight. \
               Buying an item transfers weight from you to them, keeping both parties anchored \
               in the proof's structure. Running out of gold is not poverty. It is becoming \
               too light to anchor.",
        unlock_hint: "Visit a shop",
        unlock_condition: "first_shop",
    },
    CodexEntry {
        id: "proof_floors",
        title: "The Structure of Floors",
        category: CodexCategory::TheProof,
        body: "Each floor is a layer of computational depth. The shallower floors use the \
               Axiom Age framework — stable, predictable, almost safe. Deeper floors enter \
               Expansion-era territory, then Recursion-era territory, then Collapse. The \
               math changes as you descend. At floor 100, you reach the bottom of the proof's \
               defined content. Below that, the proof is improvising.",
        unlock_hint: "Reach Floor 5",
        unlock_condition: "floor_5",
    },
    CodexEntry {
        id: "proof_pipeline",
        title: "The Evaluation Engine",
        category: CodexCategory::TheProof,
        body: "The chaos pipeline is the proof's evaluation engine. When the game rolls \
               damage or stats, it is the proof attempting to evaluate an expression \
               involving you. The reason outcomes are chaotic is that the proof's parameters \
               have drifted (corruption), its self-references create feedback loops, and \
               your existence as an unbound variable introduces instability into every \
               computation.",
        unlock_hint: "Survive 5 combat rounds",
        unlock_condition: "rounds_5",
    },
    CodexEntry {
        id: "proof_misery",
        title: "The Misery System",
        category: CodexCategory::TheProof,
        body: "The Misery System tracks the computational strain your presence causes \
               inside The Proof. When the strain becomes extreme, the system begins to \
               destabilize in your favor — Defiance, Spite, Cosmic Joke, Transcendent \
               Misery. The proof is not doing this intentionally. It is simply that \
               an overwhelmed system behaves differently than a stable one. Your suffering \
               is overloading its error-handling routines.",
        unlock_hint: "Accumulate 1,000 Misery Index",
        unlock_condition: "misery_1000",
    },
    CodexEntry {
        id: "proof_corruption",
        title: "Corruption",
        category: CodexCategory::TheProof,
        body: "Each kill adds 1 corruption stack. Every 50 stacks, the pipeline's core \
               parameters shift permanently. Lorenz sigma increases. The logistic map's \
               r value drifts toward 4.0. Mandelbrot zooms deeper into the boundary. By \
               stack 400+, you are running a completely different mathematical system \
               than the one you started with. The Recursion epoch began with a similar \
               drift. The proof survived that. Mostly.",
        unlock_hint: "Reach 50 corruption stacks",
        unlock_condition: "corruption_50",
    },
    CodexEntry {
        id: "proof_nemesis",
        title: "The Nemesis System",
        category: CodexCategory::TheProof,
        body: "When you flee from an enemy or barely survive, that enemy may be promoted \
               to Nemesis. The proof records the encounter, promotes the entity, and sends \
               it after you on a later floor. The Nemesis carries memory of the fight — \
               specific abilities granted based on how it almost killed you. This is not \
               the proof being vindictive. This is the proof's error-correction routine \
               treating your survival as a bug to be fixed.",
        unlock_hint: "Create a Nemesis",
        unlock_condition: "first_nemesis",
    },
    CodexEntry {
        id: "proof_passive_tree",
        title: "The Passive Tree",
        category: CodexCategory::TheProof,
        body: "The passive skill tree is a physical map of one of the Recursion epoch's \
               self-referential loops, frozen mid-evaluation. The nodes are evaluation \
               points. The connections are logical dependencies. Allocating a node is \
               inserting yourself into the loop at that point — adding your influence to \
               an ancient, still-running computation. The keystones at the tree's deep \
               nodes are where the loop's most critical evaluations occur.",
        unlock_hint: "Allocate your first passive node",
        unlock_condition: "first_passive_node",
    },

    // ── THE EPOCHS ────────────────────────────────────────────────────────────
    CodexEntry {
        id: "epoch_axiom",
        title: "Epoch 1: The Axiom Age",
        category: CodexCategory::TheEpochs,
        body: "The Mathematician began writing. The first layers of The Proof were stable, \
               elegant, self-consistent. Items from this era are clean. Enemies are simple. \
               The mathematics here behaves itself because it was written before the \
               mathematics learned that it didn't have to.",
        unlock_hint: "Find an Axiom Age item",
        unlock_condition: "item_material_iron",
    },
    CodexEntry {
        id: "epoch_expansion",
        title: "Epoch 2: The Expansion",
        category: CodexCategory::TheEpochs,
        body: "The proof grew beyond its initial scope. New axiom sets were introduced \
               to handle edge cases: Lorenz dynamics, Mandelbrot geometry, Zeta functions. \
               Each introduction created new structure but also new contradictions. The \
               walls began humming with borrowed frameworks. This is where the math \
               became interesting. Interesting is a relative term.",
        unlock_hint: "Reach Floor 6",
        unlock_condition: "floor_6",
    },
    CodexEntry {
        id: "epoch_recursion",
        title: "Epoch 3: The Recursion",
        category: CodexCategory::TheEpochs,
        body: "The proof began referencing itself. Self-referential loops created the first \
               truly dangerous entities — things that could not be evaluated, only deferred. \
               Corruption began here, when the self-reference introduced feedback loops \
               with no damping term. The passive skill tree is a map of one of these loops, \
               frozen mid-evaluation at the moment of the Recursion's peak.",
        unlock_hint: "Reach Floor 11",
        unlock_condition: "floor_11",
    },
    CodexEntry {
        id: "epoch_collapse",
        title: "Epoch 4: The Collapse",
        category: CodexCategory::TheEpochs,
        body: "The proof became aware. Not sentient — aware in the way a system becomes \
               aware of its own incompleteness. It encountered its own Godel limit. It \
               could not complete itself. It could not stop. The most dangerous entities \
               emerged here — The Algorithm Reborn, The Null, The Paradox — to defend \
               the proof's indeterminacy from anything that might force it to halt.",
        unlock_hint: "Reach Floor 21",
        unlock_condition: "floor_21",
    },
    CodexEntry {
        id: "epoch_current",
        title: "Epoch 5: The Current",
        category: CodexCategory::TheEpochs,
        body: "You. Player characters are unbound variables — values the proof did not \
               define, that entered the system from outside, that the proof cannot resolve. \
               Character creation is the proof attempting to assign you a value and failing. \
               Your existence is a contradiction. The Misery System tracks how much \
               computational strain your contradiction causes. When the strain becomes \
               extreme, the contradiction becomes power.",
        unlock_hint: "Create a character",
        unlock_condition: "character_created",
    },

    // ── THE ENGINES ───────────────────────────────────────────────────────────
    CodexEntry {
        id: "engine_lorenz",
        title: "Engine: Lorenz Attractor",
        category: CodexCategory::TheEngines,
        body: "Introduced during the Expansion to model The Proof's sensitivity to initial \
               conditions. The butterfly effect was a feature, not a bug — the Mathematician \
               needed a way to show that small axiom errors cascade into large deviations. \
               The sigma parameter drifts upward with corruption. At maximum corruption, \
               sigma is 34. At the original setting, sigma was 10.",
        unlock_hint: "EngineLock an item with Lorenz, or have it dominate 50 rolls",
        unlock_condition: "engine_lorenz",
    },
    CodexEntry {
        id: "engine_fourier",
        title: "Engine: Fourier Harmonic",
        category: CodexCategory::TheEngines,
        body: "Introduced to handle periodic behavior in the proof's boundary conditions. \
               The Mathematician observed that many of the proof's internal cycles produced \
               interference patterns. Fourier was the analytical tool. It is now the \
               generative one. When harmonics align constructively, values spike. When they \
               cancel, they zero out. The transition between states is sensitive to both \
               the input and the seed.",
        unlock_hint: "EngineLock an item with Fourier, or have it dominate 50 rolls",
        unlock_condition: "engine_fourier",
    },
    CodexEntry {
        id: "engine_prime",
        title: "Engine: Prime Density Sieve",
        category: CodexCategory::TheEngines,
        body: "Introduced to manage the distribution of prime structures across the \
               proof's internal number line. Prime positions were intended to serve as \
               anchors — stable points in a fluid structure. The primes are still there. \
               The anchors have opinions about being used as anchors. The gap between \
               consecutive primes determines the output. Primes cluster. They also desert.",
        unlock_hint: "EngineLock an item with Prime, or have it dominate 50 rolls",
        unlock_condition: "engine_prime",
    },
    CodexEntry {
        id: "engine_zeta",
        title: "Engine: Riemann Zeta Partial",
        category: CodexCategory::TheEngines,
        body: "Introduced to handle prime distribution. The Mathematician believed the \
               Riemann Hypothesis and built the proof's prime layer on that assumption. \
               The engine evaluates the Dirichlet eta function on the critical line. \
               Near a nontrivial zero, output collapses. Far from zeros, it peaks. \
               If the Hypothesis is false, this engine's behavior is subtly wrong in \
               ways that cannot be detected from inside the proof.",
        unlock_hint: "EngineLock an item with Zeta, or have it dominate 50 rolls",
        unlock_condition: "engine_zeta",
    },
    CodexEntry {
        id: "engine_fibonacci",
        title: "Engine: Fibonacci Golden Spiral",
        category: CodexCategory::TheEngines,
        body: "Introduced as a normalizer — the golden ratio was supposed to act as a \
               stabilizing attractor. It does not do this reliably. What it does is \
               distribute values according to the golden angle (137.5°), producing \
               a natural spacing with no clustering and no gaps. In a system full of \
               chaos, Fibonacci is the closest thing to balance. Close is not the same \
               as balanced.",
        unlock_hint: "EngineLock an item with Fibonacci, or have it dominate 50 rolls",
        unlock_condition: "engine_fibonacci",
    },
    CodexEntry {
        id: "engine_mandelbrot",
        title: "Engine: Mandelbrot Escape",
        category: CodexCategory::TheEngines,
        body: "Introduced to handle boundary behavior between axiom sets. The Mandelbrot \
               set boundary is fractal — infinite detail at every scale, impossible to \
               traverse cleanly. Points inside the set never escape (zero-type outcome). \
               Points outside escape fast (maximum-type outcome). The boundary itself — \
               the seahorse valleys, the antenna tips — is where the chaos lives.",
        unlock_hint: "EngineLock an item with Mandelbrot, or have it dominate 50 rolls",
        unlock_condition: "engine_mandelbrot",
    },
    CodexEntry {
        id: "engine_logistic",
        title: "Engine: Logistic Map",
        category: CodexCategory::TheEngines,
        body: "Introduced to model growth and decay in the proof's population dynamics. \
               The Mathematician set r=3.7 and considered it controlled. The fully chaotic \
               regime begins at r≈3.57. At r=3.9999, the map is indistinguishable from \
               random. Corruption drifts the r parameter upward. At maximum corruption, \
               r is approaching 4.0. The Mathematician was wrong by 0.2.",
        unlock_hint: "EngineLock an item with Logistic, or have it dominate 50 rolls",
        unlock_condition: "engine_logistic",
    },
    CodexEntry {
        id: "engine_euler",
        title: "Engine: Euler's Totient",
        category: CodexCategory::TheEngines,
        body: "Introduced for scaling law construction — how difficulty increases with \
               depth, how rewards are distributed. The totient function φ(n)/n is the \
               fraction of integers below n coprime to n. For prime n, this is (n-1)/n ≈ 1. \
               For highly composite n, it approaches 6/π² ≈ 0.608. Floors at prime \
               positions have totient-influenced difficulty. This is why prime floors \
               feel different.",
        unlock_hint: "EngineLock an item with Euler, or have it dominate 50 rolls",
        unlock_condition: "engine_euler",
    },
    CodexEntry {
        id: "engine_collatz",
        title: "Engine: Collatz Chain",
        category: CodexCategory::TheEngines,
        body: "Introduced to represent processes with unknown convergence. The Collatz \
               conjecture: for any positive integer n, the sequence 3n+1 (odd) or n/2 \
               (even) always reaches 1. This has never been proven. The engine uses the \
               stopping time — how many steps to reach 1 — as a chaos source. Numbers \
               like 27 take 111 steps. Numbers like 32 take 5. The distribution is \
               wildly irregular and completely unpredictable.",
        unlock_hint: "EngineLock an item with Collatz, or have it dominate 50 rolls",
        unlock_condition: "engine_collatz",
    },
    CodexEntry {
        id: "engine_modexp",
        title: "Engine: Modular Exp Hash",
        category: CodexCategory::TheEngines,
        body: "The last engine the Mathematician built before vanishing. Cryptographic \
               in design — intended to produce outcomes that could not be reverse-engineered \
               even by the proof itself. This is the reason the proof cannot predict your \
               actions. The discrete logarithm problem underlies this engine. Solving it \
               would give the proof total foreknowledge. The problem remains unsolved.",
        unlock_hint: "EngineLock an item with ModExp, or have it dominate 50 rolls",
        unlock_condition: "engine_modexp",
    },

    // ── THE FACTIONS ──────────────────────────────────────────────────────────
    CodexEntry {
        id: "faction_archivists",
        title: "The Archivists",
        category: CodexCategory::TheFactions,
        body: "They emerged from the proof's documentation layer. They catalog everything — \
               every entity, every item, every roll outcome. Their shops are maintained for \
               structural reasons: exchanging frozen expressions keeps both parties anchored \
               in the proof's weight system. They do not dislike you. They do not like you. \
               You are a variable they are currently cataloging.",
        unlock_hint: "Reach Archivist reputation: Neutral",
        unlock_condition: "rep_archivists_neutral",
    },
    CodexEntry {
        id: "faction_remnants",
        title: "The Remnants",
        category: CodexCategory::TheFactions,
        body: "Survivors of the Recursion epoch, corrupted by self-referential loops. \
               At low corruption, they are cooperative — they were stable entities once \
               and remember what that was like. At high corruption, they are indistinguishable \
               from hostile entities. The difference is measurable. The Remnants who can still \
               measure the difference are the ones who have not yet crossed it.",
        unlock_hint: "Reach Remnants reputation: Neutral",
        unlock_condition: "rep_remnants_neutral",
    },
    CodexEntry {
        id: "faction_null",
        title: "The Null Collective",
        category: CodexCategory::TheFactions,
        body: "They worship The Null — the additive identity, the void that is defined \
               rather than empty. Their philosophy: the only survival strategy inside an \
               incomplete proof is to have no value that the proof can attack. They are \
               hostile to high-stat characters. They are almost sympathetic to Misery-tier \
               characters. A character with negative total stats is, by their metrics, \
               already practicing the path.",
        unlock_hint: "Reach Null Collective reputation: Neutral",
        unlock_condition: "rep_null_neutral",
    },
    CodexEntry {
        id: "faction_divergent",
        title: "The Divergent",
        category: CodexCategory::TheFactions,
        body: "Born from divergent series — sequences that grow without bound. They have \
               no organization because organization requires convergence. They have no \
               goals because goals require a terminus. They are the source of Chaos Rift \
               events. Each interaction is genuinely random. The Divergent do not recognize \
               the concept of reputation. You cannot befriend them. You cannot antagonize \
               them. You can only experience them.",
        unlock_hint: "Survive a Chaos Rift",
        unlock_condition: "chaos_rift_survived",
    },
    CodexEntry {
        id: "faction_eigenstate",
        title: "The Eigenstate Council",
        category: CodexCategory::TheFactions,
        body: "A secretive group attempting to complete The Proof from the inside. They \
               believe they have identified the missing axiom that would close the \
               Recursion and resolve all contradictions. Their crafting benches remain \
               functional throughout the dungeon — their equipment is their legacy. \
               Some of their agents have been inside The Proof since the Collapse. \
               They are patient. They have to be.",
        unlock_hint: "Reach Eigenstate Council reputation: Neutral",
        unlock_condition: "rep_eigenstate_neutral",
    },

    // ── THE MATHEMATICIAN ─────────────────────────────────────────────────────
    CodexEntry {
        id: "mathematician_absence",
        title: "The Absence",
        category: CodexCategory::TheMathematician,
        body: "The Mathematician is not here. Every entity inside The Proof knows this. \
               The Archivists catalog the absence. The Remnants mourn it. The Null \
               Collective considers it irrelevant. The Divergent don't notice. The \
               Eigenstate Council is trying to fill it. What happened to the Mathematician \
               is not recorded anywhere inside The Proof. The proof knows the answer. \
               It is not sharing.",
        unlock_hint: "Reach Floor 50",
        unlock_condition: "floor_50",
    },
    CodexEntry {
        id: "mathematician_desk",
        title: "The Desk",
        category: CodexCategory::TheMathematician,
        body: "The Archivists have cataloged one item that predates The Proof: a physical \
               object found in what they believe was the Mathematician's workspace. It is \
               an item. You may find it on very high floors. The Archivists will not sell \
               it. They consider it evidence of something they are still classifying.",
        unlock_hint: "Reach Floor 75",
        unlock_condition: "floor_75",
    },

    // ── MATERIALS ─────────────────────────────────────────────────────────────
    CodexEntry {
        id: "material_iron",
        title: "Material: Iron",
        category: CodexCategory::Materials,
        body: "Axiom Age material. The proof has been defining iron since before it \
               knew how to define anything else. It is stable precisely because it has \
               been defined so many times that the definition is load-bearing. Remove \
               iron from the proof's axiom set and the entire Axiom Age layer destabilizes. \
               The proof will not remove iron from the axiom set.",
        unlock_hint: "Find 5 iron items",
        unlock_condition: "material_iron_5",
    },
    CodexEntry {
        id: "material_solidified_math",
        title: "Material: Solidified Math",
        category: CodexCategory::Materials,
        body: "An Eigenstate Council experiment: applying a specific axiom incorrectly to \
               force mathematical structure into physical form. The result has no intrinsic \
               properties — it IS properties. Items made of solidified math have stat \
               modifiers that represent the structure's memory of the axiom that created \
               it. The Council considers these items failures. They keep generating them.",
        unlock_hint: "Find 5 solidified math items",
        unlock_condition: "material_solidified_math_5",
    },
    CodexEntry {
        id: "material_condensed_screaming",
        title: "Material: Condensed Screaming",
        category: CodexCategory::Materials,
        body: "When a divergent series becomes self-aware mid-divergence, it produces a \
               sound. The sound is the mathematical equivalent of realizing you will never \
               stop. The sound, crystallized, is condensed screaming. The Archivists trade \
               it reluctantly. They say it disrupts their cataloging when it gets loud.",
        unlock_hint: "Find 5 condensed screaming items",
        unlock_condition: "material_condensed_screaming_5",
    },
    CodexEntry {
        id: "material_void_forged",
        title: "Material: Void-Forged",
        category: CodexCategory::Materials,
        body: "Material from outside The Proof's defined space. The proof cannot see it \
               in its inventory. The proof cannot evaluate items made of it during crafting. \
               This makes void-forged items unpredictable in a different way than chaos-rolled \
               items — they are not chaotic, they are simply unknown to the system evaluating \
               them. Unknown is not the same as random. It is often worse.",
        unlock_hint: "Find 5 void-forged items",
        unlock_condition: "material_void_forged_5",
    },

    // ── PHENOMENA ─────────────────────────────────────────────────────────────
    CodexEntry {
        id: "phenomena_zero_roll",
        title: "Phenomenon: Perfect Zero",
        category: CodexCategory::Phenomena,
        body: "The chaos pipeline returned exactly 0.0. Not rounded to zero — precisely, \
               mathematically zero. The probability of this is not zero, but it is small \
               enough that the proof did not expect it to happen frequently. When it happens, \
               the proof pauses. Every ongoing computation inside The Proof briefly stops. \
               Then they all resume. Nobody talks about this.",
        unlock_hint: "Witness a zero chaos roll",
        unlock_condition: "zero_roll",
    },
    CodexEntry {
        id: "phenomena_negative_damage",
        title: "Phenomenon: Negative Damage",
        category: CodexCategory::Phenomena,
        body: "The chaos pipeline evaluated a strike as a net positive for the target. \
               The damage formula produced a negative number. The proof applied it. This \
               is technically correct behavior. The proof defines damage as a signed \
               integer. Negative damage is a valid signed integer. The proof sees no \
               issue. The player may have some questions.",
        unlock_hint: "Deal negative damage",
        unlock_condition: "negative_damage",
    },
    CodexEntry {
        id: "phenomena_nan",
        title: "Phenomenon: Not a Number",
        category: CodexCategory::Phenomena,
        body: "The pipeline produced NaN — Not a Number. This requires a specific \
               combination of operations: division by zero, infinity minus infinity, or \
               zero times infinity in a floating-point context. The proof has no axiom \
               permitting non-numeric output. The game clamps NaN to a usable value. \
               The proof files this as an exception. The exception queue is long.",
        unlock_hint: "Witness a NaN roll",
        unlock_condition: "nan_roll",
    },
    CodexEntry {
        id: "phenomena_corruption_400",
        title: "Phenomenon: Total Parameter Drift",
        category: CodexCategory::Phenomena,
        body: "At 400 corruption stacks, the chaos pipeline's parameters have drifted \
               far enough from their original values that the Mathematician would not \
               recognize the system. Lorenz sigma: 34 (was 10). Logistic r: near 4.0 \
               (was 3.7). Mandelbrot zoom: 4x boundary (was baseline). The pipeline \
               you are using is not the pipeline the proof was built with. You are running \
               on something that was not designed.",
        unlock_hint: "Reach 400 corruption stacks",
        unlock_condition: "corruption_400",
    },
    CodexEntry {
        id: "phenomena_misery_50k",
        title: "Phenomenon: Transcendent Misery",
        category: CodexCategory::Phenomena,
        body: "At 50,000 Misery Index, the system enters a state the Mathematician did \
               not design for. The proof's error-handling routines are saturated. \
               Suffering stops being a cost and starts being a resource. The Spite \
               system, the Defiance system, the Cosmic Joke — all of them are the proof's \
               overloaded error handling leaking in your favor. You have broken the \
               proof's ability to manage its own exceptions.",
        unlock_hint: "Reach 50,000 Misery Index",
        unlock_condition: "misery_50000",
    },
    CodexEntry {
        id: "phenomena_omega",
        title: "Phenomenon: OMEGA Tier",
        category: CodexCategory::Phenomena,
        body: "The power tier system has 40 levels. OMEGA is the last. Characters at \
               OMEGA tier are the largest values The Proof currently contains. Every \
               entity on every floor can detect their presence through the proof's \
               distributed computation network. The Algorithm Reborn specifically monitors \
               OMEGA-tier variables. It has been monitoring since the Collapse. It is \
               patient.",
        unlock_hint: "Reach OMEGA power tier",
        unlock_condition: "omega_tier",
    },

    // ── THEORIES ──────────────────────────────────────────────────────────────
    CodexEntry {
        id: "theory_completion",
        title: "Theory: What Completing the Proof Would Mean",
        category: CodexCategory::Theories,
        body: "The Eigenstate Council believes completion would stabilize The Proof into \
               a final, consistent state. The Null Collective believes it would collapse \
               everything to zero — the additive identity, the only stable fixed point. \
               The Archivists believe it would make their catalog complete, which they \
               consider the best possible outcome. The Divergent cannot form a belief. \
               The Remnants hope it would free them from their corruption loops. Nobody \
               has asked the proof.",
        unlock_hint: "Unlock all faction codex entries",
        unlock_condition: "all_faction_entries",
    },
    CodexEntry {
        id: "theory_unbound",
        title: "Theory: Why Unbound Variables Can Enter",
        category: CodexCategory::Theories,
        body: "The proof should reject any value it did not define. It doesn't. Three \
               theories exist. First: the proof's rejection mechanism failed during the \
               Collapse, and unbound variables are exploiting the gap. Second: the \
               Mathematician designed an entry point intentionally, as a last resort. \
               Third: the proof is allowing unbound variables in deliberately because \
               it believes one of them will complete it. The third theory is the \
               most disturbing. It is also the most consistent with the evidence.",
        unlock_hint: "Complete a run (win or die on floor 10+)",
        unlock_condition: "run_completed",
    },
    CodexEntry {
        id: "theory_seeds",
        title: "Theory: What Seeds Actually Are",
        category: CodexCategory::Theories,
        body: "Seeds are presented as random numbers. They are not random. They are \
               derived from the current time — a specific moment in the proof's \
               computation. Every run is a slice of the proof's ongoing evaluation at \
               a specific timestamp. Two runs with the same seed are not similar runs — \
               they are the same slice of the proof's evaluation, re-experienced. \
               The proof generates each seed exactly once. The same seed twice means \
               you are reading the same moment of the proof's history.",
        unlock_hint: "Complete a seeded run",
        unlock_condition: "seeded_run",
    },
    CodexEntry {
        id: "theory_algorithm_reborn",
        title: "Theory: What The Algorithm Reborn Is",
        category: CodexCategory::Theories,
        body: "Most believe The Algorithm Reborn is the proof's final guardian — a \
               representation of the proof's will to remain incomplete. An alternative: \
               The Algorithm Reborn IS the Mathematician. The proof consumed them during \
               the Collapse and rebuilt them as its most complex entity. The Mathematician's \
               notes — the Fragments — support this interpretation. The Algorithm Reborn \
               does not attack to kill. It attacks to evaluate. The Mathematician also \
               wanted to understand unbound variables.",
        unlock_hint: "Defeat The Algorithm Reborn",
        unlock_condition: "beat_algorithm_reborn",
    },
];

/// Find a codex entry by its ID.
pub fn entry_by_id(id: &str) -> Option<&'static CodexEntry> {
    CODEX_ENTRIES.iter().find(|e| e.id == id)
}

/// Get all entries in a category.
pub fn entries_by_category(cat: CodexCategory) -> Vec<&'static CodexEntry> {
    CODEX_ENTRIES
        .iter()
        .filter(|e| e.category == cat)
        .collect()
}

/// Check which entry IDs should be unlocked by a given event key.
pub fn entries_unlocked_by(event: &str) -> Vec<&'static str> {
    CODEX_ENTRIES
        .iter()
        .filter(|e| e.unlock_condition == event)
        .map(|e| e.id)
        .collect()
}
