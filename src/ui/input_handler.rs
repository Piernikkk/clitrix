use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Generic text input handler that can be used across different screens
#[derive(Debug, Clone)]
pub struct TextInputState {
    pub value: String,
    pub cursor_position: usize,
}

impl Default for TextInputState {
    fn default() -> Self {
        Self {
            value: String::new(),
            cursor_position: 0,
        }
    }
}

impl TextInputState {
    pub fn new(value: String) -> Self {
        let cursor_position = value.len();
        Self {
            value,
            cursor_position,
        }
    }

    pub fn with_cursor(value: String, cursor_position: usize) -> Self {
        let cursor_position = cursor_position.min(value.len());
        Self {
            value,
            cursor_position,
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.value.len() {
            self.cursor_position += 1;
        }
    }

    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    pub fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.value.len();
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

    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.value.len() {
            self.value.remove(self.cursor_position);
        }
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_position = 0;
    }

    pub fn set_value(&mut self, value: String) {
        self.value = value;
        self.cursor_position = self.value.len();
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Handle common text input key events
    /// Returns true if the key was handled, false otherwise
    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'a' => {
                            self.move_cursor_to_start();
                            true
                        }
                        'e' => {
                            self.move_cursor_to_end();
                            true
                        }
                        'u' => {
                            // Clear from cursor to beginning
                            self.value.drain(..self.cursor_position);
                            self.cursor_position = 0;
                            true
                        }
                        'k' => {
                            // Clear from cursor to end
                            self.value.truncate(self.cursor_position);
                            true
                        }
                        'w' => {
                            // Delete word backwards
                            self.delete_word_backwards();
                            true
                        }
                        _ => false,
                    }
                } else {
                    self.enter_char(c);
                    true
                }
            }
            KeyCode::Backspace => {
                self.delete_char();
                true
            }
            KeyCode::Delete => {
                self.delete_char_forward();
                true
            }
            KeyCode::Left => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_cursor_word_left();
                } else {
                    self.move_cursor_left();
                }
                true
            }
            KeyCode::Right => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_cursor_word_right();
                } else {
                    self.move_cursor_right();
                }
                true
            }
            KeyCode::Home => {
                self.move_cursor_to_start();
                true
            }
            KeyCode::End => {
                self.move_cursor_to_end();
                true
            }
            _ => false,
        }
    }

    fn delete_word_backwards(&mut self) {
        let original_pos = self.cursor_position;

        // Move cursor to beginning of current word
        while self.cursor_position > 0 {
            let prev_char = self
                .value
                .chars()
                .nth(self.cursor_position - 1)
                .unwrap_or(' ');
            if prev_char.is_whitespace() {
                break;
            }
            self.cursor_position -= 1;
        }

        // Skip whitespace
        while self.cursor_position > 0 {
            let prev_char = self
                .value
                .chars()
                .nth(self.cursor_position - 1)
                .unwrap_or(' ');
            if !prev_char.is_whitespace() {
                break;
            }
            self.cursor_position -= 1;
        }

        // Delete from current position to original position
        self.value.drain(self.cursor_position..original_pos);
    }

    fn move_cursor_word_left(&mut self) {
        while self.cursor_position > 0 {
            self.cursor_position -= 1;
            let char_at_cursor = self.value.chars().nth(self.cursor_position).unwrap_or(' ');
            if char_at_cursor.is_whitespace() {
                break;
            }
        }
    }

    fn move_cursor_word_right(&mut self) {
        while self.cursor_position < self.value.len() {
            let char_at_cursor = self.value.chars().nth(self.cursor_position).unwrap_or(' ');
            self.cursor_position += 1;
            if char_at_cursor.is_whitespace() {
                break;
            }
        }
    }
}

/// Multi-field form handler for managing multiple text inputs
#[derive(Debug, Clone)]
pub struct FormHandler<T> {
    pub fields: Vec<TextInputState>,
    pub active_field: T,
    pub editing: bool,
}

impl<T> FormHandler<T>
where
    T: Clone + PartialEq,
{
    pub fn new(fields: Vec<TextInputState>, initial_field: T) -> Self {
        Self {
            fields,
            active_field: initial_field,
            editing: false,
        }
    }

    pub fn get_active_field_mut(&mut self, field_index: usize) -> Option<&mut TextInputState> {
        self.fields.get_mut(field_index)
    }

    pub fn get_active_field(&self, field_index: usize) -> Option<&TextInputState> {
        self.fields.get(field_index)
    }

    pub fn start_editing(&mut self) {
        self.editing = true;
    }

    pub fn stop_editing(&mut self) {
        self.editing = false;
    }

    pub fn is_editing(&self) -> bool {
        self.editing
    }

    pub fn clear_all(&mut self) {
        for field in &mut self.fields {
            field.clear();
        }
        self.editing = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_text_input_state_creation() {
        let state = TextInputState::new("hello".to_string());
        assert_eq!(state.value, "hello");
        assert_eq!(state.cursor_position, 5);
    }

    #[test]
    fn test_cursor_movement() {
        let mut state = TextInputState::new("hello".to_string());

        state.move_cursor_left();
        assert_eq!(state.cursor_position, 4);

        state.move_cursor_right();
        assert_eq!(state.cursor_position, 5);

        state.move_cursor_to_start();
        assert_eq!(state.cursor_position, 0);

        state.move_cursor_to_end();
        assert_eq!(state.cursor_position, 5);
    }

    #[test]
    fn test_char_operations() {
        let mut state = TextInputState::new("hello".to_string());
        state.cursor_position = 2;

        state.enter_char('x');
        assert_eq!(state.value, "hexllo");
        assert_eq!(state.cursor_position, 3);

        state.delete_char();
        assert_eq!(state.value, "hello");
        assert_eq!(state.cursor_position, 2);
    }

    #[test]
    fn test_key_event_handling() {
        let mut state = TextInputState::new("test".to_string());
        state.cursor_position = 2;

        // Test character input
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(state.handle_key_event(key));
        assert_eq!(state.value, "texto");

        // Test backspace
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(state.handle_key_event(key));
        assert_eq!(state.value, "test");

        // Test ctrl+a (move to start)
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        assert!(state.handle_key_event(key));
        assert_eq!(state.cursor_position, 0);
    }

    #[test]
    fn test_form_handler() {
        #[derive(Clone, PartialEq)]
        enum Field {
            Username,
            Password,
        }

        let fields = vec![
            TextInputState::new("user".to_string()),
            TextInputState::new("pass".to_string()),
        ];

        let mut form = FormHandler::new(fields, Field::Username);

        assert!(!form.is_editing());
        form.start_editing();
        assert!(form.is_editing());

        let username_field = form.get_active_field(0).unwrap();
        assert_eq!(username_field.value, "user");
    }
}
