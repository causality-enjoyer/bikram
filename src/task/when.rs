use chrono::{DateTime, Duration, NaiveDate, NaiveTime, TimeZone, Utc};

use crate::{
    DateError,
    calendar::{BsDate, nepal_offset},
};

/// Either a whole day or a specific instant. A date-only value is **not** midnight
/// it stays "all day" and never has a timezone to get wrong
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum When {
    /// A calendar day (AD date; the Nepal-local day)
    Day(NaiveDate),
    /// An exact instant, stored in UTC.
    Moment(DateTime<Utc>),
}

impl When {
    pub fn on(day: BsDate) -> When {
        When::Day(day.to_ad())
    }

    /// `time` is Nepal-local wall-clock time
    pub fn at(day: BsDate, time: NaiveTime) -> When {
        let local = day.to_ad().and_time(time);
        let moment = nepal_offset()
            .from_local_datetime(&local)
            .single()
            .expect("a fixed offset has no amigious local times");
        When::Moment(moment.with_timezone(&Utc))
    }

    pub fn is_all_day(&self) -> bool {
        matches!(self, When::Day(_))
    }

    /// The Nepal-local calendar day
    pub fn day(&self) -> NaiveDate {
        match self {
            When::Day(d) => *d,
            When::Moment(t) => t.with_timezone(&nepal_offset()).date_naive(),
        }
    }

    pub fn bs_day(&self) -> Result<BsDate, DateError> {
        BsDate::from_ad(self.day())
    }

    /// All-day items are overdue only once their day has fully passed
    /// (Nepal time); timed items once the instant has passed.
    pub fn is_overdue(&self, now: DateTime<Utc>) -> bool {
        match self {
            When::Day(d) => *d < now.with_timezone(&nepal_offset()).date_naive(),
            When::Moment(t) => *t < now,
        }
    }

    /// First instant of the hday, Nepal time (for range queries).
    pub fn start_of_day(&self) -> DateTime<Utc> {
        let midnight = self.day().and_time(NaiveTime::MIN);
        nepal_offset()
            .from_local_datetime(&midnight)
            .single()
            .expect("fixed offset")
            .with_timezone(&Utc)
    }

    pub fn plus_days(&self, n: i64) -> When {
        match self {
            When::Day(d) => When::Day(*d + Duration::days(n)),
            When::Moment(t) => When::Moment(*t + Duration::days(n)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bs(y: i32, m: u8, d: u8) -> BsDate {
        BsDate::new(y, m, d).unwrap()
    }

    #[test]
    fn timed_value_crosses_utc_midnight() {
        // 00:30 Nepal time on 2083-07-20 is still the previous day in UTC.
        let w = When::at(bs(2083, 7, 20), NaiveTime::from_hms_opt(0, 30, 0).unwrap());
        let When::Moment(t) = w else { panic!() };
        assert_eq!(t.date_naive(), bs(2083, 7, 20).to_ad() - Duration::days(1));
        assert_eq!(w.bs_day().unwrap(), bs(2083, 7, 20));
        assert!(!w.is_all_day());
    }

    #[test]
    fn overdue_rules() {
        let day = When::on(bs(2083, 7, 20));
        let ad = bs(2083, 7, 20).to_ad();
        let noon_that_day = ad.and_hms_opt(6, 0, 0).unwrap().and_utc();
        let next_day = (ad + Duration::days(1))
            .and_hms_opt(6, 0, 0)
            .unwrap()
            .and_utc();
        assert!(!day.is_overdue(noon_that_day));
        assert!(day.is_overdue(next_day));
        let timed = When::at(bs(2083, 7, 20), NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        assert!(!timed.is_overdue(ad.and_hms_opt(0, 0, 0).unwrap().and_utc()));
        assert!(timed.is_overdue(next_day));
    }

    #[test]
    fn start_of_day_is_nepal_midnight() {
        let w = When::on(bs(2083, 7, 20));
        // 00:00 NPT = 18:15 UTC the day before
        assert_eq!(
            w.start_of_day(),
            (bs(2083, 7, 20).to_ad() - Duration::days(1))
                .and_hms_opt(18, 15, 0)
                .unwrap()
                .and_utc()
        );
    }
}
