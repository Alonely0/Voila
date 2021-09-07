pub mod ast;
pub use ast::Script;
pub use parser::Parse;
mod error;
mod lexer;
mod parser;

pub fn parse_script(source: &str) -> parser::ParseRes<Script> {
    Script::parse_source(source)
}
