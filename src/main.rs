use clap::Parser;
use std::process;

mod cache;
mod cmds;
mod config;
mod env;
mod status;
mod sums;
use anyhow::Context;

#[derive(Debug, Parser)]
#[clap(about, version, author)]
struct Config {
    #[clap(subcommand)]
    command: Command,

    // A note (allenap) on logging. I don't like how logging works right now. It's
    // not bad, but it's not good either. However, it does work. So that I can
    // change my mind in the future, there's minimal UX to influence logging:
    // just a --verbose and a --quiet flag, which cannot be used together. No
    // short -v/-q flags, no multiple flags to incrementally increase or decrease
    // verbosity, and no fancy formatting.
    //
    /// Be more verbose
    #[clap(long, conflicts_with("quiet"))]
    verbose: bool,

    /// Be quieter
    #[clap(long, conflicts_with("verbose"))]
    quiet: bool,
}

#[derive(Debug, Parser)]
enum Command {
    Build(cmds::build::Command),
    Status(cmds::status::Command),
    Clean(cmds::clean::Command),
    Hook(cmds::hook::Command),
    Env(cmds::env::Command),
}

impl Config {
    fn main(&self) {
        if let Err(err) = self.init_logging() {
            eprintln!("{}", err);
            process::exit(2);
        }

        let result = match &self.command {
            Command::Build(build) => build.run().context("build failed"),
            Command::Status(status) => status.run().context("status failed"),
            Command::Clean(clean) => clean.run().context("clean failed"),
            Command::Hook(hook) => hook.run().context("hook failed"),
            Command::Env(env) => env.run().context("env failed"),
        };

        match result {
            Err(err) => {
                log::error!("{:?}", err);
                process::exit(2);
            }
            Ok(code) => {
                process::exit(code as i32);
            }
        };
    }

    fn init_logging(&self) -> Result<(), log::SetLoggerError> {
        let log_level = if self.verbose {
            log::LevelFilter::Debug
        } else if self.quiet {
            log::LevelFilter::Warn
        } else {
            log::LevelFilter::Info
        };

        fern::Dispatch::new()
            // Perform allocation-free log formatting.
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}  {}  {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    // record.target(),
                    record.level(),
                    message
                ))
            })
            // Add blanket level filter.
            .level(log_level)
            // Output to stderr.
            .chain(std::io::stderr())
            // Apply globally.
            .apply()
    }
}

fn main() {
    let opts = Config::parse();
    opts.main()
}
