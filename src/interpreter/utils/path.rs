use std::fs;

use async_stream::stream;
use futures_core::stream::Stream;
use path_absolutize::*;
use walkdir::WalkDir;

pub trait Path {
    fn exist(&self, input: &String) -> bool;
    fn absolutize(&self, input: &String) -> String;
    fn is_file(&self, input: &String) -> Result<bool, ()>;
}

// the compiler gets angry if I try to return that inside an impl of a trait
pub fn file_generator(interpreter: super::super::Interpreter) -> impl Stream<Item = String> {
    let generator = stream! {
        // define files' vec
        // let mut files: Vec<String> = vec![];

        // is it recursive? ok, use the library
        // no? we use our implementation
        if interpreter.__recursive__ {
            for e in WalkDir::new(interpreter.__directory__.clone())
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if e.metadata().unwrap().is_file() {
                    yield e.path().display().to_string();
                    //files.push(e.path().display().to_string());
                }
            }
            return;
        } else {
            for entry in fs::read_dir(interpreter.__directory__.clone()).unwrap() {
                let p = entry.unwrap().path();
                let path = p.absolutize().unwrap();
                let metadata = fs::metadata(&path).unwrap();
                if metadata.is_file() {
                    yield path.to_str().unwrap().to_string();
                    //files.push(path.to_str().unwrap().to_string());
                } else {
                    // next please
                    continue;
                }
            }
            return;
        }
    };
    return generator;
}
