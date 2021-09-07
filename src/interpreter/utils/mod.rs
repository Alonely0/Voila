use super::exceptions::Exceptions;
use byte_unit::{Byte, ByteUnit};
pub use bytes::ByteConversion;
use core::time::Duration;
use path_absolutize::*;
use regex::Regex;
use sha1::{Digest, Sha1};
use sha2::*;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
pub use string::Str;
pub use sum::Sum;
pub use sum::SumTypes;
pub use time::Timestamps;

pub mod bytes;
pub mod path;
pub mod regexp;
pub mod string;
pub mod sum;
pub mod time;

impl bytes::ByteConversion for super::Interpreter {
    fn convert(&self, from: u128, to: ByteUnit) -> f64 {
        // get size in format needed, then get str & convert it to chars
        let str = format!("{}", Byte::from_bytes(from).get_adjusted_unit(to));
        let mut str_chars = str.chars();

        // remove size label
        for _ in 0..3 {
            str_chars.next_back();
        }

        // convert to u128 & return
        str_chars.as_str().parse::<f64>().unwrap()
    }
}

impl path::Path for super::Interpreter {
    fn exist(&self, input: &str) -> bool {
        // absolutize path, just to be sure nothing
        // weird happens
        let path = self.absolutize(input);

        // create a new path struct with the
        // absolute path and get if exists
        Path::new(&path).exists()
    }

    fn absolutize(&self, input: &str) -> String {
        // ge the absolute path
        let p = Path::new(&input).absolutize().unwrap();

        // get the &str
        // (i think i cannot get the String directly)
        let path = p.to_str().unwrap();

        // return it converted to String
        path.to_string()
    }

    fn is_file(&self, input: &str) -> Result<bool, ()> {
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
    fn matches(&self, input: &str, regexp: &str) -> bool {
        // create a new regex struct from the regex string
        let regex = Regex::new(regexp).unwrap();

        // return an eval of the regex with the string
        regex.is_match(input)
    }
}

impl Str for super::Interpreter {
    fn trim_spaces(&self, string: &str) -> String {
        string.trim().to_string()
    }
}

impl Sum for super::Interpreter {
    fn get_sum_of(&self, file: &str, sum: SumTypes) -> String {
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
    fn read_bytes_of_file(&self, path: &str) -> Vec<u8> {
        let mut buffer = Vec::new();
        let file = File::open(path);
        if let Ok(f) = file {
            let mut reader = io::BufReader::new(f);
            reader.read_to_end(&mut buffer).unwrap();
        }
        buffer
    }
}

impl time::Timestamps for super::Interpreter {
    fn get_date(&self, timestamp: Duration) -> String {
        let chrono_duration = chrono::Duration::from_std(timestamp).unwrap();
        let unix = chrono::naive::NaiveDateTime::from_timestamp(0, 0);
        let naive = unix + chrono_duration;

        // remove hour
        let str = format!("{:?}", naive);
        let mut str_chars = str.chars();
        for _ in 0..19 {
            str_chars.next_back();
        }

        str_chars.as_str().to_string()
    }
    fn get_hour(&self, timestamp: Duration) -> String {
        let chrono_duration = chrono::Duration::from_std(timestamp).unwrap();
        let unix = chrono::naive::NaiveDateTime::from_timestamp(0, 0);
        let naive = unix + chrono_duration;

        // remove date
        let str = format!("{:?}", naive);
        let mut str_chars = str.chars();
        for _ in 0..12 {
            str_chars.next();
        }
        // remove ms
        for _ in 0..10 {
            str_chars.next_back();
        }

        str_chars.as_str().to_string()
    }
}
