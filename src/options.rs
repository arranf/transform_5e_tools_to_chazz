use clap_verbosity_flag::Verbosity;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "ssh-me-in",
    about = "A command line tool for adding your IP as a SSH rule to EC2 AWS security groups."
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
