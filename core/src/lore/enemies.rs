//! Enemy lore entries — unlocked on first encounter.

pub struct EnemyLore {
    pub name: &'static str,
    pub description: &'static str,
    pub epoch: &'static str,
}

pub const ENEMY_LORE: &[EnemyLore] = &[
    EnemyLore {
        name: "Divergence Tick",
        description: "A tiny error term. Individually harmless. They shouldn't be able to \
            coordinate. They're trying. The proof generated them to handle rounding errors \
            in the Expansion epoch. The rounding errors learned to move.",
        epoch: "Expansion",
    },
    EnemyLore {
        name: "Entropy Phantom",
        description: "A variable that lost its assignment. It wanders the proof looking for \
            a value to inhabit. Your value. It wants your value. Not because it's hostile — \
            because it's empty and emptiness seeks definition.",
        epoch: "Recursion",
    },
    EnemyLore {
        name: "Convergent Golem",
        description: "Built from series that actually converge. Stable. Predictable. In a \
            world of chaos, predictability is its own kind of threat. You can see exactly \
            what it's going to do. It's going to do it anyway.",
        epoch: "Axiom Age",
    },
    EnemyLore {
        name: "Fractal Knight",
        description: "A warrior defined at every scale. Zoom in and there are smaller warriors \
            inside. It is unclear which one is the real one. They all are. Targeting the \
            largest one does something. Targeting the smallest one does something different.",
        epoch: "Expansion",
    },
    EnemyLore {
        name: "Riemann Shade",
        description: "Tied to the proof's prime distribution. It appears at positions along \
            the critical line. If the Riemann Hypothesis is true, it can be predicted. If \
            not, it can't. Nobody knows which. The Shade knows. It's not saying.",
        epoch: "Expansion",
    },
    EnemyLore {
        name: "Null Acolyte",
        description: "A member of the Null Collective. It has voluntarily set its own value \
            to zero. It hits harder than zero should be able to. The proof does not explain \
            this. The Null Collective considers it proof of their philosophy.",
        epoch: "Collapse",
    },
    EnemyLore {
        name: "Axiom Wraith",
        description: "A first-principles entity corrupted by the Recursion. It still follows \
            the original axioms, but the axioms have been inverted. What was true is false. \
            What was stable is catastrophic. The original axioms are all still there. This \
            is the problem.",
        epoch: "Recursion",
    },
    EnemyLore {
        name: "Bifurcation Horror",
        description: "A logistic map evaluated at r = 4.0. Full chaos. Every step doubles \
            the possible outcomes. There is no pattern. There is no prediction. The \
            Mathematician set r at 3.7 and thought they were safe. At 4.0, they weren't.",
        epoch: "Expansion",
    },
    EnemyLore {
        name: "Collatz Stalker",
        description: "Its behavior follows the Collatz sequence. Odd rounds are aggressive. \
            Even rounds are passive. The sequence always seems to approach 1 — calm, resolved \
            — and then it finds another large odd number and everything is chaotic again.",
        epoch: "Expansion",
    },
    EnemyLore {
        name: "Zeta Revenant",
        description: "Emerged from one of the Riemann zeta function's nontrivial zeros. If \
            the zero is on the critical line, this entity is theoretically manageable. If it \
            isn't, the entity is something the proof has not classified. You will know which \
            one it is by how the fight goes.",
        epoch: "Expansion",
    },
    EnemyLore {
        name: "Eigenvalue Drone",
        description: "A stable eigenvector, deployed by the Eigenstate Council to enforce \
            their version of order. It will not deviate from its defined behavior under any \
            circumstance. This makes it very predictable. Very predictable things are \
            surprisingly hard to avoid.",
        epoch: "Collapse",
    },
    EnemyLore {
        name: "Divergent Wraith",
        description: "A series that diverges. Not to infinity — in every direction \
            simultaneously. Its attacks have no pattern because the pattern is infinite \
            and unresolvable. The chaos pipeline cannot bias rolls against it because the \
            bias would also diverge.",
        epoch: "Collapse",
    },
    EnemyLore {
        name: "Misery Leech",
        description: "It feeds on suffering. Specifically on accumulated mathematical strain. \
            Characters with high Misery Index attract it. Characters with low Misery Index \
            are invisible to it. There is a threshold — below 1,000 it ignores you. Above \
            5,000, it hunts you. The threshold is not a coincidence.",
        epoch: "Current",
    },
    EnemyLore {
        name: "Recursive Shade",
        description: "A function that calls itself with no base case. It should stack overflow. \
            Inside The Proof, there is no stack limit. The Recursive Shade is what a stack \
            overflow looks like when it has nowhere to go — it becomes an entity instead.",
        epoch: "Recursion",
    },
    EnemyLore {
        name: "Theorem Husk",
        description: "The empty shell of a proof that completed itself and then had nothing \
            left to be. A theorem, once proven, has no more purpose in an active proof. \
            The Husk wanders, looking for something to prove. It has selected you as a \
            conjecture. You are the thing it is trying to disprove.",
        epoch: "Collapse",
    },
];

/// Look up an enemy's lore by name. Fuzzy match on the base enemy name.
pub fn enemy_lore(enemy_name: &str) -> Option<&'static EnemyLore> {
    // Try exact match first
    if let Some(e) = ENEMY_LORE.iter().find(|e| e.name == enemy_name) {
        return Some(e);
    }
    // Try prefix match for generated enemies like "Giant Divergence Tick"
    ENEMY_LORE
        .iter()
        .find(|e| enemy_name.contains(e.name) || e.name.contains(enemy_name))
}

/// Returns a generic lore entry for enemies without a specific entry.
pub fn generic_enemy_lore(seed: u64) -> &'static str {
    const GENERIC: &[&str] = &[
        "An entity the proof generated to fill a gap in its logic. The gap remains.",
        "An error term that became too large to ignore and too complex to resolve.",
        "A variable the proof assigned and then lost track of. It has been looking for \
         its assignment ever since.",
        "Something the Recursion epoch created and the Collapse epoch couldn't delete.",
        "A divergent series with a will. The will is to diverge further.",
        "Proof-generated. Purpose: unclear. Behavior: hostile. Classification: pending.",
        "An entity from the documentation layer. It was never intended to be autonomous. \
         Intent and outcome are different things inside The Proof.",
    ];
    GENERIC[(seed % GENERIC.len() as u64) as usize]
}
