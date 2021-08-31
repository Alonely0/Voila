/// Block interpreter for a specific file
pub struct Interpreter {
    file: std::path::PathBuf,
    // here is where we can insert a cache
}

impl Interpreter {
    pub const fn new(file: std::path::PathBuf) -> Self {
        Self { file }
    }
    pub fn get_file(&self) -> &std::path::PathBuf {
        &self.file
    }
}

pub trait Resolve<T> {
    fn resolve(self, state: &Interpreter) -> T;
}

// use crate::parser2::ast::Variable;

// impl Resolve<String> for Expr {

// }
