//! Dynamic weather system affecting gameplay.
//!
//! Weather transitions use a Markov chain with an LCG random number generator.
//! Effects apply to combat accuracy and travel speed.

use std::fmt;

/// LCG constants (Numerical Recipes).
const LCG_A: u64 = 1664525;
const LCG_C: u64 = 1013904223;
const LCG_M: u64 = 1 << 32;

fn lcg_next(seed: u64) -> u64 {
    (LCG_A.wrapping_mul(seed).wrapping_add(LCG_C)) % LCG_M
}

// ---------------------------------------------------------------------------
// WeatherType
// ---------------------------------------------------------------------------

/// Distinct weather conditions in the game world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WeatherType {
    Clear,
    Cloudy,
    Rain,
    Storm,
    Fog,
    Blizzard,
}

impl WeatherType {
    /// Fraction of normal visibility (0.0 – 1.0).
    pub fn visibility_modifier(&self) -> f64 {
        match self {
            WeatherType::Clear   => 1.00,
            WeatherType::Cloudy  => 0.90,
            WeatherType::Rain    => 0.70,
            WeatherType::Storm   => 0.40,
            WeatherType::Fog     => 0.25,
            WeatherType::Blizzard => 0.15,
        }
    }

    /// Fraction by which movement is slowed (0.0 = no penalty, 1.0 = immobile).
    pub fn movement_penalty(&self) -> f64 {
        match self {
            WeatherType::Clear   => 0.00,
            WeatherType::Cloudy  => 0.05,
            WeatherType::Rain    => 0.15,
            WeatherType::Storm   => 0.35,
            WeatherType::Fog     => 0.10,
            WeatherType::Blizzard => 0.50,
        }
    }

    fn index(&self) -> usize {
        match self {
            WeatherType::Clear   => 0,
            WeatherType::Cloudy  => 1,
            WeatherType::Rain    => 2,
            WeatherType::Storm   => 3,
            WeatherType::Fog     => 4,
            WeatherType::Blizzard => 5,
        }
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => WeatherType::Clear,
            1 => WeatherType::Cloudy,
            2 => WeatherType::Rain,
            3 => WeatherType::Storm,
            4 => WeatherType::Fog,
            5 => WeatherType::Blizzard,
            _ => WeatherType::Clear,
        }
    }
}

impl fmt::Display for WeatherType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ---------------------------------------------------------------------------
// WeatherState
// ---------------------------------------------------------------------------

/// Full snapshot of current weather conditions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WeatherState {
    pub current: WeatherType,
    /// 0.0 (calm) – 1.0 (extreme).
    pub intensity: f64,
    /// Remaining ticks before a transition is re-evaluated.
    pub duration_ticks: u32,
    /// Wind speed in km/h.
    pub wind_speed: f64,
    /// Temperature in degrees Celsius.
    pub temperature: f64,
}

impl WeatherState {
    pub fn new(current: WeatherType) -> Self {
        WeatherState {
            current,
            intensity: 0.5,
            duration_ticks: 10,
            wind_speed: 10.0,
            temperature: 15.0,
        }
    }
}

// ---------------------------------------------------------------------------
// WeatherTransitionMatrix
// ---------------------------------------------------------------------------

/// 6×6 row-stochastic transition probability matrix.
///
/// Row `i` gives the probability distribution over next states given
/// the current weather is `WeatherType::from_index(i)`.
#[derive(Debug, Clone)]
pub struct WeatherTransitionMatrix {
    /// `matrix[from][to]` — values in each row must sum to 1.0.
    pub matrix: [[f64; 6]; 6],
}

impl WeatherTransitionMatrix {
    /// Default realistic transition probabilities.
    pub fn default_transitions() -> Self {
        WeatherTransitionMatrix {
            matrix: [
                // Clear
                [0.60, 0.25, 0.08, 0.02, 0.04, 0.01],
                // Cloudy
                [0.20, 0.40, 0.20, 0.05, 0.10, 0.05],
                // Rain
                [0.10, 0.20, 0.40, 0.15, 0.10, 0.05],
                // Storm
                [0.05, 0.10, 0.25, 0.40, 0.10, 0.10],
                // Fog
                [0.15, 0.25, 0.20, 0.05, 0.30, 0.05],
                // Blizzard
                [0.05, 0.10, 0.10, 0.10, 0.10, 0.55],
            ],
        }
    }

