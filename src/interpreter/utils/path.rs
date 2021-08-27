use super::Exceptions;
use async_stream::stream;
use futures_core::stream::Stream;
use path_absolutize::*;
use std::fs;
use walkdir::WalkDir;

pub trait Path {
    fn exist(&self, input: &str) -> bool;
    fn absolutize(&self, input: &str) -> String;
    fn is_file(&self, input: &str) -> Result<bool, ()>;
}

// the compiler gets angry if I try to return that inside an impl of a trait
#[allow(clippy::needless_return)]
pub fn file_generator(interpreter: super::super::Interpreter) -> impl Stream<Item = String> {
    // without this when the directory is invalid
    // Voila panics, I prefer to handle the error
    if !interpreter.__directory__.is_dir() {
        interpreter.raise_error(
            "INVALID DIRECTORY",
            format!(
                "{:?} does not exist or is a file.",
                &interpreter.__directory__.as_os_str()
            ),
        )
    }
    stream! {
        // is it recursive? ok, use the library
        // no? we use our implementation
        if interpreter.__recursive__ {
            for e in WalkDir::new(&interpreter.__directory__)
                .into_iter()
            {
                let entry = match e.and_then(|e| e.metadata().map(|m| (e, m))) {
                    Ok((e, m)) if m.is_file() => e,
                    _ => continue,
                };
                yield entry.path().display().to_string();
            }
            return;
        } else {
            for entry in fs::read_dir(&interpreter.__directory__).unwrap() {
                let p = entry.unwrap().path();
                let path = p.absolutize().unwrap();
                if let Ok(metadata) = fs::metadata(&path) {
                    if metadata.is_file() { yield path.to_str().unwrap().to_string() } else { continue };
                }
            }
            return;
        }
    }
}
