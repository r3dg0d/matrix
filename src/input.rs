//! Non-blocking keyboard input.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    TogglePause,
    SpeedUp,
    SpeedDown,
    DensityUp,
    DensityDown,
    CyclePalette,
    CyclePreset,
    ToggleHelp,
    Resize,
    None,
}

pub fn poll_action(timeout: Duration) -> anyhow::Result<Action> {
    if !event::poll(timeout)? {
        return Ok(Action::None);
    }
    match event::read()? {
        Event::Key(key) => Ok(map_key(key)),
        Event::Resize(_, _) => Ok(Action::Resize),
        _ => Ok(Action::None),
    }
}

fn map_key(key: KeyEvent) -> Action {
    // Skip key-release events when the terminal reports them.
    if key.kind == KeyEventKind::Release {
        return Action::None;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
    {
        return Action::Quit;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Action::Quit,
        KeyCode::Char(' ') => Action::TogglePause,
        KeyCode::Char('+') | KeyCode::Char('=') => Action::SpeedUp,
        KeyCode::Char('-') | KeyCode::Char('_') => Action::SpeedDown,
        KeyCode::Char(']') => Action::DensityUp,
        KeyCode::Char('[') => Action::DensityDown,
        KeyCode::Char('c') | KeyCode::Char('C') => Action::CyclePalette,
        KeyCode::Char('p') | KeyCode::Char('P') => Action::CyclePreset,
        KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Char('H') => Action::ToggleHelp,
        _ => Action::None,
    }
}
