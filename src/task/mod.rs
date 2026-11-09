//! Task domain. Depends on `calendar`
//!
//! Storage rule: dates are stored as AD (`chrono`) and converted to BS only for display and input.
//! Recurrence is the exception:
//! A BS recurrence rule (e.g., every 15th of the BS month) is itself the information,
//! not a copy of an AD date.

mod item;
mod recurrence;
mod when;

pub use item::Item;
pub use recurrence::{Calendar, Frequency, Recurrence};
pub use when::When;
