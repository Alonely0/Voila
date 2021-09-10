use super::ast::Lookup;
use super::ast::Script;
use rayon::ThreadPoolBuilder;
use std::error::Error;
use std::sync::mpsc;
use walkdir::WalkDir;
mod error;
pub use error::*;
mod cache;
pub use cache::*;
mod hash;
pub use hash::*;

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
    Date(chrono::NaiveDate),
    Time(chrono::NaiveTime),
}

impl ExprResult {
    pub fn cast_to_string(self) -> String {
        match self {
            Self::Boolean(b) => b.to_string(),
            Self::String(s) => s,
            Self::Numeric(f) => f.to_string(),
            Self::Date(d) => d.to_string(),
            Self::Time(t) => t.format("%H:%M:%S").to_string(),
        }
    }

    /// Try to reinterpret the value as another thing.
    pub fn reinterpret(self) -> Self {
        match self {
            Self::String(s) => Self::parse_str(&s),
            other => other,
        }
    }

    pub fn cast_to_bool(self) -> Result<bool, CastError> {
        Ok(match self {
            Self::Boolean(b) => b,
            Self::Numeric(n) => n != 0.0,
            Self::String(s) => !s.is_empty(),
            Self::Date(_) => {
                return Err(CastError::IncompatibleCast {
                    from: "date",
                    to: "boolean",
                })
            },
            Self::Time(_) => {
                return Err(CastError::IncompatibleCast {
                    from: "time",
                    to: "boolean",
                })
            },
        })
    }

    pub fn cast_to_number(self) -> Result<f64, CastError> {
        match self {
            Self::Boolean(_) => Err(CastError::IncompatibleCast {
                from: "boolean",
                to: "number",
            }),
            Self::Numeric(i) => Ok(i),
            Self::String(str) => str.parse().map_err(CastError::NumParseError),
            Self::Date(d) => {
                let unix = chrono::NaiveDateTime::from_timestamp(0, 0).date();
                Ok(d.signed_duration_since(unix).num_days() as f64)
            },
            Self::Time(t) => Ok(t
                .signed_duration_since(chrono::NaiveTime::from_hms(0, 0, 0))
                .num_seconds() as f64),
        }
    }

    // NOTE: this currently hinders performance since the regex will be
    // parsed on each file. It is really convenient though, since the language
    // has been simplified, and when the compiler comes into place resolving expressions
    // to their correct type, this parse (as well as the str to number one) will be done only once.
    pub fn cast_to_regex(self) -> Result<regex::Regex, CastError> {
        // only strings will be available to cast into regex.
        // NOTE: this (accidentally) supports having regexes with resolved variables.
        // this WILL be removed in the future, as keeping it with the compiler will
        // remove the ability to pre-parse it.
        match self {
            Self::Boolean(_) => Err(CastError::IncompatibleCast {
                from: "boolean",
                to: "regex",
            }),
            Self::Numeric(_) => Err(CastError::IncompatibleCast {
                from: "number",
                to: "regex",
            }),
            Self::Date(_) => Err(CastError::IncompatibleCast {
                from: "date",
                to: "regex",
            }),
            Self::Time(_) => Err(CastError::IncompatibleCast {
                from: "time",
                to: "regex",
            }),
            Self::String(s) => regex::Regex::new(&s).map_err(CastError::RegexError),
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
    pub fn parse_str(t: &str) -> Self {
        // NOTE: use `lexical::parse` for these routines if you want to be even faster :)
        fn try_time(source: &str) -> Option<chrono::NaiveTime> {
            let first_colon = source.find(':')?;
            let second_colon = source[first_colon + 1..].find(':')? + first_colon + 1;
            let hour = source[..first_colon].parse().ok()?;
            let minute = source[first_colon + 1..second_colon].parse().ok()?;
            let second = source[second_colon + 1..].parse().ok()?;
            chrono::NaiveTime::from_hms_opt(hour, minute, second)
        }
        fn try_date(source: &str) -> Option<chrono::NaiveDate> {
            let first_dash = source.find('-')?;
            let second_dash = source[first_dash + 1..].find('-')? + first_dash + 1;
            let year = source[..first_dash].parse().ok()?;
            let month = source[first_dash + 1..second_dash].parse().ok()?;
            let day = source[second_dash + 1..].parse().ok()?;
            chrono::NaiveDate::from_ymd_opt(year, month, day)
        }
        // try to parse it in different ways
        if let Ok(num) = t.parse() {
            Self::Numeric(num)
        } else if let Ok(bool) = t.parse() {
            Self::Boolean(bool)
        } else if let Some(time) = try_time(t) {
            Self::Time(time)
        } else if let Some(date) = try_date(t) {
            Self::Date(date)
        } else {
            Self::String(t.into())
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

impl From<&str> for ExprResult {
    fn from(str: &str) -> Self {
        Self::String(str.into())
    }
}

impl From<f64> for ExprResult {
    fn from(i: f64) -> Self {
        Self::Numeric(i)
    }
}
