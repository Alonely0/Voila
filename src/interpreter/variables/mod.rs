extern crate chrono;

pub mod variables;

use super::super::byte_unit;
use super::utils::path::Path;
use super::utils::SumTypes;
use super::utils::{Str, Sum};
use super::{Literal, LiteralKind};
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path;
use std::path::PathBuf;
use std::time::SystemTime;
pub use variables::Variables;

impl Variables for super::Interpreter {
    fn get_var_if_any(&self, var: &super::Literal) -> Result<Literal, String> {
        // is it a string starting with a @? ok, warn and return it as string
        // no? get the variable
        match Some(&*var.content) {
            Some("path") => {
                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: self.absolutize(&self.__file__),
                })
            }
            Some("name") => {
                // get path & convert OsStr to String using an OsString as intermediary
                let _name: OsString = path::Path::new(&self.__file__).file_name().unwrap().into();
                let name = _name.to_str().unwrap().to_string();
                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: format!("{}", name),
                });
            }
            Some("parent") => {
                let mut parent_path = PathBuf::new();
                parent_path.push(PathBuf::from(&self.__file__).parent().unwrap());
                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: self.absolutize(&parent_path.into_os_string().into_string().unwrap()),
                });
            }
            Some("size=tb") => {
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

                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: String::from(str_chars.as_str()),
                });
            }
            Some("size=gb") => {
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

                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: String::from(str_chars.as_str()),
                });
            }
            Some("size=mb") => {
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

                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: String::from(str_chars.as_str()),
                });
            }
            Some("size=kb") => {
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

                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: String::from(str_chars.as_str()),
                });
            }
            Some("size=bs") => {
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();
                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: format!("{}", metadata.len()),
                });
            }
            Some("empty") => {
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();
                let empty: bool;

                // 1 instead of 0 because sometimes a file is empty but returns 1
                if metadata.len() <= 1 {
                    empty = true;
                } else {
                    empty = false;
                }
                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: format!("{}", empty),
                });
            }
            Some("readonly") => {
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();
                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: format!("{}", metadata.permissions().readonly()),
                });
            }
            Some("sha256sum") => {
                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: self
                        .get_sum_of(&self.__file__, SumTypes::Sha256)
                        .unwrap_or_else(|x| x),
                });
            }
            Some("md5sum") => {
                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: self
                        .get_sum_of(&self.__file__, SumTypes::Md5)
                        .unwrap_or_else(|x| x),
                });
            }
            Some("ownerID") => {
                let metadata = fs::metadata(self.trim_spaces(&self.__file__)).unwrap();
                // this only works on unix
                if cfg!(unix) {
                    return Ok(Literal {
                        kind: LiteralKind::Str,
                        content: format!("{:?}", metadata.uid()),
                    });
                } else {
                    return Ok(Literal {
                        kind: LiteralKind::Str,
                        content: format!("ownerID is an Unix-Only variable!"),
                    });
                }
            }
            Some("creation=date") => {
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
                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: String::from(str_chars.as_str()),
                });
            }
            Some("creation=hour") => {
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
                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: String::from(str_chars.as_str()),
                });
            }
            Some("lastChange=date") => {
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
                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: String::from(str_chars.as_str()),
                });
            }
            Some("lastChange=hour") => {
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
                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: String::from(str_chars.as_str()),
                });
            }
            Some("lastAccess=date") => {
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

                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: String::from(str_chars.as_str()),
                });
            }
            Some("lastAccess=hour") => {
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
                return Ok(Literal {
                    kind: LiteralKind::Str,
                    content: String::from(str_chars.as_str()),
                });
            }
            _ => {
                if var.kind == LiteralKind::Var {
                    return Ok(Literal {
                        kind: LiteralKind::Str,
                        content: format!("@{}", var.content.clone()),
                    });
                } else {
                    return Ok(Literal {
                        kind: LiteralKind::Str,
                        content: var.content.clone(),
                    });
                }
            }
        }
    }
}
