use crate::cmds;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("build failed: {0}")]
    Build(cmds::build::Error),

    #[error("status failed: {0}")]
    Status(cmds::status::Error),

    #[error("clean failed: {0}")]
    Clean(cmds::clean::Error),

    #[error("hook failed: {0}")]
    Hook(cmds::hook::Error),

    #[error("env failed: {0}")]
    Env(cmds::env::Error),
}
