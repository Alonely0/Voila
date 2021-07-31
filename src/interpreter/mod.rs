extern crate futures_util;

use super::parser::ast::*;
use super::println_on_debug;
use conditionals::Conditionals;
use cycles::Cycles;
use futures_util::pin_mut;
use futures_util::stream::StreamExt;
use interpreter::Interpreter;
use std::path::PathBuf;
use utils::path;
use utils::Str;

mod conditionals;
mod cycles;
mod exceptions;
mod functions;
mod interpreter;
mod operators;
mod utils;
mod variables;

type AST = super::parser::ast::AST;

pub async fn run(ast: AST, directory: PathBuf, recursive: bool) {
    println_on_debug!("Interpreter started");

    // Initialize interpreter
    let mut __voila__: Interpreter = Interpreter::new(directory, recursive, ast);

    // start interpreter
    __voila__.exec().await;
    println_on_debug!("Interpreter ended");
}

impl Interpreter {
    fn new(dir: PathBuf, recursive: bool, ast: AST) -> Self {
        Self {
            // voila interpreter information
            __directory__: dir,
            __recursive__: recursive,
            __ast__: ast,

            // variables needed for runtime
            __files__: vec![],
            __file__: String::from(""),
        }
    }

    async fn exec(&mut self) {
        for i in 0..self.__ast__.cycles.len() {
            if i == 0 && self.__ast__.cycles.len() <= 1 {}
            // load files
            let file_generator = path::file_generator(self.clone());
            pin_mut!(file_generator);

            while let Some(path) = file_generator.next().await {
                // set current file
                self.__file__ = path;
                println_on_debug!("  File [ {} ]", &self.__file__);

                // get cycle
                let cycle = self.__ast__.cycles[i].clone();

                // get if file matches the conditionals
                if self.eval_conditionals() {
                    // execute cycle
                    self.exec_new_cycle(cycle).await;
                }
            }
        }
    }
}
