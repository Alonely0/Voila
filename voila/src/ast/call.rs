use super::parser::{ContextLevel, Parse, ParseErrorKind, ParseRes, Parser, WantedSpec};
use super::HasSpan;
use super::Str;
use super::Token;
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::io;

use std::ops::Range;

/// Represents a call in the script like
/// `shell` or `delete`. All functions receive the arguments in
/// interpolated strings, which will have their variables resolved
/// before execution.
///
/// # List of non-destructive functions
/// These functions are safe to use without any worries that
/// any data will be eliminated:
///     - `print`
///     - `mkdir`
/// `shell` won't be in the list, since you can do absolutely anything
/// when we give you a shell. The shell is an escape hatch to enable the
/// integration of Voila to the rest of the system.
#[derive(Serialize, Deserialize, Debug)]
pub struct Call<'source> {
    pub function_kind: Function,
    pub arguments: Vec<Str<'source>>,
    pub safe: bool,
    span: Range<usize>,
}

/// The function that is [`Call`]ed.
///
/// # Panic
/// The interpreter will panic when the function has not
/// enough arguments to execute.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Function {
    /// Create a directory with its parents recursively.
    /// This function is not destructive, it will error
    /// if a file with the name of the directory exists already.
    ///
    /// # Call format
    /// `mkdir` receives at least one argument: the path to create.
    /// You can put more directories to create, but make sure to separate them
    /// by commas!
    Mkdir { safe: bool },
    /// Print something to standard output
    ///
    /// # Call format
    /// `print` receives a variadic number of arguments which it prints
    /// separated by spaces (similar to python's print function without parameters), and a newline
    /// after.
    Print { safe: bool },
    /// Execute a command in a shell
    ///
    /// # Call format
    /// `shell` needs at least one argument. When called, it gets all the arguments, joins all by spaces
    /// and feeds that to `sh -c` in the case of linux and `powershell` on windows. No input is given to it, so things like `sudo`
    /// won't work unless you start voila with privileges
    ///
    /// # Safety
    /// This function may modify the outer system!
    Shell { safe: bool },
    /// Delete the given files/directories
    ///
    /// # Call format
    /// `delete` receives at least one argument: the file/directory to delete.
    /// You can put more things to remove, but make sure to separate them by commas!
    /// Directories are deleted recursively!
    ///
    /// # Safety
    /// `delete` will modify the outer system! Make sure that you're not doing
    /// accesses to the file in the argument on the same cycle, otherwise you will
    /// get undefined behavior.
    Delete { safe: bool },
    /// Moves or renames a file, with a similar behavior to the `mv` command.
    ///
    /// # Call format
    /// `move` receives two arguments: the source file/directory and the target destination
    ///
    /// # Safety
    /// `move` is a destructive call, so please make sure that you're not using it with the same file in the same cycle. Refer to [`Function::Delete`] for details
    Move { safe: bool },
    /// Copy a file or a directory. Directories are copied recursively.
    ///
    /// # Call format
    /// `copy` receives two arguments: the source file/directory and the target destination
    ///
    /// # Safety
    /// `copy` might overwrite files in the system, so use it carefully! Avoid using it in the same
    /// cycle unless you can prove it's safe to do so.
    Copy { safe: bool },
    /// Gzip a file or a directory. Directories are gzipped recursively.
    ///
    // NOTE: please rename this to `gzip` and `gunzip` like the binutils
    /// # Call format
    /// `gzc` receives two arguments: the source file/directory to compress and the file to save it
    /// into. Note that the destination name is not manipulated in any way (nothing is added or
    /// removed to it)
    ///
    /// # Safety
    /// Since `gzc` has an output file. it may overwrite another that's in the system.
    GzipCompress { safe: bool },
    /// Gunzip a file into a file/directory.
    ///
    /// # Call format
    /// `gzd` receives two arguments: the gzipped file, and the destination to decompress into.
    /// The destination, if not specified, is the directory in which the gzipped file is, **not the
    /// one that voila is executing in**
    ///
    /// # Safety
    /// since `gzd` has an output directory, it may overwrite a lot af files! Use with care.
    GzipDecompress { safe: bool },
    /// Create a file, with optional contents
    ///
    /// # Call format
    /// `create` receives the file to create and an optional second argument with the contents
    ///
    /// # Safety
    /// `create` will modify the file system!
    Create { safe: bool },
    /// Execute an executable as a child process, ignoring the cycle.await
    ///
    /// # Call format
    /// `child` receives either a binary or a path pointing to a executable. All other arguments
    /// are passed as arguments
    ///
    /// # Safety
    /// As safe as the executable is. Like in the shell function, the safety checker will treat
    /// arguments as access, modify and created as has 0 information about what the executable will do
    Child { safe: bool },
}

