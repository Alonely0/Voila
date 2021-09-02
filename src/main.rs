#![forbid(unsafe_code)] // unsafe code makes ferris get nervous
#![feature(format_args_capture)]
#![feature(decl_macro)]
#![feature(box_syntax)]
#![allow(clippy::upper_case_acronyms)]

use voila::macros::println_on_debug;

mod cli;

fn main() {
    let cli_args: cli::Cli = cli::get_cli_args();

    println_on_debug!(
        "\nVoila v{}, DEBUG ENABLED\n
Debug is always enabled on development versions.
Development versions aren't stable at all, use them at your own risk.
On non-development versions, enable debug only if you are going to report an error,
the flood of debug logs may hide warnings. Debug messages may contain sensitive information,
like the name of the files and other data related to variables.
For more information see the README.\n
------------------------------  VOILA EXECUTION STARTED  ------------------------------",
        env!("CARGO_PKG_VERSION")
    );

    if let Err(ref e) = voila::run(&cli_args.source, cli_args.dir, cli_args.recursive) {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    println_on_debug!(
        "\n------------------------------  VOILA EXECUTION ENDED  --------------------------------"
    )
}
