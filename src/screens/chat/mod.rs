use async_trait::async_trait;
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
};

use crate::{
    app::AppState,
    models::{message::Message, room::Room},
    screens::{Screen, ScreenHandler},
    ui::text_input::{TextInput, input_handler::TextInputState},
};

struct Colors {
    border: Color,
    selected_room: Color,
    unselected_room: Color,
    own_message: Color,
    other_message: Color,
    timestamp: Color,
    controls_title: Color,
    error_msg: Color,
}

const COLORS: Colors = Colors {
    border: Color::Cyan,
    selected_room: Color::Yellow,
    unselected_room: Color::White,
    own_message: Color::Green,
    other_message: Color::Blue,
    timestamp: Color::DarkGray,
    controls_title: Color::Green,
    error_msg: Color::Red,
};

pub struct ChatScreen;

#[derive(Debug)]
pub struct ChatScreenState {
    pub rooms: Vec<Room>,
    pub selected_room_index: usize,
    pub messages: Vec<Message>,
    pub message_input: TextInputState,
    pub message_scroll_offset: usize,
    pub room_list_scroll_offset: usize,
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub input_focused: bool,
}

impl Default for ChatScreenState {
    fn default() -> Self {
        Self {
            rooms: Vec::new(),
            selected_room_index: 0,
            messages: Vec::new(),
            message_input: TextInputState::new(String::new(), true),
            message_scroll_offset: 0,
            room_list_scroll_offset: 0,
            is_loading: false,
            error_message: None,
            input_focused: true,
        }
    }
}

impl ChatScreenState {
    pub fn get_selected_room(&self) -> Option<&Room> {
        self.rooms.get(self.selected_room_index)
    }

    pub fn select_next_room(&mut self) {
        if !self.rooms.is_empty() {
            self.selected_room_index = (self.selected_room_index + 1) % self.rooms.len();
        }
    }

    pub fn select_previous_room(&mut self) {
        if !self.rooms.is_empty() {
            if self.selected_room_index == 0 {
                self.selected_room_index = self.rooms.len() - 1;
            } else {
                self.selected_room_index -= 1;
            }
        }
    }

    pub fn scroll_messages_up(&mut self) {
        if self.message_scroll_offset > 0 {
            self.message_scroll_offset -= 1;
        }
    }

    pub fn scroll_messages_down(&mut self, max_offset: usize) {
        if self.message_scroll_offset < max_offset {
            self.message_scroll_offset += 1;
        }
    }
}

impl ChatScreen {
    pub fn new() -> Self {
        Self
    }

    fn render_room_list(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let room_items: Vec<ListItem> = state
            .chat_screen
            .rooms
            .iter()
            .enumerate()
            .map(|(idx, room)| {
                let is_selected = idx == state.chat_screen.selected_room_index;
                let style = if is_selected {
                    Style::default()
                        .fg(COLORS.selected_room)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(COLORS.unselected_room)
                };

                let prefix = if is_selected { "> " } else { "  " };
                let unread_indicator = if room.unread_count > 0 {
                    format!(" ({})", room.unread_count)
                } else {
                    String::new()
                };

                let display_name = room.get_display_name();
                let line = Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(display_name, style),
                    Span::styled(unread_indicator, Style::default().fg(Color::Red)),
                ]);

                ListItem::new(line)
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Rooms")
            .title_style(
                Style::default()
                    .fg(COLORS.border)
                    .add_modifier(Modifier::BOLD),
            );

        let list = List::new(room_items).block(block);

        frame.render_widget(list, area);
    }

    fn render_messages(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let title = if let Some(room) = state.chat_screen.get_selected_room() {
            format!("Chat - {}", room.get_display_name())
        } else {
            "Chat".to_string()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(
                Style::default()
                    .fg(COLORS.border)
                    .add_modifier(Modifier::BOLD),
            );

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if state.chat_screen.messages.is_empty() {
            let no_messages = Paragraph::new("No messages yet. Start chatting!")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(no_messages, inner_area);
            return;
        }

        let visible_height = inner_area.height as usize;
        let total_messages = state.chat_screen.messages.len();
        let scroll_offset = state.chat_screen.message_scroll_offset;

        let start_idx = scroll_offset;
        let end_idx = (scroll_offset + visible_height).min(total_messages);

        let mut lines = Vec::new();

        for message in &state.chat_screen.messages[start_idx..end_idx] {
            let sender_style = if message.is_own_message {
                Style::default().fg(COLORS.own_message)
            } else {
                Style::default().fg(COLORS.other_message)
            };

            let timestamp = message.timestamp.format("%H:%M:%S").to_string();
            let sender_name = message.get_sender_name();
            let content = message.get_text_content();

            lines.push(Line::from(vec![
                Span::styled(
                    format!("[{}] ", timestamp),
                    Style::default().fg(COLORS.timestamp),
                ),
                Span::styled(
                    format!("{}: ", sender_name),
                    sender_style.add_modifier(Modifier::BOLD),
                ),
                Span::raw(content),
            ]));
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner_area);

        // Render scrollbar if needed
        if total_messages > visible_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state =
                ScrollbarState::new(total_messages.saturating_sub(visible_height))
                    .position(scroll_offset);

            frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        }
    }

