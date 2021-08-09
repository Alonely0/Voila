use super::exceptions::Exceptions;
use path_absolutize::*;
use regex::Regex;
use sha1::{Digest, Sha1};
use sha2::*;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;
pub use string::Str;
pub use sum::Sum;
pub use sum::SumTypes;

pub mod path;
pub mod regexp;
pub mod string;
pub mod sum;

impl path::Path for super::Interpreter {
    fn exist(&self, input: &String) -> bool {
        // absolutize path, just to be sure nothing
        // weird happens
        let path = self.absolutize(input);

        // create a new path struct with the
        // absolute path and get if exists
        Path::new(&path).exists()
    }

    fn absolutize(&self, input: &String) -> String {
        // ge the absolute path
        let p = Path::new(&input).absolutize().unwrap();

        // get the &str
        // (i think i cannot get the String directly)
        let path = p.to_str().unwrap();

        // return it converted to String
        path.to_string()
    }

    fn is_file(&self, input: &String) -> Result<bool, ()> {
        // maybe file was delete in previous cycles.
        // if the path just was wrong, its not my fault,
        // user's fault. just re-run voila but reading what
        // you scripted before launching a tool that can be
        // (and in fact is) potentially destructive, im not
        // doing a hashmap of stuff deleted and then a checker,
        // enough overhead & bottlenecks with the async hell
        // of the cycles & the interpreter
        match fs::metadata(input) {
            Ok(md) => Ok(md.is_file()),
            Err(_) => Err(()),
        }
    }
}

impl regexp::RegExp for super::Interpreter {
    fn matches(&self, input: String, regexp: String) -> bool {
        // create a new regex struct from the regex string
        let regex = Regex::new(&regexp).unwrap();

        // return an eval of the regex with the string
        regex.is_match(&input)
    }
}

impl Str for super::Interpreter {
    fn trim_spaces(&self, str: &String) -> String {
        str.trim().to_string()
    }
}

impl Sum for super::Interpreter {
    fn get_sum_of(&self, file: &String, sum: SumTypes) -> String {
        let bytes = self.read_bytes_of_file(file);
        match sum {
            SumTypes::Md5 => format!("{:x}", md5::compute(bytes)),
            SumTypes::Sha1 => {
                let mut hasher = Sha1::new();

                hasher.update(bytes);
                format!("{:x}", hasher.finalize())
            },
            SumTypes::Sha224 => {
                let mut hasher = Sha224::new();

                hasher.update(bytes);
                format!("{:x}", hasher.finalize())
            },
            SumTypes::Sha256 => {
                let mut hasher = Sha256::new();

                hasher.update(bytes);
                format!("{:x}", hasher.finalize())
            },
            SumTypes::Sha384 => {
                let mut hasher = Sha384::new();

                hasher.update(bytes);
                format!("{:x}", hasher.finalize())
            },
            SumTypes::Sha512 => {
                let mut hasher = Sha512::new();

                hasher.update(bytes);
                format!("{:x}", hasher.finalize())
            },
        }
    }
    fn read_bytes_of_file(&self, path: &String) -> Vec<u8> {
        let mut buffer = Vec::new();
        let file = File::open(path);
        match file {
            Ok(f) => {
                let mut reader = io::BufReader::new(f);
                reader.read_to_end(&mut buffer).unwrap();
            }
            Err(e) => {
                self.raise_error("COULD NOT READ BYTES FROM FILE", format!("Cannot read {}: {:?}", self.__file__, e));
            }
        };
        buffer
    }
}
