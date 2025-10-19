# Text Input System Usage Guide

This guide demonstrates how to use the modular text input system in your Ratatui applications.

## Overview

The refactored architecture provides:

1. **Reusable TextInput Component** (`ui::text_input::TextInput`)
2. **Generic Input Handler** (`ui::input_handler::TextInputState`)
3. **Screen-based Architecture** (`screens` module)

## Basic TextInput Component Usage

### Simple Text Input

```rust
use crate::ui::text_input::TextInput;

// Basic text input
let input = TextInput::new(
    &form.username,           // Current value
    form.username_cursor,     // Cursor position
    "Username",               // Title
    "Enter your username"     // Placeholder
);

// Render with focus and editing state
let input = TextInput::new(&value, cursor, "Title", "Placeholder")
    .focused(is_active)
    .editing(is_editing);
```

### Password Fields

```rust
// Password input with masking
let password_input = TextInput::password_field(
    &form.password,
    form.password_cursor,
    "Password",
    "Enter your password"
);

// Or using the builder pattern
let password_input = TextInput::new(&password, cursor, "Password", "Enter password")
    .focused(true)
    .editing(true)
    .password(true);
```

### Readonly Display

```rust
// Display-only field
let readonly_input = TextInput::readonly(&user.email, "Email Address");
```

### Editable Field

```rust
// Ready-to-edit field
let editable_input = TextInput::editable(
    &form.message,
    form.message_cursor,
    "Message",
    "Type your message"
);
```

## Input Handler Usage

### Single Field Management

```rust
use crate::ui::input_handler::TextInputState;

// Create input state
let mut input_state = TextInputState::new("initial value".to_string());

// Handle key events automatically
if input_state.handle_key_event(key_event) {
    // Key was handled by the input system
    // The input_state is now updated
}

// Manual operations
input_state.enter_char('a');
input_state.delete_char();
input_state.move_cursor_left();
input_state.move_cursor_right();
input_state.clear();
```

### Multi-Field Forms

```rust
use crate::ui::input_handler::{TextInputState, FormHandler};

#[derive(Clone, PartialEq)]
enum FormField {
    Name,
    Email,
    Message,
}

// Create form with multiple fields
let fields = vec![
    TextInputState::new("John".to_string()),
    TextInputState::new("john@example.com".to_string()),
    TextInputState::new("".to_string()),
];

let mut form = FormHandler::new(fields, FormField::Name);

// Handle input for active field
if let Some(active_field) = form.get_active_field_mut(0) {
    active_field.handle_key_event(key_event);
}
```

## Creating New Screens

### 1. Define Screen State

```rust
#[derive(Debug)]
pub struct MyCustomScreen {
    pub text_inputs: Vec<TextInputState>,
    pub active_field: usize,
    pub editing: bool,
}

impl Default for MyCustomScreen {
    fn default() -> Self {
        Self {
            text_inputs: vec![
                TextInputState::new("".to_string()),
                TextInputState::new("".to_string()),
            ],
            active_field: 0,
            editing: false,
        }
    }
}
```

### 2. Implement Input Handling

```rust
impl MyCustomScreen {
    pub fn handle_text_input(&mut self, key: KeyEvent) -> bool {
        if self.editing {
            if let Some(active_input) = self.text_inputs.get_mut(self.active_field) {
                return active_input.handle_key_event(key);
            }
        }
        false
    }

    pub fn next_field(&mut self) {
        self.active_field = (self.active_field + 1) % self.text_inputs.len();
    }

    pub fn previous_field(&mut self) {
        self.active_field = if self.active_field == 0 {
            self.text_inputs.len() - 1
        } else {
            self.active_field - 1
        };
    }
}
```

### 3. Implement ScreenHandler

