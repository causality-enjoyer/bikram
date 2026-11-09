//! BS calendar: table-driven dates, ranges, month names and parsing.
//!
//! Gregorian arithmetic is delegated to `chrono`. Everything here is about the
//! BS side. A BS date is mapped to a *day index* (days since 1 Baisakh of the
//! first table year); AD is `anchor + day_index`.

mod confidence;
mod data;
mod date;
mod error;
mod month;
mod parse;
mod range;
mod table;

pub use confidence::{Confidence, confidence};
pub use data::FIRST_YEAR;
pub use date::{BsDate, nepal_offset};
pub use error::{DateError, ParseError};
pub use month::BsMonth;
pub use range::{BsDays, BsRange};
pub use table::LAST_YEAR;
