use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::{
    app::AppState,
    screens::{Screen, ScreenHandler},
    ui::{input_handler::TextInputState, text_input::TextInput},
};

#[derive(Debug)]
pub struct ChatScreen {
    pub message_input: TextInputState,
    pub messages: Vec<ChatMessage>,
    pub is_composing: bool,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender: String,
    pub content: String,
    pub timestamp: String,
}

impl Default for ChatScreen {
    fn default() -> Self {
        Self {
            message_input: TextInputState::default(),
            messages: vec![
                ChatMessage {
                    sender: "System".to_string(),
                    content: "Welcome to the Matrix chat! Type a message and press Enter to send."
                        .to_string(),
                    timestamp: "12:00".to_string(),
                },
                ChatMessage {
                    sender: "Alice".to_string(),
                    content: "Hello everyone! 👋".to_string(),
                    timestamp: "12:01".to_string(),
                },
                ChatMessage {
                    sender: "Bob".to_string(),
                    content: "Hey Alice! How's it going?".to_string(),
                    timestamp: "12:02".to_string(),
                },
            ],
            is_composing: false,
        }
    }
}

impl ChatScreen {
    pub fn add_message(&mut self, sender: String, content: String) {
        let timestamp = chrono::Local::now().format("%H:%M").to_string();
        self.messages.push(ChatMessage {
            sender,
            content,
            timestamp,
        });
    }

    pub fn send_current_message(&mut self) {
        if !self.message_input.is_empty() {
            let content = self.message_input.value.clone();
            self.add_message("You".to_string(), content);
            self.message_input.clear();
            self.is_composing = false;
        }
    }

    fn render_messages(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .messages
            .iter()
            .map(|msg| {
                let sender_style = match msg.sender.as_str() {
                    "System" => Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::ITALIC),
                    "You" => Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                    _ => Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                };

                let content = Line::from(vec![
                    Span::styled(
                        format!("[{}] ", msg.timestamp),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(&msg.sender, sender_style),
                    Span::raw(": "),
                    Span::styled(&msg.content, Style::default().fg(Color::White)),
                ]);

                ListItem::new(content)
            })
            .collect();

        let messages_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Messages")
                    .title_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(messages_list, area);
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let input_title = if self.is_composing {
            "Type your message (Enter to send, Esc to cancel)"
        } else {
            "Press Enter to start typing, Esc to go back to login"
        };

        let text_input = if self.is_composing {
            TextInput::editable(
                &self.message_input.value,
                self.message_input.cursor_position,
                input_title,
                "Type your message here...",
            )
        } else {
            TextInput::new(
                &self.message_input.value,
                self.message_input.cursor_position,
                input_title,
                "Press Enter to start typing",
            )
            .focused(true)
        };

        frame.render_widget(text_input, area);
    }

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let help_text = if self.is_composing {
            vec![
                Line::from("🎯 Composing Mode"),
                Line::from(""),
                Line::from("• Enter - Send message"),
                Line::from("• Esc - Cancel message"),
                Line::from("• Ctrl+A - Move to start"),
                Line::from("• Ctrl+E - Move to end"),
                Line::from("• Ctrl+U - Clear to start"),
                Line::from("• Ctrl+K - Clear to end"),
                Line::from("• Ctrl+W - Delete word"),
            ]
        } else {
            vec![
                Line::from("💬 Chat Mode"),
                Line::from(""),
                Line::from("• Enter - Start typing"),
                Line::from("• Esc - Back to login"),
                Line::from("• Ctrl+C - Quit app"),
                Line::from(""),
                Line::from("This demonstrates reusable"),
                Line::from("text input components!"),
            ]
        };

        let help_paragraph = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help")
                    .title_style(Style::default().fg(Color::Green)),
            )
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: true });

        frame.render_widget(help_paragraph, area);
    }
}

impl ScreenHandler for ChatScreen {
    fn render(&self, frame: &mut Frame, _app_state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),    // Messages area
                Constraint::Length(3),  // Input area
                Constraint::Length(11), // Help area
            ])
            .split(frame.area());

        // Create horizontal layout for messages and help
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70), // Messages
                Constraint::Percentage(30), // Help
            ])
            .split(chunks[0]);

        self.render_messages(frame, top_chunks[0]);
        self.render_help(frame, top_chunks[1]);
        self.render_input(frame, chunks[1]);

        // Status bar
        let status_text = if self.is_composing {
            "Composing message..."
        } else {
            "Ready to chat"
        };

        let status = Paragraph::new(Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Yellow)),
            Span::styled(status_text, Style::default().fg(Color::White)),
            Span::raw("  |  "),
            Span::styled(
                format!("Messages: {}", self.messages.len()),
                Style::default().fg(Color::Cyan),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Status")
                .title_style(Style::default().fg(Color::Magenta)),
        );

        frame.render_widget(status, chunks[2]);
    }

    fn handle_key_event(&self, key: KeyEvent, app_state: &mut AppState) -> Option<Screen> {
        // This is a placeholder implementation since ChatScreen is not part of AppState yet
        // In a real implementation, you would have a way to access the ChatScreen state
        // through AppState or a separate state management system

        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => None,
            KeyCode::Esc => {
                if self.is_composing {
                    // Cancel composing - this would need to modify ChatScreen state
                    Some(Screen::Chat)
                } else {
                    // Go back to login
                    Some(Screen::Login)
                }
            }
            KeyCode::Enter => {
                if self.is_composing {
                    // Send message - this would need to modify ChatScreen state
                    Some(Screen::Chat)
                } else {
                    // Start composing - this would need to modify ChatScreen state
                    Some(Screen::Chat)
                }
            }
            _ => {
                if self.is_composing {
                    // Handle text input - this would use the TextInputState
                    // let mut temp_input = self.message_input.clone();
                    // temp_input.handle_key_event(key);
                    // Update the actual state...
                }
                Some(Screen::Chat)
            }
        }
    }
}

// Example of how you might extend AppState to include chat functionality
pub trait ChatStateProvider {
    fn get_chat_screen_mut(&mut self) -> &mut ChatScreen;
    fn get_chat_screen(&self) -> &ChatScreen;
}

// You could implement this for AppState by adding a chat_screen field:
/*
impl ChatStateProvider for AppState {
    fn get_chat_screen_mut(&mut self) -> &mut ChatScreen {
        &mut self.chat_screen
    }

    fn get_chat_screen(&self) -> &ChatScreen {
        &self.chat_screen
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_screen_creation() {
        let chat = ChatScreen::default();
        assert!(!chat.is_composing);
        assert!(!chat.messages.is_empty());
        assert!(chat.message_input.is_empty());
    }

    #[test]
    fn test_add_message() {
        let mut chat = ChatScreen::default();
        let initial_count = chat.messages.len();

        chat.add_message("Test User".to_string(), "Test message".to_string());
        assert_eq!(chat.messages.len(), initial_count + 1);

        let last_message = chat.messages.last().unwrap();
        assert_eq!(last_message.sender, "Test User");
        assert_eq!(last_message.content, "Test message");
    }

    #[test]
    fn test_send_message() {
        let mut chat = ChatScreen::default();
        chat.message_input.set_value("Hello world!".to_string());
        chat.is_composing = true;

        let initial_count = chat.messages.len();
        chat.send_current_message();

        assert_eq!(chat.messages.len(), initial_count + 1);
        assert!(chat.message_input.is_empty());
        assert!(!chat.is_composing);
    }
}
