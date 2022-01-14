use crate::config;
use std::fs;
use std::io;
use thiserror::Error;

pub const NAME: &str = "clean";

type Result = std::result::Result<u8, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] config::Error),

    #[error("input/output error: {0}")]
    Io(#[from] io::Error),
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
