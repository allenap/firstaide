use crate::bash;
use crate::cache;
use crate::config;
use crate::env;
use crate::status::EnvironmentStatus;
use crate::sums;
use bstr::ByteSlice;
use std::fmt;
use std::io::{self, Write};

pub const NAME: &str = "hook";

type Result = std::result::Result<u8, Error>;

pub enum Error {
    Config(config::Error),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            Config(err) => write!(f, "{}", err),
            Io(err) => write!(f, "input/output error: {}", err),
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

pub fn argspec<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name(NAME)
        .about("Hooks the development environment; source the output from .envrc")
        .arg(
            clap::Arg::with_name("dir")
                .value_name("DIR")
                .help("The directory in which to build"),
        )
}

pub fn run(args: &clap::ArgMatches) -> Result {
    let config = config::Config::load(args.value_of_os("dir"))?;
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
    handle.write_all(&chunk(
        "Load parent environments.",
        include_bytes!("hook/parent.sh"),
    ))?;

    match cache::Cache::load(config.cache_file()) {
        Ok(cache) => {
            let sums_now = sums::Checksums::from(&config.watch_files()?)?;
            if sums::equal(&sums_now, &cache.sums) {
                let chunk_message = bash::escape(&config.messages.getting_started);
                let chunk_content =
                    include_bytes!("hook/active.sh").replace(b"__MESSAGE__", chunk_message);
                handle.write_all(&chunk(&EnvironmentStatus::Okay.display(), &chunk_content))?;
                handle.write_all(&chunk(
                    "Cached environment follows:",
                    &env_diff_dump(&cache.diff),
                ))?;
            } else {
                handle.write_all(&chunk(
                    &EnvironmentStatus::Stale.display(),
                    include_bytes!("hook/stale.sh"),
                ))?;
                handle.write_all(&chunk(
                    "Cached environment follows:",
                    &env_diff_dump(&cache.diff),
                ))?;
            }
            // We want direnv to watch every file for which we calculate a
            // checksum, AND we want it to watch the firstaide cache file.
            {
                let mut watches = Vec::with_capacity(8192); // 8kB enough?
                watches.extend(b"watch_file \\\n");
                for watch in cache.sums.into_iter() {
                    watches.extend(b"  ");
                    bash::escape_into(watch.path(), &mut watches);
                    watches.extend(b" \\\n");
                }
                watches.extend(b"  ");
                watches.extend(bash::escape(config.cache_file()));
                watches.push(b'\n');
                handle.write_all(&chunk("Watch dependencies.", &watches))?;
            }
        }
        Err(_) => {
            handle.write_all(&chunk(
                &EnvironmentStatus::Unknown.display(),
                include_bytes!("hook/inactive.sh"),
            ))?;
        }
    };

    writeln!(&mut handle, "}} # End.")?;

    Ok(0)
}

pub fn env_diff_dump(diff: &env::Diff) -> Vec<u8> {
    use crate::bash::escape as esc;
    use crate::env::Change::*;

    // Filter out DIRENV_ and SSH_ vars.
    let diff = diff
        .exclude_by_prefix(b"DIRENV_")
        .exclude_by_prefix(b"SSH_");

    let mut output: Vec<u8> = Vec::new();
    for change in &diff {
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
