use crate::ui::{ColorScheme, Layout, Rect};
use crossterm::{
    cursor,
    style::{Print, SetBackgroundColor, SetForegroundColor},
    QueueableCommand,
};
use kacemon_core::SystemSnapshot;
use std::io::{self, Write};

/// Main drawing coordinator
pub struct Drawer {
    layout: Layout,
    colors: ColorScheme,
}

impl Drawer {
    pub fn new(layout: Layout, colors: ColorScheme) -> Self {
        Self { layout, colors }
    }

    /// Draw the complete UI
    pub fn draw<W: Write>(
        &mut self,
        writer: &mut W,
        snapshot: &SystemSnapshot,
        app_state: &DrawState,
    ) -> io::Result<()> {
        // Move to top without clearing (reduces flicker)
        writer.queue(cursor::MoveTo(0, 0))?;

        let main_layout = self.layout.main_layout();

        // Draw top bar
        self.draw_top_bar(writer, main_layout.top_bar, &snapshot.system)?;

        // Draw gauges section
        self.draw_gauges(writer, main_layout.gauges, snapshot)?;

        // Draw process table
        self.draw_process_table(writer, main_layout.table, snapshot, app_state)?;

        // Draw network section
        self.draw_network_section(writer, main_layout.network, snapshot)?;

        // Draw temperature section
        self.draw_temperature_section(writer, main_layout.temperature, snapshot)?;

        // Draw footer
        self.draw_footer(writer, main_layout.footer)?;

        // Draw overlays
        if app_state.show_help {
            self.draw_help_overlay(writer)?;
        }

        if app_state.in_filter_mode {
            self.draw_filter_input(writer, main_layout.footer, &app_state.filter_text)?;
        }

        writer.flush()?;
        Ok(())
    }

    fn draw_top_bar<W: Write>(
        &self,
        writer: &mut W,
        area: Rect,
        system_info: &kacemon_core::SystemInfo,
    ) -> io::Result<()> {
        let widget = crate::ui::TopBar;
        widget.render(writer, area, system_info, &self.colors)
    }

    fn draw_gauges<W: Write>(
        &self,
        writer: &mut W,
        area: Rect,
        snapshot: &SystemSnapshot,
    ) -> io::Result<()> {
        let gauges_layout = self.layout.gauges_layout(area);

        // CPU gauges
        let cpu_widget = crate::ui::CpuGauges;
        cpu_widget.render(writer, gauges_layout.cpu, &snapshot.cpu_cores, &self.colors)?;

        // Memory gauges
        let memory_widget = crate::ui::MemoryGauges;
        memory_widget.render(writer, gauges_layout.memory, &snapshot.memory, &self.colors)?;

        Ok(())
    }

    fn draw_process_table<W: Write>(
        &self,
        writer: &mut W,
        area: Rect,
        _snapshot: &SystemSnapshot,
        app_state: &DrawState,
    ) -> io::Result<()> {
        let widget = crate::ui::ProcessTable;
        let columns: Vec<&str> = app_state.visible_columns.iter().map(|s| s.as_str()).collect();

        widget.render(
            writer,
            area,
            &app_state.filtered_processes,
            &columns,
            app_state.selected_index,
            app_state.table_start_index,
            &self.colors,
        )
    }

    fn draw_network_section<W: Write>(
        &self,
        writer: &mut W,
        area: Rect,
        snapshot: &SystemSnapshot,
    ) -> io::Result<()> {
        let widget = crate::ui::NetworkGauges;
        widget.render(writer, area, &snapshot.networks, &self.colors)
    }

    fn draw_temperature_section<W: Write>(
        &self,
        writer: &mut W,
        area: Rect,
        snapshot: &SystemSnapshot,
    ) -> io::Result<()> {
        let widget = crate::ui::TemperatureGauge;
        widget.render(writer, area, &snapshot.temperatures, &self.colors)
    }

    fn draw_footer<W: Write>(&self, writer: &mut W, area: Rect) -> io::Result<()> {
        let widget = crate::ui::Footer;
        widget.render(writer, area, &self.colors)
    }

    fn draw_help_overlay<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let terminal_rect = self.layout.terminal_rect();
        let widget = crate::ui::HelpOverlay;
        widget.render(writer, terminal_rect, &self.colors)
    }

    fn draw_filter_input<W: Write>(
        &self,
        writer: &mut W,
        area: Rect,
        filter_text: &str,
    ) -> io::Result<()> {
        writer.queue(cursor::MoveTo(area.x, area.y))?;
        writer.queue(SetForegroundColor(self.colors.accent))?;
        writer.queue(SetBackgroundColor(self.colors.background))?;
        writer.queue(Print(format!("Filter: {}_", filter_text)))?;
        Ok(())
    }

    /// Update layout for terminal size changes
    pub fn update_layout(&mut self) -> anyhow::Result<()> {
        self.layout.update_terminal_size()
    }

    /// Update color scheme
    pub fn update_colors(&mut self, colors: ColorScheme) {
        self.colors = colors;
    }
}

/// State needed for drawing
#[derive(Debug, Clone)]
pub struct DrawState {
    pub filtered_processes: Vec<kacemon_core::ProcessInfo>,
    pub visible_columns: Vec<String>,
    pub selected_index: usize,
    pub table_start_index: usize,
    pub show_help: bool,
    pub in_filter_mode: bool,
    pub filter_text: String,
}

impl Default for DrawState {
    fn default() -> Self {
        Self {
            filtered_processes: Vec::new(),
            visible_columns: vec![
                "PID".to_string(),
                "NAME".to_string(),
                "USER".to_string(),
                "CPU%".to_string(),
                "MEM%".to_string(),
                "RSS".to_string(),
                "STATE".to_string(),
            ],
            selected_index: 0,
            table_start_index: 0,
            show_help: false,
            in_filter_mode: false,
            filter_text: String::new(),
        }
    }
}
