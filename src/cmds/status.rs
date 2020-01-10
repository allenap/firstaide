use crate::cache;
use crate::config;
use crate::sums;
use std::fmt;
use std::io::{self, Write};

pub const NAME: &str = "status";

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
        .about("Hooks the development environment; source the output from .envrc")
        .arg(
            clap::Arg::with_name("dir")
                .value_name("DIR")
                .help("The directory in which to build"),
        )
}

pub fn run(args: &clap::ArgMatches) -> Result {
    let config = config::Config::new(args.value_of_os("dir"));
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    match cache::Cache::load(config.cache_file()) {
        Ok(cache) => {
            let sums_now = sums::Checksums::from(&config.watch_files()?)?;
            if sums::equal(&sums_now, &cache.sums) {
                writeln!(&mut handle, "Environment is up-to-date!")?;
                Ok(0)
            } else {
                writeln!(&mut handle, "Environment is STALE!")?;
                Ok(1)
            }
        }
        Err(_) => {
            writeln!(&mut handle, "Environment not built or otherwise broken!")?;
            Ok(2)
        }
    }
}
