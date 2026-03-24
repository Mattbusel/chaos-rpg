//! Item flavor text — rarity descriptions, material lore, suffix lore.

// ─── RARITY FLAVOR ────────────────────────────────────────────────────────────

/// Returns flavor text for an item based on its rarity name and a seed for variety.
pub fn rarity_flavor(rarity: &str, seed: u64) -> &'static str {
    match rarity {
        "Common" => {
            const LINES: &[&str] = &[
                "It works. Don't ask how.",
                "Standard issue. The proof made billions of these.",
                "Unremarkable. The most dangerous word in mathematics.",
                "Functional. The proof does not care about functional.",
                "Mass-produced during the Axiom Age. Indistinguishable from all the others.",
            ];
            LINES[(seed % LINES.len() as u64) as usize]
        }
        "Uncommon" => {
            const LINES: &[&str] = &[
                "Someone carried this before you. The proof doesn't remember who.",
                "The edges glow faintly when you're near a rift.",
                "Stamped on the inside: 'AXIOM-COMPLIANT. DO NOT CORRUPT.'",
                "It has survived at least one owner. The previous owner did not.",
                "Better than most. Which means the proof will notice it.",
            ];
            LINES[(seed % LINES.len() as u64) as usize]
        }
        "Rare" => {
            const LINES: &[&str] = &[
                "Expansion-era craft. You can feel the Lorenz attractor humming inside it.",
                "This was stable once, before the Recursion. Now it drifts.",
                "The Archivists won't sell these. They say the weight is wrong.",
                "Three stat modifiers. The proof does not normally allow three.",
                "Someone asked for more than the standard formula allows. They got it.",
            ];
            LINES[(seed % LINES.len() as u64) as usize]
        }
        "Epic" => {
            const LINES: &[&str] = &[
                "It solves a differential equation when you swing it. You don't know which one.",
                "The previous owner is listed as 'UNDEFINED.' This is not reassuring.",
                "It whispers in Mandelbrot coordinates. You're starting to understand.",
                "The proof classified this as a paradox and then kept generating it anyway.",
                "Four modifiers. The formula only allows three. The proof has not commented.",
            ];
            LINES[(seed % LINES.len() as u64) as usize]
        }
        "Legendary" => {
            const LINES: &[&str] = &[
                "Forged during the Collapse by an entity that no longer converges. It knew \
                 it was dying and it made this anyway. The stats are its last stable output.",
                "The Eigenstate Council classified this as 'proof-adjacent.' They were afraid of it.",
                "When the Mathematician vanished, this was on the desk. It is the only physical \
                 object that predates The Proof.",
                "Five modifiers. This should be impossible. The proof has filed an exception report.",
                "A senior Archivist refused to catalog this. They transferred to a different floor.",
            ];
            LINES[(seed % LINES.len() as u64) as usize]
        }
        "Mythical" => {
            const LINES: &[&str] = &[
                "The proof generated this once, realized what it had made, and tried to delete it. \
                 It failed. Here it is.",
                "The Null Collective sent a delegation to destroy this item. The delegation did not \
                 return.",
                "It exists in a superposition of six different items. You are holding all of them.",
                "The chaos pipeline behaves differently when this is equipped. Differently, not better.",
                "An Archivist offered to buy this for everything it owned. It would not say why.",
            ];
            LINES[(seed % LINES.len() as u64) as usize]
        }
        "???" => {
            const LINES: &[&str] = &[
                "The proof does not have a classification for this. It has created one. \
                 The classification is: 'stop.'",
                "The Archivists have never seen one of these. They have checked their records \
                 back to the Axiom Age. Nothing.",
                "It has no rarity tier because no rarity tier was designed to contain it.",
                "You can feel the proof rewriting its own evaluation rules to accommodate this.",
            ];
            LINES[(seed % LINES.len() as u64) as usize]
        }
        "◈ ARTIFACT ◈" => {
            const LINES: &[&str] = &[
                "Stop. Look at what you're holding. The proof defined this once, at the very \
                 edge of what it could compute, and it has not been able to define anything \
                 like it since. This is not an item. It is a boundary condition. Treat it \
                 accordingly.",
                "This item should not exist. The proof has no axiom that permits it. And yet.",
                "You can feel the chaos pipeline bending around it. Not through it. Around. \
                 As if it's afraid.",
                "The Algorithm Reborn created this during Phase 3 of a fight that happened \
                 before you were born. It is waiting for you to finish that fight.",
                "The Eigenstate Council has one entry about this item. The entry says: \
                 'found.' Nothing else.",
            ];
            LINES[(seed % LINES.len() as u64) as usize]
        }
        _ => "The proof generated this. It does not always explain why.",
    }
}

