//! Packaged entrypoint for the interactive Tsukumo host product.

use std::ffi::OsString;
use std::io::{self, Write};
use std::process::ExitCode;
use thiserror::Error;
use tsukumo_host::{
    load_presentation_pack, parse_host_args, run_tui, HostCliError, HostCommand,
    HostProductController, PresentationPackLoadError, ProductController, ProductControllerError,
    TuiError,
};

const HELP: &str = "tsukumo-host - receipt-first runtime composition root / workshop

USAGE:
    tsukumo-host [--presentation-pack <directory>] [--reduced-motion]
    tsukumo-host --help
    tsukumo-host --version

OPTIONS:
    --presentation-pack <directory>  Load one inert, versioned presentation pack
    --reduced-motion                 Freeze semantic poses on their key frames
    -h, --help                       Show this help
    -V, --version                    Show the package version

KEYS:
    W workshop  S state  P projection  R refresh  X revoke  Q quit
    Permission: 1 allow once  2 allow session  D deny

LIVE VERIFICATION:
    TSUKUMO_RUN_LIVE_SMOKE=1 cargo test -p tsukumo-host --test claude_live -- --ignored

Pack, Soul storage, and product read-model validation complete before raw mode or the alternate screen is entered.

ENVIRONMENT:
    TSUKUMO_DATA_DIR                  Soul/Chronicle directory (default: ./data)";

fn main() -> ExitCode {
    match run(std::env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let message = safe_error_message(&error);
            let write_failed = writeln!(io::stderr().lock(), "tsukumo-host: {message}").is_err();
            if write_failed {
                ExitCode::from(1)
            } else {
                ExitCode::from(2)
            }
        }
    }
}

fn safe_error_message(error: &impl std::fmt::Display) -> String {
    let redacted = tsukumo_kernel::redact_sensitive_text(&error.to_string());
    if redacted.chars().count() <= 2_048 {
        redacted
    } else {
        redacted.chars().take(2_047).collect::<String>() + "…"
    }
}
fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), CliError> {
    match parse_host_args(args)? {
        HostCommand::Help => write_stdout(HELP),
        HostCommand::Version => {
            write_stdout(&format!("tsukumo-host {}", env!("CARGO_PKG_VERSION")))
        }
        HostCommand::Run(options) => {
            let pack = load_presentation_pack(&options.presentation_pack)?;
            let data_dir = std::env::var_os("TSUKUMO_DATA_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from("data"));
            let mut controller = HostProductController::open(data_dir, &pack)?;
            let snapshot = controller.refresh()?;
            run_tui(&pack, &mut controller, snapshot, options.reduced_motion)?;
            Ok(())
        }
    }
}

fn write_stdout(value: &str) -> Result<(), CliError> {
    writeln!(io::stdout().lock(), "{value}").map_err(CliError::Output)
}

#[derive(Debug, Error)]
enum CliError {
    #[error(transparent)]
    Arguments(#[from] HostCliError),
    #[error(transparent)]
    PresentationPack(#[from] PresentationPackLoadError),
    #[error(transparent)]
    Product(#[from] ProductControllerError),
    #[error(transparent)]
    Terminal(#[from] TuiError),
    #[error("failed to write output: {0}")]
    Output(io::Error),
}
