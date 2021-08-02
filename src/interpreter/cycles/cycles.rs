use super::async_trait;
use super::Cycle;
use super::Function;

#[async_trait(?Send)]
pub trait Cycles {
    async fn exec_new_cycle(&mut self, operations: Cycle);
    fn execute_operation(&self, function: &Function);
}
