use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    screens::chat::{ChatFocus, ChatScreenState},
    ui::components::{ChatInput, MessageList, RoomList},
    ui::text_input::TextInput,
};

pub struct RoomChatLayout {
    room_list_width: u16,
    show_room_filter: bool,
    show_status_bar: bool,
    show_help: bool,
}

impl Default for RoomChatLayout {
    fn default() -> Self {
        Self {
            room_list_width: 30,
            show_room_filter: false,
            show_status_bar: true,
            show_help: true,
        }
    }
}

impl RoomChatLayout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn room_list_width(mut self, width: u16) -> Self {
        self.room_list_width = width.max(20).min(50);
        self
    }

    pub fn show_room_filter(mut self, show: bool) -> Self {
        self.show_room_filter = show;
        self
    }

    pub fn show_status_bar(mut self, show: bool) -> Self {
        self.show_status_bar = show;
        self
    }

    pub fn show_help(mut self, show: bool) -> Self {
        self.show_help = show;
        self
    }

    pub fn render(&self, frame: &mut Frame, state: &ChatScreenState) {
        let main_layout = self.create_main_layout(frame.area());

        // Render room list section
        self.render_room_section(frame, main_layout.room_section, state);

        // Render chat section
        self.render_chat_section(frame, main_layout.chat_section, state);

        // Render status bar if enabled
        if self.show_status_bar && main_layout.status_bar.is_some() {
            self.render_status_bar(frame, main_layout.status_bar.unwrap(), state);
        }

        // Render help if enabled
        if self.show_help && main_layout.help_section.is_some() {
            self.render_help_section(frame, main_layout.help_section.unwrap(), state);
        }
    }

    fn create_main_layout(&self, area: Rect) -> MainLayoutAreas {
        let mut constraints = vec![Constraint::Min(10)]; // Main content

        if self.show_status_bar {
            constraints.push(Constraint::Length(1)); // Status bar
        }

        if self.show_help {
            constraints.push(Constraint::Length(3)); // Help section
        }

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let main_content = vertical_chunks[0];
        let status_bar = if self.show_status_bar {
            Some(vertical_chunks[1])
        } else {
            None
        };
        let help_section = if self.show_help {
            Some(vertical_chunks[if self.show_status_bar { 2 } else { 1 }])
        } else {
            None
        };

        // Split main content horizontally
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(self.room_list_width),
                Constraint::Min(40),
            ])
            .split(main_content);

        MainLayoutAreas {
            room_section: horizontal_chunks[0],
            chat_section: horizontal_chunks[1],
            status_bar,
            help_section,
        }
    }

    fn render_room_section(&self, frame: &mut Frame, area: Rect, state: &ChatScreenState) {
        let room_constraints = if self.show_room_filter {
            vec![Constraint::Length(3), Constraint::Min(5)]
        } else {
            vec![Constraint::Min(5)]
        };

        let room_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(room_constraints)
            .split(area);

        // Render room filter if shown
        if self.show_room_filter {
            self.render_room_filter(frame, room_chunks[0], state);
        }

        // Render room list
        let room_list_area = if self.show_room_filter {
            room_chunks[1]
        } else {
            room_chunks[0]
        };

        let room_list = RoomList::new(&state.rooms)
            .selected(state.room_list.selected_room_index())
            .show_unread_only(state.room_list.show_unread_only)
            .filter(Some(&state.room_list.filter_text));

        frame.render_widget(room_list, room_list_area);
    }

    fn render_room_filter(&self, frame: &mut Frame, area: Rect, state: &ChatScreenState) {
        let is_focused = matches!(state.focus, ChatFocus::RoomFilter);

        let filter_input = TextInput::new(
            &state.filter_input.value,
            state.filter_input.cursor_position,
            "Filter Rooms",
            "Type to filter rooms...",
        )
        .focused(is_focused)
        .editing(is_focused);

        frame.render_widget(filter_input, area);
    }

    fn render_chat_section(&self, frame: &mut Frame, area: Rect, state: &ChatScreenState) {
        if state.selected_room().is_none() {
            self.render_no_room_selected(frame, area);
            return;
        }

        let chat_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Messages
                Constraint::Length(5), // Input
            ])
            .split(area);

        // Render messages
        self.render_message_area(frame, chat_chunks[0], state);

        // Render input area
        self.render_input_area(frame, chat_chunks[1], state);
    }

    fn render_no_room_selected(&self, frame: &mut Frame, area: Rect) {
        let welcome_text = vec![
            Line::from(""),
            Line::from("Welcome to Matrix Chat! 🚀"),
            Line::from(""),
            Line::from("Select a room from the list on the left to start chatting."),
            Line::from(""),
            Line::from("Controls:"),
            Line::from("• ↑/↓ or j/k - Navigate rooms"),
            Line::from("• Enter - Select room"),
            Line::from("• / - Filter rooms"),
            Line::from("• u - Toggle unread filter"),
            Line::from("• Tab - Switch between panels"),
            Line::from("• Ctrl+C - Quit"),
        ];

        let welcome_paragraph = Paragraph::new(welcome_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Matrix Chat")
                    .title_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .style(Style::default().fg(Color::Gray))
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(welcome_paragraph, area);
    }

    fn render_message_area(&self, frame: &mut Frame, area: Rect, state: &ChatScreenState) {
        let is_focused = matches!(state.focus, ChatFocus::MessageList);

        let message_list = MessageList::new(&state.messages)
            .current_user(state.current_user_id())
            .show_timestamps(state.show_timestamps)
            .show_reactions(true)
            .auto_scroll(state.auto_scroll);

        let message_block = if is_focused {
            Block::default()
                .borders(Borders::ALL)
                .title("Messages [Focused]")
                .title_style(Style::default().fg(Color::Yellow))
                .border_style(Style::default().fg(Color::Yellow))
        } else {
            Block::default()
                .borders(Borders::ALL)
                .title("Messages")
                .title_style(Style::default().fg(Color::Cyan))
        };

        // Create a custom widget that combines the message list with the border
        let area_with_block = message_block.inner(area);
        frame.render_widget(message_block, area);
        frame.render_widget(message_list, area_with_block);
    }

    fn render_input_area(&self, frame: &mut Frame, area: Rect, state: &ChatScreenState) {
        let is_focused = matches!(state.focus, ChatFocus::MessageInput);
        let room_name = state.selected_room().map(|r| r.display_name());

        let chat_input = ChatInput::new(&state.message_input)
            .focused(is_focused)
            .composing(is_focused)
            .room_name(room_name)
            .typing_users(&state.typing_users);

        frame.render_widget(chat_input, area);
    }

    fn render_status_bar(&self, frame: &mut Frame, area: Rect, state: &ChatScreenState) {
        let status_text = state.get_status_text();
        let unread_count = state.unread_message_count();
        let room_count = state.rooms.len();

        let mut spans = vec![Span::styled(
            format!("Rooms: {} ", room_count),
            Style::default().fg(Color::Cyan),
        )];

        if unread_count > 0 {
            spans.extend_from_slice(&[
                Span::raw("• "),
                Span::styled(
                    format!("Unread: {} ", unread_count),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]);
        }

        spans.extend_from_slice(&[
            Span::raw("• "),
            Span::styled(status_text, Style::default().fg(Color::White)),
        ]);

        // Add focus indicator
        let focus_indicator = match state.focus {
            ChatFocus::RoomList => "ROOMS",
            ChatFocus::MessageList => "MESSAGES",
            ChatFocus::MessageInput => "INPUT",
            ChatFocus::RoomFilter => "FILTER",
        };

        spans.extend_from_slice(&[
            Span::raw(" ["),
            Span::styled(
                focus_indicator,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("]"),
        ]);

        let status_line = Line::from(spans);
        let status_paragraph =
            Paragraph::new(status_line).style(Style::default().bg(Color::DarkGray));

        frame.render_widget(status_paragraph, area);
    }

    fn render_help_section(&self, frame: &mut Frame, area: Rect, state: &ChatScreenState) {
        let help_text = match state.focus {
            ChatFocus::RoomList => vec![Line::from(
                "Room List: ↑/↓ Navigate • Enter Select • / Filter • u Unread • Tab Switch",
            )],
            ChatFocus::MessageList => vec![Line::from(
                "Messages: ↑/↓ Scroll • PgUp/PgDn Page • Home/End • Enter Type • Tab Switch",
            )],
            ChatFocus::MessageInput => vec![Line::from(
                "Input: Type message • Enter Send • Esc Cancel • Ctrl+U Clear • Tab Switch",
            )],
            ChatFocus::RoomFilter => vec![Line::from(
                "Filter: Type to filter • Enter Apply • Esc Cancel • Tab Switch",
            )],
        };

        let help_paragraph = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help")
                    .title_style(Style::default().fg(Color::Green)),
            )
            .style(Style::default().fg(Color::DarkGray));

        frame.render_widget(help_paragraph, area);
    }
}

struct MainLayoutAreas {
    room_section: Rect,
    chat_section: Rect,
    status_bar: Option<Rect>,
    help_section: Option<Rect>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_creation() {
        let layout = RoomChatLayout::new();
        assert_eq!(layout.room_list_width, 30);
        assert!(!layout.show_room_filter);
        assert!(layout.show_status_bar);
        assert!(layout.show_help);
    }

    #[test]
    fn test_layout_configuration() {
        let layout = RoomChatLayout::new()
            .room_list_width(40)
            .show_room_filter(true)
            .show_status_bar(false)
            .show_help(false);

        assert_eq!(layout.room_list_width, 40);
        assert!(layout.show_room_filter);
        assert!(!layout.show_status_bar);
        assert!(!layout.show_help);
    }

    #[test]
    fn test_width_constraints() {
        let layout = RoomChatLayout::new().room_list_width(10); // Too small
        assert_eq!(layout.room_list_width, 20); // Should be clamped to minimum

        let layout = RoomChatLayout::new().room_list_width(60); // Too large
        assert_eq!(layout.room_list_width, 50); // Should be clamped to maximum
    }
}
