use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct ShortcutDisplay {
    pub id: &'static str,
    pub keys: &'static str,
    pub action: &'static str,
}

#[derive(Debug, Clone)]
pub struct ShortcutSection {
    pub title: &'static str,
    pub items: Vec<ShortcutDisplay>,
}

pub fn char_key(key: KeyEvent, expected: char) -> bool {
    matches!(key.code, KeyCode::Char(value) if value == expected)
}

pub fn plain_char(key: KeyEvent, expected: char) -> bool {
    char_key(key, expected)
        && !key.modifiers.contains(KeyModifiers::CONTROL)
        && !key.modifiers.contains(KeyModifiers::ALT)
}

pub fn ctrl_char(key: KeyEvent, expected: char) -> bool {
    char_key(key, expected) && key.modifiers.contains(KeyModifiers::CONTROL)
}

pub fn shifted_char(key: KeyEvent, expected: char) -> bool {
    char_key(key, expected) && key.modifiers.contains(KeyModifiers::SHIFT)
}

pub fn display(id: &'static str, keys: &'static str, action: &'static str) -> ShortcutDisplay {
    ShortcutDisplay { id, keys, action }
}
