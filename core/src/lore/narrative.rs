//! Auto-generated run narrative — NarrativeEvent tracking and narrative building.

use serde::{Deserialize, Serialize};

// ─── EVENT TYPES ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NarrativeEvent {
    FirstKill {
        enemy: String,
        floor: u32,
        rounds: u32,
    },
    BiggestHit {
        damage: i64,
        enemy: String,
        floor: u32,
        engine: String,
        was_crit: bool,
    },
    NearDeath {
        hp_remaining: i32,
        enemy: String,
        floor: u32,
        survived_how: String,
    },
    NemesisEncounter {
        nemesis_name: String,
        floor: u32,
        ability: String,
        outcome: String, // "defeated" | "survived" | "fled"
    },
    BossKill {
        boss: String,
        floor: u32,
        rounds: u32,
    },
    ItemFound {
        item_name: String,
        rarity: String,
        floor: u32,
    },
    CorruptionMilestone {
        stacks: u32,
        parameter_note: String,
    },
    MiseryMilestone {
        index: u64,
        milestone_name: String,
    },
    AchievementUnlock {
        name: String,
        description: String,
    },
    Death {
        enemy: String,
        damage: i64,
        floor: u32,
        was_crit: bool,
    },
    Victory {
        mode: String,
        floor: u32,
    },
}

impl NarrativeEvent {
    /// Significance score for selecting top 5 events.
    pub fn significance(&self) -> u32 {
        match self {
            NarrativeEvent::BossKill { .. } => 100,
            NarrativeEvent::NemesisEncounter { outcome, .. } => {
                if outcome == "defeated" { 90 } else { 60 }
            }
            NarrativeEvent::BiggestHit { was_crit, damage, .. } => {
                let base = if *was_crit { 70u32 } else { 50u32 };
                base + (*damage / 1000).min(30) as u32
            }
            NarrativeEvent::NearDeath { hp_remaining, .. } => {
                if *hp_remaining <= 5 { 85 } else { 65 }
            }
            NarrativeEvent::ItemFound { rarity, .. } => match rarity.as_str() {
                "◈ ARTIFACT ◈" => 80,
                "???" => 70,
                "Legendary" => 55,
                "Mythical" => 60,
                "Divine" => 65,
                _ => 30,
            },
            NarrativeEvent::CorruptionMilestone { stacks, .. } => {
                if *stacks >= 400 { 75 } else { 40 }
            }
            NarrativeEvent::MiseryMilestone { index, .. } => {
                if *index >= 100_000 { 80 }
                else if *index >= 50_000 { 65 }
                else { 40 }
            }
            NarrativeEvent::AchievementUnlock { .. } => 35,
            NarrativeEvent::FirstKill { .. } => 20,
            NarrativeEvent::Death { .. } | NarrativeEvent::Victory { .. } => 0, // always included
        }
    }
}

// ─── NARRATIVE BUILDER ────────────────────────────────────────────────────────

pub struct RunNarrative {
    pub character_name: String,
    pub character_class: String,
    pub character_background: String,
    pub difficulty: String,
    pub game_mode: String,
    pub destiny_roll_value: f64,
    pub positive_stats: Vec<(String, i64)>,
    pub negative_stats: Vec<(String, i64)>,
    pub boon_name: Option<String>,
    pub final_floor: u32,
    pub final_tier: String,
    pub total_kills: u64,
    pub total_damage: i64,
    pub events: Vec<NarrativeEvent>,
    pub custom_origin: Option<String>,
    pub epitaph: String,
    pub won: bool,
}

impl RunNarrative {
    /// Generate the full auto-narrative text for a run.
    pub fn generate(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Opening paragraph
        parts.push(self.generate_opening());

        // Select top 5 significant events (excluding death/victory)
        let mut events: Vec<&NarrativeEvent> = self
            .events
            .iter()
            .filter(|e| !matches!(e, NarrativeEvent::Death { .. } | NarrativeEvent::Victory { .. }))
            .collect();
        events.sort_by(|a, b| b.significance().cmp(&a.significance()));
        events.truncate(5);

        for event in events {
            if let Some(para) = self.event_paragraph(event) {
                parts.push(para);
            }
        }

        // Closing paragraph
        parts.push(self.generate_closing());

        parts.join("\n\n")
    }

