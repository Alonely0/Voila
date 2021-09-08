use super::ast::Lookup;
use super::ast::Script;
use rayon::ThreadPoolBuilder;
use std::error::Error;
use std::sync::mpsc;
use walkdir::WalkDir;
mod partial_reader;
pub use partial_reader::PartialReader;
mod error;
pub use error::*;
mod cache;
pub use cache::*;
mod hash;
pub use hash::*;

// TODO: follow
// <https://rust-lang-nursery.github.io/rust-cookbook/concurrency/threads.html#calculate-sha256-sum-of-iso-files-concurrently>
// for each file

pub fn run(
    script: Script<'_>,
    directory: std::path::PathBuf,
    recursive: bool,
) -> Result<(), Box<dyn Error>> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .build()
        .unwrap();
    let (tx, rx) = mpsc::channel();
    let pool = &pool;
    let tx_ = tx.clone();
    {
        let script = &script;
        pool.scope(move |s| {
            for file in get_walker(directory, recursive)
                .filter_map(Result::ok)
                .map(|x| x.path().to_owned())
                .filter(|x| x.is_file())
            {
                let tx = tx_.clone();
                s.spawn(move |_| {
                    super::ast::run_script(script, file, pool, tx);
                })
            }
        });
    }
    // for entry in get_walker(directory, recursive)
    //     .filter_map(Result::ok)
    //     .filter(|x| x.path().is_file())
    // {
    //     let tx = tx.clone();
    //     let script = &script;
    //     let path = entry.path().to_owned();
    //     let pool = &pool;
    //     pool.scope(move |_| {
    //         let ret = crate::ast::run_script(script, path, pool);
    //         tx.send(ret).unwrap();
    //     });
    // }

    drop(tx);
    rx.into_iter().next().map_or(Ok(()), |x| Err(x.into()))
}

// pub fn run_cycle(cycle: &Cycle, cache: Arc<Mutex<Cache>>) -> Result<(), ErrorKind> {
// }

fn get_walker(directory: std::path::PathBuf, recursive: bool) -> walkdir::IntoIter {
    let starting_point = WalkDir::new(directory);
    if !recursive {
        starting_point.max_depth(1)
    } else {
        starting_point
    }
    .into_iter()
}

use std::sync::Arc;
use std::sync::Mutex;

pub trait Runnable {
    fn run(&self, cache: Arc<Mutex<Cache>>) -> Result<(), ErrorKind>;
}

#[derive(Debug, Clone)]
pub enum ExprResult {
    Boolean(bool),
    String(String),
    Numeric(f64),
    Regex(regex::Regex),
}

impl ExprResult {
    pub fn cast_to_string(self) -> Result<String, CastError> {
        Ok(match self {
            Self::Boolean(b) => b.to_string(),
            Self::Numeric(n) => n.to_string(),
            Self::String(str) => str,
            // TODO: make this unreachable with the validator
            Self::Regex(_) => {
                return Err(CastError::IncompatibleCast {
                    from: "regex",
                    to: "string",
                })
            },
        })
    }

    pub fn cast_to_bool(self) -> Result<bool, CastError> {
        Ok(match self {
            Self::Regex(_) => {
                return Err(CastError::IncompatibleCast {
                    from: "regex",
                    to: "boolean",
                })
            },
            Self::Boolean(b) => b,
            Self::Numeric(n) => n != 0.0,
            Self::String(s) => !s.is_empty(),
        })
    }

    pub fn cast_to_number(self) -> Result<f64, CastError> {
        match self {
            Self::Regex(_) => Err(CastError::IncompatibleCast {
                from: "regex",
                to: "number",
            }),
            Self::Boolean(_) => Err(CastError::IncompatibleCast {
                from: "boolean",
                to: "number",
            }),
            Self::Numeric(i) => Ok(i),
            Self::String(str) => str.parse().map_err(CastError::NumParseError),
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        if let Self::String(ref str) = self {
            Some(str.as_ref())
        } else {
            None
        }
    }
    pub fn as_number(&self) -> Option<f64> {
        if let Self::Numeric(i) = self {
            Some(*i)
        } else {
            None
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }
    pub fn as_regex(&self) -> Option<&regex::Regex> {
        if let Self::Regex(r) = self {
            Some(r)
        } else {
            None
        }
    }
}

impl From<bool> for ExprResult {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<String> for ExprResult {
    fn from(str: String) -> Self {
        Self::String(str)
    }
}

impl From<f64> for ExprResult {
    fn from(i: f64) -> Self {
        Self::Numeric(i)
    }
}

impl From<&str> for ExprResult {
    fn from(t: &str) -> Self {
        Self::String(t.into())
    }
}
