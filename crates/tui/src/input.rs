use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Input events that the application can handle
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    // Navigation
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    Home,
    End,
    
    // Sorting and filtering
    CycleSort,
    StartFilter,
    ClearFilter,
    FilterChar(char),
    FilterBackspace,
    
    // Display controls
    ToggleColumns,
    ChangeRefreshRate,
    ToggleTreeView,
    
    // Process control
    KillProcess,
    
    // Application control
    ShowHelp,
    Quit,
    
    // System
    Resize,
    Tick,
    
    // Unknown/unhandled
    Unknown,
}

/// Input handler that converts crossterm events to application events
pub struct InputHandler {
    in_filter_mode: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            in_filter_mode: false,
        }
    }

    /// Poll for input events with a timeout
    pub fn poll_event(&mut self, timeout: Duration) -> anyhow::Result<Option<InputEvent>> {
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key_event) => Ok(Some(self.handle_key_event(key_event))),
                Event::Resize(_, _) => Ok(Some(InputEvent::Resize)),
                _ => Ok(Some(InputEvent::Unknown)),
            }
        } else {
            Ok(Some(InputEvent::Tick))
        }
    }

    /// Handle keyboard input
    fn handle_key_event(&mut self, key_event: KeyEvent) -> InputEvent {
        // Handle Ctrl+C for quit
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            match key_event.code {
                KeyCode::Char('c') => return InputEvent::Quit,
                _ => {}
            }
        }

        // Handle filter mode
        if self.in_filter_mode {
            return self.handle_filter_input(key_event);
        }

        // Handle normal mode
        match key_event.code {
            // Navigation
            KeyCode::Up | KeyCode::Char('k') => InputEvent::MoveUp,
            KeyCode::Down | KeyCode::Char('j') => InputEvent::MoveDown,
            KeyCode::PageUp => InputEvent::PageUp,
            KeyCode::PageDown => InputEvent::PageDown,
            KeyCode::Home => InputEvent::Home,
            KeyCode::End => InputEvent::End,
            
            // Sorting and filtering
            KeyCode::Char('s') => InputEvent::CycleSort,
            KeyCode::Char('/') => {
                self.in_filter_mode = true;
                InputEvent::StartFilter
            },
            KeyCode::Esc => InputEvent::ClearFilter,
            
            // Display controls
            KeyCode::Char('c') => InputEvent::ToggleColumns,
            KeyCode::Char('r') => InputEvent::ChangeRefreshRate,
            KeyCode::Char('t') => InputEvent::ToggleTreeView,
            
            // Process control  
            KeyCode::Char('K') => InputEvent::KillProcess, // Use uppercase K to avoid conflict with navigation
            
            // Application control
            KeyCode::Char('?') => InputEvent::ShowHelp,
            KeyCode::Char('q') => InputEvent::Quit,
            
            _ => InputEvent::Unknown,
        }
    }

    /// Handle input while in filter mode
    fn handle_filter_input(&mut self, key_event: KeyEvent) -> InputEvent {
        match key_event.code {
            KeyCode::Esc => {
                self.in_filter_mode = false;
                InputEvent::ClearFilter
            },
            KeyCode::Enter => {
                self.in_filter_mode = false;
                InputEvent::Unknown // Just exit filter mode
            },
            KeyCode::Backspace => InputEvent::FilterBackspace,
            KeyCode::Char(c) => InputEvent::FilterChar(c),
            _ => InputEvent::Unknown,
        }
    }

    /// Check if currently in filter mode
    pub fn is_in_filter_mode(&self) -> bool {
        self.in_filter_mode
    }

    /// Exit filter mode
    pub fn exit_filter_mode(&mut self) {
        self.in_filter_mode = false;
    }
}
