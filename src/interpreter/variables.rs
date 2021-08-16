extern crate chrono;

use if_chain::if_chain;
use std::ffi::OsString;
use std::fs;
use std::io::prelude::*;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path;
use std::path::PathBuf;
use std::time::SystemTime;

use super::utils::{path::Path, Str, Sum, SumTypes};
use super::{Literal, LiteralKind};

pub trait Variables {
    fn get_var_if_any(&self, var: &Literal) -> Result<Literal, String>;
}

impl Variables for super::Interpreter {
    fn get_var_if_any(&self, var: &super::Literal) -> Result<Literal, String> {
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
            "size=tb" => {
                // get metadata
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();
                // get bytes object from file size
                let byte = byte_unit::Byte::from_bytes(metadata.len() as u128);
                // get size in format needed, then get str & convert it to chars
                let str = format!("{}", byte.get_adjusted_unit(byte_unit::ByteUnit::TB));
                let mut str_chars = str.chars();
                // remove size label
                str_chars.next_back();
                str_chars.next_back();
                str_chars.next_back();

                Ok(Literal {
                    kind: LiteralKind::Str,
                    content: String::from(str_chars.as_str()),
                })
            },
            "size=gb" => {
                // get metadata
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();
                // get bytes object from file size
                let byte = byte_unit::Byte::from_bytes(metadata.len() as u128);
                // get size in format needed, then get str & convert it to chars
                let str = format!("{}", byte.get_adjusted_unit(byte_unit::ByteUnit::GB));
                let mut str_chars = str.chars();
                // remove size label
                str_chars.next_back();
                str_chars.next_back();
                str_chars.next_back();

                Ok(Literal {
                    kind: LiteralKind::Str,
                    content: String::from(str_chars.as_str()),
                })
            },
            "size=mb" => {
                // get metadata
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();
                // get bytes object from file size
                let byte = byte_unit::Byte::from_bytes(metadata.len() as u128);
                // get size in format needed, then get str & convert it to chars
                let str = format!("{}", byte.get_adjusted_unit(byte_unit::ByteUnit::MB));
                let mut str_chars = str.chars();
                // remove size label
                str_chars.next_back();
                str_chars.next_back();
                str_chars.next_back();

                Ok(Literal {
                    kind: LiteralKind::Str,
                    content: String::from(str_chars.as_str()),
                })
            },
            "size=kb" => {
                // get metadata
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();
                // get bytes object from file size
                let byte = byte_unit::Byte::from_bytes(metadata.len() as u128);
                // get size in format needed, then get str & convert it to chars
                let str = format!("{}", byte.get_adjusted_unit(byte_unit::ByteUnit::KB));
                let mut str_chars = str.chars();
                // remove size label
                str_chars.next_back();
                str_chars.next_back();
                str_chars.next_back();

                Ok(Literal {
                    kind: LiteralKind::Str,
                    content: String::from(str_chars.as_str()),
                })
            },
            "size=bs" => {
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();
                Ok(Literal {
                    kind: LiteralKind::Str,
                    content: format!("{}", metadata.len()),
                })
            },
            "empty" => {
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();

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
            "readonly" => {
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();

                Ok(Literal {
                    kind: LiteralKind::Str,
                    content: format!("{}", metadata.permissions().readonly()),
                })
            },
            "elf" => {
                // create an empty non-growable buffer
                let mut buffer = [0u8; 4];

                Ok(Literal {
                    kind: LiteralKind::Str,
                    content: if_chain! {
                        if let Ok(mut f) = fs::File::open(&self.__file__);
                        if let Ok(_) = f.read(&mut buffer);
                        then {
                            match buffer {
                                // thats the byte sequence ELF files must start with
                                [0x7f, b'E', b'L', b'F'] => "true".to_string(),
                                _ => "false".to_string(),
                            }
                        } else {
                            format!("error reading file {}", &self.__file__)
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
                let kind = LiteralKind::Str;
                // Ensures *nix specific metadata is not included on non-*nix ststems
                let content = {
                    #[cfg(unix)]
                    {
                        let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();
                        format!("{:?}", metadata.uid())
                    }

                    #[cfg(not(unix))]
                    format!("ownerID is an Unix-Only variable!")
                };

                Ok(Literal { kind, content })
            },
            "creation=date" => {
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();
                // stuff to get date & hour
                let std_duration = metadata
                    .created()
                    .unwrap()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap();
                let chrono_duration = chrono::Duration::from_std(std_duration).unwrap();
                let unix = chrono::naive::NaiveDateTime::from_timestamp(0, 0);
                let naive = unix + chrono_duration;

                // remove hour
                let str = format!("{:?}", naive);
                let mut str_chars = str.chars();
                for _ in 0..19 {
                    str_chars.next_back();
                }

                let kind = LiteralKind::Str;
                let content = String::from(str_chars.as_str());
                Ok(Literal { kind, content })
            },
            "creation=hour" => {
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();
                // stuff to get date & hour
                let std_duration = metadata
                    .created()
                    .unwrap()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap();
                let chrono_duration = chrono::Duration::from_std(std_duration).unwrap();
                let unix = chrono::naive::NaiveDateTime::from_timestamp(0, 0);
                let naive = unix + chrono_duration;

                // remove date
                let str = format!("{:?}", naive);
                let mut str_chars = str.chars();
                for _ in 0..12 {
                    str_chars.next();
                }

                // remove ms
                for _ in 0..10 {
                    str_chars.next_back();
                }

                let kind = LiteralKind::Str;
                let content = String::from(str_chars.as_str());
                Ok(Literal { kind, content })
            },
            "lastChange=date" => {
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();
                // stuff to get date & hour
                let std_duration = metadata
                    .modified()
                    .unwrap()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap();
                let chrono_duration = chrono::Duration::from_std(std_duration).unwrap();
                let unix = chrono::naive::NaiveDateTime::from_timestamp(0, 0);
                let naive = unix + chrono_duration;

                // remove hour
                let str = format!("{:?}", naive);
                let mut str_chars = str.chars();
                for _ in 0..19 {
                    str_chars.next_back();
                }

                let kind = LiteralKind::Str;
                let content = String::from(str_chars.as_str());
                Ok(Literal { kind, content })
            },
            "lastChange=hour" => {
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();
                // stuff to get date & hour
                let std_duration = metadata
                    .modified()
                    .unwrap()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap();
                let chrono_duration = chrono::Duration::from_std(std_duration).unwrap();
                let unix = chrono::naive::NaiveDateTime::from_timestamp(0, 0);
                let naive = unix + chrono_duration;

                // remove date
                let str = format!("{:?}", naive);
                let mut str_chars = str.chars();
                for _ in 0..12 {
                    str_chars.next();
                }
                // remove ms
                for _ in 0..10 {
                    str_chars.next_back();
                }

                let kind = LiteralKind::Str;
                let content = String::from(str_chars.as_str());
                Ok(Literal { kind, content })
            },
            "lastAccess=date" => {
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();
                // stuff to get date & hour
                let std_duration = metadata
                    .accessed()
                    .unwrap()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap();
                let chrono_duration = chrono::Duration::from_std(std_duration).unwrap();
                let unix = chrono::naive::NaiveDateTime::from_timestamp(0, 0);
                let naive = unix + chrono_duration;

                // remove hour
                let str = format!("{:?}", naive);
                let mut str_chars = str.chars();
                for _ in 0..19 {
                    str_chars.next_back();
                }

                let kind = LiteralKind::Str;
                let content = String::from(str_chars.as_str());
                Ok(Literal { kind, content })
            },
            "lastAccess=hour" => {
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();
                // stuff to get date & hour
                let std_duration = metadata
                    .accessed()
                    .unwrap()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap();
                let chrono_duration = chrono::Duration::from_std(std_duration).unwrap();
                let unix = chrono::naive::NaiveDateTime::from_timestamp(0, 0);
                let naive = unix + chrono_duration;

                // remove date
                let str = format!("{:?}", naive);
                let mut str_chars = str.chars();
                for _ in 0..12 {
                    str_chars.next();
                }
                // remove ms
                for _ in 0..10 {
                    str_chars.next_back();
                }

                let kind = LiteralKind::Str;
                let content = String::from(str_chars.as_str());
                Ok(Literal { kind, content })
            },
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
