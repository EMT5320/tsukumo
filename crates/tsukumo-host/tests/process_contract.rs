use std::ffi::OsString;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;
use tsukumo_adapters::{PromptDelivery, RuntimeCommandSpec, RuntimeLaunchConfig};
use tsukumo_host::{
    ProcessLaunch, ProcessLimits, ProcessRunner, RuntimeOutput, StandardProcessRunner,
};
use tsukumo_kernel::SensitiveText;

const CHILD_MODE: &str = "TSUKUMO_PROCESS_FIXTURE_MODE";
const CHILD_CAPTURE: &str = "TSUKUMO_PROCESS_FIXTURE_CAPTURE";

#[test]
fn fixture_child() {
    let Ok(mode) = std::env::var(CHILD_MODE) else {
        return;
    };
    match mode.as_str() {
        "echo" => run_echo_child(),
        "wait" => run_wait_child(),
        "stdout_overflow" => run_overflow_child(false),
        "stderr_overflow" => run_overflow_child(true),
        _ => std::process::exit(42),
    }
}

#[test]
fn standard_runner_delivers_prompt_only_through_stdin_and_reaps() {
    // Given: a child command whose diagnostics contain sensitive-looking paths and values.
    let temp = tempfile::tempdir().expect("create temp directory");
    let capture = temp.path().join("captured-prompt.bin");
    let sentinel = "sentinel-runtime-prompt\nwith-second-line";
    let spec = fixture_command("echo", &capture);
    let launch_debug = format!("{spec:?}");
    assert!(!launch_debug.contains(sentinel));
    assert!(!launch_debug.contains(capture.to_string_lossy().as_ref()));

    // When: the standard runner starts the child and drains its output incrementally.
    let runner = StandardProcessRunner;
    let mut handle = runner
        .spawn(ProcessLaunch::new(
            spec,
            Some(SensitiveText::new(sentinel)),
            ProcessLimits::default(),
        ))
        .expect("spawn fixture child");
    let mut saw_stdout = false;
    let exit = loop {
        match handle
            .next(Duration::from_secs(2))
            .expect("read child output")
        {
            RuntimeOutput::StdoutLine(line) => {
                saw_stdout |= line == r#"{"type":"fixture_progress"}"#;
            }
            RuntimeOutput::StderrLine(_) | RuntimeOutput::Idle => {}
            RuntimeOutput::Exited(exit) => break exit,
        }
    };

    // Then: stdin bytes are exact, output arrived, and repeat cleanup is idempotent.
    assert!(saw_stdout);
    assert!(exit.success);
    assert_eq!(
        fs::read(&capture).expect("read captured prompt"),
        sentinel.as_bytes()
    );
    assert_eq!(handle.cancel_and_reap().expect("repeat reap"), exit);
    assert!(!format!("{handle:?}").contains(sentinel));
}

#[test]
fn cancellation_reaps_a_waiting_child_once() {
    // Given: a child that stays alive after stdin closes.
    let temp = tempfile::tempdir().expect("create temp directory");
    let spec = fixture_command("wait", &temp.path().join("unused.bin"));
    let runner = StandardProcessRunner;
    let mut handle = runner
        .spawn(ProcessLaunch::new(
            spec,
            Some(SensitiveText::new("short prompt")),
            ProcessLimits::default(),
        ))
        .expect("spawn waiting fixture child");

    // When: cancellation is requested twice.
    let first = handle.cancel_and_reap().expect("cancel child");
    let second = handle.cancel_and_reap().expect("repeat cancellation");

    // Then: both calls report the same reaped status without another kill attempt.
    assert_eq!(first, second);
    assert!(!first.success);
}

