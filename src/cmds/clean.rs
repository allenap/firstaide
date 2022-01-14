use crate::config;
use clap::Parser;
use std::fs;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

type Result = std::result::Result<u8, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] config::Error),

    #[error("input/output error: {0}")]
    Io(#[from] io::Error),
}

/// Cleans the development environment
#[derive(Debug, Parser)]
pub struct Command {
    /// The directory to clean
    dir: Option<PathBuf>,
}

impl Command {
    pub fn run(&self) -> Result {
        let config = config::Config::load(self.dir.as_ref())?;

        // Just delete the cache directory.
        fs::remove_dir_all(&config.cache_dir)?;

        Ok(0)
    }
}
