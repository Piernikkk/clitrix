pub mod input_handler;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};

struct Colors {
    placeholder: Color,
    text: Color,
    cursor_bg: Color,
    cursor_fg: Color,
    border_focused: Color,
    border_unfocused: Color,
}

static COLORS: Colors = Colors {
    placeholder: Color::DarkGray,
    text: Color::White,
    cursor_bg: Color::Yellow,
    cursor_fg: Color::Black,
    border_focused: Color::Yellow,
    border_unfocused: Color::Gray,
};

pub struct TextInput<'a> {
    value: &'a str,
    cursor_position: usize,
    is_focused: bool,
    title: &'a str,
    placeholder: &'a str,
    mask_input: bool,
}

impl<'a> TextInput<'a> {
    pub fn new(
        value: &'a str,
        cursor_position: usize,
        title: &'a str,
        placeholder: &'a str,
        mask_input: bool,
        is_focused: bool,
    ) -> Self {
        Self {
            value,
            cursor_position,
            is_focused,
            title,
            placeholder,
            mask_input,
        }
    }

    fn render_text(&self) -> Line<'static> {
        let value = if self.mask_input {
            "*".repeat(self.value.len())
        } else {
            self.value.to_string()
        };

        if self.value.is_empty() {
            Line::from(Span::styled(
                self.placeholder.to_string(),
                Style::default()
                    .fg(COLORS.placeholder)
                    .add_modifier(Modifier::ITALIC),
            ))
        } else {
            let mut spans = Vec::new();
            let chars: Vec<char> = value.chars().collect();

            let before_cursor: String = chars.iter().take(self.cursor_position).collect();
            let at_cursor = chars.get(self.cursor_position).copied().unwrap_or(' ');
            let after_cursor: String = chars.iter().skip(self.cursor_position + 1).collect();

            spans.push(Span::styled(
                before_cursor,
                Style::default().fg(COLORS.text),
            ));
            spans.push(Span::styled(
                at_cursor.to_string(),
                Style::default().bg(COLORS.cursor_bg).fg(COLORS.cursor_fg),
            ));
            spans.push(Span::styled(after_cursor, Style::default().fg(COLORS.text)));

            Line::from(spans)
        }
    }

    fn get_border_style(&self) -> Style {
        if self.is_focused {
            Style::default().fg(COLORS.border_focused)
        } else {
            Style::default().fg(COLORS.border_unfocused)
        }
    }
}

impl<'a> Widget for TextInput<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = self.render_text();
        let border_style = self.get_border_style();

        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title)
            .border_style(border_style);

        let paragraph = Paragraph::new(text).block(block);

        paragraph.render(area, buf);
    }
}
