use super::ast::Script;
use walkdir::WalkDir;

// TODO: follow
// <https://rust-lang-nursery.github.io/rust-cookbook/concurrency/threads.html#calculate-sha256-sum-of-iso-files-concurrently>
// for each file

pub async fn run(script: Script<'_>, directory: std::path::PathBuf, recursive: bool) {
    dbg!((script, &directory, recursive));
    for entry in get_walker(directory, recursive)
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
    {
        println!("{}", entry.path().display());
    }
}

fn get_walker(directory: std::path::PathBuf, recursive: bool) -> walkdir::IntoIter {
    let starting_point = WalkDir::new(directory);
    if !recursive {
        starting_point.max_depth(1)
    } else {
        starting_point
    }
    .into_iter()
}


// fn run_script(script: &Script<'_>, directory: std::path::PathBuf) -> Result<(), RunError> {
// }

// enum RunError {
//     EvalError(EvalError),
// }

// what to do:
// 1 - evaluate the expressions (single-threaded -> &mut variables and on-time lookup)
// 2 - run the cycles (on-time lookups?)
