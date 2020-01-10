use crate::config;
use std::fmt;
use std::fs;
use std::io;

pub const NAME: &str = "clean";

type Result = std::result::Result<u8, Error>;

pub enum Error {
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            Io(err) => write!(f, "input/output error: {}", err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

pub fn argspec<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name(NAME)
        .about("Cleans the development environment")
        .arg(
            clap::Arg::with_name("dir")
                .value_name("DIR")
                .help("The directory to clean"),
        )
}

pub fn run(args: &clap::ArgMatches) -> Result {
    let config = config::Config::new(args.value_of_os("dir"));

    // Just delete the cache directory.
    fs::remove_dir_all(&config.cache_dir)?;

    Ok(0)
}
