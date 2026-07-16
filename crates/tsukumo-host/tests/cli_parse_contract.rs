use std::ffi::OsString;
use std::path::{Path, PathBuf};
use tsukumo_host::{
    parse_host_args, EpisodeCommand, HostCliError, HostCommand, PresentationPackSource,
};

#[test]
fn no_arguments_when_parsed_selects_default_interactive_run() {
    // Given: an empty product argument list.
    let args = Vec::<OsString>::new();

    // When: the pure host CLI parser runs.
    let command = parse_host_args(args).expect("default command");

    // Then: the bundled pack and normal motion are selected.
    assert!(matches!(
        command,
        HostCommand::Run(options)
            if options.presentation_pack == PresentationPackSource::EmbeddedDefault
                && !options.reduced_motion
    ));
}

#[test]
fn presentation_pack_and_reduced_motion_when_parsed_are_typed() {
    // Given: one explicit external pack plus reduced motion.
    let args = [
        OsString::from("--presentation-pack"),
        OsString::from("packs/custom"),
        OsString::from("--reduced-motion"),
    ];

    // When: arguments are parsed.
    let command = parse_host_args(args).expect("valid run options");

    // Then: path and motion preference survive as typed configuration.
    assert!(matches!(
        command,
        HostCommand::Run(options)
            if options.presentation_pack
                == PresentationPackSource::Directory(PathBuf::from("packs/custom"))
                && options.reduced_motion
    ));
}

#[test]
fn missing_pack_path_when_parsed_returns_actionable_error() {
    // Given: a path-taking flag without its value.
    let args = [OsString::from("--presentation-pack")];

    // When: parsing is attempted.
    let error = parse_host_args(args).expect_err("missing path must fail");

    // Then: the caller can report the exact missing value.
    assert!(matches!(
        error,
        HostCliError::MissingValue {
            flag: "--presentation-pack"
        }
    ));
}

#[test]
fn episode_seed_when_parsed_requires_reviewed_spec_and_data_dir() {
    let command = parse_host_args([
        "episode",
        "seed",
        "--spec",
        "episode.json",
        "--data-dir",
        "episode-data",
    ])
    .expect("parse episode seed");

    assert!(matches!(
        command,
        HostCommand::Episode(EpisodeCommand::Seed(options))
            if options.spec == Path::new("episode.json")
                && options.data_dir == Path::new("episode-data")
    ));
}

#[test]
fn episode_inspect_when_parsed_keeps_observation_inputs_explicit() {
    let command = parse_host_args([
        "episode",
        "inspect",
        "--spec",
        "episode.json",
        "--runtime-executable",
        "claude",
        "--working-dir",
        "workspace",
    ])
    .expect("parse episode inspect");

    assert!(matches!(
        command,
        HostCommand::Episode(EpisodeCommand::Inspect(options))
            if options.spec == Path::new("episode.json")
                && options.runtime_executable == Path::new("claude")
                && options.working_dir == Path::new("workspace")
    ));
}

#[test]
fn episode_inspect_missing_working_dir_is_actionable() {
    let error = parse_host_args([
        "episode",
        "inspect",
        "--spec",
        "episode.json",
        "--runtime-executable",
        "claude",
    ])
    .expect_err("working directory is required");

    assert!(matches!(
        error,
        HostCliError::MissingValue {
            flag: "--working-dir"
        }
    ));
}

#[test]
fn episode_resume_when_parsed_keeps_runtime_capability_explicit() {
    let command = parse_host_args([
        "episode",
        "resume",
        "--spec",
        "episode.json",
        "--data-dir",
        "episode-data",
        "--runtime-executable",
        "codex",
        "--working-dir",
        "workspace",
        "--workspace-write",
        "--confirm-live-run",
    ])
    .expect("parse episode resume");

    assert!(matches!(
        command,
        HostCommand::Episode(EpisodeCommand::Resume(options))
            if options.runtime_executable == Path::new("codex")
                && options.working_dir == Path::new("workspace")
                && options.workspace_write_acknowledged
                && options.live_run_confirmed
    ));
}

#[test]
fn episode_resume_missing_runtime_executable_is_actionable() {
    let error = parse_host_args([
        "episode",
        "resume",
        "--spec",
        "episode.json",
        "--data-dir",
        "episode-data",
        "--working-dir",
        "workspace",
    ])
    .expect_err("runtime executable is required");

    assert!(matches!(
        error,
        HostCliError::MissingValue {
            flag: "--runtime-executable"
        }
    ));
}
