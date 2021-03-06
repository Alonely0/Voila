use std::path::PathBuf;
use structopt::StructOpt;

pub fn get_cli_args() -> Cli {
    Cli::from_args()
}

#[derive(StructOpt)]
#[structopt(
    author = "Guillem Jara <4lon3ly0@tutanota.com>",
    about = r#"
Voila is a CLI tool for operating with files and directories in massive amounts in a fast & reliable way.

Licensed under the MIT License
Download at https://github.com/alonely0/voila/releases
Source code at https://github.com/alonely0/voila"#,
    version_short = "v"
)]
pub struct Cli {
    #[structopt(
        long,
        help = "Compile Voila script into a static binary embedding all runtime"
    )]
    pub compile: bool,
    #[structopt(
        short,
        long,
        help = "Operate recursively inside the directory provided"
    )]
    pub recursive: bool,
    #[structopt(
        name = "FOLDER",
        help = "/something/path/to/folder or ./path/to/folder"
    )]
    pub dir: PathBuf,
    #[structopt(
        name = "SOURCE",
        help = "for syntax & examples see the documentation, you can find it in the repository."
    )]
    pub source: String,
}
