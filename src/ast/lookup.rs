use super::parser::{Parse, ParseErrorKind, ParseRes, Parser};

#[derive(Debug, Clone, Copy)]
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

impl Parse<'_> for Lookup {
    // this parser assumes that the parser is already at `Token::Variable`
    fn parse(parser: &mut Parser) -> ParseRes<Self> {
        let full_var = parser.current_token_source().strip_prefix("@").unwrap();
        let (var_name, var_spec): (&str, Option<&str>) = full_var
            .find('=')
            .map(|idx| {
                let (a, b) = full_var.split_at(idx);
                (a, Some(b))
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

#[derive(Debug, Clone, Copy)]
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
            "kb" => Self::KiloBytes,
            "bs" => Self::Bytes,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SumKind {
    Md5,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
}

impl SumKind {
    const OPTS: [&'static str; 5] = ["md5", "sha224", "sha256", "sha384", "sha512"];
    fn detect(source: &str) -> Option<Self> {
        Some(match source {
            "md5" => Self::Md5,
            "sha224" => Self::Sha224,
            "sha256" => Self::Sha256,
            "sha384" => Self::Sha384,
            "sha512" => Self::Sha512,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TimeStamp {
    /// Presented to the user as yyyy-mm-dd
    Date,
    /// Presented to the user as hh:mm:ss
    Hour,
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
