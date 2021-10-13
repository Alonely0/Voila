use super::parser::{Parse, ParseErrorKind, ParseRes, Parser};
use crate::interpreter::{Hash, Hasher};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// File content
    Content,
    /// Access to a specific line of the file
    Line(usize),
    /// Whether the file is hidden
    Hidden,
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

use Lookup::*;

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
        "content",
        "lines",
        "hidden",
        "size",
        "sum",
        "creation",
        "lastChange",
        "lastAccess",
    ];
    pub fn as_str<'source>(&self) -> &'source str {
        match self {
            Name => "name",
            Path => "path",
            Parent => "parent",
            #[cfg(unix)]
            OwnerID => "ownerID",
            Empty => "empty",
            Readonly => "readonly",
            Elf => "elf",
            Text => "txt",
            Content => "content",
            Line(_) => "lines",
            Hidden => "hidden",
            Size(_) => "size=",
            Sum(_) => "sum=",
            Creation(_) => "creation=",
            LastModification(_) => "lastChange",
            LastAccess(_) => "lastAccess",
        }
    }
}

impl std::fmt::Display for Lookup {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{lookup}", lookup = self.as_str())
    }
}

impl Parse<'_> for Lookup {
    // this parser assumes that the parser is already at `Token::Variable`
    fn parse(parser: &mut Parser) -> ParseRes<Self> {
        use crate::{no_spec, spec};

        let full_var = parser.current_token_source().strip_prefix('@').unwrap();
        let (var_name, var_spec): (&str, Option<&str>) = full_var
            .find('=')
            .map(|idx| {
                let (a, b) = full_var.split_at(idx);
                (a, Some(&b[1..]))
            })
            .unwrap_or((full_var, None));
        match var_name {
            "name" => no_spec!("name", Name, var_spec),
            "path" => no_spec!("path", Path, var_spec),
            "parent" => no_spec!("parent", Parent, var_spec),
            #[cfg(unix)]
            "ownerID" => no_spec!("ownerID", OwnerID, var_spec),
            "empty" => no_spec!("empty", Empty, var_spec),
            "readonly" => no_spec!("readonly", Readonly, var_spec),
            "elf" => no_spec!("elf", Elf, var_spec),
            "txt" => no_spec!("txt", Text, var_spec),
            "content" => no_spec!("content", Content, var_spec),
            "line" => spec!("line", usize, Line, var_spec),
            "hidden" => no_spec!("hidden", Hidden, var_spec),
            "size" => spec!("size", SizeLabel, Size, var_spec),
            "sum" => spec!("sum", SumKind, Sum, var_spec),
            "creation" => spec!("creation", TimeStamp, Creation, var_spec),
            "lastChange" => spec!("lastChange", TimeStamp, LastModification, var_spec),
            "lastAccess" => spec!("lastAccess", TimeStamp, LastAccess, var_spec),
            _ => Err(ParseErrorKind::UnknownVariable),
        }
        .map_err(|e| parser.error(e))
    }
}

