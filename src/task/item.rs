use super::{Recurrence, When};

/// A task or event.
///
/// A single type with an optional `due` covers both until i implement a real split
/// split going to be based on iCalendar's VTODO and VEVENT
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: u64,
    pub title: String,
    pub due: Option<When>,
    pub recurrence: Option<Recurrence>,
    pub done: bool,
}

impl Item {
    pub fn new(id: u64, title: impl Into<String>) -> Self {
        Item {
            id,
            title: title.into(),
            due: None,
            recurrence: None,
            done: false,
        }
    }

    pub fn due(mut self, when: When) -> Self {
        self.due = Some(when);
        self
    }

    pub fn recurring(mut self, rule: Recurrence) -> Self {
        self.recurrence = Some(rule);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::BsDate;
    use crate::task::Calendar;

    #[test]
    fn builder() {
        let day = BsDate::new(2083, 7, 25).unwrap();
        let item = Item::new(1, "pay rent")
            .due(When::on(day))
            .recurring(Recurrence::monthly(Calendar::Bs));
        assert_eq!(item.due.unwrap().bs_day().unwrap(), day);
        assert!(!item.done);
        assert!(item.recurrence.is_some());
    }
}
