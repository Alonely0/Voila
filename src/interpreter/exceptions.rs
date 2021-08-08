use std::process;

pub trait Exceptions {
    fn raise_error(&self, err_type: &str, msg: String) -> !;
}

impl Exceptions for super::Interpreter {
    fn raise_error(&self, err_type: &str, msg: String) -> ! {
        eprintln!(
            "RUNTIME ERROR:\n   {}: {msg}",
            err_type.to_ascii_uppercase()
        );
        process::exit(1)
    }
}
