use path_absolutize::Absolutize;
use std::env;
use std::ffi::OsStr;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Config {
    dir: PathBuf,
    exe: PathBuf,
}

/*

A future config file (`.firstaide.toml`) might contain:

- Command to capture outside environment, with default, e.g. `direnv exec / firstaide env`
- Command to capture inside environment, with default, e.g. `nix/exec firstaide env`
- Where to store the cache file, with default, e.g. `.firstaide.cache`.
- Command to yield files to watch, with default, e.g. `.firstaide.watch`.

*/

impl Config {
    pub fn new<T: Into<PathBuf>>(dir: Option<T>) -> Self {
        let dir = match dir {
            Some(d) => d.into(),
            None => PathBuf::new(),
        };
        Config {
            dir: dir
                .absolutize()
                .expect("could not calculate absolute path to environment directory"),
            exe: env::current_exe().expect("could not obtain path to this executable"),
        }
    }

    pub fn command_to_allow_direnv(&self) -> Command {
        let mut command = self.command_direnv();
        command.arg("allow").arg("--").arg(&self.dir);
        command
    }

    pub fn command_to_dump_env_outside<T: Into<PathBuf>>(&self, out: T) -> Command {
        let mut command = self.command_direnv();
        command
            .arg("exec")
            .arg("/")
            .arg(&self.exe)
            .arg("env")
            .arg("--out")
            .arg(out.into());
        command
    }

    pub fn command_to_dump_env_inside<T: Into<PathBuf>>(
        &self,
        out: T,
        env: &[crate::env::Item],
    ) -> Command {
        let mut command = Command::new("nix/exec");
        command
            .current_dir(&self.dir)
            .arg("shell.nix")
            .arg(&self.exe)
            .arg("env")
            .arg("--out")
            .arg(out.into())
            .env_clear()
            .envs(env.iter().cloned());
        command
    }

    pub fn watch_files(&self) -> io::Result<Vec<PathBuf>> {
        let mut command = Command::new("iac/development/direnv-dependencies");
        command.current_dir(&self.dir);
        let output = command.output()?;
        let names = output
            .stdout
            .split(|&byte| byte == 0)
            .filter(|name| !name.is_empty());
        let paths = names.map(|name| OsStr::from_bytes(name));
        Ok(paths.map(|path| self.abspath(path)).collect())
    }

    /// Return an absolute path, resolved relative to `self.dir`.
    fn abspath<T: AsRef<Path>>(&self, path: T) -> PathBuf {
        let p = path.as_ref();
        if p.is_relative() {
            self.dir.join(p)
        } else {
            p.to_path_buf()
        }
    }

    pub fn command_direnv(&self) -> Command {
        Command::new("direnv")
    }

    pub fn cache_file(&self) -> PathBuf {
        self.cache_dir().join("cache")
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.dir.join("_build").join("firstaide")
    }
}
