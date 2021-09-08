use super::parser::{Parse, ParseErrorKind, ParseRes, Parser};
use super::HasSpan;
use super::Lookup;
use super::Token;
use std::fmt;
use std::io;
use std::ops::Range;

// TODO: separate the impls for parser and interpreter in two modules for ecach

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
#[derive(Debug)]
pub struct Call<'source> {
    function_kind: Function,
    arguments: Vec<Arg<'source>>,
    span: Range<usize>,
}

/// The function that is [`Call`]ed.
///
/// # Panic
/// The interpreter will panic when the function has not
/// enough arguments to execute.
#[derive(Debug, Clone, Copy)]
pub enum Function {
    /// Create a directory with its parents recursively.
    /// This function is not destructive, it will error
    /// if a file with the name of the directory exists already.
    ///
    /// # Call format
    /// `mkdir` receives at least one argument: the path to create.
    /// You can put more directories to create, but make sure to separate them
    /// by commas!
    Mkdir,
    /// Print something to standard output
    ///
    /// # Call format
    /// `print` receives a variadic number of arguments which it prints
    /// separated by spaces (similar to python's print function without parameters), and a newline
    /// after.
    Print,
    /// Execute a command in `sh`
    ///
    /// # Call format
    /// `shell` needs at least one argument. When called, it gets all the arguments, joins all by spaces
    /// and feeds that to `sh -c` in the case of linux and `powershell` on windows. No input is given to it, so things like `sudo`
    /// won't work unless you start voila with privileges
    ///
    /// # Safety
    /// This function may modify the outer system!
    Shell,
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
    Delete,
    /// Moves or renames a file, with a similar behavior to the `mv` command.
    ///
    /// # Call format
    /// `move` receives two arguments: the source file/directory and the target destination
    ///
    /// # Safety
    /// `move` is a destructive call, so please make sure that you're not using it with the same file in the same cycle. Refer to [`Function::Delete`] for details
    Move,
    /// Copy a file or a directory. Directories are copied recursively.
    ///
    /// # Call format
    /// `copy` receives two arguments: the source file/directory and the target destination
    ///
    /// # Safety
    /// `copy` might overwrite files in the system, so use it carefully! Avoid using it in the same
    /// cycle unless you can prove it's safe to do so.
    Copy,
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
    GzipCompress,
    /// Gunzip a file into a file/directory.
    ///
    /// # Call format
    /// `gzd` receives two arguments: the gzipped file, and the destination to decompress into.
    /// The destination, if not specified, is the directory in which the gzipped file is, **not the
    /// one that voila is executing in**
    ///
    /// # Safety
    /// since `gzd` has an output directory, it may overwrite a lot af files! Use with care.
    GzipDecompress,
    /// Create a file, with optional contents
    ///
    /// # Call format
    /// `create` receives the file to create and an optional second argument with the contents
    ///
    /// # Safety
    /// `create` will modify the file system!
    Create,
}

impl Function {
    pub const fn minimum_arg_count(&self) -> u8 {
        match self {
            Self::Copy | Self::Move | Self::GzipCompress | Self::GzipDecompress => 2,
            Self::Delete | Self::Shell | Self::Mkdir | Self::Create => 1,
            Self::Print => 0,
        }
    }
    fn from_name(source: &str) -> Option<Self> {
        Some(match source {
            "copy" => Self::Copy,
            "move" => Self::Move,
            "gzc" => Self::GzipCompress,
            "gzd" => Self::GzipDecompress,
            "delete" => Self::Delete,
            "shell" => Self::Shell,
            "mkdir" => Self::Mkdir,
            "print" => Self::Print,
            "create" => Self::Create,
            _ => return None,
        })
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Copy => "copy",
            Self::Move => "move",
            Self::GzipCompress => "gzc",
            Self::GzipDecompress => "gzd",
            Self::Delete => "delete",
            Self::Shell => "shell",
            Self::Mkdir => "mkdir",
            Self::Print => "print",
            Self::Create => "create",
        })
    }
}

impl Parse<'_> for Function {
    fn parse(parser: &mut Parser) -> ParseRes<Self> {
        // this doesn't accept the token because it is accepted by the calling parser (`Call`), so
        // it can use the identifier start as a more accurrate start of the function span.
        let src = parser.expect_token(Token::Identifier, Some("as the name of the function"))?;
        Self::from_name(src).ok_or_else(|| parser.error(ParseErrorKind::UnknownFunction))
    }
}

impl HasSpan for Call<'_> {
    fn span(&self) -> &Range<usize> {
        &self.span
    }
}

