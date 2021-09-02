use super::parser::Script;
pub async fn run(script: Script<'_>, directory: std::path::PathBuf, recursive: bool) {
    dbg!((script, directory, recursive));
}
