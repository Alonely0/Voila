use super::utils::regexp::RegExp;
use super::{Literal, LiteralKind};

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

impl Operators for super::Interpreter {
    // Comparing bytes works better than the eval library
    // used in the eval conditionals, specially with strings
    fn eq(&self, x: &Literal, y: &Literal) -> bool {
        x.content.as_bytes() == y.content.as_bytes()
    }

    fn ne(&self, x: &Literal, y: &Literal) -> bool {
        x.content.as_bytes() != y.content.as_bytes()
    }

    fn gt(&self, x: &Literal, y: &Literal) -> bool {
        x.content.as_bytes() > y.content.as_bytes()
    }

    fn ge(&self, x: &Literal, y: &Literal) -> bool {
        x.content.as_bytes() >= y.content.as_bytes()
    }

    fn lt(&self, x: &Literal, y: &Literal) -> bool {
        x.content.as_bytes() < y.content.as_bytes()
    }

    fn le(&self, x: &Literal, y: &Literal) -> bool {
        x.content.as_bytes() <= y.content.as_bytes()
    }

    fn re(&self, x: &Literal, y: &Literal) -> bool {
        // as the regexp can go in any of the sides, we must know
        // in which one it is before evaling them
        match x.kind {
            LiteralKind::Rgx => self.matches(&x.content, &y.content),
            _ => self.matches(&y.content, &x.content),
        }
    }

    fn rn(&self, x: &Literal, y: &Literal) -> bool {
        // why doing a new function with the same except
        // for the last line when i can call it and do the
        // same i would have done to the return?
        !self.re(x, y)
    }
}
