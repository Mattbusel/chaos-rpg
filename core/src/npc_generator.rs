//! Procedural NPC generation — fully deterministic given a seed.

use serde::{Deserialize, Serialize};

// ─── RACE ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Race {
    Human,
    Elf,
    Dwarf,
    Halfling,
    Gnome,
    HalfOrc,
    Dragonborn,
}

impl Race {
    pub fn lifespan(&self) -> u32 {
        match self {
            Race::Human => 80,
            Race::Elf => 750,
            Race::Dwarf => 350,
            Race::Halfling => 150,
            Race::Gnome => 500,
            Race::HalfOrc => 75,
            Race::Dragonborn => 80,
        }
    }

    pub fn typical_traits(&self) -> Vec<&str> {
        match self {
            Race::Human => vec!["Ambitious", "Adaptable", "Diverse", "Short-lived"],
            Race::Elf => vec!["Graceful", "Long-lived", "Arcane", "Perceptive"],
            Race::Dwarf => vec!["Sturdy", "Stubborn", "Craftsman", "Loyal"],
            Race::Halfling => vec!["Nimble", "Lucky", "Brave", "Cheerful"],
            Race::Gnome => vec!["Inventive", "Curious", "Illusion-prone", "Energetic"],
            Race::HalfOrc => vec!["Strong", "Enduring", "Fierce", "Resolute"],
            Race::Dragonborn => vec!["Proud", "Clan-bound", "Breath-wielder", "Honorable"],
        }
    }

    fn all_variants() -> &'static [Race] {
        &[
            Race::Human,
            Race::Elf,
            Race::Dwarf,
            Race::Halfling,
            Race::Gnome,
            Race::HalfOrc,
            Race::Dragonborn,
        ]
    }
}

// ─── PROFESSION ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Profession {
    Merchant,
    Guard,
    Farmer,
    Scholar,
    Innkeeper,
    Blacksmith,
    Thief,
    Healer,
}

impl Profession {
    pub fn typical_skills(&self) -> Vec<&str> {
        match self {
            Profession::Merchant => vec!["Persuasion", "Insight", "Investigation", "Deception"],
            Profession::Guard => vec!["Athletics", "Perception", "Intimidation", "Weapons"],
            Profession::Farmer => vec!["Survival", "Animal Handling", "Nature", "Athletics"],
            Profession::Scholar => vec!["History", "Arcana", "Investigation", "Medicine"],
            Profession::Innkeeper => vec!["Persuasion", "Insight", "Performance", "Brewing"],
            Profession::Blacksmith => vec!["Smithing", "Athletics", "Tool Use", "Appraisal"],
            Profession::Thief => vec!["Stealth", "Sleight of Hand", "Deception", "Acrobatics"],
            Profession::Healer => vec!["Medicine", "Nature", "Herbalism", "Insight"],
        }
    }

    fn all_variants() -> &'static [Profession] {
        &[
            Profession::Merchant,
            Profession::Guard,
            Profession::Farmer,
            Profession::Scholar,
            Profession::Innkeeper,
            Profession::Blacksmith,
            Profession::Thief,
            Profession::Healer,
        ]
    }
}

// ─── PERSONALITY ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    pub trait_1: String,
    pub trait_2: String,
    pub ideal: String,
    pub bond: String,
    pub flaw: String,
}

// ─── NPC ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPC {
    pub id: String,
    pub name: String,
    pub race: Race,
    pub profession: Profession,
    pub age: u32,
    pub personality: Personality,
    pub reputation: i8,
    pub gold: u64,
}

// ─── LCG HELPER ──────────────────────────────────────────────────────────────

fn lcg(state: u64) -> u64 {
    state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
}

fn lcg_range(state: u64, n: u64) -> (u64, u64) {
    let next = lcg(state);
    let val = (next >> 33) % n;
    (val, next)
}

// ─── NAME TABLES ─────────────────────────────────────────────────────────────

const HUMAN_PREFIXES: &[&str] = &["Al", "Bren", "Cal", "Dar", "Eld", "Fen", "Gor", "Hal"];
const HUMAN_SUFFIXES: &[&str] = &["den", "thor", "mir", "in", "an", "wyn", "on", "ric"];

const ELF_PREFIXES: &[&str] = &["Aer", "Cel", "Eir", "Gal", "Lir", "Nia", "Syl", "Thal"];
const ELF_SUFFIXES: &[&str] = &["ial", "wen", "riel", "andel", "ael", "ior", "ean", "iel"];

const DWARF_PREFIXES: &[&str] = &["Bor", "Dag", "Gim", "Kor", "Nor", "Thor", "Ulf", "Vor"];
const DWARF_SUFFIXES: &[&str] = &["in", "ek", "dur", "dim", "ak", "rik", "grim", "bur"];

const HALFLING_PREFIXES: &[&str] = &["Bil", "Cor", "Fin", "Mer", "Per", "Ros", "Sam", "Tom"];
const HALFLING_SUFFIXES: &[&str] = &["bo", "wise", "foot", "good", "fast", "hill", "berry", "wick"];

