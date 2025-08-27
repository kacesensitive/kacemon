use crate::input::{InputEvent, InputHandler};
use crate::ui::ColorScheme;
use kacemon_core::{Config, MetricsCollector, ProcessColumns, SortKey, SystemSnapshot};
use std::time::{Duration, Instant};

/// Application state
pub struct App {
    // Core components
    config: Config,
    metrics_collector: MetricsCollector,
    input_handler: InputHandler,
    
    // UI state
    colors: ColorScheme,
    layout: crate::ui::Layout,
    
    // Data
    current_snapshot: Option<SystemSnapshot>,
    last_update: Instant,
    
    // Process table state
    selected_process_index: usize,
    table_start_index: usize,
    current_sort: SortKey,
    sort_reverse: bool,
    filter_text: String,
    visible_columns: Vec<String>,
    
    // UI state
    show_help: bool,
    quit_requested: bool,
    tree_view: bool,
    
    // Performance tracking
    update_count: u64,
    render_count: u64,
}

impl App {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let mut metrics_collector = MetricsCollector::new()?;
        metrics_collector.init()?;
        
        let colors = ColorScheme::new(&config.theme, config.no_color);
        let layout = crate::ui::Layout::new()?;
        let input_handler = InputHandler::new();
        
        let visible_columns = Self::default_visible_columns(&config.process_columns);
        
