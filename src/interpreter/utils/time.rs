use core::time::Duration;

pub enum DateTime {
    Date,
    Time,
}

pub trait Timestamps {
    fn convert_timestamp(&self, timestamp: Duration, to: DateTime) -> String;
}
