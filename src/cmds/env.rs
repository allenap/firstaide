use anyhow::{Context, Result};
use bincode;
use clap::Parser;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Serialize the environment
#[derive(Debug, Parser)]
pub struct Command {
    /// Where to dump the environment; defaults to stdout;
    #[clap(long, short)]
    out: Option<PathBuf>,
}

impl Command {
    pub fn run(&self) -> Result<u8> {
        let env: Vec<(OsString, OsString)> = env::vars_os().collect();
        match &self.out {
            None => bincode::serialize_into(io::stdout().lock(), &env)
                .context("could not encode environment")?,
            Some(out) => bincode::serialize_into(
                fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&out)
                    .context("input/output error")?,
                &env,
            )?,
        };
        Ok(0)
    }
}
