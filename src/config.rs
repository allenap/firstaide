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
            .current_dir(&self.dir)
            .arg("exec")
            .arg(self.abspath(".."))
            .arg(&self.exe)
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
        let command_path = (self.dir.join(".firstaide.env"))
            .canonicalize()
            .expect(concat!(
                "could not find/resolve .firstaide.env; please create a script ",
                "called .firstaide.env that will execute its arguments in the ",
                "target environment",
            ));
        let mut command = Command::new(command_path);
        command
            .current_dir(&self.dir)
            .arg(&self.exe)
            .arg("env")
            .arg("--out")
            .arg(out.into())
            .env_clear()
            .envs(env.iter().cloned());
        command
    }

    pub fn watch_files(&self) -> io::Result<Vec<PathBuf>> {
        let command_path = (self.dir.join(".firstaide.watch"))
            .canonicalize()
            .expect(concat!(
                "could not find/resolve .firstaide.watch; please create a script ",
                "called .firstaide.watch that emits a NUL-delimited list of files ",
                "to watch for updates",
            ));
        let mut command = Command::new(command_path);
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
        self.dir
            .join(".firstaide.dir")
            .read_link()
            .expect(concat!(
                "could not determine cache directory; please symlink .firstaide.dir ",
                "to it (can be dangling symlink; the directory pointed to will be ",
                "created)",
            ))
            .absolutize()
            .expect("could not calculate absolute path to cache directory")
    }
}
