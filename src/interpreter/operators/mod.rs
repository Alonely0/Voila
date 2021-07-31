

pub mod operators;

use super::utils::regexp::RegExp;
use super::{Literal, LiteralKind};
pub use operators::Operators;

impl Operators for super::Interpreter {
    // Comparing bytes works better than the eval library
    // used in the eval conditionals, specially with strings
    fn eq(&self, x: &Literal, y: &Literal) -> bool {
        return x.content.as_bytes() == y.content.as_bytes();
    }

    fn ne(&self, x: &Literal, y: &Literal) -> bool {
        return x.content.as_bytes() != y.content.as_bytes();
    }

    fn gt(&self, x: &Literal, y: &Literal) -> bool {
        return x.content.as_bytes() > y.content.as_bytes();
    }

    fn ge(&self, x: &Literal, y: &Literal) -> bool {
        return x.content.as_bytes() >= y.content.as_bytes();
    }

    fn lt(&self, x: &Literal, y: &Literal) -> bool {
        return x.content.as_bytes() < y.content.as_bytes();
    }

    fn le(&self, x: &Literal, y: &Literal) -> bool {
        return x.content.as_bytes() <= y.content.as_bytes();
    }

    fn re(&self, x: &Literal, y: &Literal) -> bool {
        // define variables
        let rgx: String;
        let str: String;

        // as the regexp can go in any of the sides, we must know
        // in which one it is
        match x.kind {
            LiteralKind::Rgx => {
                rgx = x.content.clone();
                str = y.content.clone();
            }
            _ => {
                rgx = y.content.clone();
                str = x.content.clone();
            }
        };

        // eval the regexp with the string
        return self.matches(str, rgx);
    }

    fn rn(&self, x: &Literal, y: &Literal) -> bool {
        // why doing a new function with the same except
        // for the last line when i can call it and do the
        // same i would have done to the return?
        return !self.re(x, y);
    }
}
