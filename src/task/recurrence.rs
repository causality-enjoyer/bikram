use chrono::{Duration, Months, NaiveDate};

use crate::calendar::{BsDate, DateError};

/// Which calendar a monthly/yearly rule counts in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Calendar {
    /// Gregorian. Maps 1:1 on to RFC 5545 `RRULE`
    Ad,
    /// Bikram Sambatt. Not expressible in RRULE; expanded by this crate.
    Bs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

/// A recurrence rule: every `interval` periods of `frequency`.
///
/// `calendar` only matters for monthly/yearly; daily and weekly are the saem in both
/// calendars and are normalized to `Calendar::Ad`.
///
/// Occurrences are computed from the start date each time (`start + n * period`),
/// not by chaining, so day clamping never drifts: "monthly from the 31st" gives
/// 31, 28/29, 31 ... rather than 31, 28, 28 ...
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Recurrence {
    pub frequency: Frequency,
    pub interval: u32,
    pub calendar: Calendar,
}

impl Recurrence {
    pub fn daily() -> Self {
        Self::build(Frequency::Daily, Calendar::Ad)
    }

    pub fn weekly() -> Self {
        Self::build(Frequency::Weekly, Calendar::Ad)
    }

    pub fn monthly(calendar: Calendar) -> Self {
        Self::build(Frequency::Monthly, calendar)
    }

    pub fn yearly(calendar: Calendar) -> Self {
        Self::build(Frequency::Yearly, calendar)
    }

    /// Every `n` periods instead of every 1. `n` is clamped to atleast 1.
    pub fn every(mut self, n: u32) -> Self {
        self.interval = n.max(1);
        self
    }

    fn build(frequency: Frequency, calendar: Calendar) -> Self {
        let calendar = match frequency {
            Frequency::Daily | Frequency::Weekly => Calendar::Ad,
            _ => calendar,
        };
        Recurrence {
            frequency,
            interval: 1,
            calendar,
        }
    }

    /// The RFC 5545 `RRULE` value, or 'None' for BS-based rules
    pub fn to_rrule(&self) -> Option<String> {
        if self.calendar == Calendar::Bs {
            return None;
        }
        let freq = match self.frequency {
            Frequency::Daily => "DAILY",
            Frequency::Weekly => "WEEKLY",
            Frequency::Monthly => "MONTHLY",
            Frequency::Yearly => "YEARLY",
        };

        Some(if self.interval == 1 {
            format!("FREQ={freq}")
        } else {
            format!("FREQ={freq};INTERVAL={}", self.interval)
        })
    }

    /// The `n`th occurrence (0 is `start` itself)
    pub fn nth(&self, start: NaiveDate, n: u32) -> Result<NaiveDate, DateError> {
        let steps = self.interval as u64 * n as u64;
        let overflow = |_| DateError::OutOfRange;
        match (self.frequency, self.calendar) {
            (Frequency::Daily, _) => add_days(start, i64::try_from(steps).map_err(overflow)?),
            (Frequency::Weekly, _) => add_days(start, i64::try_from(steps * 7).map_err(overflow)?),
            (Frequency::Monthly, Calendar::Ad) => {
                add_months(start, u32::try_from(steps).map_err(overflow)?)
            }
            (Frequency::Yearly, Calendar::Ad) => {
                add_months(start, u32::try_from(steps * 12).map_err(overflow)?)
            }
            (Frequency::Monthly, Calendar::Bs) => BsDate::from_ad(start)?
                .add_months(i32::try_from(steps).map_err(overflow)?)
                .map(|d| d.to_ad()),
            (Frequency::Yearly, Calendar::Bs) => BsDate::from_ad(start)?
                .add_years(i32::try_from(steps).map_err(overflow)?)
                .map(|d| d.to_ad()),
        }
    }

    /// Occurrences from `start` (inclusive) until the supported range ends.
    pub fn iter(self, start: NaiveDate) -> impl Iterator<Item = NaiveDate> {
        (0u32..).map_while(move |n| self.nth(start, n).ok())
    }

    /// First occurence strictly after `after`
    pub fn next_after(&self, start: NaiveDate, after: NaiveDate) -> Option<NaiveDate> {
        self.iter(start).find(|d| *d > after)
    }
}

fn add_days(d: NaiveDate, n: i64) -> Result<NaiveDate, DateError> {
    Duration::try_days(n)
        .and_then(|dur| d.checked_add_signed(dur))
        .ok_or(DateError::OutOfRange)
}

fn add_months(d: NaiveDate, n: u32) -> Result<NaiveDate, DateError> {
    d.checked_add_months(Months::new(n))
        .ok_or(DateError::OutOfRange)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bs(y: i32, m: u8, d: u8) -> BsDate {
        BsDate::new(y, m, d).unwrap()
    }

    fn ad(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn daily_weekly() {
        let s = ad(2026, 11, 9);
        assert_eq!(Recurrence::daily().nth(s, 3).unwrap(), ad(2026, 11, 12));
        assert_eq!(Recurrence::weekly().nth(s, 2).unwrap(), ad(2026, 11, 23));
        assert_eq!(
            Recurrence::weekly().every(2).nth(s, 1).unwrap(),
            ad(2026, 11, 23)
        );
    }

    #[test]
    fn ad_monthly_does_not_drift() {
        let r = Recurrence::monthly(Calendar::Ad);
        let s = ad(2026, 1, 31);
        let v: Vec<_> = r.iter(s).take(3).collect();
        assert_eq!(v, [ad(2026, 1, 31), ad(2026, 2, 28), ad(2026, 3, 31)]);
    }

    #[test]
    fn bs_monthly_and_yearly() {
        let m = Recurrence::monthly(Calendar::Bs);
        let s = bs(2081, 1, 31).to_ad();
        assert_eq!(m.nth(s, 1).unwrap(), bs(2081, 2, 31).to_ad());
        let s = bs(2081, 2, 32).to_ad();
        assert_eq!(m.nth(s, 1).unwrap(), bs(2081, 3, 31).to_ad()); // clamped
        assert_eq!(m.nth(s, 2).unwrap(), bs(2081, 4, 32).to_ad()); // not drifted

        let y = Recurrence::yearly(Calendar::Bs);
        let s = bs(2081, 1, 1).to_ad();
        assert_eq!(y.nth(s, 3).unwrap(), bs(2084, 1, 1).to_ad());
    }

    #[test]
    fn bs_rule_ends_at_table_edge() {
        let y = Recurrence::yearly(Calendar::Bs);
        let s = bs(2088, 1, 1).to_ad();
        assert_eq!(y.iter(s).count(), 3); // 2088, 2089, 2090
    }

    #[test]
    fn next_after() {
        let r = Recurrence::weekly();
        let s = ad(2026, 11, 9);
        assert_eq!(r.next_after(s, ad(2026, 11, 9)), Some(ad(2026, 11, 16)));
        assert_eq!(r.next_after(s, ad(2026, 11, 8)), Some(ad(2026, 11, 9)));
    }

    #[test]
    fn rrule_export() {
        assert_eq!(Recurrence::daily().to_rrule().unwrap(), "FREQ=DAILY");
        assert_eq!(
            Recurrence::monthly(Calendar::Ad)
                .every(3)
                .to_rrule()
                .unwrap(),
            "FREQ=MONTHLY;INTERVAL=3"
        );
        assert_eq!(Recurrence::monthly(Calendar::Bs).to_rrule(), None);
        // daily/weekly are calendar-independent and always exportable
        assert!(
            Recurrence::build(Frequency::Weekly, Calendar::Bs)
                .to_rrule()
                .is_some()
        );
    }
}