    fn generate_opening(&self) -> String {
        if let Some(ref origin) = self.custom_origin {
            return format!(
                "{} the {} entered The Proof. {}",
                self.character_name, self.character_class, origin
            );
        }

        let stat_line = if self.positive_stats.is_empty() && self.negative_stats.is_empty() {
            "The chaos pipeline returned no clear assignment.".to_string()
        } else {
            let pos: Vec<String> = self
                .positive_stats
                .iter()
                .take(3)
                .map(|(s, v)| format!("{} +{}", s, v))
                .collect();
            let neg: Vec<String> = self
                .negative_stats
                .iter()
                .take(2)
                .map(|(s, v)| format!("{} {}", s, v))
                .collect();
            let mut parts = pos;
            parts.extend(neg);
            format!("The proof assigned them: {}.", parts.join(", "))
        };

        let boon_line = self
            .boon_name
            .as_ref()
            .map(|b| format!(" They chose the boon: {}.", b))
            .unwrap_or_default();

        let destiny_line = if self.destiny_roll_value > 0.8 {
            " Their Destiny Roll was exceptional — the proof briefly reconsidered its objection to their existence."
        } else if self.destiny_roll_value < -0.8 {
            " Their Destiny Roll was catastrophic — the proof evaluated them and filed an immediate exception."
        } else {
            " Their Destiny Roll was unremarkable. The proof did not notice. It would notice later."
        };

        format!(
            "{} the {} ({} / {} / {}) entered The Proof.{} {}{} The dungeon had \
             been waiting. It is always waiting.",
            self.character_name,
            self.character_class,
            self.character_background,
            self.difficulty,
            self.game_mode,
            destiny_line,
            stat_line,
            boon_line,
        )
    }

    fn event_paragraph(&self, event: &NarrativeEvent) -> Option<String> {
        match event {
            NarrativeEvent::FirstKill { enemy, floor, rounds } => Some(format!(
                "Their first kill was a {} on Floor {}. It fell in {} round{}. The proof \
                 recorded the kill. The proof records everything.",
                enemy, floor, rounds,
                if *rounds == 1 { "" } else { "s" }
            )),
            NarrativeEvent::BiggestHit { damage, enemy, floor, engine, was_crit } => {
                let crit_line = if *was_crit {
                    format!(
                        "The {} engine went critical — the chain produced a value the proof could \
                         barely contain.",
                        engine
                    )
                } else {
                    format!("The {} engine carried the chain.", engine)
                };
                Some(format!(
                    "The defining moment came on Floor {} when {} dealt {} damage to {}. {}",
                    floor, self.character_name, damage, enemy, crit_line
                ))
            }
            NarrativeEvent::NearDeath { hp_remaining, enemy, floor, survived_how } => Some(format!(
                "On Floor {}, {} brought them to {} HP. They survived by {}. The proof \
                 noted the near-termination and updated its models accordingly.",
                floor, enemy, hp_remaining, survived_how
            )),
            NarrativeEvent::NemesisEncounter { nemesis_name, floor, ability, outcome } => {
                let outcome_line = match outcome.as_str() {
                    "defeated" => format!(
                        "They defeated it. The proof revised its opinion of {} for the second time.",
                        self.character_name
                    ),
                    "fled" => "They fled again. The proof did not revise anything.".to_string(),
                    _ => "They survived the encounter. Barely.".to_string(),
                };
                Some(format!(
                    "{}, bearing the ability {} it earned from a previous encounter, returned \
                     on Floor {}. {}",
                    nemesis_name, ability, floor, outcome_line
                ))
            }
            NarrativeEvent::BossKill { boss, floor, rounds } => {
                let lore_line = crate::lore::bosses::boss_lore(boss)
                    .map(|b| format!(" — {}.", b.one_liner))
                    .unwrap_or_default();
                Some(format!(
                    "They faced {} on Floor {}{} It took {} round{}.",
                    boss, floor, lore_line, rounds,
                    if *rounds == 1 { "" } else { "s" }
                ))
            }
            NarrativeEvent::ItemFound { item_name, rarity, floor } => {
                let flavor = if rarity == "◈ ARTIFACT ◈" {
                    " The proof had not expected to generate this. Neither had they."
                } else if rarity == "Legendary" || rarity == "Mythical" || rarity == "???" {
                    " The weight of it was wrong in a way that felt correct."
                } else {
                    ""
                };
                Some(format!(
                    "In a frozen computation pocket on Floor {}, they found the {} [{}].{}",
                    floor, item_name, rarity, flavor
                ))
            }
            NarrativeEvent::CorruptionMilestone { stacks, parameter_note } => Some(format!(
                "At {} corruption stacks, the pipeline shifted. {} The math had changed.",
                stacks, parameter_note
            )),
            NarrativeEvent::MiseryMilestone { index, milestone_name } => Some(format!(
                "The Misery Index reached {}. {} The proof took notice.",
                index, milestone_name
            )),
            NarrativeEvent::AchievementUnlock { name, description } => Some(format!(
                "Somewhere in the chaos, they earned '{}' — {}.",
                name, description
            )),
            NarrativeEvent::Death { .. } | NarrativeEvent::Victory { .. } => None,
        }
    }

