use super::Cycle;
use super::Cycles;
pub use operations::Operations;

mod operations;

impl Operations for super::Parser {
    fn parse_operations(&mut self) -> Vec<Cycle> {
        self.parse_raw_cycles();
        self.parse_cycles();

        self.cycles.clone()
    }
}
