use async_trait::async_trait;
use rayon::prelude::*;

use super::functions::Functions;
use super::println_on_debug;
use super::{Cycle, Func, Function};

#[async_trait(?Send)]
pub trait Cycles {
    async fn exec_new_cycle(&mut self, operations: Cycle);
    fn execute_operation(&self, function: &Function);
}

#[async_trait(?Send)]
impl Cycles for super::Interpreter {
    async fn exec_new_cycle(&mut self, cycle: Cycle) {
        // spawn a new parallel unit of every operation,
        // that will execute its function
        cycle
            .operations
            .par_iter()
            .map(move |operation| {
                self.execute_operation(operation);
            })
            .count(); // we need to consume the map to execute the cycle, and count is the method that causes less bottleneck
    }

    fn execute_operation(&self, operation: &Function) {
        println_on_debug!("Executing [ {:#?} ]", &operation);
        let args: Vec<String> = self.supervec_literals_to_args(operation.args.to_owned());
        println_on_debug!("Args {:#?}", &args);
        match operation.function {
            Func::DELETE => self.r#delete(&args),
            Func::CREATE => self.r#create(&args),
            Func::MKDIR => self.r#mkdir(&args),
            Func::PRINT => self.r#print(&args),
            Func::MOVE => self.r#move(&args),
            Func::COPY => self.r#copy(&args),
            Func::SHELL => self.r#shell(&args),
            Func::NULL => { /* parser will already have taken care of unknown functions */ }
        }
    }
}
