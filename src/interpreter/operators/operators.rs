use super::Literal;

pub trait Operators {
    // Function operators
    fn eq(&self, x: &Literal, y: &Literal) -> bool;
    fn ne(&self, x: &Literal, y: &Literal) -> bool;
    fn gt(&self, x: &Literal, y: &Literal) -> bool;
    fn ge(&self, x: &Literal, y: &Literal) -> bool;
    fn lt(&self, x: &Literal, y: &Literal) -> bool;
    fn le(&self, x: &Literal, y: &Literal) -> bool;
    fn re(&self, x: &Literal, y: &Literal) -> bool;
    fn rn(&self, x: &Literal, y: &Literal) -> bool;
}