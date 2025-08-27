use crossterm::style::Color;
use kacemon_core::Theme;

/// Color scheme for the TUI
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub border: Color,
    pub highlight: Color,
    pub warning: Color,
    pub error: Color,
    pub success: Color,
    pub muted: Color,
    pub gauge_bg: Color,
    pub gauge_fill: Color,
    pub table_header: Color,
    pub table_row_alt: Color,
    pub table_selected: Color,
}

impl ColorScheme {
    pub fn new(theme: &Theme, no_color: bool) -> Self {
        if no_color {
            Self::no_color()
        } else {
            match theme {
                Theme::Dark => Self::dark(),
                Theme::Light => Self::light(),
            }
        }
    }

    fn dark() -> Self {
        Self {
            background: Color::Black,
            foreground: Color::White,
            accent: Color::Cyan,
            border: Color::DarkGrey,
            highlight: Color::Yellow,
            warning: Color::DarkYellow,
            error: Color::Red,
            success: Color::Green,
            muted: Color::DarkGrey,
            gauge_bg: Color::DarkGrey,
            gauge_fill: Color::Blue,
            table_header: Color::Cyan,
            table_row_alt: Color::DarkGrey,
            table_selected: Color::Yellow,
        }
    }

    fn light() -> Self {
        Self {
            background: Color::White,
            foreground: Color::Black,
            accent: Color::Blue,
            border: Color::Grey,
            highlight: Color::DarkBlue,
            warning: Color::DarkYellow,
            error: Color::DarkRed,
            success: Color::DarkGreen,
            muted: Color::Grey,
            gauge_bg: Color::Grey,
            gauge_fill: Color::Blue,
            table_header: Color::DarkBlue,
            table_row_alt: Color::Grey,
            table_selected: Color::DarkBlue,
        }
    }

    fn no_color() -> Self {
        Self {
            background: Color::Reset,
            foreground: Color::Reset,
            accent: Color::Reset,
            border: Color::Reset,
            highlight: Color::Reset,
            warning: Color::Reset,
            error: Color::Reset,
            success: Color::Reset,
            muted: Color::Reset,
            gauge_bg: Color::Reset,
            gauge_fill: Color::Reset,
            table_header: Color::Reset,
            table_row_alt: Color::Reset,
            table_selected: Color::Reset,
        }
    }

    /// Get color for CPU usage percentage
    pub fn cpu_usage_color(&self, usage: f32) -> Color {
        if usage > 90.0 {
            self.error
        } else if usage > 70.0 {
            self.warning
        } else {
            self.success
        }
    }

    /// Get color for memory usage percentage
    pub fn memory_usage_color(&self, usage: f32) -> Color {
        if usage > 90.0 {
            self.error
        } else if usage > 80.0 {
            self.warning
        } else {
            self.success
        }
    }

    /// Get color for process state
    pub fn process_state_color(&self, state: &kacemon_core::ProcessState) -> Color {
        match state {
            kacemon_core::ProcessState::Running => self.success,
            kacemon_core::ProcessState::Sleeping => self.muted,
            kacemon_core::ProcessState::Waiting => self.warning,
            kacemon_core::ProcessState::Zombie => self.error,
            kacemon_core::ProcessState::Stopped => self.error,
            kacemon_core::ProcessState::Paging => self.warning,
            kacemon_core::ProcessState::Dead => self.error,
            kacemon_core::ProcessState::Unknown => self.muted,
        }
    }
}
