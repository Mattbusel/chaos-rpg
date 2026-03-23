/// The four seasons.
#[derive(Debug, Clone, PartialEq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    /// Average daylight hours per season.
    pub fn daylight_hours(&self) -> f32 {
        match self {
            Season::Spring => 12.0,
            Season::Summer => 14.0,
            Season::Autumn => 10.0,
            Season::Winter => 8.0,
        }
    }
}

/// Time of day bucket.
#[derive(Debug, Clone, PartialEq)]
pub enum TimeOfDay {
    Dawn,
    Morning,
    Afternoon,
    Dusk,
    Evening,
    Night,
    Midnight,
}

/// Calendar date and time.
#[derive(Debug, Clone, PartialEq)]
pub struct CalendarDate {
    pub year: u32,
    pub month: u8,  // 1-12
    pub day: u8,    // 1-30
    pub hour: u8,   // 0-23
    pub minute: u8, // 0-59
}

impl CalendarDate {
    pub fn new(year: u32, month: u8, day: u8, hour: u8, minute: u8) -> Self {
        Self { year, month, day, hour, minute }
    }

    /// Advance by the given number of minutes, wrapping fields as needed.
    pub fn advance_minutes(&mut self, minutes: u32) {
        let total_minutes = self.minute as u32 + minutes;
        self.minute = (total_minutes % 60) as u8;
        let extra_hours = total_minutes / 60;

        let total_hours = self.hour as u32 + extra_hours;
        self.hour = (total_hours % 24) as u8;
        let extra_days = total_hours / 24;

        // days are 1-based, 30 per month
        let day0 = self.day as u32 - 1 + extra_days;
        let extra_months = day0 / 30;
        self.day = (day0 % 30 + 1) as u8;

        let month0 = self.month as u32 - 1 + extra_months;
        self.month = (month0 % 12 + 1) as u8;
        self.year += month0 / 12;
    }

    /// Map hour to a time-of-day bucket.
    ///
    /// Dawn 5-7, Morning 7-12, Afternoon 12-17, Dusk 17-19,
    /// Evening 19-22, Night 22-24/0-3, Midnight 3-5.
    pub fn time_of_day(&self) -> TimeOfDay {
        match self.hour {
            5..=6 => TimeOfDay::Dawn,
            7..=11 => TimeOfDay::Morning,
            12..=16 => TimeOfDay::Afternoon,
            17..=18 => TimeOfDay::Dusk,
            19..=21 => TimeOfDay::Evening,
            22..=23 | 0..=2 => TimeOfDay::Night,
            3..=4 => TimeOfDay::Midnight,
            _ => TimeOfDay::Night,
        }
    }

    /// Derive the current season from the month.
    ///
    /// 3-5 = Spring, 6-8 = Summer, 9-11 = Autumn, 12/1-2 = Winter.
    pub fn season(&self) -> Season {
        match self.month {
            3..=5 => Season::Spring,
            6..=8 => Season::Summer,
            9..=11 => Season::Autumn,
            12 | 1 | 2 => Season::Winter,
            _ => Season::Winter,
        }
    }

    /// Day of the year (1-based).
    pub fn day_of_year(&self) -> u32 {
        (self.month as u32 - 1) * 30 + self.day as u32
    }
}

/// Ambient light level for a given time of day.
/// Returns 1.0 for full daylight, 0.0 for pitch night.
pub fn light_level(time_of_day: &TimeOfDay) -> f32 {
    match time_of_day {
        TimeOfDay::Dawn => 0.4,
        TimeOfDay::Morning => 0.9,
        TimeOfDay::Afternoon => 1.0,
        TimeOfDay::Dusk => 0.5,
        TimeOfDay::Evening => 0.2,
        TimeOfDay::Night => 0.05,
        TimeOfDay::Midnight => 0.0,
    }
}

/// Environmental effects for a given season.
#[derive(Debug, Clone)]
pub struct SeasonalEffect {
    pub season: Season,
    pub temperature_mod: f32,
    pub travel_speed_mod: f32,
    pub creature_spawn_bias: String,
    pub encounter_chance_mod: f32,
}

/// Return the typical seasonal effects.
pub fn seasonal_effects(season: &Season) -> SeasonalEffect {
    match season {
        Season::Spring => SeasonalEffect {
            season: Season::Spring,
            temperature_mod: 0.0,
            travel_speed_mod: 1.0,
            creature_spawn_bias: "Beasts and Fey".to_string(),
            encounter_chance_mod: 1.1,
        },
        Season::Summer => SeasonalEffect {
            season: Season::Summer,
            temperature_mod: 10.0,
            travel_speed_mod: 1.1,
            creature_spawn_bias: "Insects and Dragons".to_string(),
            encounter_chance_mod: 1.2,
        },
        Season::Autumn => SeasonalEffect {
            season: Season::Autumn,
            temperature_mod: -5.0,
            travel_speed_mod: 0.95,
            creature_spawn_bias: "Undead and Spirits".to_string(),
            encounter_chance_mod: 1.3,
        },
        Season::Winter => SeasonalEffect {
            season: Season::Winter,
            temperature_mod: -20.0,
            travel_speed_mod: 0.75,
            creature_spawn_bias: "Giants and Wolves".to_string(),
            encounter_chance_mod: 0.8,
        },
    }
}

