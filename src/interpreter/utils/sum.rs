#![allow(dead_code)]

use std::fs::File;
use std::io::prelude::*;
use std::{panic, process};

fn get_sum_of(file: &String, sum: SumTypes) -> Result<String, String> {
    let bytes = read_bytes_of_file(file);
    match sum {
        SumTypes::Md5 => Ok(format!("{:x}", md5::compute(bytes))),
        SumTypes::Sha256 => Ok(sha256::digest_bytes(bytes)),
    }
}

fn read_bytes_of_file<'a>(path: &String) -> &'a [u8] {
    let buffer = "";
    let file = panic::catch_unwind(|| File::open(path).unwrap());
    match file {
        Ok(mut f) => f
            .read_to_string(&mut String::from(buffer))
            .unwrap_or_else(|e| {
                eprintln!("could not read one or more files:\n{:#?}", e);
                process::exit(1)
            }),
        Err(e) => {
            eprintln!("could not open one or more files:\n{:#?}", e);
            process::exit(1)
        }
    };
    buffer.as_bytes()
}

#[derive(PartialEq, Eq, Debug)]
pub enum SumTypes {
    Sha256,
    Md5,
}

pub trait Sum {
    fn get_sum_of(&self, file: &String, sum: SumTypes) -> Result<String, String>;
    fn read_bytes_of_file<'a>(&self, path: &String) -> &'a [u8];
}
