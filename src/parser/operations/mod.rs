use super::Cycle;
use super::Cycles;
pub use operations::Operations;

mod operations;

impl Operations for super::Parser {
    fn parse_operations(&mut self) -> Vec<Cycle> {
        // start parsing
        self.parse_raw_cycles();
        self.parse_cycles();

        // return value
        return self.cycles.clone();
    }
}
