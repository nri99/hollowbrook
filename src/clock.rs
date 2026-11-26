use chrono::{Datelike, Duration, Local, NaiveDateTime};

/// Gameplay-day id: rolls over at 06:00 local, distinct from the calendar date (midnight).
pub fn day_id(t: NaiveDateTime) -> i64 {
    (t - Duration::hours(6)).date().num_days_from_ce() as i64
}

/// The single authoritative clock: real local time, or (dev only) HOLLOWBROOK_DEV_TIME=YYYY-MM-DDTHH:MM:SS,
/// a start time that then advances at real speed. Interiors, menus and resting all read this same clock.
pub struct Clock {
    pub dev: Option<(NaiveDateTime, std::time::Instant)>,
}
impl Clock {
    pub fn from_env() -> Clock {
        Clock {
            dev: std::env::var("HOLLOWBROOK_DEV_TIME")
                .ok()
                .and_then(|s| s.parse().ok())
                .map(|t| (t, std::time::Instant::now())),
        }
    }
    pub fn now(&self) -> NaiveDateTime {
        match self.dev {
            Some((t, i)) => t + Duration::from_std(i.elapsed()).unwrap(),
            None => Local::now().naive_local(),
        }
    }
    pub fn source(&self) -> &'static str {
        if self.dev.is_some() {
            "dev"
        } else {
            "real"
        }
    }
}
