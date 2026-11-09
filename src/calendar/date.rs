use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, Utc, Weekday};

use super::confidence::{self, Confidence};
use super::data::{ANCHOR_AD, FIRST_YEAR, MONTH_LENGTHS};
use super::error::{DateError, ParseError};
use super::month::BsMonth;
use super::parse;
use super::range::BsRange;
use super::table::{self, LAST_YEAR, TOTAL_DAYS, YEAR_COUNT, YEAR_OFFSETS};

/// Nepal Standard Time, UTC+05:45. No Daylight Saving Time (the correct way to live ;)
/// so a fixed offset gets the job done.
pub fn nepal_offset() -> FixedOffset {
    FixedOffset::east_opt(5 * 3600 + 45 * 60).expect("valid offset")
}

fn anchor() -> NaiveDate {
    let (y, m, d) = ANCHOR_AD;
    NaiveDate::from_ymd_opt(y, m as u32, d as u32).expect("valid anchor date")
}

/// A date in the BS calendar. `month` is 1 (Baisakh) to 12 (Chatira).
/// Ordering is chronological (fields are declared as year, month, day).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BsDate {
    year: i32,
    month: u8,
    day: u8,
}

impl BsDate {
    /// Earliest supported date (1 Baisakh of the first table year)
    pub const MIN: BsDate = BsDate {
        year: FIRST_YEAR,
        month: 1,
        day: 1,
    };

    /// Latest supported date (last day of Chaitra of the last table year)
    pub const MAX: BsDate = BsDate {
        year: LAST_YEAR,
        month: 12,
        day: MONTH_LENGTHS[YEAR_COUNT - 1][11],
    };

    // ---- Construction

    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, DateError> {
        if !(FIRST_YEAR..=LAST_YEAR).contains(&year) {
            return Err(DateError::OutOfRange);
        }
        if !(1..=12).contains(&month) || day < 1 || day > table::month_length(year, month) {
            return Err(DateError::NoSuchDate);
        }
        Ok(BsDate { year, month, day })
    }

    pub fn from_parts(year: i32, month: BsMonth, day: u8) -> Result<Self, DateError> {
        Self::new(year, month.number(), day)
    }

    /// The BS day it is in Nepal at this instant (BS days roll over at Nepal-local midnight,
    /// not UTC midnight)
    pub fn from_instant(t: DateTime<Utc>) -> Result<Self, DateError> {
        Self::from_ad(t.with_timezone(&nepal_offset()).date_naive())
    }

    /// Today's date in Nepal
    pub fn today() -> Result<Self, DateError> {
        Self::from_instant(Utc::now())
    }

    pub fn from_ad(ad: NaiveDate) -> Result<Self, DateError> {
        let index = ad.signed_duration_since(anchor()).num_days();
        let index = u32::try_from(index).map_err(|_| DateError::OutOfRange)?;
        Self::from_day_index(index)
    }

    /// Parse with a reference date, enabling partial (`25 kar`) and relative
    /// (`+3d`, `tomorrow`) input. See the `parse` module for all shapes.
    pub fn parse_relative_to(input: &str, today: BsDate) -> Result<Self, ParseError> {
        parse::parse(input, Some(today))
    }

    // ---- Accessors

    pub fn year(&self) -> i32 {
        self.year
    }

    pub fn month(&self) -> u8 {
        self.month
    }

    pub fn day(&self) -> u8 {
        self.day
    }

    pub fn month_name(&self) -> BsMonth {
        BsMonth::from_number(self.month).expect("Month is validated at construction")
    }

    pub fn weekday(&self) -> Weekday {
        self.to_ad().weekday()
    }

    /// 1-based day of the BS year.
    pub fn ordinal(&self) -> u32 {
        self.day_index() - YEAR_OFFSETS[(self.year - FIRST_YEAR) as usize] + 1
    }

    /// How trustworthy this date's year data is
    pub fn confidence(&self) -> Confidence {
        confidence::confidence(self.year).expect("Year is validated at construction")
    }

    // ---- table queries

    pub fn is_valid_year(year: i32) -> bool {
        (FIRST_YEAR..=LAST_YEAR).contains(&year)
    }

    pub fn days_in_month(year: i32, month: u8) -> Result<u8, DateError> {
        Self::new(year, month, 1)?;
        Ok(table::month_length(year, month))
    }