    fn generate_closing(&self) -> String {
        // Find death or victory event
        let terminal = self.events.iter().find(|e| {
            matches!(e, NarrativeEvent::Death { .. } | NarrativeEvent::Victory { .. })
        });

        match terminal {
            Some(NarrativeEvent::Death { enemy, damage, floor, was_crit }) => {
                let crit_note = if *was_crit { ", a critical strike" } else { "" };
                format!(
                    "{} the {} fell on Floor {} to {}, who struck for {} damage{}. Their \
                     final power tier was {}. The proof recorded them in the graveyard with \
                     the epitaph: '{}.' They dealt {} damage across {} kills in {} floors. \
                     The mathematics consumed them.",
                    self.character_name,
                    self.character_class,
                    floor,
                    enemy,
                    damage,
                    crit_note,
                    self.final_tier,
                    self.epitaph,
                    self.total_damage,
                    self.total_kills,
                    self.final_floor,
                )
            }
            Some(NarrativeEvent::Victory { mode, floor }) => {
                let mode_close = match mode.as_str() {
                    "Story" => {
                        "completed The Proof's first 10 layers and emerged. Whether they escaped \
                         or simply moved to a region the proof hasn't defined yet is a question \
                         the proof refuses to answer."
                    }
                    "Daily" => {
                        "completed the Daily Seed run. Their score has been submitted. The proof \
                         records it alongside everyone else who walked the same floor plan today. \
                         Same dungeon. Different variables. Different outcomes."
                    }
                    _ => {
                        "chose to stop. The proof will remember them at their final tier. It does \
                         not understand why they stopped. It never does."
                    }
                };
                format!(
                    "{} the {} {} Floor {}, {} kills, {} total damage. Power tier: {}.",
                    self.character_name,
                    self.character_class,
                    mode_close,
                    floor,
                    self.total_kills,
                    self.total_damage,
                    self.final_tier,
                )
            }
            _ => {
                // Fallback if no terminal event was recorded
                format!(
                    "{} the {} ran ended on Floor {} with {} kills and {} total damage. \
                     Power tier: {}. The proof has filed this under: incomplete.",
                    self.character_name,
                    self.character_class,
                    self.final_floor,
                    self.total_kills,
                    self.total_damage,
                    self.final_tier,
                )
            }
        }
    }
}