/// Represents an argument to the call, with either
/// a single string like `hello world` or an interpolated
/// string with variables in it.
///
/// The argument maintains an invariant: Its sequence is
/// never empty
///
/// # Examples
///
/// - `@name`
/// - `@name is too big`
/// - `hello world`
#[derive(Debug)]
pub struct Arg<'source> {
    pub sequence: Vec<InterpolateComponent<'source>>,
    pub span: Range<usize>,
}

#[derive(Debug)]
pub enum InterpolateComponent<'source> {
    Literal(&'source str),
    Lookup(Lookup),
}

impl HasSpan for Arg<'_> {
    fn span(&self) -> &Range<usize> {
        &self.span
    }
}

// TODO: refactor this into just the interpolated sequence
impl<'source> Arg<'source> {
    /// Construct a literal string argument
    fn str(string: &'source str, span: Range<usize>) -> Self {
        Self {
            sequence: vec![InterpolateComponent::Literal(string)],
            span,
        }
    }
    /// Construct a lookup argument
    fn lookup(lookup: Lookup, span: Range<usize>) -> Self {
        Self {
            sequence: vec![InterpolateComponent::Lookup(lookup)],
            span,
        }
    }
    /// Extend the argument with a string literal, returning the next span
    /// (might be modified)
    fn extend_str(
        &mut self,
        last_span: Range<usize>,
        mut span: Range<usize>,
        source: &'source str,
    ) -> Range<usize> {
        if matches!(
            self.sequence.last().unwrap(),
            InterpolateComponent::Literal(_)
        ) {
            // if the last component was a literal,
            // we can just extend the source
            span.start = last_span.start;

            // UNSAFE: safe. we already unwrapped
            let last_ref = self.sequence.last_mut().unwrap();
            *last_ref = InterpolateComponent::Literal(&source[span.clone()]);
        } else {
            // if the last component was a variable,
            // we will extend the span to accomodate the space in between
            span.start = last_span.end;
            self.sequence
                .push(InterpolateComponent::Literal(&source[span.clone()]));
        }
        span
    }

    /// Extend the argument with a lookup, returning the next span
    fn extend_lookup(
        &mut self,
        lookup: Lookup,
        last_span: Range<usize>,
        span: Range<usize>,
        source: &'source str,
    ) -> Range<usize> {
        if matches!(
            self.sequence.last().unwrap(),
            InterpolateComponent::Literal(_)
        ) {
            // if the last component was a literal, we can
            // extend its source to accomodate the space in between
            let last_span = last_span.start..span.start;
            // UNSAFE: safe, we already unwrapped
            let last_ref = self.sequence.last_mut().unwrap();
            *last_ref = InterpolateComponent::Literal(&source[last_span]);
        } else {
            // otherwise, we will put the spaces as a literal into the sequence
            self.sequence.push(InterpolateComponent::Literal(
                &source[last_span.end..span.start],
            ));
        }

        // now we can safely push the lookup, as we already handled the space in between
        self.sequence.push(InterpolateComponent::Lookup(lookup));

        span
    }
}

