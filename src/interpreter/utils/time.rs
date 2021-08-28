use core::time::Duration;

pub trait Timestamps {
    fn get_date(&self, timestamp: Duration) -> String;
    fn get_hour(&self, timestamp: Duration) -> String;
}