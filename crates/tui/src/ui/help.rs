use crate::ui::{ColorScheme, Rect};
use crossterm::{
    cursor,
    style::{Print, SetBackgroundColor, SetForegroundColor},
    QueueableCommand,
};
use std::io::{self, Write};

/// Help overlay widget
pub struct HelpOverlay;

impl HelpOverlay {
    pub fn render<W: Write>(
        &self,
        writer: &mut W,
        area: Rect,
        colors: &ColorScheme,
    ) -> io::Result<()> {
        // Calculate centered popup area
        let popup_width = 60.min(area.width - 4);
        let popup_height = 20.min(area.height - 4);
        let popup_x = area.x + (area.width - popup_width) / 2;
        let popup_y = area.y + (area.height - popup_height) / 2;
        
        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        // Help content
        let help_lines = vec![
            "                    SRMON HELP",
            "",
            "Navigation:",
            "  ↑/k, ↓/j         Move up/down in process list",
            "  Page Up/Down     Page up/down in process list", 
            "  Home/End         Go to top/bottom of list",
            "",
            "Sorting:",
            "  s                Cycle sort (CPU% → MEM% → PID → NAME)",
            "",
            "Filtering:",
            "  /                Filter processes by name/command",
            "  Esc              Clear current filter",
            "",
            "Display:",
            "  c                Toggle column visibility",
            "  r                Change refresh rate",
            "  t                Toggle tree view",
            "",
            "Process Control:",
            "  k                Send SIGTERM to selected process",
            "",
            "Other:",
            "  ?                Show this help",
            "  q, Ctrl+C        Quit application",
        ];

        // Render popup background
        for y in popup_area.y..popup_area.bottom() {
            writer.queue(cursor::MoveTo(popup_area.x, y))?;
            writer.queue(SetBackgroundColor(colors.background))?;
            writer.queue(SetForegroundColor(colors.foreground))?;
            writer.queue(Print(" ".repeat(popup_area.width as usize)))?;
        }

        // Render border
        self.render_border(writer, popup_area, colors)?;

        // Render help content
        let content_area = popup_area.inner(1);
        for (i, line) in help_lines.iter().enumerate() {
            if i >= content_area.height as usize {
                break;
            }

            writer.queue(cursor::MoveTo(content_area.x, content_area.y + i as u16))?;
            
            // Color the title differently
            if i == 0 {
                writer.queue(SetForegroundColor(colors.accent))?;
            } else if line.trim().is_empty() {
                writer.queue(SetForegroundColor(colors.foreground))?;
            } else if line.ends_with(':') {
                writer.queue(SetForegroundColor(colors.table_header))?;
            } else {
                writer.queue(SetForegroundColor(colors.foreground))?;
            }

            let truncated = if line.len() > content_area.width as usize {
                &line[..content_area.width as usize]
            } else {
                line
            };

            writer.queue(Print(truncated))?;
        }

        // Show how to close help
        if popup_area.height > 2 {
            writer.queue(cursor::MoveTo(
                popup_area.x + popup_area.width - 20,
                popup_area.y + popup_area.height - 1,
            ))?;
            writer.queue(SetForegroundColor(colors.muted))?;
            writer.queue(Print("Press ? to close"))?;
        }

        Ok(())
    }

    fn render_border<W: Write>(
        &self,
        writer: &mut W,
        area: Rect,
        colors: &ColorScheme,
    ) -> io::Result<()> {
        writer.queue(SetForegroundColor(colors.border))?;
        writer.queue(SetBackgroundColor(colors.background))?;

        // Top border
        writer.queue(cursor::MoveTo(area.x, area.y))?;
        writer.queue(Print("┌"))?;
        writer.queue(Print("─".repeat(area.width as usize - 2)))?;
        writer.queue(Print("┐"))?;

        // Side borders
        for y in area.y + 1..area.bottom() - 1 {
            writer.queue(cursor::MoveTo(area.x, y))?;
            writer.queue(Print("│"))?;
            writer.queue(cursor::MoveTo(area.right() - 1, y))?;
            writer.queue(Print("│"))?;
        }

        // Bottom border
        writer.queue(cursor::MoveTo(area.x, area.bottom() - 1))?;
        writer.queue(Print("└"))?;
        writer.queue(Print("─".repeat(area.width as usize - 2)))?;
        writer.queue(Print("┘"))?;

        Ok(())
    }
}
