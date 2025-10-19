use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget, Wrap},
};

use crate::data::models::{Message, MessageContent};

pub struct MessageList<'a> {
    messages: &'a [Message],
    current_user_id: Option<&'a str>,
    show_timestamps: bool,
    show_reactions: bool,
    auto_scroll: bool,
}

impl<'a> MessageList<'a> {
    pub fn new(messages: &'a [Message]) -> Self {
        Self {
            messages,
            current_user_id: None,
            show_timestamps: true,
            show_reactions: true,
            auto_scroll: true,
        }
    }

    pub fn current_user(mut self, user_id: Option<&'a str>) -> Self {
        self.current_user_id = user_id;
        self
    }

    pub fn show_timestamps(mut self, show: bool) -> Self {
        self.show_timestamps = show;
        self
    }

    pub fn show_reactions(mut self, show: bool) -> Self {
        self.show_reactions = show;
        self
    }

    pub fn auto_scroll(mut self, auto_scroll: bool) -> Self {
        self.auto_scroll = auto_scroll;
        self
    }

    fn format_timestamp(&self, message: &Message) -> String {
        let now = chrono::Local::now();
        let msg_time = message.timestamp;

        if now.date_naive() == msg_time.date_naive() {
            // Same day - show only time
            msg_time.format("%H:%M").to_string()
        } else if now
            .date_naive()
            .signed_duration_since(msg_time.date_naive())
            .num_days()
            < 7
        {
            // Within a week - show day and time
            msg_time.format("%a %H:%M").to_string()
        } else {
            // Older - show date and time
            msg_time.format("%m/%d %H:%M").to_string()
        }
    }

    fn is_own_message(&self, message: &Message) -> bool {
        self.current_user_id
            .map(|id| message.is_from_user(id))
            .unwrap_or(false)
    }

    fn format_sender_name(&self, message: &Message) -> String {
        if self.is_own_message(message) {
            "You".to_string()
        } else {
            message.sender.display_name().to_string()
        }
    }

    fn get_sender_color(&self, message: &Message) -> Color {
        if self.is_own_message(message) {
            Color::Green
        } else {
            // Generate a consistent color based on user ID
            let hash = message
                .sender
                .user_id
                .bytes()
                .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));

