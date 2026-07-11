//! Thin packaged entrypoint for the receipt-first Host library.

use std::ffi::OsString;
use std::io::{self, Write};
use std::process::ExitCode;
use thiserror::Error;

const HELP: &str = "tsukumo-host - receipt-first runtime composition root

USAGE:
    tsukumo-host [--help | --version]

LIVE VERIFICATION:
    TSUKUMO_RUN_LIVE_SMOKE=1 cargo test -p tsukumo-host --test claude_live -- --ignored

The default command performs no model call and discovers no credentials.";

fn main() -> ExitCode {
    match run(std::env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let write_failed = writeln!(io::stderr().lock(), "tsukumo-host: {error}").is_err();
            if write_failed {
                ExitCode::from(1)
            } else {
                ExitCode::from(2)
            }
        }
    }
}

fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), CliError> {
    let args = args.into_iter().collect::<Vec<_>>();
    match args.as_slice() {
        [] => write_stdout(HELP),
        [value] if value == "--help" || value == "-h" => write_stdout(HELP),
        [value] if value == "--version" || value == "-V" => {
            write_stdout(&format!("tsukumo-host {}", env!("CARGO_PKG_VERSION")))
        }
        _ => Err(CliError::UnknownArgument),
    }
}

fn write_stdout(value: &str) -> Result<(), CliError> {
    writeln!(io::stdout().lock(), "{value}").map_err(CliError::Output)
}

#[derive(Debug, Error)]
enum CliError {
    #[error("unknown argument; use --help")]
    UnknownArgument,
    #[error("failed to write output: {0}")]
    Output(io::Error),
}
