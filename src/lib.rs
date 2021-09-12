#![forbid(unsafe_code)] // unsafe code makes ferris get nervous
#![feature(format_args_capture)]
#![feature(once_cell)]
#![feature(decl_macro)]
#![feature(box_syntax)]
#![feature(option_result_unwrap_unchecked)]
#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]

use std::error::Error;

mod ast;
mod cli;
mod error;
mod interpreter;
pub mod macros;
mod parser;

pub fn run(source: &str, dir: std::path::PathBuf, recursive: bool) -> Result<(), Box<dyn Error>> {
    let ast = ast::parse_script(source)?;
    interpreter::run(ast, dir, recursive)?; // wait interpreter to finish
    Ok(())
}
