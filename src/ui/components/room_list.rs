use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Widget},
};

use crate::data::models::{Room, RoomType};

pub struct RoomList<'a> {
    rooms: &'a [Room],
    selected_index: Option<usize>,
    show_unread_only: bool,
    filter_text: Option<&'a str>,
}

impl<'a> RoomList<'a> {
    pub fn new(rooms: &'a [Room]) -> Self {
        Self {
            rooms,
            selected_index: None,
            show_unread_only: false,
            filter_text: None,
        }
    }

    pub fn selected(mut self, index: Option<usize>) -> Self {
        self.selected_index = index;
        self
    }

    pub fn show_unread_only(mut self, show_unread: bool) -> Self {
        self.show_unread_only = show_unread;
        self
    }

    pub fn filter(mut self, filter_text: Option<&'a str>) -> Self {
        self.filter_text = filter_text;
        self
    }

    fn should_show_room(&self, room: &Room) -> bool {
        // Filter by unread status
        if self.show_unread_only && room.unread_count == 0 {
            return false;
        }

        // Filter by search text
        if let Some(filter) = self.filter_text {
            if !filter.is_empty() {
                let filter_lower = filter.to_lowercase();
                let name_matches = room.display_name().to_lowercase().contains(&filter_lower);
                let id_matches = room.room_id.to_lowercase().contains(&filter_lower);
                let topic_matches = room
                    .topic
                    .as_ref()
                    .map(|t| t.to_lowercase().contains(&filter_lower))
                    .unwrap_or(false);

                if !name_matches && !id_matches && !topic_matches {
                    return false;
                }
            }
        }

        true
    }

    fn format_room_item(&self, room: &Room, is_selected: bool) -> ListItem<'static> {
        let room_icon = room.type_icon();
        let encryption_icon = room.encryption_icon();
        let unread_indicator = room.unread_indicator();
        let display_name = room.display_name();

