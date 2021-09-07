#![forbid(unsafe_code)] // unsafe code makes ferris get nervous
#![feature(format_args_capture)]
#![feature(decl_macro)]
#![feature(box_syntax)]
#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]

use futures::executor::block_on;
use macros::println_on_debug;
use std::error::Error;

mod cli;
mod interpreter;
pub mod macros;
mod parser;

pub fn run(source: &str, dir: std::path::PathBuf, recursive: bool) -> Result<(), Box<dyn Error>> {
    let ast = parser::parse_script(source)?;
    block_on(interpreter::run(ast, dir, recursive)); // wait interpreter to finish
    Ok(())
}
