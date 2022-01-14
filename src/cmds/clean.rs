use crate::config;
use std::fmt;
use std::fs;
use std::io;

pub const NAME: &str = "clean";

type Result = std::result::Result<u8, Error>;

pub enum Error {
    Config(config::Error),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            Config(err) => write!(f, "{}", err),
            Io(err) => write!(f, "input/output error: {}", err),
        }
    }
}

impl From<config::Error> for Error {
    fn from(error: config::Error) -> Self {
        Error::Config(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

pub fn argspec<'a>() -> clap::App<'a> {
    clap::App::new(NAME)
        .about("Cleans the development environment")
        .arg(
            clap::Arg::new("dir")
                .value_name("DIR")
                .help("The directory to clean"),
        )
}

pub fn run(args: &clap::ArgMatches) -> Result {
    let config = config::Config::load(args.value_of_os("dir"))?;

    // Just delete the cache directory.
    fs::remove_dir_all(&config.cache_dir)?;

    Ok(0)
}