#[test]
fn stdout_lines_and_total_stderr_are_bounded_before_reap() {
    for (mode, limits, expected_stream) in [
        (
            "stdout_overflow",
            ProcessLimits::new(32, 256, 4).expect("valid stdout limits"),
            "stdout",
        ),
        (
            "stderr_overflow",
            ProcessLimits::new(1_024, 32, 4).expect("valid stderr limits"),
            "stderr",
        ),
    ] {
        // Given: a fixture child that exceeds one configured raw-output budget.
        let temp = tempfile::tempdir().expect("create overflow temp directory");
        let runner = StandardProcessRunner;
        let mut handle = runner
            .spawn(ProcessLaunch::new(
                fixture_command(mode, &temp.path().join("unused.bin")),
                Some(SensitiveText::new("bounded prompt")),
                limits,
            ))
            .expect("spawn overflow fixture");

        // When: the bounded reader reaches the configured budget.
        let error = loop {
            match handle.next(Duration::from_secs(2)) {
                Ok(
                    RuntimeOutput::StdoutLine(_)
                    | RuntimeOutput::StderrLine(_)
                    | RuntimeOutput::Idle,
                ) => {}
                Ok(RuntimeOutput::Exited(_)) => panic!("overflow child exited without an error"),
                Err(error) => break error,
            }
        };

        // Then: the error identifies only the stream/limit and cleanup still reaps the child.
        match (expected_stream, error) {
            (
                "stdout",
                tsukumo_host::ProcessError::LineLimitExceeded {
                    stream: "stdout", ..
                },
            ) => {}
            ("stderr", tsukumo_host::ProcessError::StderrLimitExceeded { .. }) => {}
            (_, other) => panic!("unexpected bounded-reader error: {other:?}"),
        }
        handle.cancel_and_reap().expect("reap overflow fixture");
    }
}

#[test]
fn configuration_and_poll_durations_reject_unbounded_allocations() {
    // Given/When: callers request allocation or wait budgets beyond Host limits.
    let limit_error =
        ProcessLimits::new(usize::MAX, 64, 4).expect_err("unbounded stdout allocation must fail");
    assert!(matches!(
        limit_error,
        tsukumo_host::ProcessConfigError::LimitTooLarge {
            field: "stdout_line_bytes",
            ..
        }
    ));

    let temp = tempfile::tempdir().expect("create wait-duration temp directory");
    let runner = StandardProcessRunner;
    let mut handle = runner
        .spawn(ProcessLaunch::new(
            fixture_command("wait", &temp.path().join("unused.bin")),
            Some(SensitiveText::new("bounded prompt")),
            ProcessLimits::default(),
        ))
        .expect("spawn wait-duration fixture");

    // Then: an unrepresentable deadline is typed and the child still reaps cleanly.
    assert!(matches!(
        handle.next(Duration::MAX),
        Err(tsukumo_host::ProcessError::WaitDurationTooLarge)
    ));
    handle
        .cancel_and_reap()
        .expect("reap wait-duration fixture");
}
fn fixture_command(mode: &str, capture: &std::path::Path) -> RuntimeCommandSpec {
    let executable = std::env::current_exe().expect("resolve test executable");
    let working_directory = std::env::current_dir().expect("resolve current directory");
    let launch = RuntimeLaunchConfig::new(executable, working_directory)
        .with_environment_override(OsString::from(CHILD_MODE), OsString::from(mode))
        .with_environment_override(
            OsString::from(CHILD_CAPTURE),
            capture.as_os_str().to_os_string(),
        );
    RuntimeCommandSpec::new(
        &launch,
        ["--exact", "fixture_child", "--nocapture"]
            .map(OsString::from)
            .to_vec(),
        PromptDelivery::Stdin,
    )
    .expect("build fixture command")
}

fn run_echo_child() -> ! {
    let mut prompt = Vec::new();
    std::io::stdin()
        .read_to_end(&mut prompt)
        .expect("read fixture stdin");
    let capture = PathBuf::from(std::env::var_os(CHILD_CAPTURE).expect("capture path"));
    fs::write(capture, prompt).expect("write captured prompt");
    std::io::stdout()
        .write_all(b"{\"type\":\"fixture_progress\"}\n")
        .expect("write fixture stdout");
    std::io::stdout().flush().expect("flush fixture stdout");
    std::process::exit(0)
}

fn run_wait_child() -> ! {
    let mut prompt = Vec::new();
    std::io::stdin()
        .read_to_end(&mut prompt)
        .expect("read fixture stdin");
    std::thread::sleep(Duration::from_secs(60));
    std::process::exit(0)
}

fn run_overflow_child(stderr: bool) -> ! {
    let mut prompt = Vec::new();
    std::io::stdin()
        .read_to_end(&mut prompt)
        .expect("read overflow fixture stdin");
    let bytes = vec![b'x'; 128];
    if stderr {
        std::io::stderr()
            .write_all(&bytes)
            .expect("write overflow stderr");
        std::io::stderr().flush().expect("flush overflow stderr");
    } else {
        std::io::stdout()
            .write_all(&bytes)
            .expect("write overflow stdout");
        std::io::stdout().flush().expect("flush overflow stdout");
    }
    std::thread::sleep(Duration::from_secs(60));
    std::process::exit(0)
}
