use std::fmt;

use super::{FIRST_YEAR, LAST_YEAR};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateError {
    /// The date doesn't exist (monthh 13, 33 Baisakh)
    NoSuchDate,
    /// The data is valid, but outside the supported table
    OutOfRange,
}

impl fmt::Display for DateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DateError::NoSuchDate => write!(f, "no such date"),
            DateError::OutOfRange => write!(
                f,
                "date is outside the supported range (BS {FIRST_YEAR} - {LAST_YEAR})"
            ),
        }
    }
}

impl std::error::Error for DateError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    /// Couldn't make sense of the shape of the input.
    BadFormat(String),
    /// A month token that is neither a number 1-12 nor a known name/alias.
    UnknownMonth(String),
    /// Parsed fine, but the resulting date is invalid or out of range.
    Date(DateError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Empty => write!(f, "empty date"),
            ParseError::BadFormat(s) => write!(f, "can't parse date: {s}"),
            ParseError::UnknownMonth(s) => write!(f, "unknown month: {s:?}"),
            ParseError::Date(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<DateError> for ParseError {
    fn from(e: DateError) -> Self {
        ParseError::Date(e)
    }
}
