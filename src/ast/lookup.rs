use super::parser::{Parse, ParseErrorKind, ParseRes, Parser};
use crate::interpreter::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lookup {
    /// The file name (basename)
    Name,
    /// The complete file path
    Path,
    /// Absolute path to the file's parent directory
    Parent,
    /// File owner ID (unix-only)
    #[cfg(unix)]
    OwnerID,
    /// Whether the file occupies than 1 byte
    Empty,
    /// Whether the file is read only (for the user that runs this process)
    Readonly,
    /// Whether the file follows the Executable & Linkable Format
    Elf,
    /// Whether the file is a valid text file
    Text,
    /// The file size
    Size(SizeLabel),
    /// A computed sum of the file's contents
    Sum(SumKind),
    /// time of file creation
    Creation(TimeStamp),
    /// time of the last modification
    LastModification(TimeStamp),
    /// time of the last access to the file
    LastAccess(TimeStamp),
}

impl Lookup {
    const VAR_OPTIONS: &'static [&'static str] = &[
        "name",
        "path",
        "parent",
        #[cfg(unix)]
        "ownerID",
        "empty",
        "readonly",
        "elf",
        "txt",
        "size",
        "sum",
        "creation",
        "lastChange",
        "lastAccess",
    ];
}

impl std::fmt::Display for Lookup {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Name => write!(f, "name"),
            Self::Path => write!(f, "path"),
            Self::Parent => write!(f, "parent"),
            #[cfg(unix)]
            Self::OwnerID => write!(f, "ownerID"),
            Self::Empty => write!(f, "empty"),
            Self::Readonly => write!(f, "readonly"),
            Self::Elf => write!(f, "elf"),
            Self::Text => write!(f, "txt"),
            Self::Size(label) => write!(f, "size={}", label),
            Self::Sum(sum) => write!(f, "sum={}", sum),
            Self::Creation(when) => write!(f, "creation={}", when),
            Self::LastModification(when) => write!(f, "lastChange={}", when),
            Self::LastAccess(when) => write!(f, "lastAccess={}", when),
        }
    }
}

impl Parse<'_> for Lookup {
    // this parser assumes that the parser is already at `Token::Variable`
    fn parse(parser: &mut Parser) -> ParseRes<Self> {
        let full_var = parser.current_token_source().strip_prefix('@').unwrap();
        let (var_name, var_spec): (&str, Option<&str>) = full_var
            .find('=')
            .map(|idx| {
                let (a, b) = full_var.split_at(idx);
                (a, Some(&b[1..]))
            })
            .unwrap_or((full_var, None));
        macro_rules! no_spec {
            ($name:literal, $value:expr) => {{
                if var_spec.is_some() {
                    Err(ParseErrorKind::VarHasNoSpec($name))
                } else {
                    Ok($value)
                }
            }};
        }
        macro_rules! spec {
            ($spec:literal, $type:tt, $ctor:expr) => {
                var_spec
                    .ok_or(ParseErrorKind::VarNeedsSpec {
                        var_name: $spec,
                        options: &$type::OPTS,
                    })
                    .and_then(|var| {
                        $type::detect(var).ok_or(ParseErrorKind::InvalidSpecifier {
                            options: &$type::OPTS,
                        })
                    })
                    .map($ctor)
            };
        }
        match var_name {
            "name" => no_spec!("name", Self::Name),
            "path" => no_spec!("path", Self::Path),
            "parent" => no_spec!("parent", Self::Parent),
            #[cfg(unix)]
            "ownerID" => no_spec!("ownerID", Self::OwnerID),
            "empty" => no_spec!("empty", Self::Empty),
            "readonly" => no_spec!("readonly", Self::Readonly),
            "elf" => no_spec!("elf", Self::Elf),
            "txt" => no_spec!("txt", Self::Text),
            "size" => spec!("size", SizeLabel, Self::Size),
            "sum" => spec!("sum", SumKind, Self::Sum),
            "creation" => spec!("creation", TimeStamp, Self::Creation),
            "lastChange" => spec!("lastChange", TimeStamp, Self::LastModification),
            "lastAccess" => spec!("lastAccess", TimeStamp, Self::LastAccess),
            _ => Err(ParseErrorKind::UnknownVariable),
        }
        .map_err(|e| parser.error(e))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SizeLabel {
    TeraBytes,
    GigaBytes,
    MegaBytes,
    KiloBytes,
    Bytes,
}

impl SizeLabel {
    const OPTS: [&'static str; 5] = ["tb", "gb", "mb", "kb", "bs"];
    fn detect(source: &str) -> Option<Self> {
        Some(match source {
            "tb" => Self::TeraBytes,
            "gb" => Self::GigaBytes,
            "mb" => Self::MegaBytes,
            "kb" => Self::KiloBytes,
            "bs" => Self::Bytes,
            _ => return None,
        })
    }
}

impl std::fmt::Display for SizeLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::TeraBytes => "tb",
            Self::GigaBytes => "gb",
            Self::MegaBytes => "mb",
            Self::KiloBytes => "kb",
            Self::Bytes => "bs",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SumKind {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
}
impl std::fmt::Display for SumKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Sha1 => "sha1",
            Self::Md5 => "md5",
            Self::Sha224 => "sha224",
            Self::Sha256 => "sha256",
            Self::Sha384 => "sha384",
            Self::Sha512 => "sha512",
        })
    }
}

