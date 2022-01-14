use bincode;
use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;

pub type Env = Vec<(OsString, OsString)>;

pub const NAME: &str = "env";

type Result = std::result::Result<u8, Error>;

pub enum Error {
    Io(io::Error),
    Encode(bincode::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            Io(err) => write!(f, "input/output error: {}", err),
            Encode(err) => write!(f, "could not encode environment: {}", err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<bincode::Error> for Error {
    fn from(error: bincode::Error) -> Self {
        Error::Encode(error)
    }
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
