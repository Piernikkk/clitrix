use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct TextInput<'a> {
    value: &'a str,
    cursor_position: usize,
    is_focused: bool,
    is_editing: bool,
    title: &'a str,
    placeholder: &'a str,
    is_password: bool,
}

impl<'a> TextInput<'a> {
    pub fn new(
        value: &'a str,
        cursor_position: usize,
        title: &'a str,
        placeholder: &'a str,
    ) -> Self {
        Self {
            value,
            cursor_position,
            is_focused: false,
            is_editing: false,
            title,
            placeholder,
            is_password: false,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    pub fn editing(mut self, editing: bool) -> Self {
        self.is_editing = editing;
        self
    }

    pub fn password(mut self, is_password: bool) -> Self {
        self.is_password = is_password;
        self
    }

    fn render_input_text(&self) -> Line<'static> {
        if self.value.is_empty() && !self.is_editing {
            // Show placeholder when empty and not editing
            Line::from(Span::styled(
                self.placeholder.to_string(),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ))
        } else if !self.is_editing || !self.is_focused {
            // Show value without cursor when not editing or not focused
            let display_value = if self.is_password && !self.value.is_empty() {
                "*".repeat(self.value.len())
            } else {
                self.value.to_string()
            };
            Line::from(Span::styled(
                display_value,
                Style::default().fg(Color::White),
            ))
        } else {
            // Show value with cursor when editing and focused
            let mut spans = Vec::new();

            if self.cursor_position == 0 {
                // Cursor at the beginning
                spans.push(Span::styled(
                    "▌".to_string(),
                    Style::default().fg(Color::Yellow),
                ));
                if !self.value.is_empty() {
                    let display_value = if self.is_password {
                        "*".repeat(self.value.len())
                    } else {
                        self.value.to_string()
                    };
                    spans.push(Span::styled(
                        display_value,
                        Style::default().fg(Color::White),
                    ));
                }
            } else if self.cursor_position >= self.value.len() {
                // Cursor at the end
                if !self.value.is_empty() {
                    let display_value = if self.is_password {
                        "*".repeat(self.value.len())
                    } else {
                        self.value.to_string()
                    };
                    spans.push(Span::styled(
                        display_value,
                        Style::default().fg(Color::White),
                    ));
                }
                spans.push(Span::styled(
                    "▌".to_string(),
                    Style::default().fg(Color::Yellow),
                ));
            } else if !self.is_password {
                // Cursor in the middle (only for non-password fields)
                let before_cursor = self.value[..self.cursor_position].to_string();
                let at_cursor = self.value.chars().nth(self.cursor_position).unwrap_or(' ');
                let after_cursor = self.value[self.cursor_position + 1..].to_string();

                spans.push(Span::styled(
                    before_cursor,
                    Style::default().fg(Color::White),
                ));
                spans.push(Span::styled(
                    at_cursor.to_string(),
                    Style::default().bg(Color::Yellow).fg(Color::Black),
                ));
                spans.push(Span::styled(
                    after_cursor,
                    Style::default().fg(Color::White),
                ));
            } else {
                // For password fields, just show masked text with cursor at end
                let display_value = "*".repeat(self.value.len());
                spans.push(Span::styled(
                    display_value,
                    Style::default().fg(Color::White),
                ));
                spans.push(Span::styled(
                    "▌".to_string(),
                    Style::default().fg(Color::Yellow),
                ));
            }

            Line::from(spans)
        }
    }

    fn get_border_style(&self) -> Style {
        if self.is_editing && self.is_focused {
            Style::default().fg(Color::Yellow)
        } else if self.is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        }
    }
}

impl<'a> Widget for TextInput<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let input_text = self.render_input_text();
        let border_style = self.get_border_style();

        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title)
            .border_style(border_style);

        let paragraph = Paragraph::new(input_text).block(block);

        paragraph.render(area, buf);
    }
}

// Utility functions for common TextInput patterns
impl<'a> TextInput<'a> {
    /// Create a basic text input that's ready for editing
    pub fn editable(
        value: &'a str,
        cursor_position: usize,
        title: &'a str,
        placeholder: &'a str,
    ) -> Self {
        Self::new(value, cursor_position, title, placeholder)
            .focused(true)
            .editing(true)
    }

    /// Create a password input field
    pub fn password_field(
        value: &'a str,
        cursor_position: usize,
        title: &'a str,
        placeholder: &'a str,
    ) -> Self {
        Self::new(value, cursor_position, title, placeholder)
            .focused(true)
            .editing(true)
            .password(true)
    }

    /// Create a readonly display field
    pub fn readonly(value: &'a str, title: &'a str) -> Self {
        Self::new(value, 0, title, "").focused(false).editing(false)
    }
}

/// Example usage patterns for the TextInput component
///
/// ```rust
/// // Basic usage in a form
/// let username_input = TextInput::new(&form.username, form.username_cursor, "Username", "Enter username")
///     .focused(current_field == Field::Username)
///     .editing(editing_mode);
///
/// // Password field
/// let password_input = TextInput::password_field(&form.password, form.password_cursor, "Password", "Enter password");
///
/// // Readonly display
/// let display_input = TextInput::readonly(&user.email, "Email Address");
///
/// // Editable field with focus
/// let editable_input = TextInput::editable(&form.message, form.message_cursor, "Message", "Type your message");
/// ```

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input_creation() {
        let input = TextInput::new("test", 0, "Title", "Placeholder");
        assert_eq!(input.value, "test");
        assert_eq!(input.cursor_position, 0);
        assert_eq!(input.title, "Title");
        assert_eq!(input.placeholder, "Placeholder");
        assert!(!input.is_focused);
        assert!(!input.is_editing);
        assert!(!input.is_password);
    }

    #[test]
    fn test_text_input_builder_pattern() {
        let input = TextInput::new("test", 0, "Title", "Placeholder")
            .focused(true)
            .editing(true)
            .password(true);

        assert!(input.is_focused);
        assert!(input.is_editing);
        assert!(input.is_password);
    }

    #[test]
    fn test_password_field_utility() {
        let input = TextInput::password_field("secret", 6, "Password", "Enter password");
        assert!(input.is_focused);
        assert!(input.is_editing);
        assert!(input.is_password);
        assert_eq!(input.value, "secret");
        assert_eq!(input.cursor_position, 6);
    }

    #[test]
    fn test_readonly_utility() {
        let input = TextInput::readonly("display_value", "Display");
        assert!(!input.is_focused);
        assert!(!input.is_editing);
        assert!(!input.is_password);
        assert_eq!(input.value, "display_value");
        assert_eq!(input.cursor_position, 0);
    }

    #[test]
    fn test_editable_utility() {
        let input = TextInput::editable("content", 4, "Edit", "Placeholder");
        assert!(input.is_focused);
        assert!(input.is_editing);
        assert!(!input.is_password);
        assert_eq!(input.cursor_position, 4);
    }
}
