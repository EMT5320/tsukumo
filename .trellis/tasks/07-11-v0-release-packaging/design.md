# V0 Release Packaging — Technical Design

## Distribution Shape

The `tsukumo-host` binary target becomes the product entry point named
`tsukumo`. Library crates remain internal workspace packages for V0 unless
publishing them serves the binary installation path.

## Reproducibility Contract

- Track the executable workspace lockfile.
- Pin a toolchain proven against all dependencies and keep `rust-version`
  aligned with the verified MSRV.
- Run identical fmt/check/clippy/test commands locally and in CI.
- Use credential-free fixtures by default; manual live jobs never receive
  repository-stored credentials.

## Documentation Contract

README follows the user journey:

1. what Tsukumo V0 does and its claim boundary;
2. prerequisites and install;
3. fixture quickstart;
4. live Claude/Codex setup;
5. TUI controls;
6. data, privacy, revoke, and removal;
7. troubleshooting and known limitations;
8. architecture/development links.

Screenshots are captured from the release-candidate binary and cannot depict
unimplemented states.

## CI Matrix

- Linux stable/pinned target: fmt, check, clippy, tests.
- Windows GNU pinned target: check, clippy, tests; format can run once in the
  matrix.
- Fixture/evidence secret and personal-path validation in both relevant paths.
- Optional manual live-smoke instructions remain outside credential-free CI.

## Release Gate

A clean checkout builds, installs, starts fixture mode, renders the TUI, runs
one state/revoke/projection flow, and passes all tests. Release notes and tag
follow that receipt.

## Rollback

A failed candidate receives no release tag. Version/toolchain/lock changes are
reverted together; durable SQLite migrations remain additive and documented.