    pub fn days_in_year(year: i32) -> Result<u32, DateError> {
        if !Self::is_valid_year(year) {
            return Err(DateError::OutOfRange);
        }
        Ok(table::year_length(year))
    }

    // ---- conversion
    pub fn to_ad(&self) -> NaiveDate {
        anchor() + Duration::days(self.day_index() as i64)
    }

    // ---- arithmetic

    pub fn add_days(&self, n: i64) -> Result<Self, DateError> {
        let index = self.day_index() as i64 + n;
        let index = u32::try_from(index).map_err(|_| DateError::OutOfRange)?;
        Self::from_day_index(index)
    }

    /// Adds calendar months. If theh target month is shorter, the day is clamped
    /// to its last day (32 Jestha + 1 month = 31 Asar).
    pub fn add_months(&self, n: i32) -> Result<Self, DateError> {
        let total = self.year as i64 * 12 + (self.month as i64 - 1) + n as i64;
        let year = i32::try_from(total.div_euclid(12)).map_err(|_| DateError::OutOfRange)?;
        if !Self::is_valid_year(year) {
            return Err(DateError::OutOfRange);
        }
        let month = total.rem_euclid(12) as u8 + 1;
        let day = self.day.min(table::month_length(year, month));
        Ok(BsDate { year, month, day })
    }

    /// Same clamping rule as [`add_months`](Self::add_months).
    pub fn add_years(&self, n: i32) -> Result<Self, DateError> {
        self.add_months(n.checked_mul(12).ok_or(DateError::OutOfRange)?)
    }

    pub fn next_day(&self) -> Result<Self, DateError> {
        self.add_days(1)
    }

    pub fn prev_days(&self) -> Result<Self, DateError> {
        self.add_days(-1)
    }

    pub fn days_until(&self, other: BsDate) -> i64 {
        other.day_index() as i64 - self.day_index() as i64
    }

    // ---- ranges

    pub fn month_range(&self) -> BsRange {
        BsRange {
            start: BsDate { day: 1, ..*self },
            end: BsDate {
                day: table::month_length(self.year, self.month),
                ..*self
            },
        }
    }

    pub fn year_range(&self) -> BsRange {
        BsRange {
            start: BsDate {
                month: 1,
                day: 1,
                ..*self
            },
            end: BsDate {
                month: 12,
                day: table::month_length(self.year, 12),
                ..*self
            },
        }
    }

    /// The 7-day week c ontaining this date. Nepal's calendar was recently changed to Monday
    pub fn week_range(&self, week_start: Weekday) -> Result<BsRange, DateError> {
        let back = self.weekday().days_since(week_start) as i64;
        let start = self.add_days(-back)?;
        let end = start.add_days(6)?;
        Ok(BsRange { start, end })
    }

    pub fn first_of_month(&self) -> BsDate {
        self.month_range().start
    }

    pub fn last_of_month(&self) -> BsDate {
        self.month_range().end
    }

    // ---- Day Index (crate-internal)

    /// Days since 1 Baisakh of the first table year.
    pub(super) fn day_index(&self) -> u32 {
        let year_start = YEAR_OFFSETS[(self.year - FIRST_YEAR) as usize];
        let days_before_month: u32 = (1..self.month)
            .map(|m| table::month_length(self.year, m) as u32)
            .sum();
        year_start + days_before_month + (self.day as u32 - 1)
    }

    pub(super) fn from_day_index(index: u32) -> Result<Self, DateError> {
        if index >= TOTAL_DAYS {
            return Err(DateError::OutOfRange);
        }

        // Count of year starts at or before `index` is the year's position + 1.
        let year_pos = YEAR_OFFSETS.partition_point(|&start| start <= index) - 1;
        let year = FIRST_YEAR + year_pos as i32;

        let mut remaining = index - YEAR_OFFSETS[year_pos];
        let mut month = 1;
        while remaining >= table::month_length(year, month) as u32 {
            remaining -= table::month_length(year, month) as u32;
            month += 1;
        }

        Ok(BsDate {
            year,
            month,
            day: remaining as u8 + 1,
        })
    }
}

impl fmt::Display for BsDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl FromStr for BsDate {
    type Err = ParseError;

    /// Strict form: needs a full date (`2083-07-25`, '25 Kartik 2083').
    /// For partial/relative input use [`BsDate::parse_relative_to`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::parse(s, None)
    }
}
