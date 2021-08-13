use super::Exceptions;
use async_stream::stream;
use futures_core::stream::Stream;
use if_chain::if_chain;
use path_absolutize::*;
use std::fs;
use walkdir::WalkDir;

pub trait Path {
    fn exist(&self, input: &String) -> bool;
    fn absolutize(&self, input: &String) -> String;
    fn is_file(&self, input: &String) -> Result<bool, ()>;
}

// the compiler gets angry if I try to return that inside an impl of a trait
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
                if_chain! {
                    if let Ok(entry) = e;
                    if let Ok(metadata) = entry.metadata();
                    if metadata.is_file();
                    then {
                        yield entry.path().display().to_string();
                    } else { continue }
                }
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
