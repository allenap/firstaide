use crate::cache;
use crate::config;
use crate::env;
use crate::status::EnvironmentStatus;
use crate::sums;
use bstr::ByteSlice;
use shell_quote::bash;
use std::env::vars_os;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use tempfile;

pub const NAME: &str = "hook";

type Result = std::result::Result<u8, Error>;

pub enum Error {
    Config(config::Error),
    Io(io::Error),
    EnvOutsideCapture,
    EnvOutsideDecode(bincode::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            Config(err) => write!(f, "{}", err),
            Io(err) => write!(f, "input/output error: {}", err),
            EnvOutsideCapture => write!(f, "could not capture outside environment"),
            EnvOutsideDecode(err) => write!(f, "problem decoding outside environment: {}", err),
        }
    }
}

impl From<config::Error> for Error {
    fn from(error: config::Error) -> Self {
        Error::Config(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

pub fn argspec<'a>() -> clap::App<'a> {
    clap::App::new(NAME)
        .about("Hooks the development environment; source the output from .envrc")
        .arg(
            clap::Arg::new("dir")
                .value_name("DIR")
                .help("The directory in which to build"),
        )
}

pub fn run(args: &clap::ArgMatches) -> Result {
    let config = config::Config::load(args.value_of_os("dir"))?;

    // Capture the environment here so we can later diff it against the
    // environment that direnv reports for the configured parent directory.
    let env_here: env::Env = vars_os().collect();
    let env_outside: env::Env = {
        // Setting up additional OS pipes for subprocesses to communicate back
        // to us is not well supported in the Rust standard library, so we use
        // files in a temporary directory instead. Here we try to create the
        // temporary directory in a preexisting cache directory, but fall back
        // to using the system's temporary directory, since we don't want to
        // write to the filesystem in the project directory until the user has
        // specifically requested it (by calling `firstaide build` for example).
        let temp_dir = tempfile::TempDir::new_in(&config.cache_dir)
            .or_else(|_err| tempfile::TempDir::new())?;
        let dump_path = temp_dir.path().join("outside");
        let mut dump_cmd = config.command_to_dump_env_outside(&dump_path);
        let mut dump_proc = dump_cmd.spawn()?;
        if !dump_proc.wait()?.success() {
            return Err(Error::EnvOutsideCapture);
        }
        match bincode::deserialize(&fs::read(dump_path)?) {
            Ok(env) => Ok(env),
            Err(err) => Err(Error::EnvOutsideDecode(err)),
        }
    }?;

    // However, we prevent the parent environment from removing or wiping
    // DIRENV_WATCHES. This mirrors the behaviour of direnv's `direnv_load`
    // function; see `direnv stdlib`. We don't use `direnv_load` because it had
    // a couple of breaking bugs in direnv 2.20.[01].
    let mut env_diff = env::diff(&env_here, &env_outside).exclude_by(|change| match change {
        env::Changed(name, _, value) if name == "DIRENV_WATCHES" && value == "" => true,
        env::Removed(name, _) if name == "DIRENV_WATCHES" => true,
        _ => false,
    });

    // Prepare to write to stdout.
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // Wrap everything in { ... } so that it's only evaluated by Bash once
    // completely written out. This is for correctness, but it might also help
    // prevent seeing broken pipe errors.
    writeln!(&mut handle, "{{ # Start.")?;
    writeln!(&mut handle)?;

    fn chunk(title: &str, chunk: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();
        let comments = title.lines().map(|line| format!("### {}\n", line));
        buf.extend(comments.map(String::into_bytes).flatten());
        buf.extend(chunk);
        buf.push(b'\n');
        buf
    }

    handle.write_all(&chunk("Helpers.", include_bytes!("hook/helpers.sh")))?;

    let sums_now = sums::Checksums::from(&config.watch_files()?)?;
    let cache_file = config.cache_file(&sums_now);
    let cache_file_fallback = config.cache_file_most_recent();

    match cache::Cache::load_with_fallback(&cache_file, &cache_file_fallback) {
        Ok(cache) => {
            // Filter out DIRENV_ and SSH_ vars from cached diff, then use it to
            // extend the parent's environment diff.
            env_diff.extend(
                cache
                    .diff
                    .exclude_by_prefix(b"DIRENV_")
                    .exclude_by_prefix(b"SSH_"),
            );
            env_diff.simplify();
            if sums::equal(&sums_now, &cache.sums) {
                let chunk_message = bash::escape(&config.messages.getting_started);
                let chunk_content =
                    include_bytes!("hook/active.sh").replace(b"__MESSAGE__", chunk_message);
                handle.write_all(&chunk(&EnvironmentStatus::Okay.display(), &chunk_content))?;
                handle.write_all(&chunk(
                    "Computed environment follows (includes parent environment):",
                    &env_diff_dump(&env_diff),
                ))?;
            } else {
                handle.write_all(&chunk(
                    &EnvironmentStatus::Stale.display(),
                    include_bytes!("hook/stale.sh"),
                ))?;
                handle.write_all(&chunk(
                    "Computed environment follows (includes parent environment):",
                    &env_diff_dump(&env_diff),
                ))?;
            }
            // We want direnv to watch every file for which we calculate a
            // checksum, AND we want it to watch the firstaide cache file.
            {
                let mut watches = Vec::with_capacity(8192); // 8kB enough?
                watches.extend(b"watch_file \\\n  ");
                for watch in cache.sums.into_iter() {
                    bash::escape_into(watch.path(), &mut watches);
                    watches.extend(b" \\\n  ");
                }
                // Also watch the cache file, the build log, the build
                // executable, and the watch executable.
                bash::escape_into(&cache_file, &mut watches);
                watches.extend(b" \\\n  ");
                bash::escape_into(&config.build_log_file(), &mut watches);
                watches.extend(b" \\\n  ");
                bash::escape_into(&config.build_exe, &mut watches);
                watches.extend(b" \\\n  ");
                bash::escape_into(&config.watch_exe, &mut watches);
                watches.push(b'\n');

                handle.write_all(&chunk("Watch dependencies.", &watches))?;
            }
        }
        Err(_) => {
            handle.write_all(&chunk(
                &EnvironmentStatus::Unknown.display(),
                include_bytes!("hook/inactive.sh"),
            ))?;
            handle.write_all(&chunk(
                "Parent environment follows:",
                &env_diff_dump(&env_diff),
            ))?;
        }
    };

    writeln!(&mut handle, "}} # End.")?;

    Ok(0)
}

fn env_diff_dump(diff: &env::Diff) -> Vec<u8> {
    use bash::escape as esc;
    use env::Change::*;

    let mut output: Vec<u8> = Vec::new();
    for change in diff {
        match change {
            Added(k, vb) => {
                output.extend(b"export ");
                output.extend(esc(k));
                output.extend(b"=");
                output.extend(esc(vb));
            }
            Changed(k, _va, vb) => {
                output.extend(b"export ");
                output.extend(esc(k));
                output.extend(b"=");
                output.extend(esc(vb));
            }
            Removed(k, _va) => {
                output.extend(b"unset ");
                output.extend(esc(k));
            }
        }
        output.push(b'\n');
    }
    output
}
