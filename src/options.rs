use clap_verbosity_flag::Verbosity;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "transform-5e-tools-to-chazz",
    about = "A command line tool for transforming data from 5e.tools to a format Chazz understands."
)]
pub struct Options {
    #[structopt(flatten)]
    pub verbosity: Verbosity,

    /// Input folder
    #[structopt(parse(from_os_str))]
    pub input: PathBuf,

    /// JSON key
    pub key: String,

    /// Output location
    pub output: PathBuf,
}
