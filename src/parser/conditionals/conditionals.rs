pub trait Conditionals {
    fn parse_next_conditional(&mut self) -> super::Conditional;
    fn reset_conditionals(&mut self);
}