/// World-level time manager with a configurable speed multiplier.
pub struct WorldTime {
    pub date: CalendarDate,
    /// How many in-game minutes pass per real minute.
    pub speed_multiplier: u32,
}

impl WorldTime {
    /// Create a new WorldTime starting at year 1, month 3, day 1, 06:00.
    pub fn new(speed: u32) -> Self {
        Self {
            date: CalendarDate::new(1, 3, 1, 6, 0),
            speed_multiplier: speed,
        }
    }

    /// Advance game time by real_minutes * speed_multiplier game minutes.
    pub fn tick(&mut self, real_minutes: u32) {
        self.date.advance_minutes(real_minutes * self.speed_multiplier);
    }

    /// Returns true if the current time is daytime (Dawn through Dusk).
    pub fn is_daytime(&self) -> bool {
        matches!(
            self.date.time_of_day(),
            TimeOfDay::Dawn | TimeOfDay::Morning | TimeOfDay::Afternoon | TimeOfDay::Dusk
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advance_minutes_wraps_days() {
        let mut d = CalendarDate::new(1, 1, 30, 23, 50);
        d.advance_minutes(20); // 20 min -> next day
        assert_eq!(d.day, 1);
        assert_eq!(d.month, 2);
    }

    #[test]
    fn test_advance_minutes_wraps_months() {
        let mut d = CalendarDate::new(1, 12, 30, 23, 59);
        d.advance_minutes(2);
        assert_eq!(d.month, 1);
        assert_eq!(d.year, 2);
    }

    #[test]
    fn test_advance_minutes_simple() {
        let mut d = CalendarDate::new(1, 6, 15, 10, 30);
        d.advance_minutes(90);
        assert_eq!(d.hour, 12);
        assert_eq!(d.minute, 0);
    }

    #[test]
    fn test_time_of_day_correct_for_hour() {
        let d = CalendarDate::new(1, 1, 1, 6, 0);
        assert_eq!(d.time_of_day(), TimeOfDay::Dawn);
        let d2 = CalendarDate::new(1, 1, 1, 14, 0);
        assert_eq!(d2.time_of_day(), TimeOfDay::Afternoon);
        let d3 = CalendarDate::new(1, 1, 1, 23, 0);
        assert_eq!(d3.time_of_day(), TimeOfDay::Night);
        let d4 = CalendarDate::new(1, 1, 1, 4, 0);
        assert_eq!(d4.time_of_day(), TimeOfDay::Midnight);
    }

    #[test]
    fn test_season_from_month() {
        assert_eq!(CalendarDate::new(1, 4, 1, 0, 0).season(), Season::Spring);
        assert_eq!(CalendarDate::new(1, 7, 1, 0, 0).season(), Season::Summer);
        assert_eq!(CalendarDate::new(1, 10, 1, 0, 0).season(), Season::Autumn);
        assert_eq!(CalendarDate::new(1, 1, 1, 0, 0).season(), Season::Winter);
        assert_eq!(CalendarDate::new(1, 12, 1, 0, 0).season(), Season::Winter);
    }

    #[test]
    fn test_light_level_day_night() {
        assert_eq!(light_level(&TimeOfDay::Afternoon), 1.0);
        assert_eq!(light_level(&TimeOfDay::Midnight), 0.0);
        assert!(light_level(&TimeOfDay::Dawn) > 0.0);
        assert!(light_level(&TimeOfDay::Night) < 0.2);
    }

    #[test]
    fn test_seasonal_effects_non_nil() {
        let fx = seasonal_effects(&Season::Winter);
        assert!(!fx.creature_spawn_bias.is_empty());
        assert!(fx.travel_speed_mod < 1.0);

        let fx2 = seasonal_effects(&Season::Summer);
        assert!(fx2.temperature_mod > 0.0);
        assert!(!fx2.creature_spawn_bias.is_empty());
    }

    #[test]
    fn test_world_time_is_daytime() {
        let mut wt = WorldTime::new(1);
        // starts at 6:00 (Dawn) — is daytime
        assert!(wt.is_daytime());
        wt.tick(1000); // advance many minutes into night
        // Just check it returns a bool without panic
        let _ = wt.is_daytime();
    }
}
