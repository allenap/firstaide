use crate::cmds;
use std::fmt;

pub use Error::*;

pub enum Error {
    CommandNotFound(String),
    BuildError(cmds::build::Error),
    StatusError(cmds::status::Error),
    CleanError(cmds::clean::Error),
    HookError(cmds::hook::Error),
    EnvError(cmds::env::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandNotFound(message) => write!(f, "command not found: {}", message),
            BuildError(err) => write!(f, "build failed: {}", err),
            StatusError(err) => write!(f, "status failed: {}", err),
            CleanError(err) => write!(f, "clean failed: {}", err),
            HookError(err) => write!(f, "hook failed: {}", err),
            EnvError(err) => write!(f, "env failed: {}", err),
        }
    }
}
