pub mod login;
pub mod search;

pub use search::{SearchKeyOutcome, SearchMode, SearchModalState};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextInputState {
    pub value: String,
    /// Cursor position in characters, not bytes.
    pub cursor: usize,
    pub mask: bool,
}

impl TextInputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from(value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            cursor: value.chars().count(),
            value,
            mask: false,
        }
    }

    pub fn with_mask(mut self, mask: bool) -> Self {
        self.mask = mask;
        self
    }

    pub fn set(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.value.chars().count();
    }

    pub fn insert(&mut self, ch: char) {
        let byte_idx = self.byte_index_at_cursor();
        self.value.insert(byte_idx, ch);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        let byte_idx = self.byte_index_at_cursor();
        let next_byte_idx = self
            .value
            .char_indices()
            .nth(self.cursor + 1)
            .map(|(idx, _)| idx)
            .unwrap_or(self.value.len());
        self.value.replace_range(byte_idx..next_byte_idx, "");
    }

    pub fn backspace_word(&mut self) {
        while self.cursor > 0 && self.char_before_cursor().is_some_and(|c| c.is_whitespace()) {
            self.backspace();
        }
        while self.cursor > 0 && self.char_before_cursor().is_some_and(|c| !c.is_whitespace()) {
            self.backspace();
        }
    }

    fn char_before_cursor(&self) -> Option<char> {
        if self.cursor == 0 {
            return None;
        }
        self.value.chars().nth(self.cursor - 1)
    }

    pub fn delete(&mut self) {
        if self.cursor >= self.value.chars().count() {
            return;
        }
        let byte_idx = self.byte_index_at_cursor();
        let next_byte_idx = self
            .value
            .char_indices()
            .nth(self.cursor + 1)
            .map(|(idx, _)| idx)
            .unwrap_or(self.value.len());
        self.value.replace_range(byte_idx..next_byte_idx, "");
    }

    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.value.chars().count());
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.value.chars().count();
    }

    pub fn display(&self) -> String {
        if self.mask {
            "*".repeat(self.value.chars().count())
        } else {
            self.value.clone()
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Home => self.move_home(),
            KeyCode::End => self.move_end(),
            KeyCode::Backspace
                if key.modifiers.contains(KeyModifiers::ALT)
                    || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.backspace_word()
            }
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete(),
            KeyCode::Char(ch)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.insert(ch)
            }
            _ => return false,
        }
        true
    }

    fn byte_index_at_cursor(&self) -> usize {
        self.value
            .char_indices()
            .nth(self.cursor)
            .map(|(idx, _)| idx)
            .unwrap_or(self.value.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backspace_word_deletes_previous_word() {
        let mut input = TextInputState::from("hello world foo");
        input.backspace_word();
        assert_eq!(input.value, "hello world ");
        input.backspace_word();
        assert_eq!(input.value, "hello ");
        input.backspace_word();
        assert_eq!(input.value, "");
        input.backspace_word();
        assert_eq!(input.value, "");
    }

    #[test]
    fn backspace_word_handles_mid_cursor_and_unicode() {
        let mut input = TextInputState::from("foo bär baz");
        input.move_left();
        input.move_left();
        input.move_left();
        input.move_left();
        input.backspace_word();
        assert_eq!(input.value, "foo  baz");
    }

    #[test]
    fn handle_key_alt_backspace_invokes_word_delete() {
        let mut input = TextInputState::from("one two");
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::ALT);
        assert!(input.handle_key(key));
        assert_eq!(input.value, "one ");
    }

    #[test]
    fn edits_unicode_safely() {
        let mut input = TextInputState::from("aäb");
        input.move_left();
        input.backspace();
        assert_eq!(input.value, "ab");
        input.insert('ß');
        assert_eq!(input.value, "aßb");
    }
}
