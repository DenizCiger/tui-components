use crate::input::TextInputState;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginKeyOutcome {
    None,
    Edited,
    FocusChanged,
    Submit,
    SavedLogin,
    TogglePassword,
    Quit,
}

#[derive(Debug, Clone, Copy)]
pub struct LoginKeyBindings {
    pub submit_on_password: bool,
}

impl Default for LoginKeyBindings {
    fn default() -> Self {
        Self {
            submit_on_password: true,
        }
    }
}

pub fn handle_login_key<F, Next, Prev>(
    key: KeyEvent,
    focus: &mut F,
    password_focus: F,
    submit_focus: F,
    current_input: Option<&mut TextInputState>,
    next_focus: Next,
    prev_focus: Prev,
    bindings: LoginKeyBindings,
) -> LoginKeyOutcome
where
    F: Copy + PartialEq,
    Next: Fn(F) -> F,
    Prev: Fn(F) -> F,
{
    match key.code {
        KeyCode::Esc => return LoginKeyOutcome::Quit,
        KeyCode::Tab => {
            *focus = if key.modifiers.contains(KeyModifiers::SHIFT) {
                prev_focus(*focus)
            } else {
                next_focus(*focus)
            };
            return LoginKeyOutcome::FocusChanged;
        }
        KeyCode::BackTab | KeyCode::Up => {
            *focus = prev_focus(*focus);
            return LoginKeyOutcome::FocusChanged;
        }
        KeyCode::Down => {
            *focus = next_focus(*focus);
            return LoginKeyOutcome::FocusChanged;
        }
        KeyCode::Char('v') | KeyCode::Char('V') if key.modifiers.contains(KeyModifiers::ALT) => {
            return LoginKeyOutcome::TogglePassword;
        }
        KeyCode::Char('l') | KeyCode::Char('L') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return LoginKeyOutcome::SavedLogin;
        }
        KeyCode::Enter => {
            if *focus == submit_focus || (bindings.submit_on_password && *focus == password_focus) {
                return LoginKeyOutcome::Submit;
            }
            *focus = next_focus(*focus);
            return LoginKeyOutcome::FocusChanged;
        }
        _ => {}
    }

    if *focus == submit_focus {
        return LoginKeyOutcome::None;
    }

    if let Some(input) = current_input {
        if input.handle_key(key) {
            return LoginKeyOutcome::Edited;
        }
    }

    LoginKeyOutcome::None
}