```rust
impl ScreenHandler for MyCustomScreen {
    fn render(&self, frame: &mut Frame, _app_state: &AppState) {
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(frame.area());

        // Render each text input
        for (i, input_state) in self.text_inputs.iter().enumerate() {
            let is_active = i == self.active_field;
            let text_input = TextInput::new(
                &input_state.value,
                input_state.cursor_position,
                &format!("Field {}", i + 1),
                "Enter value"
            )
            .focused(is_active)
            .editing(self.editing && is_active);

            frame.render_widget(text_input, chunks[i]);
        }
    }

    fn handle_key_event(&self, key: KeyEvent, app_state: &mut AppState) -> Option<Screen> {
        match key.code {
            KeyCode::Tab => {
                // Handle field navigation
                // Note: In real implementation, you'd need mutable access to self
                Some(Screen::MyCustom)
            }
            KeyCode::Enter => {
                // Toggle editing mode
                Some(Screen::MyCustom)
            }
            KeyCode::Esc => {
                // Exit or stop editing
                Some(Screen::Login)
            }
            _ => {
                // Handle text input
                // Note: You'd need mutable access to handle actual input
                Some(Screen::MyCustom)
            }
        }
    }
}
```

## Advanced Features

### Custom Key Bindings

```rust
impl TextInputState {
    pub fn handle_custom_keys(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                match c {
                    'a' => { self.move_cursor_to_start(); true }
                    'e' => { self.move_cursor_to_end(); true }
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
                        // Delete word backwards (implemented in TextInputState)
                        true 
                    }
                    _ => false,
                }
            }
            _ => self.handle_key_event(key),
        }
    }
}
```

### Validation

```rust
impl MyCustomScreen {
    pub fn validate_current_field(&self) -> Result<(), String> {
        if let Some(input) = self.text_inputs.get(self.active_field) {
            match self.active_field {
                0 => { // Name field
                    if input.value.trim().is_empty() {
                        Err("Name cannot be empty".to_string())
                    } else {
                        Ok(())
                    }
                }
                1 => { // Email field
                    if !input.value.contains('@') {
                        Err("Invalid email format".to_string())
                    } else {
                        Ok(())
                    }
                }
                _ => Ok(())
            }
        } else {
            Err("Invalid field".to_string())
        }
    }
}
```

## Integration with Matrix SDK

```rust
// Example of how to integrate with Matrix login
impl LoginScreen {
    pub async fn attempt_login(&self, app_state: &AppState) -> Result<(), matrix_sdk::Error> {
        let username = &app_state.login_form.username;
        let password = &app_state.login_form.password;
        let homeserver = &app_state.login_form.homeserver;

        // Use Matrix SDK for actual login
        // This is where you'd integrate with matrix_sdk crate
        todo!("Implement Matrix SDK login")
    }
}
```

## Best Practices

1. **Keep State Separate**: Store text input state separately from UI rendering logic
2. **Use Builder Pattern**: Leverage the TextInput builder methods for clean code
3. **Handle Focus Properly**: Always indicate which field is active
4. **Validate Input**: Implement validation for each field type
5. **Consistent Key Bindings**: Use standard terminal key bindings (Ctrl+A, Ctrl+E, etc.)
6. **Error Handling**: Provide clear feedback for validation errors
7. **Accessibility**: Use proper titles and placeholders for screen readers

## Common Patterns

### Multi-step Forms

```rust
pub struct WizardScreen {
    pub steps: Vec<FormStep>,
    pub current_step: usize,
}

pub struct FormStep {
    pub title: String,
    pub fields: Vec<TextInputState>,
    pub active_field: usize,
}
```

### Dynamic Fields

```rust
pub struct DynamicFormScreen {
    pub fields: Vec<(String, TextInputState)>, // (label, input)
    pub active_field: usize,
}

impl DynamicFormScreen {
    pub fn add_field(&mut self, label: String) {
        self.fields.push((label, TextInputState::default()));
    }

    pub fn remove_field(&mut self, index: usize) {
        if index < self.fields.len() {
            self.fields.remove(index);
            if self.active_field >= self.fields.len() && !self.fields.is_empty() {
                self.active_field = self.fields.len() - 1;
            }
        }
    }
}
```

This modular approach allows you to create any type of form or input screen while reusing the core text input functionality!