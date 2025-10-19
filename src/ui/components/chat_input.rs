use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::ui::input_handler::TextInputState;
use crate::ui::text_input::TextInput;

pub struct ChatInput<'a> {
    input_state: &'a TextInputState,
    is_focused: bool,
    is_composing: bool,
    room_name: Option<&'a str>,
    typing_users: &'a [String],
    max_lines: usize,
}

impl<'a> ChatInput<'a> {
    pub fn new(input_state: &'a TextInputState) -> Self {
        Self {
            input_state,
            is_focused: false,
            is_composing: false,
            room_name: None,
            typing_users: &[],
            max_lines: 5,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    pub fn composing(mut self, composing: bool) -> Self {
        self.is_composing = composing;
        self
    }

    pub fn room_name(mut self, room_name: Option<&'a str>) -> Self {
        self.room_name = room_name;
        self
    }

    pub fn typing_users(mut self, typing_users: &'a [String]) -> Self {
        self.typing_users = typing_users;
        self
    }

    pub fn max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = max_lines;
        self
    }

    fn get_title(&self) -> String {
        let base_title = if let Some(room) = self.room_name {
            format!("Message to {}", room)
        } else {
            "Compose Message".to_string()
        };

        if self.is_composing {
            format!("{} [Composing...]", base_title)
        } else {
            base_title
        }
    }

    fn get_placeholder(&self) -> &'static str {
        if self.is_composing {
            "Type your message (Enter to send, Esc to cancel)..."
        } else {
            "Press Enter to start typing, Tab to switch focus"
        }
    }

    fn format_typing_indicator(&self) -> Option<Line> {
        if self.typing_users.is_empty() {
            return None;
        }

        let typing_text = match self.typing_users.len() {
            1 => format!("{} is typing...", self.typing_users[0]),
            2 => format!(
                "{} and {} are typing...",
                self.typing_users[0], self.typing_users[1]
            ),
            n => format!(
                "{}, {} and {} others are typing...",
                self.typing_users[0],
                self.typing_users[1],
                n - 2
            ),
        };

        Some(Line::from(Span::styled(
            typing_text,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC),
        )))
    }

    fn calculate_input_height(&self, area_width: u16) -> u16 {
        if self.input_state.value.is_empty() {
            return 1;
        }

        let content_width = area_width.saturating_sub(4) as usize; // Account for borders and padding
        if content_width == 0 {
            return 1;
        }

        let lines = self.wrap_text(&self.input_state.value, content_width);
        (lines.len() as u16).min(self.max_lines as u16).max(1)
    }

    fn wrap_text(&self, text: &str, width: usize) -> Vec<String> {
        if width == 0 {
            return vec![text.to_string()];
        }

        let mut lines = Vec::new();
        for line in text.lines() {
            if line.len() <= width {
                lines.push(line.to_string());
            } else {
                // Simple word wrapping
                let mut current_line = String::new();
                for word in line.split_whitespace() {
                    if current_line.is_empty() {
                        current_line = word.to_string();
                    } else if current_line.len() + word.len() + 1 <= width {
                        current_line.push(' ');
                        current_line.push_str(word);
                    } else {
                        lines.push(current_line);
                        current_line = word.to_string();
                    }
                }
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
            }
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }
}

impl<'a> Widget for ChatInput<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate dynamic height for input
        let input_height = self.calculate_input_height(area.width).max(3);
        let typing_height = if self.typing_users.is_empty() { 0 } else { 1 };
        let help_height = 2;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(input_height),
                Constraint::Length(typing_height),
                Constraint::Length(help_height),
                Constraint::Min(0),
            ])
            .split(area);

        // Main input field
        let title = self.get_title();
        let placeholder = self.get_placeholder();

        let text_input = TextInput::new(
            &self.input_state.value,
            self.input_state.cursor_position,
            &title,
            placeholder,
        )
        .focused(self.is_focused)
        .editing(self.is_composing);

        text_input.render(chunks[0], buf);

        // Typing indicator
        if typing_height > 0 {
            if let Some(typing_line) = self.format_typing_indicator() {
                let typing_paragraph = Paragraph::new(typing_line)
                    .block(Block::default().borders(Borders::LEFT | Borders::RIGHT))
                    .wrap(Wrap { trim: true });

                typing_paragraph.render(chunks[1], buf);
            }
        }

        // Help text
        let help_text = if self.is_composing {
            vec![Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" Send • "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" Cancel • "),
                Span::styled("Ctrl+U", Style::default().fg(Color::Yellow)),
                Span::raw(" Clear • "),
                Span::styled("Ctrl+W", Style::default().fg(Color::Yellow)),
                Span::raw(" Delete word"),
            ])]
        } else {
            vec![Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" Start typing • "),
                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                Span::raw(" Switch focus • "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" Back to rooms"),
            ])]
        };

        let help_paragraph = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Controls")
                    .title_style(Style::default().fg(Color::Green)),
            )
            .style(Style::default().fg(Color::DarkGray))
            .wrap(Wrap { trim: true });

        help_paragraph.render(chunks[2], buf);
    }
}

/// Extended chat input with additional features
pub struct ExtendedChatInput<'a> {
    base: ChatInput<'a>,
    show_character_count: bool,
    max_characters: Option<usize>,
    show_send_button: bool,
    draft_indicator: bool,
}

