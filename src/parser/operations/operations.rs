pub trait Operations {
    fn parse_operations(&mut self) -> Vec<super::Cycle>;
}