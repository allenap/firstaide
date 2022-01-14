use crate::cmds;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("command not found: {0}")]
    CommandNotFound(String),

    #[error("build failed: {0}")]
    BuildError(cmds::build::Error),

    #[error("status failed: {0}")]
    StatusError(cmds::status::Error),

    #[error("clean failed: {0}")]
    CleanError(cmds::clean::Error),

    #[error("hook failed: {0}")]
    HookError(cmds::hook::Error),

    #[error("env failed: {0}")]
    EnvError(cmds::env::Error),
}
