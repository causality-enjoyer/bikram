//! Bikram Sambat calendar, tasks, and a small CLI.
//!
//! Layering (dependencies point downward only):
//!
//! ```text
//! cli  ->  task  ->  calendar
//! ```
//!
//! `calendar` knows nothing about tasks; `task` knows nothing about the CLI.
//! When a GUI arrives it becomes a second frontend next to `cli`.
//!
//! ```rust
//! use bikram::BsDate;
//! use chrono::NaiveDate;
//!
//! let bs = BsDate::new(2081, 1, 1).unwrap();
//! assert_eq!(bs.to_ad(), NaiveDate::from_ymd_opt(2024, 4, 13).unwrap());
//!

pub mod calendar;
pub mod cli;
pub mod task;

pub use calendar::{
    BsDate, BsMonth, BsRange, Confidence, DateError, FIRST_YEAR, LAST_YEAR, ParseError,
};

pub use task::{Calendar, Frequency, Recurrence};
