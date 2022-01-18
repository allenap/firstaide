use crate::cmds;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("build failed: {0}")]
    Build(anyhow::Error),

    #[error("status failed: {0}")]
    Status(anyhow::Error),

    #[error("clean failed: {0}")]
    Clean(anyhow::Error),

    #[error("hook failed: {0}")]
    Hook(anyhow::Error),

    #[error("env failed: {0}")]
    Env(anyhow::Error),
}
