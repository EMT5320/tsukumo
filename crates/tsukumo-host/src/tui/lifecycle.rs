//! RAII ownership of raw mode, alternate screen, and cursor restoration.

use super::TuiError;
use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

static TERMINAL_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);

pub(super) struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    active: bool,
}

impl TerminalGuard {
    pub fn enter() -> Result<Self, TuiError> {
        let mut operations = CrosstermModeOperations;
        enter_mode(&mut operations)?;
        TERMINAL_MODE_ACTIVE.store(true, Ordering::Release);
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(source) => {
                TERMINAL_MODE_ACTIVE.store(false, Ordering::Release);
                restore_mode(&mut operations);
                return Err(TuiError::Io(source));
            }
        };
        Ok(Self {
            terminal,
            active: true,
        })
    }

    pub fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }

    fn restore(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        if TERMINAL_MODE_ACTIVE.swap(false, Ordering::AcqRel) {
            let mut operations = BackendModeOperations {
                backend: self.terminal.backend_mut(),
            };
            restore_mode(&mut operations);
            let _ = self.terminal.show_cursor();
        }
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        self.restore();
    }
}

pub(super) fn install_panic_restoration_hook() {
    static INSTALL_HOOK: Once = Once::new();

    // A process-wide hook wraps the previous hook once and acts only during an active TUI.
    INSTALL_HOOK.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |information| {
            invoke_panic_restoration(restore_terminal_if_active, || previous(information));
        }));
    });
}

trait ModeOperations {
    fn enable_raw(&mut self) -> io::Result<()>;
    fn enter_alternate_and_hide(&mut self) -> io::Result<()>;
    fn disable_raw(&mut self);
    fn leave_alternate_and_show(&mut self);
}

struct CrosstermModeOperations;

impl ModeOperations for CrosstermModeOperations {
    fn enable_raw(&mut self) -> io::Result<()> {
        enable_raw_mode()
    }

    fn enter_alternate_and_hide(&mut self) -> io::Result<()> {
        execute!(io::stdout(), EnterAlternateScreen, Hide)
    }

    fn disable_raw(&mut self) {
        let _ = disable_raw_mode();
    }

    fn leave_alternate_and_show(&mut self) {
        let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
    }
}

struct BackendModeOperations<'a> {
    backend: &'a mut CrosstermBackend<Stdout>,
}

impl ModeOperations for BackendModeOperations<'_> {
    fn enable_raw(&mut self) -> io::Result<()> {
        enable_raw_mode()
    }

    fn enter_alternate_and_hide(&mut self) -> io::Result<()> {
        execute!(self.backend, EnterAlternateScreen, Hide)
    }

    fn disable_raw(&mut self) {
        let _ = disable_raw_mode();
    }

    fn leave_alternate_and_show(&mut self) {
        let _ = execute!(self.backend, LeaveAlternateScreen, Show);
    }
}

fn enter_mode(operations: &mut impl ModeOperations) -> io::Result<()> {
    operations.enable_raw()?;
    if let Err(source) = operations.enter_alternate_and_hide() {
        restore_mode(operations);
        return Err(source);
    }
    Ok(())
}

fn restore_mode(operations: &mut impl ModeOperations) {
    operations.disable_raw();
    operations.leave_alternate_and_show();
}

fn restore_terminal_if_active() {
    if !TERMINAL_MODE_ACTIVE.swap(false, Ordering::AcqRel) {
        return;
    }
    restore_terminal_best_effort();
}

fn restore_terminal_best_effort() {
    let mut operations = CrosstermModeOperations;
    restore_mode(&mut operations);
}

fn invoke_panic_restoration(restore: impl FnOnce(), previous: impl FnOnce()) {
    restore();
    previous();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeOperations {
        calls: Vec<&'static str>,
        fail_enter: bool,
    }

    impl ModeOperations for FakeOperations {
        fn enable_raw(&mut self) -> io::Result<()> {
            self.calls.push("enable_raw");
            Ok(())
        }

        fn enter_alternate_and_hide(&mut self) -> io::Result<()> {
            self.calls.push("enter_alternate_and_hide");
            if self.fail_enter {
                Err(io::Error::other("injected alternate-screen failure"))
            } else {
                Ok(())
            }
        }

        fn disable_raw(&mut self) {
            self.calls.push("disable_raw");
        }

        fn leave_alternate_and_show(&mut self) {
            self.calls.push("leave_alternate_and_show");
        }
    }

    #[test]
    fn alternate_screen_failure_when_raw_was_enabled_restores_every_mode() {
        // Given: a deterministic failure after raw mode succeeds.
        let mut operations = FakeOperations {
            fail_enter: true,
            ..FakeOperations::default()
        };

        // When: terminal mode entry is attempted.
        let error = enter_mode(&mut operations).expect_err("injected entry failure");

        // Then: all possibly-partial terminal modes are restored.
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert_eq!(
            operations.calls,
            [
                "enable_raw",
                "enter_alternate_and_hide",
                "disable_raw",
                "leave_alternate_and_show",
            ]
        );
    }

    #[test]
    fn panic_hook_path_restores_before_delegating_to_the_previous_hook() {
        // Given: observable stand-ins for restoration and the previous panic hook.
        let calls = std::cell::RefCell::new(Vec::new());

        // When: the ordered helper used by the process-wide hook runs.
        invoke_panic_restoration(
            || calls.borrow_mut().push("restore"),
            || calls.borrow_mut().push("previous"),
        );

        // Then: terminal cleanup always precedes diagnostic delegation.
        assert_eq!(calls.into_inner(), ["restore", "previous"]);
    }

    #[test]
    fn restoration_when_called_addresses_every_terminal_mode() {
        // Given: a fake terminal mode operation sink.
        let mut operations = FakeOperations::default();

        // When: the shared normal/error/panic restoration sequence runs.
        restore_mode(&mut operations);

        // Then: raw mode, alternate screen, and cursor visibility are all addressed.
        assert_eq!(
            operations.calls,
            ["disable_raw", "leave_alternate_and_show"]
        );
    }
}
