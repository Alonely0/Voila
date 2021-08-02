#[derive(Clone)]
pub struct Interpreter {
    // voila interpreter information
    pub __directory__: super::PathBuf,
    pub __recursive__: bool,
    pub __ast__: super::AST,

    // variables needed for runtime
    pub __files__: Vec<String>,
    pub __file__: String,
}
