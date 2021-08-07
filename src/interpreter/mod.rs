use std::path::PathBuf;

use crate::{parser::ast::*, println_on_debug};

mod conditionals;
mod cycles;
mod exceptions;
mod functions;
mod interpreter;
mod operators;
mod utils;
mod variables;

use conditionals::Conditionals;
use cycles::Cycles;
use futures_util::{pin_mut, stream::StreamExt};
use interpreter::Interpreter;
use utils::{path, Str};

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
            __file__: String::from(""),
        }
    }

    async fn exec(&mut self) {
        for i in 0..self.__ast__.cycles.len() {
            if i == 0 && self.__ast__.cycles.len() <= 1 {}
            // load files
            let file_generator = path::file_generator(self.to_owned());
            pin_mut!(file_generator);

            while let Some(path) = file_generator.next().await {
                // set current file
                self.__file__ = path;
                println_on_debug!("  File [ {} ]", &self.__file__);

                // get cycle
                let cycle = self.__ast__.cycles[i].to_owned();

                // get if file matches the conditionals
                if self.eval_conditionals() {
                    // execute cycle
                    self.exec_new_cycle(cycle).await;
                }
            }
        }
    }
}
