//! Crossterm input and terminal color capability mapping.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tsukumo_theater::{ColorCapability, UiInput, UiKey};

pub fn map_terminal_key(event: KeyEvent) -> Option<UiInput> {
    if event.kind == KeyEventKind::Release {
        return None;
    }
    if event.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(event.code, KeyCode::Char('c' | 'C'))
    {
        return Some(UiInput::Key(UiKey::Quit));
    }
    if event
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
    {
        return None;
    }

    let key = match event.code {
        KeyCode::Esc => UiKey::Escape,
        KeyCode::Up => UiKey::Up,
        KeyCode::Down => UiKey::Down,
        KeyCode::Left | KeyCode::PageUp => UiKey::PreviousPage,
        KeyCode::Right | KeyCode::PageDown => UiKey::NextPage,
        KeyCode::Char('1') => UiKey::AllowOnce,
        KeyCode::Char('2') => UiKey::AllowSession,
        KeyCode::Char(character) => match character.to_ascii_lowercase() {
            'w' => UiKey::OpenWorkshop,
            's' => UiKey::OpenStates,
            'p' => UiKey::OpenProjection,
            'r' => UiKey::Refresh,
            'x' => UiKey::Revoke,
            'd' => UiKey::Deny,
            'q' => UiKey::Quit,
            _ => return None,
        },
        KeyCode::Backspace
        | KeyCode::Enter
        | KeyCode::Home
        | KeyCode::End
        | KeyCode::Tab
        | KeyCode::BackTab
        | KeyCode::Delete
        | KeyCode::Insert
        | KeyCode::F(_)
        | KeyCode::Null
        | KeyCode::CapsLock
        | KeyCode::ScrollLock
        | KeyCode::NumLock
        | KeyCode::PrintScreen
        | KeyCode::Pause
        | KeyCode::Menu
        | KeyCode::KeypadBegin
        | KeyCode::Media(_)
        | KeyCode::Modifier(_) => return None,
    };
    if event.kind == KeyEventKind::Repeat
        && !matches!(
            key,
            UiKey::Up | UiKey::Down | UiKey::PreviousPage | UiKey::NextPage
        )
    {
        return None;
    }
    Some(UiInput::Key(key))
}

pub fn color_capability_from_env(
    no_color: bool,
    color_term: Option<&str>,
    term: Option<&str>,
    windows_terminal: bool,
) -> ColorCapability {
    if no_color {
        return ColorCapability::Monochrome;
    }
    if windows_terminal
        || color_term.is_some_and(|value| {
            value.eq_ignore_ascii_case("truecolor") || value.eq_ignore_ascii_case("24bit")
        })
    {
        return ColorCapability::TrueColor;
    }
    if term.is_some_and(|value| value.to_ascii_lowercase().contains("256color")) {
        return ColorCapability::Ansi256;
    }
    ColorCapability::Ansi256
}

pub(super) fn detect_color_capability() -> ColorCapability {
    let color_term = std::env::var("COLORTERM").ok();
    let term = std::env::var("TERM").ok();
    color_capability_from_env(
        std::env::var_os("NO_COLOR").is_some(),
        color_term.as_deref(),
        term.as_deref(),
        std::env::var_os("WT_SESSION").is_some(),
    )
}
