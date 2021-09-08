use super::Lookup;
use enum_dispatch::enum_dispatch;
use std::error::Error;
use std::fmt;
use std::io::Error as IOError;

#[derive(Debug)]
pub enum CastError {
    IncompatibleCast {
        from: &'static str,
        to: &'static str,
    },
    NumParseError(<f64 as std::str::FromStr>::Err),
}

impl Error for CastError {}
impl fmt::Display for CastError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IncompatibleCast { from, to } => write!(f, "can't cast {} to {}", from, to),
            Self::NumParseError(err) => write!(f, "errar at parsing number: {}", err),
        }
    }
}

#[enum_dispatch]
trait Error1: Error {}

#[enum_dispatch(Error1)]
#[derive(Debug)]
pub enum ErrorKind {
    IOError,
    LookupError,
    CastError,
    ArgCountMismatched,
}

impl Error for ErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(match self {
            Self::IOError(err) => err,
            Self::LookupError(err) => err,
            Self::CastError(err) => err,
            Self::ArgCountMismatched(err) => err,
        })
    }
}

// TODO: make this thing not look like shit
impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.source().unwrap())
    }
}

#[derive(Debug)]
pub struct LookupError {
    lookup: Lookup,
}

impl LookupError {
    pub const fn new(lookup: Lookup) -> Self {
        Self { lookup }
    }
}

impl Error for LookupError {}
impl fmt::Display for LookupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lookup {} has no initializer", self.lookup)
    }
}

use crate::ast::Function;

#[derive(Debug)]
pub struct ArgCountMismatched {
    function: Function,
    got: usize,
}

impl ArgCountMismatched {
    pub const fn new(function: Function, got: usize) -> Self {
        Self { function, got }
    }

    pub const fn check(function: Function, arg_count: usize) -> Result<(), Self> {
        if arg_count < function.minimum_arg_count() as usize {
            Err(Self::new(function, arg_count))
        } else {
            Ok(())
        }
    }
}

impl Error for ArgCountMismatched {}

impl fmt::Display for ArgCountMismatched {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{function} expected at least {expected_count} arguments, but received {actual_count} instead", function = self.function, expected_count = self.function.minimum_arg_count(), actual_count = self.got)
    }
}