impl Function {
    pub const fn minimum_arg_count(&self) -> u8 {
        match self {
            Self::Copy { safe: _ }
            | Self::Move { safe: _ }
            | Self::GzipCompress { safe: _ }
            | Self::GzipDecompress { safe: _ } => 2,
            Self::Delete { safe: _ }
            | Self::Shell { safe: _ }
            | Self::Mkdir { safe: _ }
            | Self::Create { safe: _ }
            | Self::Child { safe: _ } => 1,
            Self::Print { safe: _ } => 0,
        }
    }
    fn from_name(source: &str, safe: bool) -> Option<Self> {
        Some(match source.trim() {
            "copy" => Self::Copy { safe },
            "move" => Self::Move { safe },
            "gzc" => Self::GzipCompress { safe },
            "gzd" => Self::GzipDecompress { safe },
            "delete" => Self::Delete { safe },
            "shell" => Self::Shell { safe },
            "mkdir" => Self::Mkdir { safe },
            "print" => Self::Print { safe },
            "create" => Self::Create { safe },
            "child" => Self::Child { safe },
            _ => return None,
        })
    }
    fn is_safe(&self) -> bool {
        match *self {
            Function::Mkdir { safe }
            | Function::Print { safe }
            | Function::Shell { safe }
            | Function::Delete { safe }
            | Function::Copy { safe }
            | Function::Move { safe }
            | Function::GzipCompress { safe }
            | Function::GzipDecompress { safe }
            | Function::Create { safe }
            | Function::Child { safe } => safe,
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Copy { safe: _ } => "copy",
            Self::Move { safe: _ } => "move",
            Self::GzipCompress { safe: _ } => "gzc",
            Self::GzipDecompress { safe: _ } => "gzd",
            Self::Delete { safe: _ } => "delete",
            Self::Shell { safe: _ } => "shell",
            Self::Mkdir { safe: _ } => "mkdir",
            Self::Print { safe: _ } => "print",
            Self::Create { safe: _ } => "create",
            Self::Child { safe: _ } => "child",
        })
    }
}

impl Parse<'_> for Function {
    fn parse(parser: &mut Parser) -> ParseRes<Self> {
        // this doesn't accept the token (unless unsafe is found) because
        // it is accepted by the calling parser (`Call`), so it can use
        // the identifier start as a more accurate start of the function span.
        let mut safe = true;
        let mut src = parser.expect_token(
            Token::Identifier,
            Some("as the name of the function or unsafe statement"),
        )?;
        if src == "unsafe" {
            safe = false;
            parser.accept_current();
            src = parser.expect_token(Token::Identifier, Some("as the name of the function"))?
        }
        Self::from_name(src, safe).ok_or_else(|| parser.error(ParseErrorKind::UnknownFunction))
    }
}

impl<'source> Call<'source> {
    pub fn offset(&self) -> usize {
        self.span.start
    }
}

impl HasSpan for Call<'_> {
    fn span(&self) -> &Range<usize> {
        &self.span
    }
}

impl<'source> Parse<'source> for Call<'source> {
    fn parse(parser: &mut Parser<'source>) -> ParseRes<Self> {
        parser.with_context(ContextLevel::Call, |parser| {
            let function_kind = parser.parse()?;
            let start = parser.current_token_span().start;
            parser.accept_current();
            parser.expect_token(
                Token::OpenParen,
                Some("to begin the function call arguments"),
            )?;
            parser.accept_current();
            let mut arguments = Vec::new();

            loop {
                if parser.expect_any_token(Some(
                    WantedSpec::explicit_multiple(vec![
                        Token::Variable,
                        Token::Identifier,
                        Token::CloseParen,
                    ])
                    .with_explanation("end of argument list or argument to the function"),
                ))? == Token::CloseParen
                {
                    break;
                }
                arguments.push(parser.parse()?);
                if parser.expect_any_token(Some(
                    WantedSpec::explicit_multiple(vec![Token::CloseParen, Token::Identifier])
                        .with_explanation("end of argument list or comma to continue it"),
                ))? != Token::Comma
                {
                    break;
                }
                parser.accept_current()
            }
            parser.expect_token(Token::CloseParen, Some("to end the argument list"))?;
            let end = parser.current_token_span().end;
            parser.accept_current();
            Ok(Self {
                function_kind,
                arguments,
                safe: function_kind.is_safe(),
                span: start..end,
            })
        })
    }
}
use crate::interpreter::{Cache, ErrorKind, ExprResult};
use path_absolutize::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub fn run_call(call: &Call, cache: Arc<Mutex<Cache>>) -> Result<(), ErrorKind> {
    use crate::interpreter::ArgCountMismatched;

    // note: already considered streaming the arguments instead
    // of collecting all of them, but the number of arguments is very low (1 or 2),
    // so there is no real performance hit if we evaluate all of them now.
    let mut args: Vec<String> = call
        .arguments
        .iter()
        // note: grabbing the cache lock on each argument separately to prevent locking
        // the cache too much time, e.g in a cycle like
        // `print(@sum=sha256 @sum=sha512 @sum=md5) print(@path)`,
        // if the first `print` grabs the cache first, it will only prevent the second `print` from
        // executing while it's calculating the SHA256 sum, then the second print will be executed
        .map(|arg| cache.lock().unwrap().resolve(arg))
        .map(|x| x.map(ExprResult::cast_to_string))
        .collect::<Result<_, _>>()?;
    // drop the guard now since we're finished
    drop(cache);

    ArgCountMismatched::check(call.function_kind, args.len())?;

    // todo: error contexts in interpreter errors...
    match call.function_kind {
        Function::Print { safe: _ } => print(args),
        Function::Create { safe: _ } => create(&args[0], args.get(1).map(String::as_str)),
        Function::Mkdir { safe: _ } => mkdir(args),
        Function::Delete { safe: _ } => delete(args),
        Function::Copy { safe: _ } => {
            copy_file_or_dir(args[0].as_str().into(), args[1].as_str().into())
        },
        Function::Move { safe: _ } => move_file(&args[0], &args[1]),
        Function::GzipCompress { safe: _ } => gzc(&args[0], &args[1]),
        Function::GzipDecompress { safe: _ } => gzd(&args[0], &args[1]),
        Function::Shell { safe: _ } => shell(args),
        Function::Child { safe: _ } => child(&args.remove(0), args),
    }
    .map_err(Into::into)
}

