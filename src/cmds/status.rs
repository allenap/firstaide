use crate::cache;
use crate::config;
use crate::status::EnvironmentStatus;
use crate::sums;
use std::fmt;
use std::io::{self, Write};

pub const NAME: &str = "status";

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
        .about("Reports the status of the development environment")
        .long_about(concat!(
            "Reports the status of the development environment.\n",
            "- Exits 0 when the environment is up to date.\n",
            "- Exits 1 when the environment is stale.\n",
            "- Exits 2 when the environment is unbuilt, or when an error occurs.",
        ))
        .arg(
            clap::Arg::new("dir")
                .value_name("DIR")
                .help("The directory in which to build"),
        )
}

pub fn run(args: &clap::ArgMatches) -> Result {
    let config = config::Config::load(args.value_of_os("dir"))?;
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    let sums_now = sums::Checksums::from(&config.watch_files()?)?;
    let cache_file = config.cache_file(&sums_now);
    let cache_file_fallback = config.cache_file_most_recent();

    let status = match cache::Cache::load_with_fallback(&cache_file, &cache_file_fallback) {
        Ok(cache) => {
            if sums::equal(&sums_now, &cache.sums) {
                EnvironmentStatus::Okay
            } else {
                EnvironmentStatus::Stale
            }
        }
        Err(_) => EnvironmentStatus::Unknown,
    };

    writeln!(&mut handle, "{}", status)?;
    Ok(status.code())
}
