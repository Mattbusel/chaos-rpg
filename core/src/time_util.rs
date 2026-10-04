//! Calendar dates for scoreboards, the daily leaderboard and the bestiary.
//!
//! Built on the [`time`] crate. The frontends used to carry four separate
//! hand-written conversions from Unix time to a date. Two of them assumed
//! 365-day years and 30-day months, so on 2025-10-03 they printed
//! `2025-10-20`; the Proof Engine frontend then asked the leaderboard server
//! for the wrong day.
//!
//! ```
//! use chaos_rpg_core::time_util::{date_from_unix, timestamp_from_unix};
//! assert_eq!(date_from_unix(1_759_449_600), "2025-10-03");
//! assert_eq!(timestamp_from_unix(1_709_210_096), "2024-02-29 12:34Z");
//! ```

use time::OffsetDateTime;

fn utc(secs: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(secs).unwrap_or(OffsetDateTime::UNIX_EPOCH)
}

/// Seconds since the Unix epoch, now (0 if the clock is before 1970).
pub fn now_unix() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp().max(0)
}

/// UTC calendar date of a Unix timestamp as `YYYY-MM-DD`.
pub fn date_from_unix(secs: i64) -> String {
    let d = utc(secs).date();
    format!("{:04}-{:02}-{:02}", d.year(), u8::from(d.month()), d.day())
}

/// UTC date and minute of a Unix timestamp as `YYYY-MM-DD HH:MMZ`.
pub fn timestamp_from_unix(secs: i64) -> String {
    let t = utc(secs);
    format!(
        "{} {:02}:{:02}Z",
        date_from_unix(secs),
        t.hour(),
        t.minute()
    )
}

/// Today's UTC date as `YYYY-MM-DD`: the key the daily leaderboard server
/// uses for a day's scores.
pub fn today_utc() -> String {
    date_from_unix(now_unix())
}

/// The current UTC date and minute as `YYYY-MM-DD HH:MMZ`.
pub fn now_timestamp() -> String {
    timestamp_from_unix(now_unix())
}

/// Number of whole UTC days since 1970-01-01. Daily seeds are derived from it.
pub fn utc_day_number() -> u64 {
    (now_unix() / 86_400) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_dates() {
        assert_eq!(date_from_unix(0), "1970-01-01");
        assert_eq!(date_from_unix(951_782_400), "2000-02-29");
        assert_eq!(date_from_unix(1_709_164_800), "2024-02-29");
        assert_eq!(date_from_unix(1_735_603_200), "2024-12-31");
        assert_eq!(date_from_unix(1_735_689_600), "2025-01-01");
        assert_eq!(date_from_unix(1_759_449_600), "2025-10-03");
        assert_eq!(date_from_unix(1_759_449_600 + 86_399), "2025-10-03");
    }

    #[test]
    fn timestamps() {
        assert_eq!(timestamp_from_unix(0), "1970-01-01 00:00Z");
        assert_eq!(timestamp_from_unix(1_759_449_600 + 23 * 3600 + 59 * 60 + 59), "2025-10-03 23:59Z");
    }

    /// The approximation the bestiary, achievements and Proof Engine
    /// leaderboard used before (365-day years, 30-day months) is wrong for
    /// almost every day; keep a copy here so the difference stays visible.
    #[test]
    fn old_approximation_was_wrong() {
        fn old(secs: u64) -> String {
            let days = secs / 86400;
            let year = 1970 + days / 365;
            let doy = days % 365;
            format!("{:04}-{:02}-{:02}", year, (doy / 30 + 1).min(12), (doy % 30 + 1).min(31))
        }
        assert_eq!(old(1_759_449_600), "2025-10-20");
        assert_eq!(date_from_unix(1_759_449_600), "2025-10-03");
    }

    #[test]
    fn today_is_well_formed() {
        let t = today_utc();
        assert_eq!(t.len(), 10);
        assert!(t.starts_with("20"));
    }
}
