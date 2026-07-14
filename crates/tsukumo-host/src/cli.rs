//! Pure command-line parsing for the interactive host product.

use crate::presentation_pack::PresentationPackSource;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use thiserror::Error;

/// Typed options required to enter the terminal product.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostRunOptions {
    pub presentation_pack: PresentationPackSource,
    pub reduced_motion: bool,
}

/// Inputs required to persist one reviewed episode seed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeSeedOptions {
    pub spec: PathBuf,
    pub data_dir: PathBuf,
}

/// Inputs required to resume one receipt-committed episode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeResumeOptions {
    pub spec: PathBuf,
    pub data_dir: PathBuf,
    pub runtime_executable: PathBuf,
    pub working_dir: PathBuf,
    pub workspace_write_acknowledged: bool,
    pub live_run_confirmed: bool,
}

/// One bounded evidence-collection action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpisodeCommand {
    Seed(EpisodeSeedOptions),
    Resume(EpisodeResumeOptions),
}

/// One top-level host command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostCommand {
    Run(HostRunOptions),
    Episode(EpisodeCommand),
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
    #[error("missing episode action; expected seed or resume")]
    MissingEpisodeAction,
    #[error("unknown episode action {action}; expected seed or resume")]
    UnknownEpisodeAction { action: String },
}

/// Parses host arguments without touching files, the terminal, or a runtime.
pub fn parse_host_args<I, S>(args: I) -> Result<HostCommand, HostCliError>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let values = args.into_iter().map(Into::into).collect::<Vec<_>>();
    if values
        .first()
        .is_some_and(|value| value == OsStr::new("episode"))
    {
        return parse_episode_args(&values[1..]).map(HostCommand::Episode);
    }
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

fn parse_episode_args(values: &[OsString]) -> Result<EpisodeCommand, HostCliError> {
    let Some(action) = values.first() else {
        return Err(HostCliError::MissingEpisodeAction);
    };
    match action.to_string_lossy().as_ref() {
        "seed" => parse_episode_seed(&values[1..]).map(EpisodeCommand::Seed),
        "resume" => parse_episode_resume(&values[1..]).map(EpisodeCommand::Resume),
        action => Err(HostCliError::UnknownEpisodeAction {
            action: action.to_owned(),
        }),
    }
}

fn parse_episode_seed(values: &[OsString]) -> Result<EpisodeSeedOptions, HostCliError> {
    let mut spec = None;
    let mut data_dir = None;
    let mut index = 0;
    while index < values.len() {
        if values[index] == OsStr::new("--spec") {
            set_path_flag(values, &mut index, &mut spec, "--spec")?;
        } else if values[index] == OsStr::new("--data-dir") {
            set_path_flag(values, &mut index, &mut data_dir, "--data-dir")?;
        } else {
            return Err(unknown_argument(&values[index]));
        }
    }
    Ok(EpisodeSeedOptions {
        spec: required_path(spec, "--spec")?,
        data_dir: required_path(data_dir, "--data-dir")?,
    })
}

fn parse_episode_resume(values: &[OsString]) -> Result<EpisodeResumeOptions, HostCliError> {
    let mut spec = None;
    let mut data_dir = None;
    let mut runtime_executable = None;
    let mut working_dir = None;
    let mut workspace_write_acknowledged = false;
    let mut live_run_confirmed = false;
    let mut index = 0;
    while index < values.len() {
        if values[index] == OsStr::new("--spec") {
            set_path_flag(values, &mut index, &mut spec, "--spec")?;
        } else if values[index] == OsStr::new("--data-dir") {
            set_path_flag(values, &mut index, &mut data_dir, "--data-dir")?;
        } else if values[index] == OsStr::new("--runtime-executable") {
            set_path_flag(
                values,
                &mut index,
                &mut runtime_executable,
                "--runtime-executable",
            )?;
        } else if values[index] == OsStr::new("--working-dir") {
            set_path_flag(values, &mut index, &mut working_dir, "--working-dir")?;
        } else if values[index] == OsStr::new("--workspace-write") {
            if workspace_write_acknowledged {
                return Err(HostCliError::DuplicateFlag {
                    flag: "--workspace-write",
                });
            }
            workspace_write_acknowledged = true;
            index += 1;
        } else if values[index] == OsStr::new("--confirm-live-run") {
            if live_run_confirmed {
                return Err(HostCliError::DuplicateFlag {
                    flag: "--confirm-live-run",
                });
            }
            live_run_confirmed = true;
            index += 1;
        } else {
            return Err(unknown_argument(&values[index]));
        }
    }
    Ok(EpisodeResumeOptions {
        spec: required_path(spec, "--spec")?,
        data_dir: required_path(data_dir, "--data-dir")?,
        runtime_executable: required_path(runtime_executable, "--runtime-executable")?,
        working_dir: required_path(working_dir, "--working-dir")?,
        workspace_write_acknowledged,
        live_run_confirmed,
    })
}

fn set_path_flag(
    values: &[OsString],
    index: &mut usize,
    target: &mut Option<PathBuf>,
    flag: &'static str,
) -> Result<(), HostCliError> {
    if target.is_some() {
        return Err(HostCliError::DuplicateFlag { flag });
    }
    let value = values
        .get(*index + 1)
        .ok_or(HostCliError::MissingValue { flag })?;
    *target = Some(PathBuf::from(value));
    *index += 2;
    Ok(())
}

fn required_path(value: Option<PathBuf>, flag: &'static str) -> Result<PathBuf, HostCliError> {
    value.ok_or(HostCliError::MissingValue { flag })
}

fn unknown_argument(value: &OsStr) -> HostCliError {
    HostCliError::UnknownArgument {
        argument: value.to_string_lossy().into_owned(),
    }
}

fn is_flag(value: &OsStr, long: &str, short: &str) -> bool {
    value == OsStr::new(long) || value == OsStr::new(short)
}
