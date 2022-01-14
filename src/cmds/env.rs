use bincode;
use clap::Parser;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

pub type Env = Vec<(OsString, OsString)>;

type Result = std::result::Result<u8, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("input/output error: {0}")]
    Io(#[from] io::Error),

    #[error("could not encode environment: {0}")]
    Encode(#[from] bincode::Error),
}

/// Serialize the environment
#[derive(Debug, Parser)]
pub struct Command {
    /// Where to dump the environment; defaults to stdout;
    #[clap(long, short)]
    out: Option<PathBuf>,
}

impl Command {
    pub fn run(&self) -> Result {
        let env: Env = env::vars_os().collect();
        match &self.out {
            None => bincode::serialize_into(io::stdout().lock(), &env)?,
            Some(out) => bincode::serialize_into(
                fs::OpenOptions::new().write(true).create(true).open(&out)?,
                &env,
            )?,
        };
        Ok(0)
    }
}
