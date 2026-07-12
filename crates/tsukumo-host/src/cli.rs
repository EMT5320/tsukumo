//! Pure command-line parsing for the interactive host product.

use crate::presentation_pack::PresentationPackSource;
use std::ffi::{OsStr, OsString};
use thiserror::Error;

/// Typed options required to enter the terminal product.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostRunOptions {
    pub presentation_pack: PresentationPackSource,
    pub reduced_motion: bool,
}

/// One top-level host command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostCommand {
    Run(HostRunOptions),
    Help,
    Version,
}

/// User-facing command-line contract failures.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum HostCliError {
    #[error("missing value for {flag}")]
    MissingValue { flag: &'static str },
    #[error("flag may be supplied only once: {flag}")]
    DuplicateFlag { flag: &'static str },
    #[error("{command} cannot be combined with run options")]
    ConflictingCommand { command: &'static str },
    #[error("unknown argument {argument}; use --help")]
    UnknownArgument { argument: String },
}

/// Parses host arguments without touching files, the terminal, or a runtime.
pub fn parse_host_args<I, S>(args: I) -> Result<HostCommand, HostCliError>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let values = args.into_iter().map(Into::into).collect::<Vec<_>>();
    if values.len() == 1 && is_flag(&values[0], "--help", "-h") {
        return Ok(HostCommand::Help);
    }
    if values.len() == 1 && is_flag(&values[0], "--version", "-V") {
        return Ok(HostCommand::Version);
    }
    if values.iter().any(|value| is_flag(value, "--help", "-h")) {
        return Err(HostCliError::ConflictingCommand { command: "--help" });
    }
    if values.iter().any(|value| is_flag(value, "--version", "-V")) {
        return Err(HostCliError::ConflictingCommand {
            command: "--version",
        });
    }

    let mut source = PresentationPackSource::EmbeddedDefault;
    let mut reduced_motion = false;
    let mut index = 0;
    while index < values.len() {
        let value = &values[index];
        if value == OsStr::new("--presentation-pack") {
            if !matches!(source, PresentationPackSource::EmbeddedDefault) {
                return Err(HostCliError::DuplicateFlag {
                    flag: "--presentation-pack",
                });
            }
            let path = values
                .get(index + 1)
                .ok_or(HostCliError::MissingValue {
                    flag: "--presentation-pack",
                })?
                .clone();
            source = PresentationPackSource::Directory(path.into());
            index += 2;
        } else if value == OsStr::new("--reduced-motion") {
            if reduced_motion {
                return Err(HostCliError::DuplicateFlag {
                    flag: "--reduced-motion",
                });
            }
            reduced_motion = true;
            index += 1;
        } else {
            return Err(HostCliError::UnknownArgument {
                argument: value.to_string_lossy().into_owned(),
            });
        }
    }

    Ok(HostCommand::Run(HostRunOptions {
        presentation_pack: source,
        reduced_motion,
    }))
}

fn is_flag(value: &OsStr, long: &str, short: &str) -> bool {
    value == OsStr::new(long) || value == OsStr::new(short)
}