fn print(args: Vec<String>) -> Result<(), io::Error> {
    let args = args
        .into_iter()
        .enumerate()
        .fold(String::new(), |acc, (i, next)| {
            acc + if i > 0 { " " } else { "" } + &next
        });
    let stdout = io::stdout();
    // lock stdout since we're executing in multithread
    let mut stdout = stdout.lock();
    use io::Write;

    stdout.write_all(args.as_bytes())?;
    stdout.write_all(b"\n")?;
    stdout.flush()
}

fn create(dest: &str, content: Option<&str>) -> Result<(), io::Error> {
    use std::fs;
    fs::write(dest, content.unwrap_or(""))
}

fn mkdir(dirs: Vec<String>) -> Result<(), io::Error> {
    use std::fs;
    dirs.into_iter().try_for_each(fs::create_dir_all)
}

fn delete(files: Vec<String>) -> Result<(), io::Error> {
    files.into_iter().try_for_each(|x| delete_file_or_dir(&x))
}

fn delete_file_or_dir(target: &str) -> Result<(), io::Error> {
    use std::fs;
    let mut t = PathBuf::from(target);
    let metadata = match fs::metadata(target) {
        Ok(meta) => meta,
        Err(_) => return Ok(()),
    };

    if t.is_relative() {
        t = t.absolutize()?.into();
    }

    if metadata.is_dir() {
        fs::remove_dir_all(t)
    } else {
        fs::remove_file(t)
    }
}

fn copy_file_or_dir(mut source: PathBuf, mut dest: PathBuf) -> Result<(), io::Error> {
    use std::fs;

    if source.is_relative() {
        source = source.absolutize()?.into();
    }
    if dest.is_relative() {
        dest = dest.absolutize()?.into();
    }

    if dest.exists() && dest.is_dir() {
        dest = dest.join(
            source
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap(),
        );
    }

    if source.is_dir() {
        fs::create_dir_all(dest)?;
    } else {
        fs::copy(source, dest)?;
    }
    Ok(())
}

fn move_file(source: &str, dest: &str) -> Result<(), io::Error> {
    copy_file_or_dir(source.into(), dest.into())?;
    delete_file_or_dir(source)
}

fn gzc(source: &str, dest: &str) -> Result<(), io::Error> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::fs;

    let dest = fs::File::create(dest)?;
    let encoder = GzEncoder::new(dest, Compression::default());
    let mut tar = tar::Builder::new(encoder);
    let source = PathBuf::from(source);
    if source.is_dir() {
        tar.append_dir_all(source.clone(), source)
    } else {
        tar.append_path(source)
    }
}

fn gzd(source: &str, dest: &str) -> Result<(), io::Error> {
    use flate2::read::GzDecoder;
    use std::fs::File;
    use tar::Archive;

    let mut archive = Archive::new(GzDecoder::new(File::open(source)?));
    archive.unpack(dest)
}

use std::process::Command;

fn shell(commands: Vec<String>) -> Result<(), io::Error> {
    commands.into_iter().try_for_each(|cmd| {
        let complete_command: Result<_, std::io::Error> = {
            #[cfg(windows)]
            {
                let mut initial = Command::new("powershell");
                initial.arg("-Command");
                Ok(initial)
            }

            #[cfg(unix)]
            {
                let mut initial = Command::new("sh");
                initial.arg("-c");
                Ok(initial)
            }

            #[cfg(not(any(unix, windows)))]
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Voila's shell is only supported on Windows & Unix-like systems",
            ))
        };
        let mut complete_command = complete_command?;

        complete_command.arg(cmd);
        // question is: will this thread join with rayon threadpool?
        // TODO: refactor this to use the thread pool.
        complete_command.spawn()?.wait()?;

        Ok(())
    })
}

fn child(executable: &str, arguments: Vec<String>) -> Result<(), io::Error> {
    Command::new(executable)
        .args(arguments)
        .spawn()
        .map(|_| ())
}
