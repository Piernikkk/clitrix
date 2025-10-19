use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug)]
pub struct TextInputState {
    pub value: String,
    pub cursor_position: usize,
    pub is_focused: bool,
}

impl Default for TextInputState {
    fn default() -> Self {
        Self {
            value: String::new(),
            cursor_position: 0,
            is_focused: false,
        }
    }
}

impl TextInputState {
    pub fn new(value: String, is_focused: bool) -> Self {
        let cursor_position = value.chars().count();
        Self {
            value,
            cursor_position,
            is_focused,
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.value.chars().count() {
            self.cursor_position += 1;
        }
    }

    pub fn enter_char(&mut self, c: char) {
        self.value.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.value.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_position = 0;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(char) => {
                self.enter_char(char);
                true
            }
            KeyCode::Left => {
                self.move_cursor_left();
                true
            }
            KeyCode::Right => {
                self.move_cursor_right();
                true
            }
            KeyCode::Backspace => {
                if key.modifiers.contains(KeyModifiers::ALT) {
                    self.clear();
                    return true;
                }
                self.delete_char();
                true
            }
            _ => false,
        }
    }
}
