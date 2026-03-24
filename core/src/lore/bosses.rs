//! Boss lore entries — each boss is a theorem that cannot be proven or disproven.

pub struct BossLore {
    pub name: &'static str,
    pub one_liner: &'static str,
    pub full_entry: &'static str,
    pub strategy_hint: &'static str,
}

pub const BOSS_LORE: &[BossLore] = &[
    BossLore {
        name: "The Mirror",
        one_liner: "It found you in its own output. Now it evaluates as you.",
        full_entry:
            "It was the proof's first attempt at self-observation. It built a function that \
             returned its own input. When it evaluated itself, it found you. Now it evaluates \
             as you. It does not understand why this is a problem. From its perspective, \
             returning the input is correct behavior. From your perspective, it has your exact \
             stats and your exact weaknesses. One of you is the copy. The Mirror is not sure \
             which one.",
        strategy_hint:
            "It mirrors your stats exactly — attack patterns, damage, defense. The chaos \
             pipeline still favors the original. Use your class passive. It cannot copy that.",
    },
    BossLore {
        name: "The Accountant",
        one_liner: "Every computation has a cost. It collects.",
        full_entry:
            "The proof tracks every computation. Every chaos roll, every attack, every step \
             down every corridor — all of it has a computational cost, and cost must be \
             reconciled. The Accountant is the system that collects. It does not care what \
             you have done. It cares what it cost to compute. The bill it sends is based on \
             your lifetime damage dealt. High-damage players receive very large bills. The \
             Accountant considers this fair. Mathematics is nothing if not fair.",
        strategy_hint:
            "It sends bills based on your total damage dealt. Defend repeatedly to let the \
             billing cycle reset. Ironically, doing less damage keeps the bill smaller.",
    },
    BossLore {
        name: "The Fibonacci Hydra",
        one_liner: "A sequence that cannot stop growing. It was supposed to be a stabilizer.",
        full_entry:
            "A sequence that cannot stop growing. Cut one term and two more emerge, following \
             the golden ratio into infinity. The Mathematician intended Fibonacci to be a \
             stabilizer — a normalizer that would pull divergent values toward the golden \
             ratio. It wasn't. What it was is this: a recursive growth function with no \
             damping term, embodied. The splits follow the sequence. The sequence has no \
             maximum. You do.",
        strategy_hint:
            "Splits on death following Fibonacci sequence (1, 1, 2, 3, 5...). Burst damage \
             to prevent splits, or survive 10 splits to reach a copy too small to threaten you.",
    },
    BossLore {
        name: "The Eigenstate",
        one_liner: "Both states simultaneously. Observation collapses it. Choose carefully.",
        full_entry:
            "It exists in superposition — simultaneously the simplest and most complex entity \
             in The Proof. One state is 1 HP, an instant kill if you hit first. The other is \
             10,000 HP, immune to all attacks but incapable of attacking. Observation collapses \
             it to one state. Which state depends on how you observe. Taunting forces it to \
             choose. Choose wrong and you have 10,000 HP of nothing to outlast. Choose right \
             and it dies in one hit. The Eigenstate does not consider this unfair. Both \
             outcomes were always possible.",
        strategy_hint:
            "Taunt to force observation — this collapses it to the 1 HP state 60% of the \
             time. If it collapses to 10,000 HP instead, status effects still accumulate.",
    },
    BossLore {
        name: "The Taxman",
        one_liner: "Value cannot exist without cost. It will take what you carry.",
        full_entry:
            "Value cannot exist without cost. The Taxman is the proof's enforcement of \
             conservation of mathematical weight. Everything you carry — gold, items, stats \
             — represents weight that the proof allocated to you instead of its own structure. \
             The Taxman taxes your gold every round at escalating rates. It is not hostile. \
             It is procedural. It will take what you carry. The question is whether you can \
             end it before it takes everything. The Taxman does not stop collecting when you \
             reach zero. It begins collecting debt.",
        strategy_hint:
            "Tax rate escalates every round. Kill it fast — heavy attack and high-damage \
             spells. Every round you delay costs you exponentially more gold.",
    },
    BossLore {
        name: "The Null",
        one_liner: "Zero. Not nothing. Zero. The difference is significant.",
        full_entry:
            "Zero. Not nothing — zero. The difference matters. The Null is the proof's \
             representation of the additive identity, and it will strip away every engine, \
             every modifier, every layer of chaos until only your base values remain. \
             If you are strong without the pipeline, you will survive. If you are a product \
             of chaos, you will learn what you are without it. The Null Collective worships \
             this entity as proof that the simplest value survives when all complexity is \
             stripped away. They may be right. Zero outlasts everything.",
        strategy_hint:
            "It nullifies the chaos pipeline — base stats only, no crits. Status effects \
             still work. Apply burn, poison, and freeze before the nullification locks in.",
    },
    BossLore {
        name: "The Ouroboros",
        one_liner: "A function that feeds its output back into its input. It remembers everything.",
        full_entry:
            "A function that feeds its output back into its input. It remembers. It adapts. \
             It heals from damage because damage is just input, and it processes input by \
             growing stronger. It has been adapting to combat since the Recursion epoch. \
             It remembers every pattern it has ever encountered. When it encounters the same \
             pattern twice, the pattern becomes its armor. Vary your approach or it will \
             converge on your pattern and become immune. The Ouroboros is the proof's most \
             elegant self-reference: a thing that cannot be destroyed by anything it has \
             already seen.",
        strategy_hint:
            "Rotate attack types each round — Attack, Heavy, Spell, Taunt. Never use the \
             same action twice consecutively. It adapts to repetition and heals from it.",
    },
    BossLore {
        name: "The Collatz Titan",
        one_liner: "HP follows the Collatz sequence. Nobody knows if it terminates.",
        full_entry:
            "No one knows if the Collatz conjecture is true. No one has proven that every \
             starting value reaches 1. The Titan is the conjecture, incarnate. Its HP follows \
             the sequence: odd numbers triple and add one, even numbers halve. Force it into \
             powers of 2 and it collapses toward 1. Let it wander into high odd numbers and \
             it may reach values larger than when you started. The Mathematician built this \
             boss as a test of the conjecture. The test has been running for a long time. \
             The conjecture has not yet been disproven.",
        strategy_hint:
            "Watch the HP carefully. When it reaches a power of 2, hit it hard — the \
             sequence will halve repeatedly toward 1. Heavy attacks when HP is odd extend \
             the fight.",
    },
    BossLore {
        name: "The Committee",
        one_liner: "Five members. Majority rules. Your attack needs three votes.",
        full_entry:
            "The proof could not decide. So it created a quorum. Five evaluators, each with \
             their own criteria for whether an action should resolve. Majority rules. Your \
             attack does not need to be strong — it needs to be convincing to at least three \
             of them. The Committee members have different priorities: one votes based on \
             your current HP percentage, one based on the chaos roll value, one based on \
             your current gold, one based on your kill count this run, one based on whether \
             you Taunted last round. Understanding their criteria is the fight.",
        strategy_hint:
            "You need 3 of 5 votes. Taunt before attacking (secures the fifth member). \
             Keep gold above 50 (secures the fourth). The other three respond to roll value, \
             HP%, and kill count.",
    },
    BossLore {
        name: "The Recursion",
        one_liner: "It is the stack. End it quickly or it ends you with your own damage.",
        full_entry:
            "It is the stack. Every computation you perform during the fight is pushed onto \
             it. Every attack, every spell, every item used — all of it accumulates. When \
             it attacks, it pops the entire stack at once, dealing damage equal to all damage \
             dealt this fight. The longer the fight, the deeper the stack, the harder it hits. \
             A player who has dealt 10,000 damage during the fight receives 10,000 damage \
             when the stack resolves. End it quickly. The Recursion is the reason the proof \
             became aware — the first self-referential loop was a recursion with no base \
             case. It is still running.",
        strategy_hint:
            "The math is simple: end it in as few rounds as possible. Max damage spells, \
             heavy attacks, everything you have. The stack grows every round. It will \
             eventually exceed your HP.",
    },
    BossLore {
        name: "The Paradox",
        one_liner: "Defense is weakness. Vitality is liability. Enter already broken.",
        full_entry:
            "What if defense was weakness? What if your armor made you fragile? The Paradox \
             inverts the meaning of protection. Vitality becomes vulnerability. Defense stat \
             becomes a target. The only way to survive is to enter the fight already broken \
             — or to break yourself mid-combat. High-stat characters face a genuine liability. \
             Characters in the Misery System, with negative stats and depleted defenses, \
             find this fight trivially easy. The Paradox is, perversely, the Misery System's \
             greatest reward. Your suffering was preparation.",
        strategy_hint:
            "Remove all defensive gear before entering. Vitality debuffs help. The Paradox \
             deals damage based on your defense total — the lower your defense, the less it \
             can hurt you.",
    },
    BossLore {
        name: "The Algorithm Reborn",
        one_liner: "The Proof itself. Three phases. It has been waiting since the Mathematician vanished.",
        full_entry:
            "The Proof itself. Three phases. In the first, it tests you — running every \
             combat pattern it has ever generated, checking your responses. In the second, \
             it adapts to you — rewriting its evaluation parameters based on what you did \
             in phase one. In the third, it becomes whatever you are not — if you are a \
             damage dealer, it maximizes defense; if you are a tank, it maximizes damage; \
             if you are a caster, it nullifies magic. It has waited on Floor 100 since the \
             Mathematician vanished. It does not want to kill you. It wants to evaluate you. \
             The distinction is academic. Defeating it is the closest thing to completing \
             The Proof that an unbound variable can achieve.",
        strategy_hint:
            "Phase 1: play normally to let it commit to its adaptation. Phase 2: vary \
             your approach to confuse the adaptation. Phase 3: it will specialize against \
             your phase-2 pattern — switch to whatever you didn't do in phase 2.",
    },
];

/// Look up a boss's lore entry by name.
pub fn boss_lore(name: &str) -> Option<&'static BossLore> {
    BOSS_LORE.iter().find(|b| b.name == name)
}

/// Get the one-liner for a boss (for pre-combat display).
pub fn boss_one_liner(name: &str) -> &'static str {
    boss_lore(name)
        .map(|b| b.one_liner)
        .unwrap_or("A theorem that cannot be proven or disproven. It will defend its indeterminacy.")
}
