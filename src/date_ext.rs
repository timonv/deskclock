use std::borrow::Borrow;

use chrono::{DateTime, Local};

pub trait DateExt {
    fn is_today(&self) -> bool;
    fn is_after_today(&self) -> bool;
    fn is_on_same_day_as(&self, other: impl Borrow<DateTime<Local>>) -> bool;
    fn is_before(&self, other: impl Borrow<DateTime<Local>>) -> bool;
    fn is_after(&self, other: impl Borrow<DateTime<Local>>) -> bool;
}

impl DateExt for DateTime<Local> {
    fn is_today(&self) -> bool {
        self.date_naive() == Local::now().date_naive()
    }

    fn is_after_today(&self) -> bool {
        self.date_naive() > Local::now().date_naive()
    }

    fn is_on_same_day_as(&self, other: impl Borrow<DateTime<Local>>) -> bool {
        self.date_naive() == other.borrow().date_naive()
    }

    fn is_before(&self, other: impl Borrow<DateTime<Local>>) -> bool {
        self > other.borrow()
    }

    fn is_after(&self, other: impl Borrow<DateTime<Local>>) -> bool {
        self < other.borrow()
    }
}