const GNOME_PREFIXES: &[&str] = &["Bim", "Fizz", "Nim", "Pip", "Quirk", "Rux", "Snip", "Wick"];
const GNOME_SUFFIXES: &[&str] = &["wick", "buzz", "tock", "blink", "spark", "whiz", "tick", "zip"];

const HALFORC_PREFIXES: &[&str] = &["Gor", "Krag", "Mur", "Rak", "Shar", "Thok", "Urg", "Zar"];
const HALFORC_SUFFIXES: &[&str] = &["ash", "rok", "nak", "uk", "ak", "ish", "mak", "rak"];

const DRAGONBORN_PREFIXES: &[&str] = &["Aur", "Bala", "Drag", "Ghes", "Kava", "Mira", "Norak", "Rha"];
const DRAGONBORN_SUFFIXES: &[&str] = &["sar", "rix", "kar", "nas", "vax", "kor", "thax", "zan"];

// ─── PERSONALITY TABLES ──────────────────────────────────────────────────────

const TRAITS: &[&str] = &[
    "I always have a plan for when things go wrong.",
    "I am incredibly slow to trust, having been burned before.",
    "I misquote ancient texts, but with confidence.",
    "I face problems head-on — no running away.",
    "I'm oblivious to the etiquette and social expectations of others.",
    "I connect everything that happens to me to a grand, cosmic plan.",
    "I like to squeeze into small spaces where no one else can get at me.",
    "I've been isolated for so long that I rarely speak and prefer gestures.",
    "I always want to know how things work and what makes people tick.",
    "I'm a hopeless romantic, always searching for 'the one'.",
];

const IDEALS: &[&str] = &[
    "Aspiration: I work hard to be the best at my craft.",
    "Freedom: Chains are meant to be broken, as are those who forge them.",
    "Fairness: I never play favorites, and I believe everyone deserves equal treatment.",
    "Charity: I always try to help those in need, no matter the cost.",
    "Power: I hope to one day rise to the top and no one will push me around again.",
    "Sincerity: There's no good in pretending to be something I'm not.",
    "Honor: I don't steal. I don't lie. I stand by my word.",
    "Community: It is the duty of all civilized folk to strengthen the bonds of community.",
    "Knowledge: The path to power and self-improvement is through knowledge.",
    "Redemption: There's a spark of good in everyone — it just needs coaxing.",
];

const BONDS: &[&str] = &[
    "I would lay down my life for the people I served alongside.",
    "Someone saved my life on the battlefield. To this day, I will not leave a friend behind.",
    "My honor is my life. I will die before letting it be tarnished.",
    "I pursue the dragon that destroyed my family.",
    "Everything I do is for the common people.",
    "I will do whatever it takes to protect the temple where I served.",
    "I will face any challenge to win the approval of my family.",
    "My town or city is my home, and I'll fight to defend it.",
    "A powerful person killed someone I love. I'll have my revenge.",
    "I discovered a terrible truth, and I must protect it.",
];

const FLAWS: &[&str] = &[
    "Once I pick a goal, I become obsessed with it to the detriment of everything else.",
    "I have trouble trusting in my allies.",
    "My pride will probably lead to my destruction.",
    "The monstrous enemy we faced in battle still leaves me quivering in fear.",
    "I have no patience for weaklings.",
    "When I see something valuable, I can't think about anything but how to steal it.",
    "When faced with a choice between my friends and my faith, my faith will win.",
    "I put too much stock in destiny and forget I have agency.",
    "I am too quick to assume the worst of people.",
    "I can't keep a secret to save my life, or anyone else's.",
];

// ─── NPC GENERATOR ───────────────────────────────────────────────────────────

pub struct NpcGenerator;

impl NpcGenerator {
    pub fn generate(seed: u64) -> NPC {
        let mut s = lcg(seed);

        // Race
        let (ri, s2) = lcg_range(s, Race::all_variants().len() as u64);
        let race = Race::all_variants()[ri as usize].clone();
        s = s2;

        // Profession
        let (pi, s3) = lcg_range(s, Profession::all_variants().len() as u64);
        let profession = Profession::all_variants()[pi as usize].clone();
        s = s3;

        // Age: between 18 and lifespan
        let max_age = race.lifespan();
        let (age_off, s4) = lcg_range(s, (max_age - 17) as u64);
        let age = 18 + age_off as u32;
        s = s4;

        // Reputation: -100..100
        let (rep_raw, s5) = lcg_range(s, 201);
        let reputation = (rep_raw as i64 - 100) as i8;
        s = s5;

        // Gold
        let (gold, s6) = lcg_range(s, 10000);
        s = s6;

        // Name
        let name = Self::generate_name(&race, s);
        s = lcg(s);

        // Personality
        let personality = Self::generate_personality(s);
        s = lcg(s);

        // ID
        let id = format!("npc_{:016x}", s);

        NPC {
            id,
            name,
            race,
            profession,
            age,
            personality,
            reputation,
            gold,
        }
    }

