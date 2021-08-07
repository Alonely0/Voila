use std::process;

pub trait Exceptions {
    fn raise_parse_error(&self, err_type: &str, msg: String);
}

impl Exceptions for super::parser::Parser {
    fn raise_parse_error(&self, err_type: &str, msg: String) {
        eprintln!("PARSE ERROR:\n   {}: {msg}", err_type.to_ascii_uppercase());
        process::exit(1)
    }
}
