use std::ffi::OsString;
use std::path::PathBuf;
use tsukumo_host::{parse_host_args, HostCliError, HostCommand, PresentationPackSource};

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
