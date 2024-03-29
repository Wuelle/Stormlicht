//! Provides various Date and Time utilities

use std::time::{SystemTime, UNIX_EPOCH};

pub mod consts;
mod date;
mod time;

pub use date::Date;
pub use time::Time;

use self::date::{Month, Year, YearRange};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    InvalidWeekday,
    InvalidMonth,
    InvalidDay,
    InvalidYear,
    InvalidHour,
    InvalidMinute,
    InvalidSecond,
    MissingDay,
    MissingMonth,
    MissingYear,
    MissingTime,
    MissingHour,
    MissingMinute,
    IncorrectWeekday,
}

impl Weekday {
    /// Parse a [Weekday] as defined in [RFC 822](https://datatracker.ietf.org/doc/html/rfc822)
    pub fn from_rfc822(s: &str) -> Result<Self, ParseError> {
        match s {
            "Mon" => Ok(Self::Monday),
            "Tue" => Ok(Self::Tuesday),
            "Wed" => Ok(Self::Wednesday),
            "Thu" => Ok(Self::Thursday),
            "Fri" => Ok(Self::Friday),
            "Sat" => Ok(Self::Saturday),
            "Sun" => Ok(Self::Sunday),
            _ => Err(ParseError::InvalidWeekday),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime {
    date: Date,
    time: Time,
}

impl DateTime {
    /// Return the current [DateTime]
    ///
    /// # Panics
    /// This function panics if the current system time is before [UNIX_EPOCH].
    #[must_use]
    pub fn now() -> Self {
        let since_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before unix epoch");
        Self::from_unix_timestamp(since_unix.as_secs())
    }

    #[must_use]
    pub const fn from_unix_timestamp(seconds: u64) -> Self {
        let days = seconds / consts::SECONDS_PER_DAY as u64;
        let seconds = seconds % consts::SECONDS_PER_DAY as u64;

        let date = Date::new_from_days_since_unix(days as i32);
        let time = Time::new_from_n_seconds_since_midnight(seconds);

        Self { date, time }
    }

    pub fn from_ymd_hms(
        year: u64,
        month: u8,
        day: u8,
        hour: u64,
        minute: u64,
        second: u64,
    ) -> Option<Self> {
        let date = Date::from_ymd(Year::new(year as YearRange), Month::from_index(month), day);
        let time = Time::from_hms(hour, minute, second)?;

        Some(Self { date, time })
    }

    pub fn date(&self) -> Date {
        self.date
    }

    pub fn time(&self) -> Time {
        self.time
    }
}