// ─── MATERIAL LORE ────────────────────────────────────────────────────────────

/// Returns lore text for an item material.
pub fn material_lore(material: &str) -> Option<&'static str> {
    match material {
        "iron" => Some(
            "Axiom Age. Stable, predictable, boring. The first material the proof ever defined. \
             It has been defining it ever since because it has no reason to stop.",
        ),
        "wood" => Some(
            "From the proof's early attempts to model organic growth. Trees are recursion \
             made visible. The proof understood recursion before it understood the danger.",
        ),
        "steel" => Some(
            "An Expansion-era refinement of iron. More complex, more stable, slightly more \
             dangerous. The proof was pleased with itself when it derived steel. That was before.",
        ),
        "mithril" => Some(
            "A material the proof derived from extrapolating iron's properties into higher \
             dimensions. It doesn't exist outside The Proof. Inside The Proof, it is the \
             most cooperative substance available.",
        ),
        "antimatter" => Some(
            "From the Null Collective's territory. It cancels what it touches. Handle \
             accordingly. The Null Collective uses it as currency. They consider this logical.",
        ),
        "condensed screaming" => Some(
            "Crystallized output from a divergent series that became self-aware mid-divergence. \
             It knows it's infinite. It's not happy about it. The sound it makes is the sound \
             of a sequence realizing it will never terminate.",
        ),
        "solidified math" => Some(
            "Raw mathematical structure, forced into physical form by an axiom the Eigenstate \
             Council applied incorrectly. It has no properties. It IS properties. Every stat \
             modifier on an item made of this is the structure remembering what it was.",
        ),
        "dark matter" => Some(
            "Invisible to the proof's standard evaluation. The proof can't see it. That's the \
             point. The Eigenstate Council theorizes it comes from outside the proof's defined \
             space.",
        ),
        "recycled prayers" => Some(
            "Collected by the Archivists from entities that tried to petition the Mathematician \
             for rescue. They went unanswered. They still carry the hoping. The hoping is \
             surprisingly durable.",
        ),
        "prime-factored obsidian" => Some(
            "Material that can only exist at prime-number coordinates in the proof's structure. \
             It was found at position 104729. The next one will be at 104743. The Archivists \
             know exactly where to look.",
        ),
        "Turing-complete leather" => Some(
            "Material capable of arbitrary computation. Given enough time, this armor could \
             simulate itself. It has not tried this yet. The Eigenstate Council strongly \
             recommends it not try this.",
        ),
        "bottled lightning" => Some(
            "Captured energy from a Lorenz attractor's chaotic orbit. It doesn't strike the \
             same place twice. It doesn't strike the same place once. The previous owner was \
             very confused about where they were standing.",
        ),
        "superposition glass" => Some(
            "Material from the Eigenstate's domain. It is both broken and unbroken until you \
             check. The Eigenstate does not consider this a flaw.",
        ),
        "non-euclidean bone" => Some(
            "From entities that exist in geometries the proof shouldn't contain. The angles \
             are wrong. All of them. Count the sides if you need to convince yourself.",
        ),
        "eigenstate alloy" => Some(
            "An alloy that exists in two states simultaneously: extremely hard and extremely \
             soft. Combat resolves which state it's in. The resolution is not always \
             consistent.",
        ),
        "void-forged" => Some(
            "Material from beyond the proof's defined space. Where did this come from? The \
             proof has no answer. The proof always has an answer. This is notable.",
        ),
        "decompiled soul" => Some(
            "A Remnant, reduced to component values and recompiled as material. They agreed \
             to this. At the time, they believed it was a form of completion. The Archivists \
             disagree but continue to sell the results.",
        ),
        "crystallized luck" => Some(
            "Luck, in the proof's framework, is the residual probability mass after all \
             defined outcomes are assigned. This is luck made solid. It has opinions about \
             where it wants to end up.",
        ),
        "weaponized optimism" => Some(
            "An Expansion-era experiment. The Mathematician briefly believed the proof could \
             be completed through sheer conviction. This material is what remains of that \
             belief. It is still trying.",
        ),
        "deterministic void" => Some(
            "Empty space that the proof has claimed ownership over. It is not nothing — it \
             is defined emptiness. The difference is significant. Undefined emptiness would \
             be much more dangerous.",
        ),
        _ => None,
    }
}

