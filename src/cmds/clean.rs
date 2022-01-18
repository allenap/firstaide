use crate::config;
use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

/// Cleans the development environment
#[derive(Debug, Parser)]
pub struct Command {
    /// The directory to clean
    dir: Option<PathBuf>,
}

impl Command {
    pub fn run(&self) -> Result<u8> {
        let config = config::Config::load(self.dir.as_ref()).context("could not load config")?;

        // Just delete the cache directory.
        fs::remove_dir_all(&config.cache_dir).context("could not remove cache directory")?;

        Ok(0)
    }
}
