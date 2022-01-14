use bincode;
use crypto_hash::{hex_digest, Algorithm};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Checksums(Vec<Checksum>);

impl Checksums {
    pub fn from<T>(filenames: &[T]) -> io::Result<Self>
    where
        T: AsRef<Path>,
    {
        let mut sums = Vec::new();
        for filename in filenames {
            let sum = Checksum::from(filename)?;
            sums.push(sum);
        }
        Ok(Self(sums))
    }

    pub fn sig(&self) -> String {
        // Default bincode config is unlimited so should not error, hence
        // unwrapping is safe.
        hex_digest(Algorithm::SHA1, &bincode::serialize(self).unwrap())
    }
}

impl IntoIterator for Checksums {
    type Item = Checksum;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Checksum {
    Found(PathBuf, Sha1),
    NotFound(PathBuf),
}

impl Checksum {
    pub fn from<T>(filename: T) -> io::Result<Self>
    where
        T: AsRef<Path>,
    {
        let path = filename.as_ref().to_path_buf();
        match Sha1::from(&filename) {
            Ok(sha1) => Ok(Checksum::Found(path, sha1)),
            Err(ref err) if err.kind() == io::ErrorKind::NotFound => Ok(Checksum::NotFound(path)),
            Err(err) => Err(err),
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Checksum::Found(path, _) => path,
            Checksum::NotFound(path) => path,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Sha1(pub String);

impl Sha1 {
    pub fn from<T>(filename: T) -> io::Result<Self>
    where
        T: AsRef<Path>,
    {
        Ok(Self(hex_digest(Algorithm::SHA1, &fs::read(filename)?)))
    }
}

pub fn equal(a: &Checksums, b: &Checksums) -> bool {
    a.0.iter().eq(b.0.iter())
}
