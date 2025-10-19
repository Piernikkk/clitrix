use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    app::AppState,
    screens::{Screen, ScreenHandler},
};

#[derive(Debug, Clone, PartialEq)]
pub enum LoginField {
    Username,
    Password,
    Homeserver,
}

impl Default for LoginField {
    fn default() -> Self {
        LoginField::Username
    }
}

pub struct LoginScreen;

impl LoginScreen {
    fn get_field_title(&self, field: &LoginField) -> &'static str {
        match field {
            LoginField::Username => "Username",
            LoginField::Password => "Password",
            LoginField::Homeserver => "Homeserver",
        }
    }

    fn get_field_placeholder(&self, field: &LoginField) -> &'static str {
        match field {
            LoginField::Username => "@username:matrix.org",
            LoginField::Password => "Enter your password",
            LoginField::Homeserver => "matrix.org",
        }
    }

    fn get_field_value<'a>(&self, field: &LoginField, app_state: &'a AppState) -> &'a str {
        match field {
            LoginField::Username => &app_state.login_form.username,
            LoginField::Password => &app_state.login_form.password,
            LoginField::Homeserver => &app_state.login_form.homeserver,
        }
    }

    fn render_field_input(
        &self,
        frame: &mut Frame,
        area: Rect,
        field: &LoginField,
        app_state: &AppState,
        is_active: bool,
    ) {
        let value = self.get_field_value(field, app_state);
        let title = self.get_field_title(field);
        let placeholder = self.get_field_placeholder(field);

        // Create display text (mask password)
        let display_text = if matches!(field, LoginField::Password) && !value.is_empty() {
            "*".repeat(value.len())
        } else if value.is_empty() {
            placeholder.to_string()
        } else {
            value.to_string()
        };

        // Style based on whether this field is active
        let border_style = if is_active {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        let text_style = if value.is_empty() {
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC)
        } else if matches!(field, LoginField::Password) {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::White)
        };

        // Show cursor if this field is active and in editing mode
        let mut spans = vec![Span::styled(display_text, text_style)];

        if is_active && app_state.login_form.editing {
            if matches!(field, LoginField::Password) {
                // For password, just show cursor at end
                spans.push(Span::styled("▌", Style::default().fg(Color::Yellow)));
            } else {
                // For other fields, show cursor at actual position
                spans.clear();
                let cursor_pos = match field {
                    LoginField::Username => app_state.login_form.username_cursor,
                    LoginField::Homeserver => app_state.login_form.homeserver_cursor,
                    _ => 0,
                };

                if value.is_empty() {
                    spans.push(Span::styled(
                        placeholder,
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::ITALIC),
                    ));
                    if cursor_pos == 0 {
                        spans.insert(0, Span::styled("▌", Style::default().fg(Color::Yellow)));
                    }
                } else {
                    if cursor_pos == 0 {
                        spans.push(Span::styled("▌", Style::default().fg(Color::Yellow)));
                        spans.push(Span::styled(value, Style::default().fg(Color::White)));
                    } else if cursor_pos >= value.len() {
                        spans.push(Span::styled(value, Style::default().fg(Color::White)));
                        spans.push(Span::styled("▌", Style::default().fg(Color::Yellow)));
                    } else {
                        let before = &value[..cursor_pos];
                        let at_cursor = value.chars().nth(cursor_pos).unwrap_or(' ');
                        let after = &value[cursor_pos + 1..];

                        spans.push(Span::styled(before, Style::default().fg(Color::White)));
                        spans.push(Span::styled(
                            at_cursor.to_string(),
                            Style::default().bg(Color::Yellow).fg(Color::Black),
                        ));
                        spans.push(Span::styled(after, Style::default().fg(Color::White)));
                    }
                }
            }
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

        let paragraph = Paragraph::new(Line::from(spans)).block(block);

        frame.render_widget(paragraph, area);
    }

    fn center_rect(&self, percent_x: u16, percent_y: u16, area: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

impl ScreenHandler for LoginScreen {
    fn render(&self, frame: &mut Frame, app_state: &AppState) {
        // Clear the entire area
        frame.render_widget(Clear, frame.area());

        // Create centered login form
        let form_area = self.center_rect(50, 60, frame.area());

        // Main container
        let main_block = Block::default()
            .borders(Borders::ALL)
            .title("Matrix Login")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(main_block, form_area);

        // Inner area for form fields
        let inner_area = form_area.inner(Margin {
            horizontal: 2,
            vertical: 1,
        });

        // Layout for form fields and instructions
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Username
                Constraint::Length(3), // Password
                Constraint::Length(3), // Homeserver
                Constraint::Length(2), // Spacer
                Constraint::Length(4), // Instructions
                Constraint::Min(0),    // Remaining space
            ])
            .split(inner_area);

        // Render form fields
        self.render_field_input(
            frame,
            chunks[0],
            &LoginField::Username,
            app_state,
            app_state.login_form.active_field == LoginField::Username,
        );

        self.render_field_input(
            frame,
            chunks[1],
            &LoginField::Password,
            app_state,
            app_state.login_form.active_field == LoginField::Password,
        );

        self.render_field_input(
            frame,
            chunks[2],
            &LoginField::Homeserver,
            app_state,
            app_state.login_form.active_field == LoginField::Homeserver,
        );

        // Instructions
        let instructions = if app_state.login_form.editing {
            vec![
                Line::from("ESC - Stop editing"),
                Line::from("Tab/Shift+Tab - Switch fields"),
                Line::from("Enter - Submit login"),
                Line::from("Ctrl+C - Quit"),
            ]
        } else {
            vec![
                Line::from("Enter/Space - Start editing field"),
                Line::from("Tab/Shift+Tab - Switch fields"),
                Line::from("Ctrl+L - Submit login"),
                Line::from("Ctrl+C - Quit"),
            ]
        };

        let instructions_paragraph = Paragraph::new(instructions)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Controls")
                    .title_style(Style::default().fg(Color::Green)),
            )
            .style(Style::default().fg(Color::Gray));

        frame.render_widget(instructions_paragraph, chunks[4]);
    }

    fn handle_key_event(&self, key: KeyEvent, app_state: &mut AppState) -> Option<Screen> {
        if app_state.login_form.editing {
            // Handle editing mode
            match key.code {
                KeyCode::Esc => {
                    app_state.login_form.editing = false;
                    Some(Screen::Login)
                }
                KeyCode::Enter => {
                    app_state.login_form.editing = false;
                    // TODO: Trigger login attempt
                    // For now, just switch to chat screen as placeholder
                    Some(Screen::Chat)
                }
                KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        app_state.login_form.previous_field();
                    } else {
                        app_state.login_form.next_field();
                    }
                    Some(Screen::Login)
                }
                KeyCode::Char(c) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        match c {
                            'c' => None, // Quit
                            _ => Some(Screen::Login),
                        }
                    } else {
                        app_state.login_form.enter_char(c);
                        Some(Screen::Login)
                    }
                }
                KeyCode::Backspace => {
                    app_state.login_form.delete_char();
                    Some(Screen::Login)
                }
                KeyCode::Left => {
                    app_state.login_form.move_cursor_left();
                    Some(Screen::Login)
                }
                KeyCode::Right => {
                    app_state.login_form.move_cursor_right();
                    Some(Screen::Login)
                }
                _ => Some(Screen::Login),
            }
        } else {
            // Handle navigation mode
            match key.code {
                KeyCode::Char('q') => None,
                KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        app_state.login_form.previous_field();
                    } else {
                        app_state.login_form.next_field();
                    }
                    Some(Screen::Login)
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    app_state.login_form.editing = true;
                    Some(Screen::Login)
                }
                KeyCode::Char(c) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        match c {
                            'c' => None, // Quit
                            'l' => {
                                // TODO: Trigger login attempt
                                // For now, just switch to chat screen as placeholder
                                Some(Screen::Chat)
                            }
                            _ => Some(Screen::Login),
                        }
                    } else {
                        Some(Screen::Login)
                    }
                }
                _ => Some(Screen::Login),
            }
        }
    }
}
