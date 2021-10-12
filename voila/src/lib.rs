#![forbid(unsafe_code)] // unsafe code makes ferris get nervous
#![feature(format_args_capture)]
#![feature(once_cell)]
#![feature(decl_macro)]
#![feature(option_result_unwrap_unchecked)]
#![feature(never_type)]
#![allow(dead_code)]

use std::error::Error;

mod ast;
mod cli;
mod error;
mod interpreter;
mod lexer;
pub mod macros;
mod parser;
mod safety;

pub fn run(source: String, dir: std::path::PathBuf, recursive: bool) -> Result<(), Box<dyn Error>> {
    exec(get_checked_ast(&source)?, dir, recursive)?;
    Ok(())
}

pub fn get_checked_ast(source: &str) -> Result<ast::Script, Box<dyn Error>> {
    let ast = ast::parse_script(source)?;
    ast.ub_checks(source)?;
    Ok(ast)
}

pub fn exec(
    ast: ast::Script,
    dir: std::path::PathBuf,
    recursive: bool,
) -> Result<(), Box<dyn Error>> {
    interpreter::run(ast, dir, recursive)?;
    Ok(())
}
