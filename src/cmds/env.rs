use bincode;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use thiserror::Error;

pub type Env = Vec<(OsString, OsString)>;

pub const NAME: &str = "env";

type Result = std::result::Result<u8, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("input/output error: {0}")]
    Io(#[from] io::Error),

    #[error("could not encode environment: {0}")]
    Encode(#[from] bincode::Error),
}

pub fn argspec<'a>() -> clap::App<'a> {
    clap::App::new(NAME).about("Serialize the environment").arg(
        clap::Arg::new("out")
            .short('o')
            .long("out")
            .value_name("OUT")
            .help("Where to dump the environment; defaults to stdout"),
    )
}

pub fn run(args: &clap::ArgMatches) -> Result {
    let env: Env = env::vars_os().collect();
    match args.value_of_os("out") {
        None => bincode::serialize_into(io::stdout().lock(), &env)?,
        Some(out) => bincode::serialize_into(
            fs::OpenOptions::new().write(true).create(true).open(&out)?,
            &env,
        )?,
    };
    Ok(0)
}
