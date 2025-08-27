/// Rectangle for layout calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { x, y, width, height }
    }

    pub fn from_terminal_size() -> anyhow::Result<Self> {
        let (width, height) = crossterm::terminal::size()?;
        Ok(Self::new(0, 0, width, height))
    }

    pub fn right(&self) -> u16 {
        self.x.saturating_add(self.width)
    }

    pub fn bottom(&self) -> u16 {
        self.y.saturating_add(self.height)
    }

    pub fn area(&self) -> u16 {
        self.width.saturating_mul(self.height)
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right() && self.right() > other.x && self.y < other.bottom() && self.bottom() > other.y
    }

    pub fn inner(&self, margin: u16) -> Self {
        let doubled_margin = margin.saturating_mul(2);
        Self {
            x: self.x.saturating_add(margin),
            y: self.y.saturating_add(margin),
            width: self.width.saturating_sub(doubled_margin),
            height: self.height.saturating_sub(doubled_margin),
        }
    }
}

/// Layout manager for the TUI
pub struct Layout {
    terminal_rect: Rect,
}

impl Layout {
    pub fn new() -> anyhow::Result<Self> {
        let terminal_rect = Rect::from_terminal_size()?;
        Ok(Self { terminal_rect })
    }

    pub fn update_terminal_size(&mut self) -> anyhow::Result<()> {
        self.terminal_rect = Rect::from_terminal_size()?;
        Ok(())
    }

    pub fn terminal_rect(&self) -> Rect {
        self.terminal_rect
    }

    /// Calculate layout for the main screen
    pub fn main_layout(&self) -> MainLayout {
        let rect = self.terminal_rect;
        
        // Top bar: hostname, uptime, etc. (1 line)
        let top_bar = Rect::new(rect.x, rect.y, rect.width, 1);
        
        // CPU and memory gauges (4 lines)
        let gauges_height = 4;
        let gauges = Rect::new(
            rect.x,
            top_bar.bottom(),
            rect.width,
            gauges_height.min(rect.height.saturating_sub(1).saturating_sub(1)),
        );
        
        // Footer: keybind hints (1 line)
        let footer = Rect::new(
            rect.x,
            rect.bottom().saturating_sub(1),
            rect.width,
            1,
        );
        
        // Bottom section: split into network (left) and temperature (right)
        let available_height = footer.y.saturating_sub(gauges.bottom());
        let bottom_height = (available_height / 3).max(4);
        let bottom_y = footer.y.saturating_sub(bottom_height);
        
        // Split bottom section vertically (50/50)
        let bottom_width = rect.width / 2;
        let network = Rect::new(
            rect.x,
            bottom_y,
            bottom_width,
            bottom_height,
        );
        
        let temperature = Rect::new(
            rect.x + bottom_width,
            bottom_y,
            rect.width - bottom_width,
            bottom_height,
        );
        
        // Process table: remaining space between gauges and bottom section
        let table = Rect::new(
            rect.x,
            gauges.bottom(),
            rect.width,
            bottom_y.saturating_sub(gauges.bottom()),
        );

        MainLayout {
            top_bar,
            gauges,
            table,
            network,
            temperature,
            footer,
        }
    }

    /// Calculate layout for gauges section
    pub fn gauges_layout(&self, area: Rect) -> GaugesLayout {
        let width = area.width;
        let height = area.height;
        
        // Split into two columns for CPU and memory
        let col_width = width / 2;
        
        let cpu_area = Rect::new(area.x, area.y, col_width, height);
        let memory_area = Rect::new(area.x + col_width, area.y, width - col_width, height);
        
        GaugesLayout {
            cpu: cpu_area,
            memory: memory_area,
        }
    }

    /// Calculate layout for process table columns
    pub fn table_layout(&self, area: Rect, columns: &[&str]) -> Vec<Rect> {
        if columns.is_empty() || area.width == 0 {
            return Vec::new();
        }

        let total_width = area.width as usize;
        let num_columns = columns.len();
        
        // Define preferred widths for different column types
        let column_widths: Vec<usize> = columns.iter().map(|&col| {
            match col {
                "PID" => 8,
                "USER" => 12,
                "CPU%" => 6,
                "MEM%" => 6,
                "RSS" => 8,
                "VSZ" => 8,
                "THR" => 4,
                "STATE" => 6,
                "TIME" => 8,
                "NAME" => 20, // This will expand to fill remaining space
                _ => 10,
            }
        }).collect();

        // Calculate total preferred width
        let total_preferred: usize = column_widths.iter().sum();
        
        // Calculate actual widths
        let actual_widths: Vec<u16> = if total_preferred <= total_width {
            // We have enough space, expand the NAME column to fill remaining space
            let mut widths = column_widths.clone();
            if let Some(name_idx) = columns.iter().position(|&col| col == "NAME") {
                let used_width: usize = widths.iter().enumerate()
                    .filter(|(i, _)| *i != name_idx)
                    .map(|(_, w)| *w)
                    .sum();
                widths[name_idx] = total_width.saturating_sub(used_width);
            }
            widths.into_iter().map(|w| w as u16).collect()
        } else {
            // Not enough space, scale down proportionally
            column_widths.into_iter().map(|w| {
                ((w * total_width) / total_preferred).max(1) as u16
            }).collect()
        };

        // Create rectangles
        let mut rects = Vec::with_capacity(num_columns);
        let mut x = area.x;
        
        for width in actual_widths {
            rects.push(Rect::new(x, area.y, width, area.height));
            x = x.saturating_add(width);
        }

        rects
    }
}

#[derive(Debug, Clone)]
pub struct MainLayout {
    pub top_bar: Rect,
    pub gauges: Rect,
    pub table: Rect,
    pub network: Rect,
    pub temperature: Rect,
    pub footer: Rect,
}

#[derive(Debug, Clone)]
pub struct GaugesLayout {
    pub cpu: Rect,
    pub memory: Rect,
}
