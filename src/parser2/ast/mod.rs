use super::lexer::Token;
use super::parser;

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
}