impl<'a> ExtendedChatInput<'a> {
    pub fn new(input_state: &'a TextInputState) -> Self {
        Self {
            base: ChatInput::new(input_state),
            show_character_count: false,
            max_characters: None,
            show_send_button: true,
            draft_indicator: false,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.base = self.base.focused(focused);
        self
    }

    pub fn composing(mut self, composing: bool) -> Self {
        self.base = self.base.composing(composing);
        self
    }

    pub fn room_name(mut self, room_name: Option<&'a str>) -> Self {
        self.base = self.base.room_name(room_name);
        self
    }

    pub fn typing_users(mut self, typing_users: &'a [String]) -> Self {
        self.base = self.base.typing_users(typing_users);
        self
    }

    pub fn show_character_count(mut self, show: bool) -> Self {
        self.show_character_count = show;
        self
    }

    pub fn max_characters(mut self, max: Option<usize>) -> Self {
        self.max_characters = max;
        self
    }

    pub fn show_send_button(mut self, show: bool) -> Self {
        self.show_send_button = show;
        self
    }

    pub fn draft_indicator(mut self, has_draft: bool) -> Self {
        self.draft_indicator = has_draft;
        self
    }

    fn get_character_count_text(&self) -> Option<String> {
        if !self.show_character_count {
            return None;
        }

        let current_count = self.base.input_state.value.len();

        if let Some(max) = self.max_characters {
            Some(format!("{}/{}", current_count, max))
        } else {
            Some(current_count.to_string())
        }
    }

    fn is_over_limit(&self) -> bool {
        if let Some(max) = self.max_characters {
            self.base.input_state.value.len() > max
        } else {
            false
        }
    }
}

impl<'a> Widget for ExtendedChatInput<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let show_footer =
            self.show_character_count || self.show_send_button || self.draft_indicator;
        let footer_height = if show_footer { 1 } else { 0 };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(footer_height)])
            .split(area);

        // Get values before rendering base to avoid borrow checker issues
        let input_value = self.base.input_state.value.clone();
        let is_over_limit = self.is_over_limit();
        let count_text = self.get_character_count_text();
        let draft_indicator = self.draft_indicator;
        let _show_character_count = self.show_character_count;
        let show_send_button = self.show_send_button;

        // Render base chat input
        self.base.render(chunks[0], buf);

        // Render footer with additional info
        if show_footer {
            let mut footer_spans = Vec::new();

            // Draft indicator
            if draft_indicator {
                footer_spans.push(Span::styled(
                    "💾 Draft saved",
                    Style::default().fg(Color::Green),
                ));
                footer_spans.push(Span::raw(" • "));
            }

            // Character count
            if let Some(count_text) = count_text {
                let count_color = if is_over_limit {
                    Color::Red
                } else {
                    Color::DarkGray
                };

                footer_spans.push(Span::styled(count_text, Style::default().fg(count_color)));
                footer_spans.push(Span::raw(" chars"));

                if show_send_button {
                    footer_spans.push(Span::raw(" • "));
                }
            }

            // Send button indicator
            if show_send_button {
                let can_send = !input_value.trim().is_empty() && !is_over_limit;
                let button_text = if can_send {
                    "Ready to send"
                } else {
                    "Enter message"
                };
                let button_color = if can_send {
                    Color::Green
                } else {
                    Color::DarkGray
                };

                footer_spans.push(Span::styled(button_text, Style::default().fg(button_color)));
            }

            if !footer_spans.is_empty() {
                let footer_paragraph = Paragraph::new(Line::from(footer_spans))
                    .block(
                        Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM),
                    )
                    .style(Style::default());

                footer_paragraph.render(chunks[1], buf);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_input_creation() {
        let input_state = TextInputState::default();
        let chat_input = ChatInput::new(&input_state);

        assert!(!chat_input.is_focused);
        assert!(!chat_input.is_composing);
        assert_eq!(chat_input.max_lines, 5);
    }

    #[test]
    fn test_text_wrapping() {
        let input_state = TextInputState::default();
        let chat_input = ChatInput::new(&input_state);

        let wrapped = chat_input.wrap_text("This is a very long line that should be wrapped", 10);
        assert!(wrapped.len() > 1);

        for line in &wrapped {
            assert!(line.len() <= 10);
        }
    }

    #[test]
    fn test_extended_chat_input() {
        let input_state = TextInputState::default();
        let extended_input = ExtendedChatInput::new(&input_state)
            .show_character_count(true)
            .max_characters(Some(100))
            .draft_indicator(true);

        assert!(extended_input.show_character_count);
        assert_eq!(extended_input.max_characters, Some(100));
        assert!(extended_input.draft_indicator);
    }

    #[test]
    fn test_character_count() {
        let mut input_state = TextInputState::default();
        input_state.set_value("Hello world".to_string());

        let extended_input = ExtendedChatInput::new(&input_state)
            .show_character_count(true)
            .max_characters(Some(20));

        let count_text = extended_input.get_character_count_text();
        assert_eq!(count_text, Some("11/20".to_string()));
        assert!(!extended_input.is_over_limit());
    }

    #[test]
    fn test_over_limit() {
        let mut input_state = TextInputState::default();
        input_state.set_value("This is a very long message that exceeds the limit".to_string());

        let extended_input = ExtendedChatInput::new(&input_state).max_characters(Some(20));

        assert!(extended_input.is_over_limit());
    }
}
