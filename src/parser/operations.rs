use super::{Cycle, Cycles};

pub trait Operations {
    fn parse_operations(&mut self) -> &Vec<Cycle>;
}

impl Operations for super::Parser {
    fn parse_operations(&mut self) -> &Vec<Cycle> {
        self.parse_raw_cycles();
        self.parse_cycles();

        &self.cycles
    }
}
