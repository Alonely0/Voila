extern crate md5;
extern crate sha256;

use std::fs::File;
use std::io::prelude::*;
use std::panic;
use std::process;

fn get_sum_of(file: &String, sum: SumTypes) -> Result<String, String> {
    let bytes = read_bytes_of_file(file);
    println!("{:?}", sum);
    if sum == SumTypes::Sha256 {
        println!("sha256");
        return Ok(sha256::digest_bytes(bytes));
    } else {
        println!("md5");
        return Ok(format!("{:x}", md5::compute(bytes)));
    }
}

fn read_bytes_of_file<'a>(path: &String) -> &'a [u8] {
    let buffer = "";
    let file = panic::catch_unwind(|| {
        return File::open(path).unwrap();
    });
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
    return buffer.as_bytes();
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