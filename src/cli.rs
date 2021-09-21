use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "super-octo-engine", about = "Super CSV engine")]
pub struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

impl Opt {
    pub fn get_input_file() -> PathBuf {
        Opt::from_args().input
    }
}
