use crate::input::TextInputState;
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// Filter narrows live; edits reset selection to 0.
    Live,
    /// Query commits on Enter; selection is independent of input edits.
    Deferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchKeyOutcome {
    None,
    Edited,
    Moved,
    Submit,
    Cancel,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchModalState {
    pub input: TextInputState,
    pub selected: usize,
}

impl SearchModalState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_query(query: &str) -> Self {
        Self {
            input: TextInputState::from(query),
            selected: 0,
        }
    }

    pub fn reset(&mut self) {
        self.input = TextInputState::default();
        self.selected = 0;
    }

    pub fn clamp(&mut self, max: usize) {
        if max == 0 {
            self.selected = 0;
        } else if self.selected >= max {
            self.selected = max - 1;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, max: usize, mode: SearchMode) -> SearchKeyOutcome {
        match key.code {
            KeyCode::Esc => SearchKeyOutcome::Cancel,
            KeyCode::Enter => SearchKeyOutcome::Submit,
            KeyCode::Up => {
                if self.selected == 0 {
                    SearchKeyOutcome::None
                } else {
                    self.selected -= 1;
                    SearchKeyOutcome::Moved
                }
            }
            KeyCode::Down => {
                if max == 0 || self.selected + 1 >= max {
                    SearchKeyOutcome::None
                } else {
                    self.selected += 1;
                    SearchKeyOutcome::Moved
                }
            }
            _ => {
                if self.input.handle_key(key) {
                    if matches!(mode, SearchMode::Live) {
                        self.selected = 0;
                    }
                    SearchKeyOutcome::Edited
                } else {
                    SearchKeyOutcome::None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn esc_cancels_and_enter_submits() {
        let mut s = SearchModalState::new();
        assert_eq!(s.handle_key(key(KeyCode::Esc), 5, SearchMode::Live), SearchKeyOutcome::Cancel);
        assert_eq!(s.handle_key(key(KeyCode::Enter), 5, SearchMode::Deferred), SearchKeyOutcome::Submit);
    }

    #[test]
    fn up_down_move_with_clamp() {
        let mut s = SearchModalState::new();
        assert_eq!(s.handle_key(key(KeyCode::Up), 3, SearchMode::Live), SearchKeyOutcome::None);
        assert_eq!(s.handle_key(key(KeyCode::Down), 3, SearchMode::Live), SearchKeyOutcome::Moved);
        assert_eq!(s.selected, 1);
        assert_eq!(s.handle_key(key(KeyCode::Down), 3, SearchMode::Live), SearchKeyOutcome::Moved);
        assert_eq!(s.handle_key(key(KeyCode::Down), 3, SearchMode::Live), SearchKeyOutcome::None);
        assert_eq!(s.selected, 2);
    }

    #[test]
    fn down_with_zero_max_no_op() {
        let mut s = SearchModalState::new();
        assert_eq!(s.handle_key(key(KeyCode::Down), 0, SearchMode::Live), SearchKeyOutcome::None);
        assert_eq!(s.selected, 0);
    }

    #[test]
    fn live_edit_resets_selection() {
        let mut s = SearchModalState::new();
        s.selected = 3;
        let outcome = s.handle_key(key(KeyCode::Char('a')), 10, SearchMode::Live);
        assert_eq!(outcome, SearchKeyOutcome::Edited);
        assert_eq!(s.selected, 0);
        assert_eq!(s.input.value, "a");
    }

    #[test]
    fn deferred_edit_preserves_selection() {
        let mut s = SearchModalState::new();
        s.selected = 3;
        let outcome = s.handle_key(key(KeyCode::Char('a')), 10, SearchMode::Deferred);
        assert_eq!(outcome, SearchKeyOutcome::Edited);
        assert_eq!(s.selected, 3);
    }

    #[test]
    fn alt_backspace_word_deletes_via_input() {
        let mut s = SearchModalState::from_query("foo bar");
        let outcome = s.handle_key(
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::ALT),
            10,
            SearchMode::Live,
        );
        assert_eq!(outcome, SearchKeyOutcome::Edited);
        assert_eq!(s.input.value, "foo ");
    }

    #[test]
    fn clamp_handles_shrinking_list() {
        let mut s = SearchModalState::new();
        s.selected = 5;
        s.clamp(3);
        assert_eq!(s.selected, 2);
        s.clamp(0);
        assert_eq!(s.selected, 0);
    }
}