    /// Choose the next weather type using the given uniform random value in [0, 1).
    pub fn next_state(&self, current: WeatherType, r: f64) -> WeatherType {
        let row = &self.matrix[current.index()];
        let mut cumulative = 0.0;
        for (i, &p) in row.iter().enumerate() {
            cumulative += p;
            if r < cumulative {
                return WeatherType::from_index(i);
            }
        }
        // Fallback due to floating-point rounding.
        WeatherType::from_index(5)
    }
}

// ---------------------------------------------------------------------------
// WeatherSystem
// ---------------------------------------------------------------------------

/// Manages weather simulation and modifier calculations.
pub struct WeatherSystem {
    pub transition_matrix: WeatherTransitionMatrix,
    lcg_state: u64,
}

impl WeatherSystem {
    pub fn new() -> Self {
        WeatherSystem {
            transition_matrix: WeatherTransitionMatrix::default_transitions(),
            lcg_state: 42,
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        WeatherSystem {
            transition_matrix: WeatherTransitionMatrix::default_transitions(),
            lcg_state: seed,
        }
    }

    /// Advance weather by one tick using the provided seed and the internal LCG.
    ///
    /// The `seed` parameter is XOR'd into the LCG state for external entropy.
    pub fn tick(&mut self, seed: u64, current: &WeatherState) -> WeatherState {
        self.lcg_state = lcg_next(self.lcg_state ^ seed);
        let r_transition = (self.lcg_state as f64) / (LCG_M as f64);

        self.lcg_state = lcg_next(self.lcg_state);
        let r_intensity = (self.lcg_state as f64) / (LCG_M as f64);

        self.lcg_state = lcg_next(self.lcg_state);
        let r_wind = (self.lcg_state as f64) / (LCG_M as f64);

        self.lcg_state = lcg_next(self.lcg_state);
        let r_temp = (self.lcg_state as f64) / (LCG_M as f64);

        let next_type = if current.duration_ticks == 0 {
            self.transition_matrix.next_state(current.current, r_transition)
        } else {
            current.current
        };

        let new_duration = if next_type != current.current || current.duration_ticks == 0 {
            5 + (r_transition * 20.0) as u32
        } else {
            current.duration_ticks.saturating_sub(1)
        };

        let intensity = (0.2 + r_intensity * 0.8).clamp(0.0, 1.0);
        let wind_speed = r_wind * 120.0;
        let temperature = -30.0 + r_temp * 60.0;

        WeatherState {
            current: next_type,
            intensity,
            duration_ticks: new_duration,
            wind_speed,
            temperature,
        }
    }

    /// Apply weather penalty to base accuracy (0.0 – 1.0).
    pub fn apply_combat_modifier(base_accuracy: f64, state: &WeatherState) -> f64 {
        let vis_penalty = state.current.visibility_modifier();
        let intensity_penalty = 1.0 - state.intensity * 0.2;
        (base_accuracy * vis_penalty * intensity_penalty).clamp(0.0, 1.0)
    }

    /// Apply weather penalty to base speed.
    pub fn apply_travel_modifier(base_speed: f64, state: &WeatherState) -> f64 {
        let movement_penalty = state.current.movement_penalty();
        let intensity_factor = 1.0 - state.intensity * movement_penalty;
        (base_speed * intensity_factor).max(0.0)
    }

    /// Generate a narrative description of the current weather.
    pub fn weather_description(state: &WeatherState) -> String {
        let intensity_word = if state.intensity > 0.75 {
            "severe"
        } else if state.intensity > 0.5 {
            "moderate"
        } else if state.intensity > 0.25 {
            "light"
        } else {
            "mild"
        };

        let base = match state.current {
            WeatherType::Clear   => format!("The skies are clear and bright. Wind: {:.0} km/h.", state.wind_speed),
            WeatherType::Cloudy  => format!("Thick clouds roll overhead, dimming the world. Wind: {:.0} km/h.", state.wind_speed),
            WeatherType::Rain    => format!("{} rain falls from grey skies. Wind: {:.0} km/h.", intensity_word, state.wind_speed),
            WeatherType::Storm   => format!("A {} storm rages — lightning, thunder, torrential rain. Wind: {:.0} km/h.", intensity_word, state.wind_speed),
            WeatherType::Fog     => format!("{} fog clings to the ground, muffling all sound. Wind: {:.0} km/h.", intensity_word, state.wind_speed),
            WeatherType::Blizzard => format!("A {} blizzard howls. Ice and snow obscure everything. Wind: {:.0} km/h.", intensity_word, state.wind_speed),
        };

        format!("{} Temperature: {:.1}°C.", base, state.temperature)
    }
}

impl Default for WeatherSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_modifier_clear_is_one() {
        assert!((WeatherType::Clear.visibility_modifier() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_visibility_modifier_blizzard_is_lowest() {
        let blizzard_vis = WeatherType::Blizzard.visibility_modifier();
        assert!(blizzard_vis < WeatherType::Storm.visibility_modifier());
        assert!(blizzard_vis < WeatherType::Fog.visibility_modifier());
    }

    #[test]
    fn test_movement_penalty_clear_is_zero() {
        assert!((WeatherType::Clear.movement_penalty()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_movement_penalty_blizzard_is_highest() {
        let blizzard = WeatherType::Blizzard.movement_penalty();
        assert!(blizzard > WeatherType::Storm.movement_penalty());
        assert!(blizzard > WeatherType::Rain.movement_penalty());
    }

    #[test]
    fn test_transition_matrix_rows_sum_to_one() {
        let m = WeatherTransitionMatrix::default_transitions();
        for row in &m.matrix {
            let sum: f64 = row.iter().sum();
            assert!((sum - 1.0).abs() < 1e-9, "Row does not sum to 1: {}", sum);
        }
    }

    #[test]
    fn test_transition_matrix_next_state_r_zero_stays_in_first_high_prob_state() {
        let m = WeatherTransitionMatrix::default_transitions();
        // Clear -> Clear should dominate at r=0.0.
        let next = m.next_state(WeatherType::Clear, 0.0);
        assert_eq!(next, WeatherType::Clear);
    }

    #[test]
    fn test_tick_returns_valid_weather_state() {
        let mut sys = WeatherSystem::with_seed(12345);
        let state = WeatherState::new(WeatherType::Clear);
        let next = sys.tick(99, &state);
        // intensity must be in [0, 1]
        assert!(next.intensity >= 0.0 && next.intensity <= 1.0);
    }

    #[test]
    fn test_apply_combat_modifier_reduces_accuracy_in_storm() {
        let state = WeatherState {
            current: WeatherType::Storm,
            intensity: 1.0,
            duration_ticks: 5,
            wind_speed: 80.0,
            temperature: -5.0,
        };
        let modified = WeatherSystem::apply_combat_modifier(1.0, &state);
        assert!(modified < 1.0);
    }

    #[test]
    fn test_apply_combat_modifier_clear_near_base() {
        let state = WeatherState {
            current: WeatherType::Clear,
            intensity: 0.0,
            duration_ticks: 10,
            wind_speed: 5.0,
            temperature: 20.0,
        };
        let modified = WeatherSystem::apply_combat_modifier(0.9, &state);
        assert!((modified - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_apply_travel_modifier_slows_in_blizzard() {
        let state = WeatherState {
            current: WeatherType::Blizzard,
            intensity: 1.0,
            duration_ticks: 5,
            wind_speed: 100.0,
            temperature: -20.0,
        };
        let modified = WeatherSystem::apply_travel_modifier(10.0, &state);
        assert!(modified < 10.0);
    }

    #[test]
    fn test_weather_description_contains_temperature() {
        let state = WeatherState {
            current: WeatherType::Rain,
            intensity: 0.5,
            duration_ticks: 3,
            wind_speed: 20.0,
            temperature: 12.3,
        };
        let desc = WeatherSystem::weather_description(&state);
        assert!(desc.contains("12.3"), "Description: {}", desc);
    }

    #[test]
    fn test_weather_description_mentions_wind() {
        let state = WeatherState::new(WeatherType::Clear);
        let desc = WeatherSystem::weather_description(&state);
        assert!(desc.contains("km/h"), "Description: {}", desc);
    }

    #[test]
    fn test_multiple_ticks_produce_deterministic_sequence() {
        let mut sys1 = WeatherSystem::with_seed(7777);
        let mut sys2 = WeatherSystem::with_seed(7777);
        let state = WeatherState::new(WeatherType::Cloudy);
        let s1 = sys1.tick(42, &state);
        let s2 = sys2.tick(42, &state);
        assert_eq!(s1.current, s2.current);
        assert!((s1.intensity - s2.intensity).abs() < f64::EPSILON);
    }

    #[test]
    fn test_index_roundtrip() {
        for i in 0..6 {
            assert_eq!(WeatherType::from_index(i).index(), i);
        }
    }
}