            match hash % 6 {
                0 => Color::Cyan,
                1 => Color::Magenta,
                2 => Color::Yellow,
                3 => Color::Blue,
                4 => Color::Red,
                _ => Color::White,
            }
        }
    }

    fn format_message_content(&self, message: &Message, area_width: u16) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let content_width = area_width.saturating_sub(4) as usize; // Account for padding

        match &message.content {
            MessageContent::Text { body } => {
                // Handle text wrapping
                let wrapped_lines = self.wrap_text(body, content_width);
                for line in wrapped_lines {
                    lines.push(Line::from(Span::styled(
                        line.to_string(),
                        Style::default().fg(Color::White),
                    )));
                }
            }
            MessageContent::Emote { body } => {
                let emote_text = format!("* {}", body);
                let wrapped_lines = self.wrap_text(&emote_text, content_width);
                for line in wrapped_lines {
                    lines.push(Line::from(Span::styled(
                        line.to_string(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::ITALIC),
                    )));
                }
            }
            MessageContent::Notice { body } => {
                let notice_text = format!("ℹ️ {}", body);
                let wrapped_lines = self.wrap_text(&notice_text, content_width);
                for line in wrapped_lines {
                    lines.push(Line::from(Span::styled(
                        line.to_string(),
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::ITALIC),
                    )));
                }
            }
            MessageContent::System { message: msg } => {
                let system_text = format!("🔧 {}", msg);
                let wrapped_lines = self.wrap_text(&system_text, content_width);
                for line in wrapped_lines {
                    lines.push(Line::from(Span::styled(
                        line.to_string(),
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::ITALIC),
                    )));
                }
            }
            MessageContent::Image { body, .. } => {
                lines.push(Line::from(vec![
                    Span::styled("📷 ".to_string(), Style::default().fg(Color::Yellow)),
                    Span::styled(body.clone(), Style::default().fg(Color::White)),
                ]));
            }
            MessageContent::File { filename, size, .. } => {
                let size_text = size
                    .map(|s| format!(" ({})", self.format_file_size(s)))
                    .unwrap_or_default();
                lines.push(Line::from(vec![
                    Span::styled("📎 ".to_string(), Style::default().fg(Color::Yellow)),
                    Span::styled(filename.clone(), Style::default().fg(Color::White)),
                    Span::styled(size_text, Style::default().fg(Color::DarkGray)),
                ]));
            }
            MessageContent::Audio { body, duration, .. } => {
                let duration_text = duration
                    .map(|d| format!(" ({})", self.format_duration(d)))
                    .unwrap_or_default();
                lines.push(Line::from(vec![
                    Span::styled("🎵 ".to_string(), Style::default().fg(Color::Yellow)),
                    Span::styled(body.clone(), Style::default().fg(Color::White)),
                    Span::styled(duration_text, Style::default().fg(Color::DarkGray)),
                ]));
            }
            MessageContent::Video { body, duration, .. } => {
                let duration_text = duration
                    .map(|d| format!(" ({})", self.format_duration(d)))
                    .unwrap_or_default();
                lines.push(Line::from(vec![
                    Span::styled("🎥 ".to_string(), Style::default().fg(Color::Yellow)),
                    Span::styled(body.clone(), Style::default().fg(Color::White)),
                    Span::styled(duration_text, Style::default().fg(Color::DarkGray)),
                ]));
            }
        }

        lines
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

    fn format_file_size(&self, size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    fn format_duration(&self, seconds: u64) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if hours > 0 {
            format!("{}:{:02}:{:02}", hours, minutes, secs)
        } else {
            format!("{}:{:02}", minutes, secs)
        }
    }

    fn format_message_item(&self, message: &Message, area_width: u16) -> ListItem {
        let mut lines = Vec::new();

        // Handle system messages differently
        if matches!(message.content, MessageContent::System { .. }) {
            let content_lines = self.format_message_content(message, area_width);
            return ListItem::new(content_lines);
        }

        // Header line with sender and timestamp
        let sender_name = self.format_sender_name(message);
        let sender_color = self.get_sender_color(message);
        let timestamp = if self.show_timestamps {
            format!(" [{}]", self.format_timestamp(message))
        } else {
            String::new()
        };

        let edited_indicator = if message.is_edited { " (edited)" } else { "" };

        let mut header_spans = vec![
            Span::styled(
                sender_name,
                Style::default()
                    .fg(sender_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(timestamp, Style::default().fg(Color::DarkGray)),
            Span::styled(
                edited_indicator,
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ),
        ];

        // Add presence indicator for non-own messages
        if !self.is_own_message(message) {
            header_spans.insert(
                0,
                Span::styled(
                    message.sender.presence_icon(),
                    Style::default().fg(Color::Green),
                ),
            );
            header_spans.insert(1, Span::raw(" "));
        }

        lines.push(Line::from(header_spans));

        // Reply indicator
        if message.reply_to.is_some() {
            lines.push(Line::from(Span::styled(
                "  ↳ replying to message",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )));
        }

        // Message content
        let content_lines = self.format_message_content(message, area_width);
        for mut content_line in content_lines {
            // Indent content slightly
            content_line.spans.insert(0, Span::raw("  "));
            lines.push(content_line);
        }

        // Reactions
        if self.show_reactions && !message.reactions.is_empty() {
            let reactions_text = message.format_reactions();
            lines.push(Line::from(Span::styled(
                format!("  {}", reactions_text),
                Style::default().fg(Color::Yellow),
            )));
        }

        // Add spacing between messages
        lines.push(Line::from(""));

        ListItem::new(lines)
    }
}

impl<'a> Widget for MessageList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.messages.is_empty() {
            let empty_message = Paragraph::new("No messages yet. Start the conversation!")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Messages")
                        .title_style(
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                )
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            empty_message.render(area, buf);
            return;
        }

        let items: Vec<ListItem> = self
            .messages
            .iter()
            .map(|message| self.format_message_item(message, area.width))
            .collect();

        let title = format!("Messages ({})", self.messages.len());

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .title_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .style(Style::default());

        list.render(area, buf);
    }
}

/// Stateful wrapper for MessageList that manages scrolling
#[derive(Debug)]
pub struct StatefulMessageList {
    pub scroll_offset: usize,
    pub auto_scroll: bool,
}