trait Specifier<T: Sized, const O: usize> {
    const OPTS: [&'static str; O];
    fn detect(source: &str) -> Option<T>;
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SizeLabel {
    TeraBytes,
    GigaBytes,
    MegaBytes,
    KiloBytes,
    Bytes,
}

impl Specifier<Self, 5> for SizeLabel {
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SumKind {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
}

impl Specifier<Self, 6> for SumKind {
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeStamp {
    /// Presented to the user as yyyy-mm-dd
    Date,
    /// Presented to the user as hh:mm:ss
    Hour,
}

impl Specifier<Self, 2> for TimeStamp {
    const OPTS: [&'static str; 2] = ["date", "hour"];
    fn detect(source: &str) -> Option<Self> {
        Some(match source {
            "date" => Self::Date,
            "hour" => Self::Hour,
            _ => return None,
        })
    }
}

impl std::fmt::Display for TimeStamp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Date => "date",
            Self::Hour => "hour",
        })
    }
}

impl Specifier<Self, 1> for usize {
    const OPTS: [&'static str; 1] = ["any positive integer"];
    fn detect(source: &str) -> Option<Self> {
        source.parse().ok()
    }
}

use crate::interpreter::{with_blocks, Cache, CachedResolve, ErrorKind, ExprResult, Resolve};
use std::time::SystemTime;
impl CachedResolve for Lookup {
    fn cached_resolve(&self, cache: &mut Cache) -> Result<ExprResult, ErrorKind> {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        match self {
            Name => Ok(cache
                .get_path()
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap()
                .into()),
            Path => cache
                .get_path()
                .canonicalize()
                .map_err(ErrorKind::from)
                .map(|path| path.to_str().unwrap().into()),
            // TODO: add error for not having parent
            Parent => cache
                .get_path()
                .parent()
                .unwrap()
                .canonicalize()
                .map_err(ErrorKind::from)
                .map(|path| path.to_str().unwrap().into()),

            #[cfg(unix)]
            OwnerID => cache
                .get_file_metadata()
                .map(|m| m.uid() as f64)
                .map(ExprResult::from),
            Empty => cache
                .get_file_metadata()
                .map(|m| m.len() <= 1)
                .map(ExprResult::from),
            Readonly => cache
                .get_file_metadata()
                .map(|m| m.permissions().readonly())
                .map(ExprResult::from),
            Elf => {
                use std::io::Read;
                use std::io::Seek;
                let mut rb = [0u8; 4];
                let br = cache.get_file_mut()?;
                // impl a partial reader that can seek wherever
                br.rewind().map_err(ErrorKind::from)?;
                let size = br.read(&mut rb).map_err(ErrorKind::from)?;
                Ok(size >= 4 && rb == [0x7f, b'E', b'L', b'F']).map(ExprResult::from)
            },
            Text => {
                use std::io::Read;
                use std::io::Seek;
                use std::io::SeekFrom;

                let bufreader = cache.get_file_mut()?;
                bufreader.seek(SeekFrom::End(-1))?;
                let mut buf = [0u8; 1];
                let last = if bufreader.read(&mut buf)? > 0 {
                    Some(buf[0])
                } else {
                    None
                };
                Ok(last == Some(b'\n') || last == Some(b'\r')).map(ExprResult::from)
            },
            Content => {
                use std::ops::Deref;

                let mut reader = cache.get_file_mut()?;
                let mut buffer = Vec::new();

                with_blocks(&mut reader, |block| buffer.extend_from_slice(block)).unwrap_or(());

                Ok(String::from_utf8_lossy(&buffer).deref().into())
            },
            Line(l) => {
                use std::io::BufRead;

                let reader = cache.get_file_mut().unwrap();
                let offset = vec![Ok(String::from(""))].into_iter();

                Ok(offset
                    .chain(reader.lines())
                    .nth(*l)
                    .unwrap_or_else(|| Ok(String::new()))
                    .unwrap_or(String::new())
                    .into())
            },
            Hidden => {
                #[cfg(not(any(unix, windows)))]
                {
                    println!("`hidden` variable is not natively supported in the current OS, falling back to unix implementation");
                }
                #[cfg(not(windows))]
                {
                    use std::ffi::OsStr;
                    Ok(ExprResult::from(
                        cache
                            .get_path()
                            .file_name()
                            .unwrap_or_else(|| OsStr::new(""))
                            .to_str()
                            .unwrap_or("")
                            .starts_with('.'),
                    ))
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    Ok(ExprResult::from(
                        cache.get_file_metadata()?.file_attributes() & 0x2 > 0,
                    )) // https://docs.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants
                }
            },
            Creation(ts) => {
                let created_time = cache.get_file_metadata()?.created()?;
                Ok(get_timestamp(created_time, ts))
            },
            LastModification(ts) => {
                let mod_time = cache.get_file_metadata()?.modified()?;
                Ok(get_timestamp(mod_time, ts))
            },
            LastAccess(ts) => {
                let last_access_time = cache.get_file_metadata()?.created()?;
                Ok(get_timestamp(last_access_time, ts))
            },
            // note: think about using Decimal (for the 2 decimal imposed precision):
            // https://crates.io/crates/rust-decimal
            Size(sz) => Ok(cache.get_file_metadata()?.len() as f64
                / match sz {
                    SizeLabel::Bytes => 1.0,
                    SizeLabel::KiloBytes => 1_000.0,
                    SizeLabel::MegaBytes => 1_000_000.0,
                    SizeLabel::GigaBytes => 1_000_000_000.0,
                    SizeLabel::TeraBytes => 1_000_000_000_000.0,
                })
            .map(ExprResult::from),
            Sum(sum) => {
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
    match timestamp {
        TimeStamp::Date => ExprResult::Date(datetime.date()),
        TimeStamp::Hour => ExprResult::Time(datetime.time()),
    }
}
