use super::parser;
use parser::Token;
use std::ops::Range;

// This trait will be useful later
pub trait HasSpan {
    fn span(&self) -> &Range<usize>;
}

#[macro_use]
mod macros {
    macro_rules! mod_use {
        { $(use $mod:ident;)+ } => { $(
                mod $mod;
                pub use $mod::*;
            )+ };
    }
}

mod_use! {
    use script;
    use target;
    use expr;
    use cycle;
    use call;
    use lookup;
    use string;
}

pub fn parse_script(source: &str) -> parser::ParseRes<Script> {
    use parser::Parse;
    Script::parse_source(source)
}
