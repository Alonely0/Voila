#![forbid(unsafe_code)] // unsafe code makes ferris get nervous
#![feature(format_args_capture)]
#![feature(decl_macro)]
#![feature(box_syntax)]
#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]

use futures::executor::block_on;
use macros::println_on_debug;

#[path = "cli.rs"]
mod cli;
#[path = "interpreter/mod.rs"]
mod interpreter;
#[path = "lexer.rs"]
mod lexer;
#[path = "macros.rs"]
mod macros;
#[path = "parser/mod.rs"]
mod parser;

pub fn run(source: String, dir: std::path::PathBuf, recursive: bool) {
    let tokens: Vec<lexer::Token> = lexer::lex(&source); // lex source
    let ast = parser::parse(tokens); // parse tokens
    block_on(interpreter::run(ast, dir, recursive)); // wait interpreter to finish
}