        Ok(Self {
            config,
            metrics_collector,
            input_handler,
            colors,
            layout,
            current_snapshot: None,
            last_update: Instant::now(),
            selected_process_index: 0,
            table_start_index: 0,
            current_sort: SortKey::Cpu,
            sort_reverse: true, // Default to descending for CPU usage
            filter_text: String::new(),
            visible_columns,
            show_help: false,
            quit_requested: false,
            tree_view: false,
            update_count: 0,
            render_count: 0,
        })
    }

    /// Main application loop
    pub fn run<W: std::io::Write>(&mut self, writer: &mut W) -> anyhow::Result<()> {
        // Initialize terminal
        self.setup_terminal()?;
        
        // Ensure we restore terminal on exit
        let _terminal_guard = TerminalGuard;
        
        // Clear screen once at startup
        crossterm::execute!(
            writer,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )?;
        
        // Initial data collection and render
        self.update_data()?;
        self.render(writer)?;
        
        // Main loop
        let refresh_interval = self.config.refresh_interval();
        let mut last_refresh = Instant::now();
        
        while !self.quit_requested {
            // Calculate timeout for next update
            let elapsed_since_refresh = last_refresh.elapsed();
            let timeout = if elapsed_since_refresh >= refresh_interval {
                Duration::from_millis(10) // Very short timeout to update immediately
            } else {
                refresh_interval - elapsed_since_refresh
            };
            
            // Poll for input
            if let Some(event) = self.input_handler.poll_event(timeout)? {
                let needs_redraw = self.needs_redraw_after_input(&event);
                self.handle_event(event);
                
                // Only render after input if it affects display (reduce unnecessary redraws)
                if needs_redraw {
                    self.render(writer)?;
                }
            }
            
            // Update data if it's time
            if last_refresh.elapsed() >= refresh_interval {
                self.update_data()?;
                self.render(writer)?;
                last_refresh = Instant::now();
            }
        }
        
        Ok(())
    }

    /// Update system metrics
    fn update_data(&mut self) -> anyhow::Result<()> {
        self.current_snapshot = Some(self.metrics_collector.collect()?);
        self.update_count += 1;
        self.last_update = Instant::now();
        Ok(())
    }

    /// Handle input events
    fn handle_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::Quit => self.quit_requested = true,
            InputEvent::ShowHelp => self.show_help = !self.show_help,
            InputEvent::Resize => {
                if let Ok(()) = self.layout.update_terminal_size() {
                    // Terminal size updated, render will handle the new layout
                }
            },
            InputEvent::Tick => {
                // Periodic tick, no action needed in normal mode
            },
            
            // Navigation (only when not showing help)
            _ if self.show_help => {
                if event == InputEvent::ShowHelp {
                    self.show_help = false;
                }
            },
            
            InputEvent::MoveUp => self.move_selection(-1),
            InputEvent::MoveDown => self.move_selection(1),
            InputEvent::PageUp => self.move_selection(-(self.get_visible_rows() as isize)),
            InputEvent::PageDown => self.move_selection(self.get_visible_rows() as isize),
            InputEvent::Home => self.selected_process_index = 0,
            InputEvent::End => {
                if let Some(snapshot) = &self.current_snapshot {
                    let process_count = self.get_filtered_processes(&snapshot.processes).len();
                    self.selected_process_index = process_count.saturating_sub(1);
                }
            },
            
            // Sorting
            InputEvent::CycleSort => {
                self.current_sort = self.current_sort.next();
                self.sort_reverse = matches!(self.current_sort, SortKey::Cpu | SortKey::Memory);
            },
            
            // Filtering
            InputEvent::StartFilter => {
                // Filter mode handled by input handler
            },
            InputEvent::ClearFilter => {
                self.filter_text.clear();
                self.input_handler.exit_filter_mode();
            },
            InputEvent::FilterChar(c) => {
                self.filter_text.push(c);
            },
            InputEvent::FilterBackspace => {
                self.filter_text.pop();
            },
            
            // Display controls
            InputEvent::ToggleTreeView => {
                self.tree_view = !self.tree_view;
            },
            
            // Process control
            InputEvent::KillProcess => {
                if let Some(snapshot) = &self.current_snapshot {
                    let filtered_processes = self.get_filtered_processes(&snapshot.processes);
                    if let Some(_process) = filtered_processes.get(self.selected_process_index) {
                        // Attempt to kill the process
                        // Note: kill_process is not available on MetricsCollector
                        // We would need to add this functionality or use the platform provider
                    }
                }
            },
            
            _ => {
                // Unhandled event
            }
        }
        
        // Ensure selection is within bounds after any changes
        self.clamp_selection();
    }

    /// Check if input event requires a redraw
    fn needs_redraw_after_input(&self, event: &InputEvent) -> bool {
        match event {
            InputEvent::Quit => false, // No need to redraw before quitting
            _ => true, // All other inputs affect display
        }
    }

    /// Render the UI
    fn render<W: std::io::Write>(&mut self, writer: &mut W) -> anyhow::Result<()> {
        // Clear screen only once during setup, then just move cursor
        crossterm::queue!(
            writer,
            crossterm::cursor::MoveTo(0, 0),
        )?;

        let terminal_rect = self.layout.terminal_rect();
        let main_layout = self.layout.main_layout();

        if let Some(snapshot) = &self.current_snapshot {
            // Render top bar
            let top_bar = crate::ui::TopBar;
            top_bar.render(writer, main_layout.top_bar, &snapshot.system, &self.colors)?;

            // Render gauges
            let gauges_layout = self.layout.gauges_layout(main_layout.gauges);
            
            let cpu_gauges = crate::ui::CpuGauges;
            cpu_gauges.render(writer, gauges_layout.cpu, &snapshot.cpu_cores, &self.colors)?;
            
            let memory_gauges = crate::ui::MemoryGauges;
            memory_gauges.render(writer, gauges_layout.memory, &snapshot.memory, &self.colors)?;

            // Render process table
            let filtered_processes = self.get_filtered_sorted_processes(&snapshot.processes);
            let columns: Vec<&str> = self.visible_columns.iter().map(|s| s.as_str()).collect();
            
            let process_table = crate::ui::ProcessTable;
            process_table.render(
                writer,
                main_layout.table,
                &filtered_processes,
                &columns,
                self.selected_process_index,
                self.table_start_index,
                &self.colors,
            )?;

            // Render network section
            let network_gauges = crate::ui::NetworkGauges;
            network_gauges.render(writer, main_layout.network, &snapshot.networks, &self.colors)?;

            // Render temperature section
            let temperature_gauge = crate::ui::TemperatureGauge;
            temperature_gauge.render(writer, main_layout.temperature, &snapshot.temperatures, &self.colors)?;
        }

        // Render footer
        let footer = crate::ui::Footer;
        footer.render(writer, main_layout.footer, &self.colors)?;

        // Render help overlay if shown
        if self.show_help {
            let help = crate::ui::HelpOverlay;
            help.render(writer, terminal_rect, &self.colors)?;
        }

        // Show filter text if in filter mode
        if self.input_handler.is_in_filter_mode() {
            self.render_filter_prompt(writer, main_layout.footer)?;
        }

        writer.flush()?;
        self.render_count += 1;
        Ok(())
    }

    /// Render filter prompt
    fn render_filter_prompt<W: std::io::Write>(
        &self,
        writer: &mut W,
        area: crate::ui::Rect,
    ) -> anyhow::Result<()> {
        use crossterm::{cursor, style::Print, QueueableCommand};
        
        writer.queue(cursor::MoveTo(area.x, area.y))?;
        writer.queue(Print(format!("Filter: {}_", self.filter_text)))?;
        Ok(())
    }

    /// Get filtered processes
    fn get_filtered_processes(&self, processes: &[kacemon_core::ProcessInfo]) -> Vec<kacemon_core::ProcessInfo> {
        if self.filter_text.is_empty() {
            processes.to_vec()
        } else {
            let filter_lower = self.filter_text.to_lowercase();
            processes
                .iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&filter_lower)
                        || p.cmd.join(" ").to_lowercase().contains(&filter_lower)
                        || p.user.to_lowercase().contains(&filter_lower)
                        || p.pid.to_string().contains(&filter_lower)
                })
                .cloned()
                .collect()
        }
    }

    /// Get filtered and sorted processes
    fn get_filtered_sorted_processes(&self, processes: &[kacemon_core::ProcessInfo]) -> Vec<kacemon_core::ProcessInfo> {
        let mut filtered = self.get_filtered_processes(processes);
        self.sort_processes(&mut filtered);
        filtered
    }

    /// Sort processes according to current sort settings
    fn sort_processes(&self, processes: &mut [kacemon_core::ProcessInfo]) {
        match self.current_sort {
            SortKey::Cpu => {
                processes.sort_by(|a, b| {
                    if self.sort_reverse {
                        b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        a.cpu_percent.partial_cmp(&b.cpu_percent).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            }
            SortKey::Memory => {
                processes.sort_by(|a, b| {
                    if self.sort_reverse {
                        b.memory_percent.partial_cmp(&a.memory_percent).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        a.memory_percent.partial_cmp(&b.memory_percent).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            }
            SortKey::Pid => {
                processes.sort_by(|a, b| {
                    if self.sort_reverse {
                        b.pid.cmp(&a.pid)
                    } else {
                        a.pid.cmp(&b.pid)
                    }
                });
            }
            SortKey::Name => {
                processes.sort_by(|a, b| {
                    if self.sort_reverse {
                        b.name.cmp(&a.name)
                    } else {
                        a.name.cmp(&b.name)
                    }
                });
            }
        }
    }

    /// Move selection by delta
    fn move_selection(&mut self, delta: isize) {
        let new_index = if delta < 0 {
            self.selected_process_index.saturating_sub((-delta) as usize)
        } else {
            self.selected_process_index.saturating_add(delta as usize)
        };
        
        self.selected_process_index = new_index;
        self.clamp_selection();
    }

    /// Ensure selection is within valid bounds
    fn clamp_selection(&mut self) {
        if let Some(snapshot) = &self.current_snapshot {
            let process_count = self.get_filtered_processes(&snapshot.processes).len();
            if process_count == 0 {
                self.selected_process_index = 0;
                self.table_start_index = 0;
            } else {
                self.selected_process_index = self.selected_process_index.min(process_count - 1);
                
                // Adjust table start index to keep selection visible
                let visible_rows = self.get_visible_rows();
                if self.selected_process_index < self.table_start_index {
                    self.table_start_index = self.selected_process_index;
                } else if self.selected_process_index >= self.table_start_index + visible_rows {
                    self.table_start_index = self.selected_process_index.saturating_sub(visible_rows - 1);
                }
            }
        }
    }

    /// Get number of visible rows in process table
    fn get_visible_rows(&self) -> usize {
        let main_layout = self.layout.main_layout();
        (main_layout.table.height.saturating_sub(1)) as usize // Subtract header row
    }

    /// Get default visible columns from configuration
    fn default_visible_columns(columns: &ProcessColumns) -> Vec<String> {
        let mut visible = Vec::new();
        
        if columns.pid { visible.push("PID".to_string()); }
        if columns.name { visible.push("NAME".to_string()); }
        if columns.user { visible.push("USER".to_string()); }
        if columns.cpu_percent { visible.push("CPU%".to_string()); }
        if columns.memory_percent { visible.push("MEM%".to_string()); }
        if columns.memory_rss { visible.push("RSS".to_string()); }
        if columns.memory_vsz { visible.push("VSZ".to_string()); }
        if columns.threads { visible.push("THR".to_string()); }
        if columns.state { visible.push("STATE".to_string()); }
        if columns.start_time { visible.push("TIME".to_string()); }
        
        visible
    }

    fn setup_terminal(&self) -> anyhow::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::EnterAlternateScreen,
            crossterm::cursor::Hide
        )?;
        Ok(())
    }
}

/// RAII guard to restore terminal state on drop
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::cursor::Show,
            crossterm::terminal::LeaveAlternateScreen
        );
        let _ = crossterm::terminal::disable_raw_mode();
    }
}
