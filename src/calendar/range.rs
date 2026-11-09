use chrono::NaiveDate;

use crate::calendar::{BsDate, DateError, table::TOTAL_DAYS};

/// An inclusive range of BS days (`start <= end`)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BsRange {
    pub(super) start: BsDate,
    pub(super) end: BsDate,
}

impl BsRange {
    /// `None` if `end < start`.
    pub fn new(start: BsDate, end: BsDate) -> Option<Self> {
        (start <= end).then_some(BsRange { start, end })
    }

    pub fn month(year: i32, month: u8) -> Result<Self, DateError> {
        Ok(BsDate::new(year, month, 1)?.month_range())
    }

    pub fn year(year: i32) -> Result<Self, DateError> {
        Ok(BsDate::new(year, 1, 1)?.year_range())
    }

    pub fn start(&self) -> BsDate {
        self.start
    }

    pub fn end(&self) -> BsDate {
        self.end
    }

    pub fn contains(&self, date: BsDate) -> bool {
        self.start <= date && date <= self.end
    }

    /// Number of days (always at least 1)
    pub fn len_days(&self) -> u32 {
        self.end.day_index() - self.start.day_index() + 1
    }

    /// The same range in AD, inclusive on both ends
    pub fn to_ad(&self) -> (NaiveDate, NaiveDate) {
        (self.start.to_ad(), self.end.to_ad())
    }

    pub fn iter(&self) -> BsDays {
        BsDays {
            front: self.start.day_index(),
            back: self.end.day_index() + 1,
        }
    }
}

impl IntoIterator for BsRange {
    type Item = BsDate;
    type IntoIter = BsDays;
    fn into_iter(self) -> BsDays {
        self.iter()
    }
}

impl IntoIterator for &BsRange {
    type Item = BsDate;
    type IntoIter = BsDays;
    fn into_iter(self) -> BsDays {
        self.iter()
    }
}

/// Iterator over the days of a [`BsRange`]
#[derive(Debug, Clone)]
pub struct BsDays {
    front: u32,
    /// Exclusive
    back: u32,
}

impl Iterator for BsDays {
    type Item = BsDate;

    fn next(&mut self) -> Option<BsDate> {
        if self.front >= self.back || self.back > TOTAL_DAYS {
            return None;
        }
        let d = BsDate::from_day_index(self.front).ok();
        self.front += 1;
        d
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.back.saturating_sub(self.front) as usize;
        (n, Some(n))
    }
}

impl DoubleEndedIterator for BsDays {
    fn next_back(&mut self) -> Option<BsDate> {
        if self.front >= self.back {
            return None;
        }
        self.back -= 1;
        BsDate::from_day_index(self.back).ok()
    }
}

impl ExactSizeIterator for BsDays {}
