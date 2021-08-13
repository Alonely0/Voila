use std::path::PathBuf;
use structopt::StructOpt;

pub fn get_cli_args() -> Cli {
    Cli::from_args()
}

#[derive(StructOpt)]
#[structopt(
    author = "Guillem Jara <4lon3ly0@tutanota.com>",
    about = "Licensed under the MIT License\n
Download at https://github.com/alonely0/voila/releases\n
Source code at https://github.com/alonely0/voila\n\n
Voila is a CLI tool for operating with files and directories in massive amounts in a fast & reliable way.",
    version_short = "v"
)]
pub struct Cli {
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