impl SumKind {
    const OPTS: [&'static str; 6] = ["md5", "sha224", "sha256", "sha384", "sha512", "sha1"];
    fn detect(source: &str) -> Option<Self> {
        Some(match source {
            "md5" => Self::Md5,
            "sha1" => Self::Sha1,
            "sha224" => Self::Sha224,
            "sha256" => Self::Sha256,
            "sha384" => Self::Sha384,
            "sha512" => Self::Sha512,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeStamp {
    /// Presented to the user as yyyy-mm-dd
    Date,
    /// Presented to the user as hh:mm:ss
    Hour,
}

impl std::fmt::Display for TimeStamp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Date => "date",
            Self::Hour => "hour",
        })
    }
}

impl TimeStamp {
    const OPTS: [&'static str; 2] = ["date", "hour"];
    fn detect(source: &str) -> Option<Self> {
        Some(match source {
            "date" => Self::Date,
            "hour" => Self::Hour,
            _ => return None,
        })
    }
}

use crate::interpreter::{Cache, CachedResolve, ErrorKind, ExprResult, Resolve};
use std::time::SystemTime;
impl CachedResolve for Lookup {
    fn cached_resolve(&self, cache: &mut Cache) -> Result<ExprResult, ErrorKind> {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        match self {
            Self::Name => Ok(cache
                .get_path()
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap()
                .into()),
            Self::Path => cache
                .get_path()
                .canonicalize()
                .map_err(ErrorKind::from)
                .map(|path| path.to_str().unwrap().into()),
            // TODO: add error for not having parent
            Self::Parent => cache
                .get_path()
                .parent()
                .unwrap()
                .canonicalize()
                .map_err(ErrorKind::from)
                .map(|path| path.to_str().unwrap().into()),

            #[cfg(unix)]
            Self::OwnerID => cache
                .get_file_metadata()
                .map(|m| m.uid() as f64)
                .map(ExprResult::from),
            Self::Empty => cache
                .get_file_metadata()
                .map(|m| m.len() <= 1)
                .map(ExprResult::from),
            Self::Readonly => cache
                .get_file_metadata()
                .map(|m| m.permissions().readonly())
                .map(ExprResult::from),
            Self::Elf => {
                use std::io::Read;
                use std::io::Seek;
                let mut rb = [0u8; 4];
                let br = cache.get_file_mut()?;
                // impl a partial reader that can seek wherever
                br.rewind().map_err(ErrorKind::from)?;
                let size = br.read(&mut rb).map_err(ErrorKind::from)?;
                Ok(size >= 4 && rb == [0x7f, b'E', b'L', b'F']).map(ExprResult::from)
            },
            Self::Text => {
                let file = cache.get_file_mut()?.fill()?;
                Ok(file.last().filter(|x| *x == &b'\n').is_some()).map(ExprResult::from)
            },
            Self::Creation(ts) => {
                let created_time = cache.get_file_metadata()?.created()?;
                Ok(get_timestamp(created_time, ts))
            },
            Self::LastModification(ts) => {
                let mod_time = cache.get_file_metadata()?.modified()?;
                Ok(get_timestamp(mod_time, ts))
            },
            Self::LastAccess(ts) => {
                let last_access_time = cache.get_file_metadata()?.created()?;
                Ok(get_timestamp(last_access_time, ts))
            },
            // note: think about using Decimal (for the 2 decimal imposed precision):
            // https://crates.io/crates/rust-decimal
            Self::Size(sz) => Ok(cache.get_file_metadata()?.len() as f64
                / match sz {
                    SizeLabel::Bytes => 1.0,
                    SizeLabel::KiloBytes => 1_000.0,
                    SizeLabel::MegaBytes => 1_000_000.0,
                    SizeLabel::GigaBytes => 1_000_000_000.0,
                    SizeLabel::TeraBytes => 1_000_000_000_000.0,
                })
            .map(ExprResult::from),
            Self::Sum(sum) => {
                let hasher = Hasher::select_from_sum(*sum);
                hasher
                    .hash_reader(cache.get_file_mut()?)
                    .map(ExprResult::from)
                    .map_err(ErrorKind::from)
            },
        }
    }
}
impl Resolve for Lookup {
    fn resolve(&self, cache: &mut Cache) -> Result<ExprResult, ErrorKind> {
        cache.resolve_var(*self)
    }
}

use std::time::Duration;
fn get_naive_datetime(duration: Duration) -> chrono::NaiveDateTime {
    let duration = chrono::Duration::from_std(duration).unwrap();
    let unix = chrono::NaiveDateTime::from_timestamp(0, 0);
    unix + duration
}

fn get_timestamp(time: SystemTime, timestamp: &TimeStamp) -> ExprResult {
    let datetime = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(get_naive_datetime)
        .unwrap();
    let str = match timestamp {
        TimeStamp::Date => datetime.date().to_string(),
        TimeStamp::Hour => datetime.time().format("%H:%M:%S").to_string(),
    };
    str.into()
}