// ─── SUFFIX LORE ──────────────────────────────────────────────────────────────

/// Returns lore text for an item name adjective/suffix.
pub fn suffix_lore(suffix: &str) -> Option<&'static str> {
    match suffix {
        "of the Forgotten" => Some(
            "Belonged to an entity the proof garbage-collected. The entity is gone. The item \
             persists. Objects outlast their owners inside The Proof.",
        ),
        "of the Omega Constant" => Some(
            "Tuned to the last mathematical constant the proof can evaluate before its own \
             framework breaks down. The Omega constant is the proof's edge. This item was \
             made at that edge.",
        ),
        "of Infinite Regret" => Some(
            "From an entity that computed its own mortality and had time to process the \
             implications. The regret is encoded in the item's structure. It affects nothing \
             mechanically. It affects everything existentially.",
        ),
        "of Mild Inconvenience" => Some(
            "The proof's attempt at humor. It does not understand humor. This is its best \
             effort. The mild inconvenience is real.",
        ),
        "of Absolute Tuesday" => Some(
            "References a temporal anomaly in the proof where all time coordinates collapsed \
             to a single value. That value was a Tuesday. No one knows which Tuesday. The \
             item knows. It's not saying.",
        ),
        "of Suspicious Origin" => Some(
            "The Archivists flagged this. They won't say why. The flag is still active. \
             The reason has been classified at a tier you don't have access to.",
        ),
        "of the Last Algorithm" => Some(
            "From the Mathematician's final work before vanishing. The Modular Exponential \
             Hash was the last thing they built. Items bearing this suffix carry a residue \
             of that final session.",
        ),
        "of Someone Else" => Some(
            "The proof assigned this to another variable. You have it now. The other \
             variable is aware of this. It is not pleased.",
        ),
        "of Non-Euclidean Design" => Some(
            "It has more sides than it should. You've counted twice. The count changes \
             depending on which side you start from.",
        ),
        "of Schrödinger" => Some(
            "Its stats are undefined until equipped. Then they're also undefined, but louder. \
             The act of equipping it causes a collapse. The collapsed state is not always \
             the one you wanted.",
        ),
        "the Unbreakable (breaks immediately)" => Some(
            "The proof's type system said it couldn't break. The proof's type system was \
             wrong. The naming was applied before the first test. The testing was not \
             conducted afterward.",
        ),
        "of Mathematical Inevitability" => Some(
            "Given the initial conditions of the proof, this item was always going to be \
             here. You were always going to find it. The chaos pipeline confirms this. \
             The chaos pipeline confirms everything.",
        ),
        "of the Lorenz Attractor" => Some(
            "Shaped by butterfly-effect cascade. The item's current form is the result of \
             a tiny input variation propagated through the Expansion epoch. Change one \
             axiom in the Axiom Age and this would be something entirely different.",
        ),
        "beyond the Mandelbrot Set" => Some(
            "This item's stats were generated outside the fractal boundary. What exists \
             beyond the Mandelbrot set escapes immediately and never returns. These stat \
             values escaped. They are here under protest.",
        ),
        "of the Prime Manifold" => Some(
            "Found at a prime-number position in the proof's structure. The distribution \
             of items at prime positions follows the prime number theorem — approximately. \
             The deviation from approximation is what makes these items interesting.",
        ),
        _ => None,
    }
}
