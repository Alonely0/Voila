use std::process;

pub trait Exceptions {
    fn raise_parse_error(&self, err_type: &str, msg: String) -> !;
}

impl Exceptions for super::Parser {
    fn raise_parse_error(&self, err_type: &str, msg: String) -> ! {
        eprintln!("PARSE ERROR:\n   {et}: {msg}", et = err_type.to_ascii_uppercase());
        process::exit(1)
    }
}
