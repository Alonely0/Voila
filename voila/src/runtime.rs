pub use std::process::exit;

pub fn interpret(args: crate::cli::Cli) -> Result<(), String> {
    crate::voila::run(args.source, args.dir, args.recursive).map_err(|e| format!("{e}"))
}

pub fn compile(args: crate::cli::Cli) -> Result<(), String> {
    crate::compiler::compile([
        &format!(
            "{:?}",
            bincode::serialize(&voila::get_checked_ast(&args.source).map_err(|e| format!("{e}"))?).unwrap()
        ),
        args.dir.as_os_str().to_str().unwrap(),
        &format!("{r}", r = args.recursive),
    ]).map_err(|e| e.to_string())
}