impl<'source> Parse<'source> for Call<'source> {
    fn parse(parser: &mut Parser<'source>) -> ParseRes<Self> {
        parser.with_context("parsing function call", |parser| {
            let function_kind = parser.parse()?;
            let start = parser.current_token_span().start;
            parser.accept_current();
            parser.expect_token(
                Token::OpenParen,
                Some("to begin the function call arguments"),
            )?;
            parser.accept_current();
            let mut arguments = Vec::new();

            // parsing arguments
            'outer: loop {
                let (mut arg, mut arg_span) = match parser.expect_one_of_tokens(
                    &[Token::CloseParen, Token::Identifier, Token::Variable],
                    Some("argument to the function call or end the call"),
                )? {
                    Token::CloseParen => {
                        break;
                    },
                    Token::Identifier => {
                        let src = parser.current_token_source();
                        let span = parser.current_token_span().clone();
                        (Arg::str(src, span.clone()), span)
                    },
                    // TODO: check lookups? as they are known at first time
                    Token::Variable => {
                        let span = parser.current_token_span().clone();
                        (
                            match parser.parse() {
                                Ok(lookup) => Arg::lookup(lookup, span.clone()),
                                Err(e) if matches!(e.kind, ParseErrorKind::UnknownVariable) => {
                                    Arg::str(parser.current_token_source(), span.clone())
                                },
                                Err(e) => return Err(e),
                            },
                            span,
                        )
                    },
                    _ => unreachable!(),
                };
                parser.accept_current();
                loop {
                    match parser.expect_one_of_tokens(
                        &[
                            Token::CloseParen,
                            Token::Identifier,
                            Token::Variable,
                            Token::Comma,
                        ],
                        None,
                    )? {
                        Token::CloseParen => {
                            arguments.push(arg);
                            break 'outer;
                        },
                        Token::Comma => {
                            parser.accept_current();
                            break;
                        },
                        Token::Variable => {
                            let span = parser.current_token_span().clone();
                            arg_span = match parser.parse() {
                                Ok(lookup) => {
                                    arg.extend_lookup(lookup, arg_span, span, parser.source())
                                },
                                Err(e) if matches!(e.kind, ParseErrorKind::UnknownVariable) => {
                                    arg.extend_str(arg_span, span, parser.source())
                                },
                                Err(e) => return Err(e),
                            };
                            parser.accept_current();
                        },
                        Token::Identifier => {
                            let span = parser.current_token_span().clone();
                            arg_span = arg.extend_str(arg_span, span, parser.source());
                            parser.accept_current();
                        },
                        _ => unreachable!(),
                    }
                }
                arguments.push(arg);
            }
            //     arguments.push(arg);
            // }
            parser.expect_token(Token::CloseParen, Some("to end the argument list"))?;
            let end = parser.current_token_span().end;
            parser.accept_current();

            Ok(Self {
                function_kind,
                arguments,
                span: start..end,
            })
        })
    }
}
use crate::interpreter::{Cache, ErrorKind, ExprResult, Resolve};
impl Resolve for Arg<'_> {
    fn resolve(&self, cache: &mut Cache) -> Result<ExprResult, ErrorKind> {
        let mut str = String::new();
        for x in &self.sequence {
            match x {
                InterpolateComponent::Literal(lit) => str.push_str(lit),
                InterpolateComponent::Lookup(look) => {
                    str.push_str(&cache.resolve(look)?.cast_to_string()?);
                },
            }
        }
        Ok(str.into())
    }
}

use std::sync::{Arc, Mutex};
pub fn run_call(call: &Call, cache: Arc<Mutex<Cache>>) -> Result<(), ErrorKind> {
    use crate::interpreter::ArgCountMismatched;

    // note: already considered streaming the arguments instead
    // of collecting all of them, but the number of arguments is very low (1 or 2),
    // so there is no real performance hit if we evaluate all of them now.
    let args: Vec<String> = call
        .arguments
        .iter()
        // note: grabbing the cache lock on each argument separately to prevent locking
        // the cache too much time, e.g in a cycle like
        // `print(@sum=sha256 @sum=sha512 @sum=md5) print(@path)`,
        // if the first `print` grabs the cache first, it will only prevent the second `print` from
        // executing while it's calculating the SHA256 sum, then the second print will be executed
        .map(|arg| cache.lock().unwrap().resolve(arg))
        .map(|x| x.and_then(|x| x.cast_to_string().map_err(Into::into)))
        .collect::<Result<_, _>>()?;
    // drop the guard now since we're finished
    drop(cache);

    ArgCountMismatched::check(call.function_kind, args.len())?;

    // todo: error contexts in interpreter errors...
    match call.function_kind {
        Function::Print => print(args),
        Function::Create => create(&args[0], args.get(1).map(String::as_str)),
        Function::Mkdir => mkdir(args),
        Function::Delete => delete(args),
        Function::Copy => copy_file_or_dir(args[0].as_str().into(), args[1].as_str().into()),
        Function::Move => move_file(&args[0], &args[1]),
        Function::GzipCompress => gzc(&args[0], &args[1]),
        Function::GzipDecompress => gzd(&args[0], &args[1]),
        Function::Shell => shell(args),
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
    let metadata = match fs::metadata(target) {
        Ok(meta) => meta,
        Err(_) => return Ok(()),
    };
    if metadata.is_dir() {
        fs::remove_dir_all(target)
    } else {
        fs::remove_file(target)
    }
}

use std::path::PathBuf;
fn copy_file_or_dir(source: PathBuf, mut dest: PathBuf) -> Result<(), io::Error> {
    use std::fs;

    if dest.exists() && dest.is_dir() {
        dest = dest.join(
            source
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap(),
        );
    }

    if source.is_dir() {
        fs::create_dir_all(dest)
    } else {
        fs::copy(source, dest).map(|_| {})
    }
}

fn move_file(source: &str, dest: &str) -> Result<(), io::Error> {
    copy_file_or_dir(source.into(), dest.into())?;
    delete_file_or_dir(source)
}

fn gzc(source: &str, dest: &str) -> Result<(), io::Error> {
    use std::fs;

    use flate2::write::GzEncoder;
    use flate2::Compression;

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

fn shell(commands: Vec<String>) -> Result<(), io::Error> {
    use std::process::Command;
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
        complete_command.spawn()?;

        Ok(())
    })
}