    fn render_message_input(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let input = TextInput::new(
            &state.chat_screen.message_input.value,
            state.chat_screen.message_input.cursor_position,
            "Message",
            "Type your message...",
            false,
            state.chat_screen.input_focused,
        );

        frame.render_widget(input, area);
    }

    fn render_controls(&self, frame: &mut Frame, area: Rect, _state: &AppState) {
        let instructions = vec![
            Line::from(
                "↑/↓ - Select room | PgUp/PgDn - Scroll messages | Tab - Toggle input focus",
            ),
            Line::from("Enter - Send message | ESC - Exit"),
        ];

        let instructions_paragraph = Paragraph::new(instructions)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Controls")
                    .title_style(Style::default().fg(COLORS.controls_title)),
            )
            .style(Style::default().fg(Color::Gray));

        frame.render_widget(instructions_paragraph, area);
    }

    fn render_error(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        if let Some(error) = &state.chat_screen.error_message {
            let error_msg =
                Paragraph::new(error.as_str()).style(Style::default().fg(COLORS.error_msg));
            frame.render_widget(error_msg, area);
        }
    }
}

#[async_trait]
impl ScreenHandler for ChatScreen {
    fn render(&self, frame: &mut Frame, state: &AppState) {
        frame.render_widget(Clear, frame.area());

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Main content
                Constraint::Length(3), // Message input
                Constraint::Length(3), // Controls
                Constraint::Length(1), // Error message
            ])
            .split(frame.area());

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Room list
                Constraint::Percentage(70), // Messages
            ])
            .split(main_chunks[0]);

        self.render_room_list(frame, content_chunks[0], state);
        self.render_messages(frame, content_chunks[1], state);
        self.render_message_input(frame, main_chunks[1], state);
        self.render_controls(frame, main_chunks[2], state);
        self.render_error(frame, main_chunks[3], state);
    }

    async fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> Option<Screen> {
        match key.code {
            KeyCode::Esc => None,
            KeyCode::Tab => {
                state.chat_screen.input_focused = !state.chat_screen.input_focused;
                state.chat_screen.message_input.is_focused = state.chat_screen.input_focused;
                Some(Screen::Chat)
            }
            KeyCode::Up => {
                if !state.chat_screen.input_focused {
                    state.chat_screen.select_previous_room();

                    // Load messages for the newly selected room
                    if let Some(room) = state.chat_screen.get_selected_room() {
                        let room_id = room.room_id.clone();
                        match state.matrix_service.get_messages(&room_id, 100).await {
                            Ok(messages) => {
                                state.chat_screen.messages = messages;
                                state.chat_screen.message_scroll_offset = 0;
                                state.chat_screen.error_message = None;
                            }
                            Err(e) => {
                                state.chat_screen.error_message =
                                    Some(format!("Failed to load messages: {}", e));
                            }
                        }
                    }
                }
                Some(Screen::Chat)
            }
            KeyCode::Down => {
                if !state.chat_screen.input_focused {
                    state.chat_screen.select_next_room();

                    // Load messages for the newly selected room
                    if let Some(room) = state.chat_screen.get_selected_room() {
                        let room_id = room.room_id.clone();
                        match state.matrix_service.get_messages(&room_id, 100).await {
                            Ok(messages) => {
                                state.chat_screen.messages = messages;
                                state.chat_screen.message_scroll_offset = 0;
                                state.chat_screen.error_message = None;
                            }
                            Err(e) => {
                                state.chat_screen.error_message =
                                    Some(format!("Failed to load messages: {}", e));
                            }
                        }
                    }
                }
                Some(Screen::Chat)
            }
            KeyCode::PageUp => {
                state.chat_screen.scroll_messages_up();
                Some(Screen::Chat)
            }
            KeyCode::PageDown => {
                let max_offset = state.chat_screen.messages.len().saturating_sub(1);
                state.chat_screen.scroll_messages_down(max_offset);
                Some(Screen::Chat)
            }
            KeyCode::Enter => {
                if state.chat_screen.input_focused {
                    let message = state.chat_screen.message_input.value.trim().to_string();

                    if !message.is_empty() {
                        if let Some(room) = state.chat_screen.get_selected_room() {
                            let room_id = room.room_id.clone();

                            match state.matrix_service.send_message(&room_id, &message).await {
                                Ok(_) => {
                                    state.chat_screen.message_input.clear();
                                    state.chat_screen.error_message = None;

                                    // Reload messages to show the new message
                                    match state.matrix_service.get_messages(&room_id, 100).await {
                                        Ok(messages) => {
                                            state.chat_screen.messages = messages;
                                        }
                                        Err(e) => {
                                            state.chat_screen.error_message =
                                                Some(format!("Failed to reload messages: {}", e));
                                        }
                                    }
                                }
                                Err(e) => {
                                    state.chat_screen.error_message =
                                        Some(format!("Failed to send message: {}", e));
                                }
                            }
                        }
                    }
                }
                Some(Screen::Chat)
            }
            _ => {
                if state.chat_screen.input_focused {
                    state.chat_screen.message_input.handle_key_event(key);
                }
                Some(Screen::Chat)
            }
        }
    }
}
