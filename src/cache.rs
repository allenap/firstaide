use crate::env;
use crate::sums;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Cache {
    pub diff: env::Diff,
    pub sums: sums::Checksums,
}

impl Cache {
    pub fn load<T: AsRef<Path>>(filename: T) -> bincode::Result<Self> {
        let data = fs::read(filename)?;
        bincode::deserialize(&data)
    }

    pub fn load_with_fallback<T: AsRef<Path>>(filename: T, fallback: T) -> bincode::Result<Self> {
        Self::load(filename).or_else(|_err| {
            log::debug!("Loading fallback, {}", fallback.as_ref().display());
            Self::load(fallback)
        })
    }

    pub fn save<T: AsRef<Path>>(&self, filename: T) -> bincode::Result<()> {
        Ok(fs::write(filename, bincode::serialize(self)?)?)
    }
}
