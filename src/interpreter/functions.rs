use std::fs;
use std::process;

use super::exceptions::Exceptions;
use super::utils::path::Path;
use super::utils::Sum;
use super::variables::Variables;
use super::Literal;
use super::Str;
use flate2::bufread::MultiGzDecoder;
use fs_extra::dir;
use gzp::pargz::ParGz;
use std::io::{self, prelude::*, Write};

type Args = Vec<String>;

pub trait Functions {
    // Parser returns the args of a function as a vector of vector of Literals,
    // because an argument might have Text & Variables. Each vector inside the
    // super-vector is a function argument (those separated by ","), and the
    // Literals inside those vectors must be merged into a unique Literal,
    // creating a vector of literals, being each one a function argument
    // for making something that a function can deal with more easily.
    // Then, the best is to create directly a vector of strings, because
    // we do not care anymore about the type of the literal.
    fn supervec_literals_to_args(&self, supervec: &[Vec<Literal>]) -> Args;

    // Functions definitions
    fn r#print(&self, args: Args);
    fn r#create(&self, args: Args);
    fn r#mkdir(&self, args: Args);
    fn r#delete(&self, args: Args);
    fn r#move(&self, args: Args);
    fn r#copy(&self, args: Args);
    fn r#gzc(&self, args: Args);
    fn r#gzd(&self, args: Args);
    fn r#shell(&self, args: Args);
}

impl Functions for super::Interpreter {
    // Parser returns the args of a function as a vector of vector of Literals,
    // because an argument might have Text & Variables. Each vector inside the
    // super-vector is a function argument (those separated by ","), and the
    // Literals inside those vectors must be merged into a unique Literal,
    // creating a vector of literals, being each one a function argument
    // for making something that a function can deal with more easily.
    // Then, the best is to create directly a vector of strings, because
    // we do not care anymore about the type of the literal.
    fn supervec_literals_to_args(&self, supervec: &[Vec<Literal>]) -> Args {
        supervec
            .iter()
            .map(|literals| {
                literals
                    .iter()
                    .map(|literal| self.get_var_if_any(literal).unwrap().content)
                    .collect::<Vec<String>>()
                    .join("")
            })
            .collect()
    }

