use path_absolutize::Absolutize;
use serde::Deserialize;
use std::env;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml;

type Result = std::result::Result<Config, Error>;

pub enum Error {
    Io(io::Error),
    NotFound(PathBuf),
    Invalid(toml::de::Error),
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            Io(err) => write!(f, "input/output error: {}", err),
            NotFound(path) => write!(f, "configuration file not found; started from {:?}", path),
            Invalid(err) => write!(f, "configuration file not valid: {}", err),
            Other(message) => write!(f, "could not use configuration: {}", message),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Self {
        Error::Invalid(error)
    }
}

#[derive(Debug)]
pub struct Config {
    pub build_dir: PathBuf,
    pub cache_dir: PathBuf,
    build_exe: PathBuf,
    watch_exe: PathBuf,
    self_exe: PathBuf,
    parent_dir: PathBuf,
    pub messages: ConfigMessages,
}

#[derive(Debug, Deserialize)]
struct ConfigData {
    cache_dir: PathBuf,
    build_exe: PathBuf,
    watch_exe: PathBuf,
    #[serde(default)]
    parent_dir: ParentDir,
    #[serde(default)]
    messages: ConfigMessages,
}

#[derive(Debug, Deserialize)]
pub struct ParentDir(pub PathBuf);

impl Default for ParentDir {
    fn default() -> Self {
        Self("..".into())
    }
}

impl AsRef<Path> for ParentDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfigMessages {
    pub getting_started: String,
}

impl Default for ConfigMessages {
    fn default() -> Self {
        Self {
            getting_started: "aide --help".into(),
        }
    }
}

impl Config {
    pub fn load<T: Into<PathBuf>>(dir: Option<T>) -> Result {
        let dir = match dir {
            Some(d) => d.into().absolutize()?,
            None => PathBuf::new().absolutize()?,
        };

        // Find and load a configuration file.
        let config_file: PathBuf = dir
            .ancestors()
            .map(|path| path.join(".firstaide.toml"))
            .find(|path| path.is_file())
            .ok_or(Error::NotFound(dir))?;
        let config_bytes: Vec<u8> = fs::read(&config_file)?;
        let config_data: ConfigData = toml::from_slice(&config_bytes)?;

        // All paths are resolved relative to the directory where we found the
        // configuration file.
        let datum_dir = config_file.parent().ok_or(Error::Other(
            "could not get directory of configuration file".into(),
        ))?;

        Ok(Config {
            build_dir: datum_dir.to_path_buf(),
            cache_dir: datum_dir.join(config_data.cache_dir).absolutize()?,
            build_exe: datum_dir.join(config_data.build_exe).absolutize()?,
            watch_exe: datum_dir.join(config_data.watch_exe).absolutize()?,
            parent_dir: datum_dir.join(config_data.parent_dir).absolutize()?,
            self_exe: env::current_exe()?,
            messages: config_data.messages,
        })
    }

    pub fn command_to_allow_direnv(&self) -> Command {
        let mut command = self.command_direnv();
        command.arg("allow").arg("--").arg(&self.build_dir);
        command
    }

    /// Capture the environment from outside of the Nix environment.
    ///
    /// We also ask direnv to load as if from the *parent* of the build
    /// directory. We do want to include all parent `.envrc`s, but we do NOT
    /// want the `.envrc` in the build directory itself from influencing the
    /// build: *it* is a consumer of *this*, not the other way around or some
    /// weird ouroboros of both.
    pub fn command_to_dump_env_outside<T: Into<PathBuf>>(&self, out: T) -> Command {
        let mut command = self.command_direnv();
        command
            .current_dir(&self.build_dir)
            .arg("exec")
            .arg(&self.parent_dir)
            .arg(&self.self_exe)
            .arg("env")
            .arg("--out")
            .arg(out.into());
        command
    }

    /// Capture the environment from inside the Nix environment.
    ///
    /// We invoke the build with exactly the environment captured from the
    /// outside, which should include parent `.envrc`s.
    pub fn command_to_dump_env_inside<T: Into<PathBuf>>(
        &self,
        out: T,
        env: &[crate::env::Item],
    ) -> Command {
        let mut command = Command::new(&self.build_exe);
        command
            .current_dir(&self.build_dir)
            .arg(&self.self_exe)
            .arg("env")
            .arg("--out")
            .arg(out.into())
            .env_clear()
            .envs(env.iter().cloned());
        command
    }

    pub fn watch_files(&self) -> io::Result<Vec<PathBuf>> {
        let mut command = Command::new(&self.watch_exe);
        command.current_dir(&self.build_dir);
        let output = command.output()?;
        let names = output
            .stdout
            .split(|&byte| byte == 0)
            .filter(|name| !name.is_empty());
        let paths = names.map(|name| OsStr::from_bytes(name));
        Ok(paths.map(|path| self.abspath(path)).collect())
    }

    /// Return an absolute path, resolved relative to `self.build_dir`.
    fn abspath<T: AsRef<Path>>(&self, path: T) -> PathBuf {
        let p = path.as_ref();
        if p.is_relative() {
            self.build_dir.join(p)
        } else {
            p.to_path_buf()
        }
    }

    pub fn command_direnv(&self) -> Command {
        Command::new("direnv")
    }

    pub fn cache_file(&self) -> PathBuf {
        self.cache_dir.join("cache")
    }
}
