pub trait Conditionals {
    fn eval_conditionals(&mut self) -> bool;
    fn eval_relationships(&self, cond: String) -> bool;
}
