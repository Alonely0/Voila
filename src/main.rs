#![forbid(unsafe_code)] // unsafe code makes ferris get nervous
#![feature(format_args_capture)]
#![feature(decl_macro)]
#![allow(clippy::upper_case_acronyms)]

use futures::executor::block_on;
use macros::println_on_debug;

mod cli;
mod interpreter;
mod lexer;
mod macros;
mod parser;

fn main() {
    let cli_args: cli::Cli = cli::get_cli_args();

    println_on_debug!(
        "\nVoila v{ver}, DEBUG ENABLED\n
Debug is always enabled on development versions.
Development versions aren't stable at all, use them at your own risk.
On non-development versions, enable debug only if you are going to report an error,
the flood of debug logs may hide warnings. Debug messages may contain sensitive information,
like the name of the files and other data related to variables.
For more information see the README.\n
------------------------------  VOILA EXECUTION STARTED  ------------------------------",
        ver = env!("CARGO_PKG_VERSION")
    );

    let tokens: Vec<lexer::Token> = lexer::lex(&cli_args.source); // lex source
    let ast: parser::ast::AST = parser::parse(tokens); // parse tokens
    block_on(interpreter::run(ast, cli_args.dir, cli_args.recursive)); // wait interpreter to finish

    println_on_debug!(
        "\n------------------------------  VOILA EXECUTION ENDED  --------------------------------"
    )
}
