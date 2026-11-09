use crate::calendar::{
    data::{ATTESTED_THROUGH, PUBLISHED_THROUGH},
    table::LAST_YEAR,
};

use super::data::FIRST_YEAR;

/// How trustworthy the month lengths for a BS year are
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    /// Confirmed by multiple sources
    Attested,
    /// Officially published, not independently cross-checked
    Published,
    /// Extrapolated from patterns; may change.
    Provisional,
}

pub fn confidence(bs_year: i32) -> Option<Confidence> {
    match bs_year {
        y if !(FIRST_YEAR..=LAST_YEAR).contains(&y) => None,
        y if y <= ATTESTED_THROUGH => Some(Confidence::Attested),
        y if y <= PUBLISHED_THROUGH => Some(Confidence::Published),
        _ => Some(Confidence::Provisional),
    }
}
