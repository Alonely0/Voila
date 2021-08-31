#![forbid(unsafe_code)] // unsafe code makes ferris get nervous
#![feature(format_args_capture)]
#![feature(decl_macro)]
#![feature(box_syntax)]
#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]

use futures::executor::block_on;
use macros::println_on_debug;

mod cli;
mod interpreter;
mod lexer;
pub mod macros;
mod parser;
mod parser2; // temporary.

pub fn run(source: &str, dir: std::path::PathBuf, recursive: bool) {
    let tokens: Vec<lexer::Token> = lexer::lex(&source); // lex source
    let ast = parser::parse(tokens); // parse tokens
    block_on(interpreter::run(ast, dir, recursive)); // wait interpreter to finish
}