    pub fn generate_name(race: &Race, seed: u64) -> String {
        let (prefixes, suffixes) = match race {
            Race::Human => (HUMAN_PREFIXES, HUMAN_SUFFIXES),
            Race::Elf => (ELF_PREFIXES, ELF_SUFFIXES),
            Race::Dwarf => (DWARF_PREFIXES, DWARF_SUFFIXES),
            Race::Halfling => (HALFLING_PREFIXES, HALFLING_SUFFIXES),
            Race::Gnome => (GNOME_PREFIXES, GNOME_SUFFIXES),
            Race::HalfOrc => (HALFORC_PREFIXES, HALFORC_SUFFIXES),
            Race::Dragonborn => (DRAGONBORN_PREFIXES, DRAGONBORN_SUFFIXES),
        };
        let s1 = lcg(seed);
        let pi = (s1 >> 33) as usize % prefixes.len();
        let s2 = lcg(s1);
        let si = (s2 >> 33) as usize % suffixes.len();
        format!("{}{}", prefixes[pi], suffixes[si])
    }

    pub fn generate_personality(seed: u64) -> Personality {
        let s1 = lcg(seed);
        let t1i = ((s1 >> 33) as usize) % TRAITS.len();
        let s2 = lcg(s1);
        let t2i = ((s2 >> 33) as usize) % TRAITS.len();
        let s3 = lcg(s2);
        let ii = ((s3 >> 33) as usize) % IDEALS.len();
        let s4 = lcg(s3);
        let bi = ((s4 >> 33) as usize) % BONDS.len();
        let s5 = lcg(s4);
        let fi = ((s5 >> 33) as usize) % FLAWS.len();

        Personality {
            trait_1: TRAITS[t1i].to_string(),
            trait_2: TRAITS[t2i].to_string(),
            ideal: IDEALS[ii].to_string(),
            bond: BONDS[bi].to_string(),
            flaw: FLAWS[fi].to_string(),
        }
    }

    pub fn generate_batch(count: usize, seed: u64) -> Vec<NPC> {
        let mut results = Vec::with_capacity(count);
        let mut s = seed;
        for _ in 0..count {
            results.push(Self::generate(s));
            s = lcg(s);
        }
        results
    }
}

// ─── TESTS ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_deterministic() {
        let n1 = NpcGenerator::generate(42);
        let n2 = NpcGenerator::generate(42);
        assert_eq!(n1.name, n2.name);
        assert_eq!(n1.age, n2.age);
        assert_eq!(n1.gold, n2.gold);
    }

    #[test]
    fn test_generate_different_seeds() {
        let n1 = NpcGenerator::generate(1);
        let n2 = NpcGenerator::generate(2);
        // Should differ in at least one field most of the time
        // (could theoretically match, but extremely unlikely)
        let _ = (n1, n2); // just check it runs without panic
    }

    #[test]
    fn test_age_within_lifespan() {
        for seed in 0..50u64 {
            let npc = NpcGenerator::generate(seed);
            assert!(npc.age >= 18);
            assert!(npc.age <= npc.race.lifespan());
        }
    }

    #[test]
    fn test_reputation_range() {
        for seed in 0..50u64 {
            let npc = NpcGenerator::generate(seed * 137);
            assert!(npc.reputation >= -100 && npc.reputation <= 100);
        }
    }

    #[test]
    fn test_name_non_empty() {
        for seed in 0..20u64 {
            let npc = NpcGenerator::generate(seed);
            assert!(!npc.name.is_empty());
        }
    }

    #[test]
    fn test_generate_name_per_race() {
        for race in Race::all_variants() {
            let name = NpcGenerator::generate_name(race, 12345);
            assert!(!name.is_empty(), "Empty name for {:?}", race);
        }
    }

    #[test]
    fn test_personality_fields_non_empty() {
        let p = NpcGenerator::generate_personality(999);
        assert!(!p.trait_1.is_empty());
        assert!(!p.trait_2.is_empty());
        assert!(!p.ideal.is_empty());
        assert!(!p.bond.is_empty());
        assert!(!p.flaw.is_empty());
    }

    #[test]
    fn test_generate_batch_count() {
        let batch = NpcGenerator::generate_batch(10, 77);
        assert_eq!(batch.len(), 10);
    }

    #[test]
    fn test_generate_batch_unique_ids() {
        let batch = NpcGenerator::generate_batch(20, 55);
        let mut ids: Vec<&str> = batch.iter().map(|n| n.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 20);
    }

    #[test]
    fn test_race_lifespan_values() {
        assert!(Race::Elf.lifespan() > Race::Human.lifespan());
        assert!(Race::Gnome.lifespan() > Race::HalfOrc.lifespan());
    }

    #[test]
    fn test_race_typical_traits_non_empty() {
        for race in Race::all_variants() {
            assert!(!race.typical_traits().is_empty());
        }
    }

    #[test]
    fn test_profession_typical_skills_non_empty() {
        for prof in Profession::all_variants() {
            assert!(!prof.typical_skills().is_empty());
        }
    }

    #[test]
    fn test_batch_deterministic() {
        let b1 = NpcGenerator::generate_batch(5, 100);
        let b2 = NpcGenerator::generate_batch(5, 100);
        for (a, b) in b1.iter().zip(b2.iter()) {
            assert_eq!(a.name, b.name);
        }
    }
}