        // Style based on room type and selection
        let (name_style, icon_style) = if is_selected {
            (
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(Color::Yellow),
            )
        } else {
            let color = match room.room_type {
                RoomType::DirectMessage => Color::Green,
                RoomType::PublicRoom => Color::Cyan,
                RoomType::PrivateRoom => Color::Magenta,
                RoomType::Space => Color::Blue,
            };

            let name_color = if room.unread_count > 0 {
                Color::White
            } else {
                Color::Gray
            };

            (
                Style::default()
                    .fg(name_color)
                    .add_modifier(if room.unread_count > 0 {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
                Style::default().fg(color),
            )
        };

        // Build the room line
        let mut spans = Vec::new();

        // Room type icon
        spans.push(Span::styled(room_icon.to_string(), icon_style));
        spans.push(Span::raw(" ".to_string()));

        // Encryption icon if encrypted
        if !encryption_icon.is_empty() {
            spans.push(Span::styled(
                encryption_icon.to_string(),
                Style::default().fg(Color::Green),
            ));
            spans.push(Span::raw(" ".to_string()));
        }

        // Room name
        spans.push(Span::styled(display_name.to_string(), name_style));

        // Unread count
        if !unread_indicator.is_empty() {
            spans.push(Span::styled(
                unread_indicator.to_string(),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ));
        }

        // Member count for non-DM rooms
        if !matches!(room.room_type, RoomType::DirectMessage) && room.member_count > 0 {
            spans.push(Span::styled(
                format!(" [{}]", room.member_count),
                Style::default().fg(Color::DarkGray),
            ));
        }

        // Topic preview on second line for selected room
        let mut lines = vec![Line::from(spans)];

        if is_selected {
            if let Some(ref topic) = room.topic {
                let topic_text = if topic.len() > 60 {
                    format!("  💬 {}...", &topic[..57])
                } else {
                    format!("  💬 {}", topic)
                };

                lines.push(Line::from(Span::styled(
                    topic_text,
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                )));
            }

            // Last message preview
            if room.last_message.is_some() {
                let preview = room.last_message_preview();
                let preview_text = if preview.len() > 60 {
                    format!("  📝 {}...", &preview[..57])
                } else {
                    format!("  📝 {}", preview)
                };

                lines.push(Line::from(Span::styled(
                    preview_text,
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }

        ListItem::new(lines)
    }

    fn get_filtered_rooms(&self) -> Vec<(usize, &Room)> {
        self.rooms
            .iter()
            .enumerate()
            .filter(|(_, room)| self.should_show_room(room))
            .collect()
    }

    fn get_title(&self) -> String {
        let total_rooms = self.rooms.len();
        let filtered_rooms = self.get_filtered_rooms().len();
        let unread_count: usize = self.rooms.iter().map(|r| r.unread_count).sum();

        let mut title = format!("Rooms ({}/{})", filtered_rooms, total_rooms);

        if unread_count > 0 {
            title.push_str(&format!(" • {} unread", unread_count));
        }

        if self.show_unread_only {
            title.push_str(" [Unread Only]");
        }

        if let Some(filter) = self.filter_text {
            if !filter.is_empty() {
                title.push_str(&format!(" [Filter: {}]", filter));
            }
        }

        title
    }
}

impl<'a> Widget for RoomList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let filtered_rooms = self.get_filtered_rooms();

        let items: Vec<ListItem> = filtered_rooms
            .iter()
            .enumerate()
            .map(|(_filtered_index, (original_index, room))| {
                let is_selected = self
                    .selected_index
                    .map(|selected| selected == *original_index)
                    .unwrap_or(false);
                self.format_room_item(room, is_selected)
            })
            .collect();

        let title = self.get_title();

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
            .style(Style::default())
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        list.render(area, buf);
    }
}

/// Stateful wrapper for RoomList that manages selection
#[derive(Debug)]
pub struct StatefulRoomList {
    pub state: ListState,
    pub show_unread_only: bool,
    pub filter_text: String,
}

impl Default for StatefulRoomList {
    fn default() -> Self {
        Self {
            state: ListState::default(),
            show_unread_only: false,
            filter_text: String::new(),
        }
    }
}

impl StatefulRoomList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select_first(&mut self, rooms: &[Room]) {
        if !rooms.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn select_next(&mut self, rooms: &[Room]) {
        let filtered_rooms = self.get_filtered_room_indices(rooms);
        if filtered_rooms.is_empty() {
            return;
        }

        let selected = match self.state.selected() {
            Some(i) => {
                // Find current position in filtered list
                if let Some(pos) = filtered_rooms.iter().position(|&idx| idx == i) {
                    // Move to next in filtered list, wrap around
                    let next_pos = (pos + 1) % filtered_rooms.len();
                    filtered_rooms[next_pos]
                } else {
                    // Current selection not in filtered list, select first
                    filtered_rooms[0]
                }
            }
            None => filtered_rooms[0],
        };

        self.state.select(Some(selected));
    }

    pub fn select_previous(&mut self, rooms: &[Room]) {
        let filtered_rooms = self.get_filtered_room_indices(rooms);
        if filtered_rooms.is_empty() {
            return;
        }

        let selected = match self.state.selected() {
            Some(i) => {
                // Find current position in filtered list
                if let Some(pos) = filtered_rooms.iter().position(|&idx| idx == i) {
                    // Move to previous in filtered list, wrap around
                    let prev_pos = if pos == 0 {
                        filtered_rooms.len() - 1
                    } else {
                        pos - 1
                    };
                    filtered_rooms[prev_pos]
                } else {
                    // Current selection not in filtered list, select last
                    filtered_rooms[filtered_rooms.len() - 1]
                }
            }
            None => filtered_rooms[0],
        };

        self.state.select(Some(selected));
    }

    pub fn selected_room_index(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn selected_room<'a>(&self, rooms: &'a [Room]) -> Option<&'a Room> {
        self.state.selected().and_then(|i| rooms.get(i))
    }

    pub fn toggle_unread_filter(&mut self) {
        self.show_unread_only = !self.show_unread_only;
    }

    pub fn set_filter(&mut self, filter: String) {
        self.filter_text = filter;
    }

    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
    }

    fn should_show_room(&self, room: &Room) -> bool {
        // Filter by unread status
        if self.show_unread_only && room.unread_count == 0 {
            return false;
        }

        // Filter by search text
        if !self.filter_text.is_empty() {
            let filter_lower = self.filter_text.to_lowercase();
            let name_matches = room.display_name().to_lowercase().contains(&filter_lower);
            let id_matches = room.room_id.to_lowercase().contains(&filter_lower);
            let topic_matches = room
                .topic
                .as_ref()
                .map(|t| t.to_lowercase().contains(&filter_lower))
                .unwrap_or(false);

            if !name_matches && !id_matches && !topic_matches {
                return false;
            }
        }

        true
    }

    fn get_filtered_room_indices(&self, rooms: &[Room]) -> Vec<usize> {
        rooms
            .iter()
            .enumerate()
            .filter(|(_, room)| self.should_show_room(room))
            .map(|(index, _)| index)
            .collect()
    }

    pub fn render<'a>(&'a mut self, area: Rect, buf: &mut Buffer, rooms: &'a [Room]) {
        let filtered_rooms = self.get_filtered_room_indices(rooms);

        let items: Vec<ListItem> = filtered_rooms
            .iter()
            .map(|&room_index| {
                let room = &rooms[room_index];
                let is_selected = self
                    .state
                    .selected()
                    .map(|selected| selected == room_index)
                    .unwrap_or(false);

                // Create the item directly instead of using format_room_item
                let room_icon = room.type_icon();
                let encryption_icon = room.encryption_icon();
                let unread_indicator = room.unread_indicator();
                let display_name = room.display_name();

                let mut spans = Vec::new();
                spans.push(Span::styled(
                    room_icon.to_string(),
                    Style::default().fg(Color::Cyan),
                ));
                spans.push(Span::raw(" ".to_string()));

                if !encryption_icon.is_empty() {
                    spans.push(Span::styled(
                        encryption_icon.to_string(),
                        Style::default().fg(Color::Green),
                    ));
                    spans.push(Span::raw(" ".to_string()));
                }

                spans.push(Span::styled(
                    display_name.to_string(),
                    if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ));

                if !unread_indicator.is_empty() {
                    spans.push(Span::styled(
                        unread_indicator.to_string(),
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ));
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        let title = self.get_title(rooms);

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
            .style(Style::default())
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        // Create a subarea for the list content to properly handle the filtered state
        let _filtered_state = if filtered_rooms.is_empty() {
            ListState::default()
        } else {
            let mut state = ListState::default();
            if let Some(selected) = self.state.selected() {
                if let Some(filtered_pos) = filtered_rooms.iter().position(|&idx| idx == selected) {
                    state.select(Some(filtered_pos));
                }
            }
            state
        };

        Widget::render(list, area, buf);
    }

    fn get_title(&self, rooms: &[Room]) -> String {
        let total_rooms = rooms.len();
        let filtered_rooms = self.get_filtered_room_indices(rooms);
        let unread_count: usize = rooms.iter().map(|r| r.unread_count).sum();

        let mut title = format!("Rooms ({}/{})", filtered_rooms.len(), total_rooms);

        if unread_count > 0 {
            title.push_str(&format!(" • {} unread", unread_count));
        }

        if self.show_unread_only {
            title.push_str(" [Unread Only]");
        }

        if !self.filter_text.is_empty() {
            title.push_str(&format!(" [Filter: {}]", self.filter_text));
        }

        title
    }
}
