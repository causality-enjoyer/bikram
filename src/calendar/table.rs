//! Derived lookup tables over the raw month-length data.

use super::data::{FIRST_YEAR, MONTH_LENGTHS};

pub(super) const YEAR_COUNT: usize = MONTH_LENGTHS.len();

/// Last BS year in the table
pub const LAST_YEAR: i32 = FIRST_YEAR + YEAR_COUNT as i32 - 1;

/// Day index of the first day of each BS year, with the grand total last
pub(super) const YEAR_OFFSETS: [u32; YEAR_COUNT + 1] = build_year_offsets();

pub(super) const TOTAL_DAYS: u32 = YEAR_OFFSETS[YEAR_COUNT];

const fn build_year_offsets() -> [u32; YEAR_COUNT + 1] {
    let mut offsets = [0; YEAR_COUNT + 1];
    let mut year = 0;
    while year < YEAR_COUNT {
        let mut days_in_year = 0;
        let mut month = 0;
        while month < 12 {
            days_in_year += MONTH_LENGTHS[year][month] as u32;
            month += 1;
        }
        offsets[year + 1] = offsets[year] + days_in_year;
        year += 1;
    }
    offsets
}

/// Caller guarantees `year` is in range and `month` is 1..=12
pub(super) const fn month_length(year: i32, month: u8) -> u8 {
    MONTH_LENGTHS[(year - FIRST_YEAR) as usize][(month - 1) as usize]
}

/// Caller guarantees `year` is in range
pub(super) const fn year_length(year: i32) -> u32 {
    let i = (year - FIRST_YEAR) as usize;
    YEAR_OFFSETS[i + 1] - YEAR_OFFSETS[i]
}
