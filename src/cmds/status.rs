use crate::cache;
use crate::config;
use crate::status::EnvironmentStatus;
use crate::sums;
use clap::Parser;
use std::io::{self, Write};
use std::path::PathBuf;
use thiserror::Error;

type Result = std::result::Result<u8, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] config::Error),

    #[error("input/output error: {0}")]
    Io(#[from] io::Error),
}

/// Reports the status of the development environment
///
/// Exits 0 when the environment is up to date, 1 when the environment is stale,
/// and 2 when the environment is unbuilt, or when an error occurs.
#[derive(Debug, Parser)]
pub struct Command {
    /// The directory in which to build
    dir: Option<PathBuf>,
}

impl Command {
    pub fn run(&self) -> Result {
        let config = config::Config::load(self.dir.as_ref())?;
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
}