    // Functions definitions
    fn r#print(&self, args: Args) {
        // mitigate printing bottleneck by locking the stdout
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        writeln!(handle, "{}", args.join("\n")).unwrap();
        handle.flush().unwrap();
    }

    fn r#create(&self, args: Args) {
        if args.len() != 2 {
            self.raise_error(
                "UNEXPECTED QUANTITY ARGUMENTS",
                "create() function is expected to take only 2 args, the file and the content"
                    .to_string(),
            );
        } else if let Err(err) = fs::write(self.trim_spaces(&args[0]), self.trim_spaces(&args[1])) {
            self.raise_error(
                "ERROR CREATING FILE",
                format!("echo {a1} > {a0}': {err}", a1 = args[1], a0 = args[0]),
            );
        }
    }

    fn r#mkdir(&self, args: Args) {
        for arg in args {
            if let Err(err) = fs::create_dir_all(self.trim_spaces(&arg)) {
                self.raise_error(
                    "ERROR WHILE CREATING DIRECTORY",
                    format!("An error occurred:\n'mkdir --parent {arg}': {err}"),
                );
            };
        }
    }

    fn r#delete(&self, args: Args) {
        for arg in args {
            let is_file: Result<bool, ()> = self.is_file(&self.trim_spaces(&arg));

            // maybe file was delete in previous cycles.
            // if the path just was wrong, its not my fault,
            // user's fault. just re-run voila but reading what
            // you scripted before launching a tool that can be
            // (and in fact is) potentially destructive, im not
            // doing a hashmap of stuff deleted and then a checker,
            // enough overhead & bottlenecks with the async hell
            // of the cycles & the interpreter
            if is_file.is_ok() {
                // there is not a way of deleting something without
                // without caring if its a directory or a file, so
                // we have to get its type and call whatever needed
                if is_file.unwrap() {
                    if let Err(err) = fs::remove_file(self.trim_spaces(&arg)) {
                        self.raise_error(
                            "ERROR WHILE DELETING FILE",
                            format!("An error occurred:\n'rm -f {arg}': {err}"),
                        );
                    };
                } else if let Err(err) = fs::remove_dir_all(self.trim_spaces(&arg)) {
                    self.raise_error(
                        "ERROR WHILE DELETING DIRECTORY",
                        format!("An error occurred:\n'rm -rf {arg}': {err}"),
                    );
                };
            }
        }
    }

    fn r#move(&self, args: Args) {
        // moving is literally copying and then deleting,
        // so i prefer to call their respective functions
        // instead of mashing them up
        self.r#copy(args.clone());
        self.r#delete(vec![args[0].clone()]);
    }

    fn r#copy(&self, args: Args) {
        // arguments must be exactly 2
        if args.len() != 2 {
            self.raise_error(
                "UNEXPECTED QUANTITY ARGUMENTS",
                "copy() & move() functions are expected to take only 2 args, the origin and the destination".to_string(),
            );
        } else {
            let is_file: Result<bool, ()> = self.is_file(&self.trim_spaces(&args[0]));

            // maybe file was delete in previous cycles.
            // if the path just was wrong, its not my fault,
            // user's fault. just re-run voila but reading what
            // you scripted before launching a tool that can be
            // (and in fact is) potentially destructive, im not
            // doing a hashmap of stuff deleted and then a checker,
            // enough overhead & bottlenecks with the async hell
            // of the cycles & the interpreter
            if let Ok(result) = is_file {
                if result {
                    if let Err(err) =
                        fs::copy(self.trim_spaces(&args[0]), self.trim_spaces(&args[1]))
                    {
                        self.raise_error(
                            "ERROR WHILE COPYING FILE",
                            format!(
                                "An error occurred:\n'cp {a0} {a1}': {err}",
                                a0 = args[0],
                                a1 = args[1]
                            ),
                        );
                    };
                } else if let Err(err) = dir::copy(
                    self.trim_spaces(&args[0]),
                    self.trim_spaces(&args[1]),
                    &dir::CopyOptions::new(),
                ) {
                    self.raise_error(
                        "ERROR WHILE COPYING DIR",
                        format!(
                            "An error occurred:\n'cp -r --parents --copy-contents {a0} {a1}': {err}",
                            a0 = args[0], a1 = args[1]
                        ),
                    );
                };
            }
        }
    }
    fn r#gzc(&self, args: Args) {
        // check arguments
        if args.len() != 2 {
            self.raise_error(
                "NOT ENOUGH ARGUMENTS",
                format!("Expected 2 arguments, found {len}", len = args.len()),
            )
        }

        // get bytes of file to compress
        // and store them in a buffer
        let content = self.read_bytes_of_file(&self.trim_spaces(&args[0]));

        // init a compressed bytes writer
        // with destination to the file
        let mut compressor =
            ParGz::builder(fs::File::create(self.trim_spaces(&args[1])).unwrap()).build();

        // send bytes to compressor and write them
        compressor.write_all(&content).unwrap();

        // drop safely compressor
        compressor.finish().unwrap();
    }
    fn gzd(&self, args: Args) {
        // check arguments
        if args.len() != 2 {
            self.raise_error(
                "NOT ENOUGH ARGUMENTS",
                format!("Expected 2 arguments, found {len}", len = args.len()),
            )
        }

        // get compressed file
        let compressed_file = fs::File::open(self.trim_spaces(&args[0])).unwrap();

        // init a decompressor over
        // a readbuffer of the file
        let mut decompressor = MultiGzDecoder::new(io::BufReader::new(compressed_file));

        // init a buffer for
        // decompressed bytes
        let mut buffer: Vec<u8> = Vec::new();

        // dump decompressed bytes
        // into the buffer
        decompressor.read_to_end(&mut buffer).unwrap();

        // dump buffer into the file
        fs::write(self.trim_spaces(&args[1]), &buffer).unwrap();
    }
    fn r#shell(&self, args: Args) {
        for arg in args {
            // Determine operating system and launch associated process
            #[cfg(windows)]
            {
                // Windows' shell is powershell/pwsh
                if let Err(err) = process::Command::new("powershell")
                    .arg("-Command")
                    .arg(self.trim_spaces(&arg))
                    .output()
                // spawn() does not await the cmd to finish, output() does
                {
                    self.raise_error(
                        "ERROR WHILE EXECUTING SHELL",
                        format!("An error occurred:\n'powershell -Command {arg}': {err}"),
                    );
                }
            }
            #[cfg(unix)]
            {
                // unix' shell is the bourne shell, aka sh
                if let Err(err) = process::Command::new("sh")
                    .arg("-c")
                    .arg(self.trim_spaces(&arg))
                    .output()
                // spawn() does not await the cmd to finish, output() does
                {
                    self.raise_error(
                        "ERROR WHILE EXECUTING SHELL",
                        format!("An error occurred:\n'sh -c {arg}': {err}"),
                    );
                }
            }
            #[cfg(not(any(unix, windows)))]
            {
                self.raise_error(
                    "UNSUPPORTED PLATFORM",
                    "Voila is only supported on Windows & Unix-like systems. That is mostly because this function and some variables like `ownerID`.\n                         I won't be limiting you to just use Voila on those OSs, but you will be using it at your own risk.\n                         I'm not planning to support more OSs in the short term, but any contribution is welcome!".to_string(),
                )
            }
        }
    }
}