impl Default for StatefulMessageList {
    fn default() -> Self {
        Self {
            scroll_offset: 0,
            auto_scroll: true,
        }
    }
}

impl StatefulMessageList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            self.auto_scroll = false;
        }
    }

    pub fn scroll_down(&mut self, messages: &[Message], visible_lines: usize) {
        let max_scroll = messages.len().saturating_sub(visible_lines);
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        } else {
            // If we're at the bottom, enable auto-scroll
            self.auto_scroll = true;
        }
    }

    pub fn scroll_to_bottom(&mut self, messages: &[Message], visible_lines: usize) {
        self.scroll_offset = messages.len().saturating_sub(visible_lines);
        self.auto_scroll = true;
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
        self.auto_scroll = false;
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
        self.auto_scroll = false;
    }

    pub fn page_down(&mut self, messages: &[Message], visible_lines: usize, page_size: usize) {
        let max_scroll = messages.len().saturating_sub(visible_lines);
        self.scroll_offset = (self.scroll_offset + page_size).min(max_scroll);

        // Check if we're at the bottom
        if self.scroll_offset >= max_scroll {
            self.auto_scroll = true;
        }
    }

    pub fn on_new_message(&mut self, messages: &[Message], visible_lines: usize) {
        if self.auto_scroll {
            self.scroll_to_bottom(messages, visible_lines);
        }
    }

    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll = !self.auto_scroll;
    }

    pub fn render<'a>(
        &self,
        area: Rect,
        buf: &mut Buffer,
        messages: &'a [Message],
        current_user_id: Option<&'a str>,
    ) {
        let message_list = MessageList::new(messages)
            .current_user(current_user_id)
            .auto_scroll(self.auto_scroll);

        message_list.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::models::{User, UserPresence};
    use chrono::Local;

    fn create_test_message(sender_id: &str, content: &str) -> Message {
        Message::new_text(
            format!("$event_{}", sender_id),
            User {
                user_id: sender_id.to_string(),
                display_name: Some(format!("User {}", sender_id)),
                avatar_url: None,
                presence: UserPresence::Online,
            },
            content.to_string(),
        )
    }

    #[test]
    fn test_message_list_creation() {
        let messages = vec![
            create_test_message("@alice:matrix.org", "Hello"),
            create_test_message("@bob:matrix.org", "Hi there!"),
        ];

        let message_list = MessageList::new(&messages);
        assert_eq!(message_list.messages.len(), 2);
        assert!(message_list.show_timestamps);
        assert!(message_list.show_reactions);
    }

    #[test]
    fn test_text_wrapping() {
        let message_list = MessageList::new(&[]);
        let wrapped = message_list.wrap_text("This is a very long line that should be wrapped", 10);

        assert!(wrapped.len() > 1);
        for line in &wrapped {
            assert!(line.len() <= 10);
        }
    }

    #[test]
    fn test_file_size_formatting() {
        let message_list = MessageList::new(&[]);

        assert_eq!(message_list.format_file_size(512), "512 B");
        assert_eq!(message_list.format_file_size(1536), "1.5 KB");
        assert_eq!(message_list.format_file_size(1048576), "1.0 MB");
    }

    #[test]
    fn test_duration_formatting() {
        let message_list = MessageList::new(&[]);

        assert_eq!(message_list.format_duration(30), "0:30");
        assert_eq!(message_list.format_duration(90), "1:30");
        assert_eq!(message_list.format_duration(3661), "1:01:01");
    }

    #[test]
    fn test_stateful_message_list() {
        let mut stateful_list = StatefulMessageList::new();
        let messages = vec![
            create_test_message("@user1:matrix.org", "Message 1"),
            create_test_message("@user2:matrix.org", "Message 2"),
            create_test_message("@user3:matrix.org", "Message 3"),
        ];

        assert_eq!(stateful_list.scroll_offset, 0);
        assert!(stateful_list.auto_scroll);

        stateful_list.scroll_down(&messages, 2);
        assert_eq!(stateful_list.scroll_offset, 1);

        stateful_list.scroll_to_top();
        assert_eq!(stateful_list.scroll_offset, 0);
        assert!(!stateful_list.auto_scroll);
    }
}
