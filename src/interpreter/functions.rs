use std::fs;
use std::process;

use fs_extra::dir;

use super::exceptions::Exceptions;
use super::utils::path::Path;
use super::variables::Variables;
use super::Literal;
use super::Str;
use std::io::{self, Write};

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
    fn supervec_literals_to_args(&self, supervec: Vec<Vec<Literal>>) -> Args;

    // Functions definitions
    fn r#print(&self, args: &Args);
    fn r#create(&self, args: &Args);
    fn r#mkdir(&self, args: &Args);
    fn r#delete(&self, args: &Args);
    fn r#move(&self, args: &Args);
    fn r#copy(&self, args: &Args);
    fn r#shell(&self, args: &Args);
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
    fn supervec_literals_to_args(&self, supervec: Vec<Vec<Literal>>) -> Vec<String> {
        let mut final_args: Args = vec![];
        for vec_of_literals in supervec {
            let mut literals_str = String::from("");
            for literal in vec_of_literals {
                let str: String = self.get_var_if_any(&literal).unwrap().content.to_owned();

                literals_str = format!("{literals_str}{str}")
            }
            final_args.push(literals_str.to_owned());
        }

        final_args
    }

    // Functions definitions
    fn r#print(&self, args: &Args) {
        // mitigate printing bottleneck by locking the stdout
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        writeln!(handle, "{}", args.join("\n")).unwrap();
        handle.flush().unwrap();
    }

    fn r#create(&self, args: &Args) {
        if args.len() != 2 {
            self.raise_error(
                "UNEXPECTED QUANTITY ARGUMENTS",
                "create() function is expected to take only 2 args, the file and the content"
                    .to_string(),
            );
        } else {
            match fs::write(self.trim_spaces(&args[0]), self.trim_spaces(&args[1])) {
                Err(err) => self.raise_error(
                    "ERROR CREATING FILE",
                    format!("echo {} > {}': {err}", args[1], args[0]),
                ),
                _ => {}
            }
        }
    }

    fn r#mkdir(&self, args: &Args) {
        for arg in args {
            match fs::create_dir_all(self.trim_spaces(&arg)) {
                Err(err) => self.raise_error(
                    "ERROR WHILE CREATING DIRECTORY",
                    format!("An error occurred:\n'mkdir --parent {arg}': {err}"),
                ),
                _ => {}
            };
        }
    }

    fn r#delete(&self, args: &Args) {
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
            if let Ok(_) = is_file {
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
                } else {
                    if let Err(err) = fs::remove_dir_all(self.trim_spaces(&arg)) {
                        self.raise_error(
                            "ERROR WHILE DELETING DIRECTORY",
                            format!("An error occurred:\n'rm -rf {arg}': {err}"),
                        );
                    };
                }
            }
        }
    }

    fn r#move(&self, args: &Args) {
        // moving is literally copying and then deleting,
        // so i prefer to call their respective functions
        // instead of mashing them up
        self.r#copy(&args);
        self.r#delete(&vec![args[0].clone()]);
    }

    fn r#copy(&self, args: &Args) {
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
            if let Ok(_) = is_file {
                if is_file.unwrap() {
                    if let Err(err) =
                        fs::copy(self.trim_spaces(&args[0]), self.trim_spaces(&args[1]))
                    {
                        self.raise_error(
                            "ERROR WHILE COPYING FILE",
                            format!("An error occurred:\n'cp {} {}': {err}", args[0], args[1]),
                        );
                    };
                } else {
                    if let Err(err) = dir::copy(
                        self.trim_spaces(&args[0]),
                        self.trim_spaces(&args[1]),
                        &dir::CopyOptions::new(),
                    ) {
                        self.raise_error(
                            "ERROR WHILE COPYING DIR",
                            format!("An error occurred:\n'cp -r --parents --copy-contents {} {}': {err}", args[0], args[1]),
                        );
                    };
                }
            }
        }
    }
    fn r#shell(&self, args: &Args) {
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
                        format!("An error occurred:\n'powershell -Command {}': {err}", arg),
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
                        format!("An error occurred:\n'sh -c {}': {err}", arg),
                    );
                }
            }
            #[cfg(not(any(unix, windows)))]
            {
                self.raise_error(
                    "UNSUPPORTED PLATFORM",
                    "Voila is only supported on Windows & Unix-like systems".to_string(),
                )
            }
        }
    }
}
