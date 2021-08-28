use super::utils::{path::Path, ByteConversion, Str, Sum, SumTypes, Timestamps};
use super::{Literal, LiteralKind};
use byte_unit::ByteUnit;
use std::ffi::OsString;
use std::fs;
use std::io::prelude::*;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path;
use std::path::PathBuf;
use std::time::SystemTime;

pub trait Variables {
    fn get_var_if_any(&self, var: &Literal) -> Result<Literal, String>;
}

impl Variables for super::Interpreter {
    fn get_var_if_any(&self, var: &super::Literal) -> Result<Literal, String> {
        // get file metadata,
        // it'll be useful later
        let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();

        // is it a string starting with a @? ok, warn and return it as string
        // no? get the variable
        match var.content.as_str() {
            "path" => Ok(Literal {
                kind: LiteralKind::Str,
                content: self.absolutize(&self.__file__),
            }),
            "name" => {
                // get path & convert OsStr to String using an OsString as intermediary
                let _name: OsString = path::Path::new(&self.__file__).file_name().unwrap().into();
                let name = _name.to_str().unwrap().to_string();
                Ok(Literal {
                    kind: LiteralKind::Str,
                    content: name,
                })
            },
            "parent" => {
                let mut parent_path = PathBuf::new();
                parent_path.push(PathBuf::from(&self.__file__).parent().unwrap());

                Ok(Literal {
                    kind: LiteralKind::Str,
                    content: self.absolutize(&parent_path.into_os_string().into_string().unwrap()),
                })
            },
            "size=tb" => Ok(Literal {
                kind: LiteralKind::Str,
                content: format!("{}", self.convert(metadata.len() as u128, ByteUnit::TB)),
            }),
            "size=gb" => Ok(Literal {
                kind: LiteralKind::Str,
                content: format!("{}", self.convert(metadata.len() as u128, ByteUnit::GB)),
            }),
            "size=mb" => Ok(Literal {
                kind: LiteralKind::Str,
                content: format!("{}", self.convert(metadata.len() as u128, ByteUnit::MB)),
            }),
            "size=kb" => Ok(Literal {
                kind: LiteralKind::Str,
                content: format!("{}", self.convert(metadata.len() as u128, ByteUnit::KB)),
            }),
            "size=bs" => Ok(Literal {
                kind: LiteralKind::Str,
                content: format!("{}", metadata.len()),
            }),
            "empty" => {
                Ok(Literal {
                    kind: LiteralKind::Str,
                    content: if metadata.len() <= 1
                    // 1 instead of 0 because sometimes a file is empty but returns 1
                    {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    },
                })
            },
            "readonly" => Ok(Literal {
                kind: LiteralKind::Str,
                content: format!("{}", metadata.permissions().readonly()),
            }),
            "elf" => {
                // create an empty non-growable buffer
                let mut buffer = [0u8; 4];

                Ok(Literal {
                    kind: LiteralKind::Str,
                    content: if let Err(err) =
                        fs::File::open(&self.__file__).and_then(|mut f| f.read(&mut buffer))
                    {
                        format!("error reading file {}: {}", self.__file__, err)
                    } else {
                        match buffer {
                            // thats the byte sequence ELF files must start with
                            [0x7f, b'E', b'L', b'F'] => "true".to_string(),
                            _ => "false".to_string(),
                        }
                    },
                })
            },
            "txt" => {
                // create a growable buffer
                let mut buffer = vec![0u8];

                Ok(Literal {
                    kind: LiteralKind::Str,
                    content: if let Err(err) =
                        fs::File::open(&self.__file__).and_then(|mut f| f.read_to_end(&mut buffer))
                    {
                        format!("error reading file {}: {}", self.__file__, err)
                    } else {
                        match buffer[buffer.len() - 1] {
                            // CR or LF
                            b'\r' | b'\n' => "true".to_string(),
                            _ => "false".to_string(),
                        }
                    },
                })
            },
            "sum=md5" => Ok(Literal {
                kind: LiteralKind::Str,
                content: self.get_sum_of(&self.__file__, SumTypes::Md5),
            }),
            "sum=sha1" => Ok(Literal {
                kind: LiteralKind::Str,
                content: self.get_sum_of(&self.__file__, SumTypes::Sha1),
            }),
            "sum=sha224" => Ok(Literal {
                kind: LiteralKind::Str,
                content: self.get_sum_of(&self.__file__, SumTypes::Sha224),
            }),
            "sum=sha256" => Ok(Literal {
                kind: LiteralKind::Str,
                content: self.get_sum_of(&self.__file__, SumTypes::Sha256),
            }),
            "sum=sha384" => Ok(Literal {
                kind: LiteralKind::Str,
                content: self.get_sum_of(&self.__file__, SumTypes::Sha384),
            }),
            "sum=sha512" => Ok(Literal {
                kind: LiteralKind::Str,
                content: self.get_sum_of(&self.__file__, SumTypes::Sha512),
            }),
            "ownerID" => {
                Ok(Literal {
                    kind: LiteralKind::Str,
                    content: {
                        // Ensures *nix specific metadata is not included on non-*nix systems
                        #[cfg(unix)]
                        {
                            format!("{}", metadata.uid())
                        }

                        #[cfg(not(unix))]
                        {
                            format!("ownerID is an Unix-Only variable!")
                        }
                    },
                })
            },
            "creation=date" => Ok(Literal {
                kind: LiteralKind::Str,
                content: self.get_date(
                    metadata
                        .created()
                        .unwrap()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap(),
                ),
            }),
            "creation=hour" => Ok(Literal {
                kind: LiteralKind::Str,
                content: self.get_hour(
                    metadata
                        .created()
                        .unwrap()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap(),
                ),
            }),
            "lastChange=date" => Ok(Literal {
                kind: LiteralKind::Str,
                content: self.get_date(
                    metadata
                        .modified()
                        .unwrap()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap(),
                ),
            }),
            "lastChange=hour" => Ok(Literal {
                kind: LiteralKind::Str,
                content: self.get_hour(
                    metadata
                        .modified()
                        .unwrap()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap(),
                ),
            }),
            "lastAccess=date" => Ok(Literal {
                kind: LiteralKind::Str,
                content: self.get_date(
                    metadata
                        .accessed()
                        .unwrap()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap(),
                ),
            }),
            "lastAccess=hour" => Ok(Literal {
                kind: LiteralKind::Str,
                content: self.get_hour(
                    metadata
                        .accessed()
                        .unwrap()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap(),
                ),
            }),
            _ => {
                let kind = LiteralKind::Str;
                let content = if var.kind == LiteralKind::Var {
                    format!("@{}", &var.content)
                } else {
                    var.content.clone()
                };
                Ok(Literal { kind, content })
            },
        }
    }
}
