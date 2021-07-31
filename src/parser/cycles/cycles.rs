pub trait Cycles {
    fn push_current_cycle(&mut self, tokens_index_list: &mut Vec<usize>);
    fn parse_raw_cycles(&mut self);
    fn parse_cycles(&mut self);
}