use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tsukumo_host::{color_capability_from_env, map_terminal_key};
use tsukumo_theater::{ColorCapability, UiInput, UiKey};

#[test]
fn terminal_keys_when_pressed_map_to_semantic_inputs() {
    // Given: product keys and one release event.
    let open_states = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
    let deny = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::SHIFT);
    let release = KeyEvent {
        code: KeyCode::Char('q'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Release,
        state: crossterm::event::KeyEventState::NONE,
    };

    // When/Then: presses map semantically and releases do not double-trigger.
    assert_eq!(
        map_terminal_key(open_states),
        Some(UiInput::Key(UiKey::OpenStates))
    );
    assert_eq!(map_terminal_key(deny), Some(UiInput::Key(UiKey::Deny)));
    assert_eq!(map_terminal_key(release), None);
}

#[test]
fn terminal_capability_when_environment_varies_has_safe_fallbacks() {
    // Given/When/Then: explicit no-color, truecolor, 256-color, and plain terminals.
    assert_eq!(
        color_capability_from_env(true, None, None, false),
        ColorCapability::Monochrome
    );
    assert_eq!(
        color_capability_from_env(false, Some("truecolor"), None, false),
        ColorCapability::TrueColor
    );
    assert_eq!(
        color_capability_from_env(false, None, Some("xterm-256color"), false),
        ColorCapability::Ansi256
    );
    assert_eq!(
        color_capability_from_env(false, None, None, true),
        ColorCapability::TrueColor
    );
}

#[test]
fn repeated_or_modified_authority_keys_do_not_trigger_actions() {
    // Given: repeated and modifier-bearing permission or destructive keys.
    let repeat = KeyEvent {
        code: KeyCode::Char('x'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Repeat,
        state: crossterm::event::KeyEventState::NONE,
    };
    let control = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
    let alt = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::ALT);

    // When/Then: only edge-triggered bare product keys may change authority.
    assert_eq!(map_terminal_key(repeat), None);
    assert_eq!(map_terminal_key(control), None);
    assert_eq!(map_terminal_key(alt), None);
}

#[test]
fn page_keys_when_pressed_map_to_bounded_inspector_navigation() {
    // Given: left/right and page navigation keys.
    let left = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
    let page_down = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);

    // When/Then: the terminal adapter keeps pagination semantic.
    assert_eq!(
        map_terminal_key(left),
        Some(UiInput::Key(UiKey::PreviousPage))
    );
    assert_eq!(
        map_terminal_key(page_down),
        Some(UiInput::Key(UiKey::NextPage))
    );
}
