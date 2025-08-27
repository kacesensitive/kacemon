use crate::ui::{ColorScheme, Rect};
use crossterm::{
    cursor,
    style::{Print, SetBackgroundColor, SetForegroundColor},
    QueueableCommand,
};
use kacemon_core::{CpuCore, MemoryInfo, NetworkInfo, ProcessInfo, SystemInfo, TemperatureInfo};
use std::io::{self, Write};

/// Top bar widget showing system information
pub struct TopBar;

impl TopBar {
    pub fn render<W: Write>(
        &self,
        writer: &mut W,
        area: Rect,
        system_info: &SystemInfo,
        colors: &ColorScheme,
    ) -> io::Result<()> {
        if area.height == 0 {
            return Ok(());
        }

        writer.queue(cursor::MoveTo(area.x, area.y))?;
        writer.queue(SetForegroundColor(colors.foreground))?;
        writer.queue(SetBackgroundColor(colors.background))?;

        let uptime_secs = system_info.uptime.as_secs();
        let uptime_str = format!(
            "{}d {}h {}m",
            uptime_secs / 86400,
            (uptime_secs % 86400) / 3600,
            (uptime_secs % 3600) / 60
        );

        let load_str = format!(
            "Load: {:.2} {:.2} {:.2}",
            system_info.load_avg_1, system_info.load_avg_5, system_info.load_avg_15
        );

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let time_str = format!("{:02}:{:02}:{:02}", 
            (now.as_secs() % 86400) / 3600,
            (now.as_secs() % 3600) / 60,
            now.as_secs() % 60
        );

        let content = format!(
            "{} | {} {} | Up: {} | {} | {}",
            system_info.hostname,
            system_info.os_name,
            system_info.os_version,
            uptime_str,
            load_str,
            time_str
        );

        // Truncate if too long
        let max_width = area.width as usize;
        let truncated = if content.len() > max_width {
            format!("{}...", &content[..max_width.saturating_sub(3)])
        } else {
            format!("{:width$}", content, width = max_width)
        };

        writer.queue(Print(truncated))?;
        Ok(())
    }
}

/// Gauge widget for displaying usage percentages
pub struct Gauge;

impl Gauge {
    pub fn render<W: Write>(
        &self,
        writer: &mut W,
        area: Rect,
        title: &str,
        percentage: f32,
        label: &str,
        colors: &ColorScheme,
    ) -> io::Result<()> {
        if area.height < 2 || area.width < 3 {
            return Ok(());
        }

        let percentage = percentage.clamp(0.0, 100.0);
        let gauge_width = (area.width as usize).saturating_sub(2); // Account for borders
        let fill_width = ((percentage / 100.0) * gauge_width as f32) as usize;

        // Title line
        writer.queue(cursor::MoveTo(area.x, area.y))?;
        writer.queue(SetForegroundColor(colors.table_header))?;
        writer.queue(SetBackgroundColor(colors.background))?;
        writer.queue(Print(format!("{:width$}", title, width = area.width as usize)))?;

        // Gauge line
        if area.height > 1 {
            writer.queue(cursor::MoveTo(area.x, area.y + 1))?;
            writer.queue(SetForegroundColor(colors.foreground))?;
            writer.queue(Print("["))?;

            // Fill portion
            writer.queue(SetBackgroundColor(colors.gauge_fill))?;
            writer.queue(SetForegroundColor(colors.background))?;
            for _ in 0..fill_width {
                writer.queue(Print("█"))?;
            }

            // Empty portion
            writer.queue(SetBackgroundColor(colors.gauge_bg))?;
            writer.queue(SetForegroundColor(colors.muted))?;
            for _ in fill_width..gauge_width {
                writer.queue(Print("░"))?;
            }

            writer.queue(SetBackgroundColor(colors.background))?;
            writer.queue(SetForegroundColor(colors.foreground))?;
            writer.queue(Print("]"))?;
        }

        // Label line
        if area.height > 2 {
            writer.queue(cursor::MoveTo(area.x, area.y + 2))?;
            writer.queue(SetForegroundColor(colors.muted))?;
            let label_text = format!("{:.1}% - {}", percentage, label);
            let truncated = if label_text.len() > area.width as usize {
                format!("{}...", &label_text[..area.width as usize - 3])
            } else {
                label_text
            };
            writer.queue(Print(truncated))?;
        }

        Ok(())
    }
}

/// CPU gauges widget
pub struct CpuGauges;

impl CpuGauges {
    pub fn render<W: Write>(
        &self,
        writer: &mut W,
        area: Rect,
        cpu_cores: &[CpuCore],
        colors: &ColorScheme,
    ) -> io::Result<()> {
        if cpu_cores.is_empty() || area.height < 2 {
            return Ok(());
        }

        // Calculate overall CPU usage
        let overall_usage = cpu_cores.iter().map(|c| c.usage_percent).sum::<f32>() / cpu_cores.len() as f32;

        // Render overall CPU gauge
        let gauge = Gauge;
        let cores_info = format!("{} cores", cpu_cores.len());
        gauge.render(writer, area, "CPU", overall_usage, &cores_info, colors)?;

        // If we have space, show individual core usage in a compact format
        if area.height > 3 && cpu_cores.len() <= 16 {
            writer.queue(cursor::MoveTo(area.x, area.y + 3))?;
            writer.queue(SetForegroundColor(colors.muted))?;
            
            let cores_per_line = (area.width as usize) / 8; // "C0:100% "
            for (i, core) in cpu_cores.iter().enumerate() {
                if i > 0 && i % cores_per_line == 0 && (area.y + 3 + (i / cores_per_line) as u16) < area.bottom() {
                    writer.queue(cursor::MoveTo(area.x, area.y + 3 + (i / cores_per_line) as u16))?;
                }
                
                let usage_color = colors.cpu_usage_color(core.usage_percent);
                writer.queue(SetForegroundColor(usage_color))?;
                writer.queue(Print(format!("C{}:{:3.0}% ", core.id, core.usage_percent)))?;
                writer.queue(SetForegroundColor(colors.muted))?;
            }
        }

        Ok(())
    }
}

/// Memory gauges widget
pub struct MemoryGauges;

impl MemoryGauges {
    pub fn render<W: Write>(
        &self,
        writer: &mut W,
        area: Rect,
        memory: &MemoryInfo,
        colors: &ColorScheme,
    ) -> io::Result<()> {
        if area.height < 2 {
            return Ok(());
        }

        let memory_usage = if memory.total > 0 {
            (memory.used as f32 / memory.total as f32) * 100.0
        } else {
            0.0
        };

        let memory_label = format!("{} / {}", 
            format_bytes(memory.used), 
            format_bytes(memory.total)
        );

        let gauge = Gauge;
        gauge.render(writer, area, "Memory", memory_usage, &memory_label, colors)?;

        // Show swap if available
        if area.height > 3 && memory.swap_total > 0 {
            let swap_usage = (memory.swap_used as f32 / memory.swap_total as f32) * 100.0;
            let swap_label = format!("{} / {}", 
                format_bytes(memory.swap_used), 
                format_bytes(memory.swap_total)
            );

            let swap_area = Rect::new(area.x, area.y + 3, area.width, 1);
            gauge.render(writer, swap_area, "Swap", swap_usage, &swap_label, colors)?;
        }

        Ok(())
    }
}

/// Process table widget
pub struct ProcessTable;

impl ProcessTable {
    pub fn render<W: Write>(
        &self,
        writer: &mut W,
        area: Rect,
        processes: &[ProcessInfo],
        columns: &[&str],
        selected_index: usize,
        start_index: usize,
        colors: &ColorScheme,
    ) -> io::Result<()> {
        if area.height < 2 {
            return Ok(());
        }

        // Calculate column layout
        let layout = crate::ui::Layout::new().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let column_rects = layout.table_layout(area, columns);

        // Render header
        writer.queue(cursor::MoveTo(area.x, area.y))?;
        writer.queue(SetForegroundColor(colors.table_header))?;
        writer.queue(SetBackgroundColor(colors.background))?;

        for (i, column) in columns.iter().enumerate() {
            if i < column_rects.len() {
                let rect = column_rects[i];
                writer.queue(cursor::MoveTo(rect.x, rect.y))?;
                let header = format!("{:width$}", column, width = rect.width as usize);
                writer.queue(Print(header))?;
            }
        }

        // Render process rows
        let visible_rows = (area.height as usize).saturating_sub(1); // Subtract header
        let end_index = (start_index + visible_rows).min(processes.len());

        for (row_idx, process) in processes[start_index..end_index].iter().enumerate() {
            let y = area.y + 1 + row_idx as u16;
            let is_selected = start_index + row_idx == selected_index;
            let is_alternate = row_idx % 2 == 1;

            // Set row background
            if is_selected {
                writer.queue(SetBackgroundColor(colors.table_selected))?;
                writer.queue(SetForegroundColor(colors.background))?;
            } else if is_alternate {
                writer.queue(SetBackgroundColor(colors.table_row_alt))?;
                writer.queue(SetForegroundColor(colors.foreground))?;
            } else {
                writer.queue(SetBackgroundColor(colors.background))?;
                writer.queue(SetForegroundColor(colors.foreground))?;
            }

            for (col_idx, &column) in columns.iter().enumerate() {
                if col_idx < column_rects.len() {
                    let rect = column_rects[col_idx];
                    writer.queue(cursor::MoveTo(rect.x, y))?;

                    let content = match column {
                        "PID" => process.pid.to_string(),
                        "NAME" => process.name.clone(),
                        "USER" => process.user.clone(),
                        "CPU%" => format!("{:5.1}", process.cpu_percent),
                        "MEM%" => format!("{:5.1}", process.memory_percent),
                        "RSS" => format_bytes(process.memory_rss),
                        "VSZ" => format_bytes(process.memory_vsz),
                        "THR" => process.threads.to_string(),
                        "STATE" => format!("{:?}", process.state),
                        "TIME" => {
                            let elapsed = std::time::SystemTime::now()
                                .duration_since(process.start_time)
                                .unwrap_or_default();
                            format!("{:02}:{:02}", elapsed.as_secs() / 60, elapsed.as_secs() % 60)
                        },
                        _ => String::new(),
                    };

                    // Apply column-specific colors
                    if !is_selected {
                        match column {
                            "STATE" => { writer.queue(SetForegroundColor(colors.process_state_color(&process.state)))?; },
                            "CPU%" if process.cpu_percent > 50.0 => { writer.queue(SetForegroundColor(colors.warning))?; },
                            "MEM%" if process.memory_percent > 50.0 => { writer.queue(SetForegroundColor(colors.warning))?; },
                            _ => {}
                        }
                    }

                    let truncated = if content.len() > rect.width as usize {
                        format!("{}...", &content[..rect.width as usize - 3])
                    } else {
                        format!("{:width$}", content, width = rect.width as usize)
                    };

                    writer.queue(Print(truncated))?;
                }
            }
        }

        // Reset colors
        writer.queue(SetBackgroundColor(colors.background))?;
        writer.queue(SetForegroundColor(colors.foreground))?;

        Ok(())
    }
}

/// Footer widget for keybind hints
pub struct Footer;

impl Footer {
    pub fn render<W: Write>(
        &self,
        writer: &mut W,
        area: Rect,
        colors: &ColorScheme,
    ) -> io::Result<()> {
        if area.height == 0 {
            return Ok(());
        }

        writer.queue(cursor::MoveTo(area.x, area.y))?;
        writer.queue(SetForegroundColor(colors.muted))?;
        writer.queue(SetBackgroundColor(colors.background))?;

        let keybinds = "q:quit ↑↓:navigate s:sort /:filter c:columns r:refresh ?:help k:kill";
        let truncated = if keybinds.len() > area.width as usize {
            format!("{}...", &keybinds[..area.width as usize - 3])
        } else {
            format!("{:width$}", keybinds, width = area.width as usize)
        };

        writer.queue(Print(truncated))?;
        Ok(())
    }
}

/// Network gauges widget with creative visual elements
pub struct NetworkGauges;

impl NetworkGauges {
    pub fn render<W: Write>(
        &self,
        writer: &mut W,
        area: Rect,
        networks: &[NetworkInfo],
        colors: &ColorScheme,
    ) -> io::Result<()> {
        if area.height < 2 {
            return Ok(());
        }

        // Calculate aggregate network stats
        let (total_rx_rate, total_tx_rate, _total_rx, _total_tx) = networks.iter().fold(
            (0u64, 0u64, 0u64, 0u64),
            |(rx_rate, tx_rate, rx_total, tx_total), net| {
                (
                    rx_rate + net.rx_bytes_delta,
                    tx_rate + net.tx_bytes_delta,
                    rx_total + net.rx_bytes,
                    tx_total + net.tx_bytes,
                )
            },
        );

        // Draw creative title with network activity indicator
        writer.queue(cursor::MoveTo(area.x, area.y))?;
        writer.queue(SetForegroundColor(colors.table_header))?;
        writer.queue(SetBackgroundColor(colors.background))?;
        
        let (activity_indicator, activity_color) = if total_rx_rate > 0 || total_tx_rate > 0 {
            if total_rx_rate > 1_000_000 || total_tx_rate > 1_000_000 { 
                ("●●●", colors.error) // High activity - red
            } else if total_rx_rate > 100_000 || total_tx_rate > 100_000 { 
                ("●●○", colors.warning) // Medium activity - yellow/orange
            } else { 
                ("●○○", colors.success) // Low activity - green
            }
        } else { 
            ("○○○", colors.muted) // No activity - gray
        };
        
        writer.queue(SetForegroundColor(activity_color))?;
        writer.queue(Print(activity_indicator))?;
        writer.queue(SetForegroundColor(colors.table_header))?;
        let title = format!(" NETWORK ({} interfaces)", networks.len());
        let full_title = format!("{}{}", activity_indicator, title);
        
        // Ensure we clear the full width
        if full_title.len() < area.width as usize {
            writer.queue(Print(&title))?;
            let padding = " ".repeat(area.width as usize - full_title.len());
            writer.queue(Print(padding))?;
        } else {
            let truncated = format!("{}...", &title[..area.width as usize - 7]);
            writer.queue(Print(truncated))?;
        }

        if area.height >= 2 {
            // Draw decorative border
            writer.queue(cursor::MoveTo(area.x, area.y + 1))?;
            writer.queue(SetForegroundColor(colors.muted))?;
            let border = "─".repeat((area.width as usize).min(80));
            writer.queue(Print(border))?;
        }

        if area.height >= 3 {
            // Draw aggregate stats with visual bars
            writer.queue(cursor::MoveTo(area.x, area.y + 2))?;
            writer.queue(SetForegroundColor(colors.accent))?;
            
            let max_bar_width = 15;
            let max_rate = (total_rx_rate.max(total_tx_rate)).max(1_000); // At least 1KB for scaling
            
            // RX bar
            let rx_bar_width = ((total_rx_rate * max_bar_width as u64) / max_rate).min(max_bar_width as u64) as usize;
            let rx_bar = "▓".repeat(rx_bar_width) + &"░".repeat(max_bar_width - rx_bar_width);
            
            // TX bar  
            let tx_bar_width = ((total_tx_rate * max_bar_width as u64) / max_rate).min(max_bar_width as u64) as usize;
            let tx_bar = "▓".repeat(tx_bar_width) + &"░".repeat(max_bar_width - tx_bar_width);
            
            writer.queue(SetForegroundColor(colors.success))?; // Green for download
            writer.queue(Print("⬇ "))?;
            writer.queue(Print(&rx_bar))?;
            writer.queue(Print(format!(" {}/s", format_rate(total_rx_rate))))?;
            
            // Clear rest of line and add upload stats on same line if space
            let rx_part_len = 2 + max_bar_width + format!(" {}/s", format_rate(total_rx_rate)).len();
            if area.width as usize > rx_part_len + 25 {
                writer.queue(Print("   "))?;
                writer.queue(SetForegroundColor(colors.warning))?; // Orange for upload
                writer.queue(Print("⬆ "))?;
                writer.queue(Print(&tx_bar))?;
                writer.queue(Print(format!(" {}/s", format_rate(total_tx_rate))))?;
                
                // Clear remaining space on this line
                let total_used = rx_part_len + 25 + format!(" {}/s", format_rate(total_tx_rate)).len();
                if area.width as usize > total_used {
                    let spaces = " ".repeat(area.width as usize - total_used);
                    writer.queue(Print(spaces))?;
                }
            } else {
                // Clear remaining space on this line
                if area.width as usize > rx_part_len {
                    let spaces = " ".repeat(area.width as usize - rx_part_len);
                    writer.queue(Print(spaces))?;
                }
            }

            // Draw individual interfaces with creative visualizations
            if area.height > 3 && !networks.is_empty() {
                let available_lines = (area.height as usize).saturating_sub(3);
                
                // Sort interfaces: active first, then by name, limit to max 6
                let mut sorted_networks: Vec<_> = networks.iter().collect();
                sorted_networks.sort_by(|a, b| {
                    let a_active = a.rx_bytes_delta > 0 || a.tx_bytes_delta > 0;
                    let b_active = b.rx_bytes_delta > 0 || b.tx_bytes_delta > 0;
                    
                    match (a_active, b_active) {
                        (true, false) => std::cmp::Ordering::Less,  // Active interfaces first
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.interface_name.cmp(&b.interface_name), // Then alphabetical
                    }
                });
                
                // Show all interfaces that fit in available space
                let interfaces_to_show = sorted_networks.len().min(available_lines);
                
                for (i, net) in sorted_networks.iter().take(interfaces_to_show).enumerate() {
                    let y_pos = area.y + 3 + i as u16;
                    writer.queue(cursor::MoveTo(area.x, y_pos))?;
                    
                    // Interface status icon with activity level
                    let (status_icon, status_color) = if net.rx_bytes_delta > 0 || net.tx_bytes_delta > 0 {
                        if net.rx_bytes_delta > 10_000_000 || net.tx_bytes_delta > 10_000_000 {
                            ("◉", colors.error) // Very high activity - red
                        } else if net.rx_bytes_delta > 1_000_000 || net.tx_bytes_delta > 1_000_000 {
                            ("◎", colors.warning) // High activity - orange
                        } else if net.rx_bytes_delta > 100_000 || net.tx_bytes_delta > 100_000 {
                            ("◐", colors.accent) // Medium activity - blue
                        } else {
                            ("◯", colors.success) // Low activity - green
                        }
                    } else {
                        ("○", colors.muted) // No activity - gray
                    };
                    
                    writer.queue(SetForegroundColor(status_color))?;
                    writer.queue(Print(status_icon))?;
                    
                    // Interface name with color coding
                    let name_color = if net.interface_name.starts_with("en") || net.interface_name.starts_with("eth") {
                        colors.success // Ethernet - green
                    } else if net.interface_name.starts_with("wl") || net.interface_name.contains("wifi") {
                        colors.accent // WiFi - blue  
                    } else {
                        colors.muted // Other - gray
                    };
                    
                    writer.queue(SetForegroundColor(name_color))?;
                    writer.queue(Print(format!("{:<8}", net.interface_name)))?;
                    
                    // Always show bars for consistency, but with different styles
                    let mini_bar_width = 8;
                    
                    if net.rx_bytes_delta > 0 || net.tx_bytes_delta > 0 {
                        // Active interface - show proportional bars
                        let total_max_rate = networks.iter()
                            .map(|n| n.rx_bytes_delta.max(n.tx_bytes_delta))
                            .max()
                            .unwrap_or(1_000)
                            .max(1_000); // At least 1KB for scaling
                        
                        let rx_width = if net.rx_bytes_delta > 0 {
                            ((net.rx_bytes_delta * mini_bar_width as u64) / total_max_rate).min(mini_bar_width as u64).max(1) as usize
                        } else { 0 };
                        
                        let tx_width = if net.tx_bytes_delta > 0 {
                            ((net.tx_bytes_delta * mini_bar_width as u64) / total_max_rate).min(mini_bar_width as u64).max(1) as usize
                        } else { 0 };
                        
                        writer.queue(SetForegroundColor(colors.success))?;
                        writer.queue(Print(" ⬇"))?;
                        writer.queue(Print(&"█".repeat(rx_width)))?;
                        writer.queue(SetForegroundColor(colors.muted))?;
                        writer.queue(Print(&"▁".repeat(mini_bar_width - rx_width)))?;
                        
                        writer.queue(SetForegroundColor(colors.warning))?;
                        writer.queue(Print(" ⬆"))?;
                        writer.queue(Print(&"█".repeat(tx_width)))?;
                        writer.queue(SetForegroundColor(colors.muted))?;
                        writer.queue(Print(&"▁".repeat(mini_bar_width - tx_width)))?;
                        
                        // Data rates
                        writer.queue(SetForegroundColor(colors.foreground))?;
                        let rate_text = format!(" {}/{}",
                            format_rate(net.rx_bytes_delta),
                            format_rate(net.tx_bytes_delta)
                        );
                        writer.queue(Print(rate_text))?;
                    } else {
                        // Idle interface - show empty bars
                        writer.queue(SetForegroundColor(colors.muted))?;
                        writer.queue(Print(" ⬇"))?;
                        writer.queue(Print(&"▁".repeat(mini_bar_width)))?;
                        writer.queue(Print(" ⬆"))?;
                        writer.queue(Print(&"▁".repeat(mini_bar_width)))?;
                        writer.queue(Print(" [idle]"))?;
                    }
                    
                    // Clear rest of the line to prevent text bleeding
                    let used_width = 1 + 8 + if net.rx_bytes_delta > 0 || net.tx_bytes_delta > 0 { 
                        2 + mini_bar_width + 2 + mini_bar_width + format!(" {}/{}", format_rate(net.rx_bytes_delta), format_rate(net.tx_bytes_delta)).len()
                    } else { 
                        2 + mini_bar_width + 2 + mini_bar_width + 7 // " [idle]"
                    };
                    
                    if area.width as usize > used_width {
                        let clear_space = " ".repeat(area.width as usize - used_width);
                        writer.queue(Print(clear_space))?;
                    }
                }
                
                // Show summary if there are more interfaces than can fit
                if sorted_networks.len() > interfaces_to_show {
                    let remaining = sorted_networks.len() - interfaces_to_show;
                    if area.height > 3 + interfaces_to_show as u16 {
                        writer.queue(cursor::MoveTo(area.x, area.y + 3 + interfaces_to_show as u16))?;
                        writer.queue(SetForegroundColor(colors.muted))?;
                        let summary_text = format!("... and {} more interfaces (increase terminal height to see all)", remaining);
                        writer.queue(Print(&summary_text))?;
                        
                        // Clear rest of summary line
                        if area.width as usize > summary_text.len() {
                            let clear_space = " ".repeat(area.width as usize - summary_text.len());
                            writer.queue(Print(clear_space))?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Format bytes with rate suffix (no extra "/s" since we add it in display)
fn format_rate(bytes_per_sec: u64) -> String {
    if bytes_per_sec == 0 {
        return "0B".to_string();
    }
    
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes_per_sec as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{}{}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1}{}", size, UNITS[unit_index])
    }
}

/// Temperature gauge widget
pub struct TemperatureGauge;

impl TemperatureGauge {
    pub fn render<W: Write>(
        &self,
        writer: &mut W,
        area: Rect,
        temperatures: &[TemperatureInfo],
        colors: &ColorScheme,
    ) -> io::Result<()> {
        if area.height < 2 {
            return Ok(());
        }

        // Find the highest temperature for the main gauge
        let max_temp_info = temperatures.iter()
            .max_by(|a, b| a.temperature.partial_cmp(&b.temperature).unwrap_or(std::cmp::Ordering::Equal));

        // Draw title with temperature status
        writer.queue(cursor::MoveTo(area.x, area.y))?;
        writer.queue(SetBackgroundColor(colors.background))?;

        let (title_text, title_color) = if let Some(temp_info) = max_temp_info {
            let temp = temp_info.temperature;
            let status_icon = if temp >= 80.0 {   // 176°F
                "🔥" // Critical
            } else if temp >= 65.0 {             // 149°F
                "🌡️" // Warning  
            } else if temp >= 45.0 {             // 113°F
                "♨️" // Warm
            } else {
                "❄️" // Cool
            };

            let color = if temp >= 80.0 {
                colors.error // Red for critical
            } else if temp >= 65.0 {
                colors.warning // Orange for warning
            } else if temp >= 45.0 {
                colors.accent // Blue for warm
            } else {
                colors.success // Green for cool
            };

            let temp_f = temp * 9.0 / 5.0 + 32.0;
            (format!("{} TEMPERATURE ({:.0}°F)", status_icon, temp_f), color)
        } else {
            ("🌡️ TEMPERATURE (No sensors)".to_string(), colors.muted)
        };

        writer.queue(SetForegroundColor(title_color))?;
        let title_truncated = if title_text.len() > area.width as usize {
            format!("{}...", &title_text[..area.width as usize - 3])
        } else {
            format!("{:width$}", title_text, width = area.width as usize)
        };
        writer.queue(Print(title_truncated))?;

        if area.height >= 2 {
            // Draw decorative border
            writer.queue(cursor::MoveTo(area.x, area.y + 1))?;
            writer.queue(SetForegroundColor(colors.muted))?;
            let border = "─".repeat(area.width as usize);
            writer.queue(Print(border))?;
        }

        if area.height >= 4 && max_temp_info.is_some() {
            let temp_info = max_temp_info.unwrap();
            let temp = temp_info.temperature;
            
            // Draw large temperature gauge
            writer.queue(cursor::MoveTo(area.x, area.y + 2))?;
            
            // Calculate gauge parameters
            let gauge_width = (area.width as usize).saturating_sub(2).min(30);
            let max_temp = temp_info.critical.or(temp_info.max).unwrap_or(100.0);
            let percentage = ((temp / max_temp) * 100.0).min(100.0).max(0.0) as usize;
            let fill_width = (gauge_width * percentage) / 100;
            
            // Temperature gauge with gradient effect
            writer.queue(SetForegroundColor(colors.foreground))?;
            writer.queue(Print("┌"))?;
            
            // Create the gauge bar with color gradient
            for i in 0..gauge_width {
                let pos_percentage = (i * 100) / gauge_width;
                let gauge_color = if pos_percentage >= 80 {
                    colors.error // Red zone
                } else if pos_percentage >= 60 {
                    colors.warning // Yellow zone
                } else if pos_percentage >= 40 {
                    colors.accent // Blue zone
                } else {
                    colors.success // Green zone
                };
                
                writer.queue(SetForegroundColor(gauge_color))?;
                if i < fill_width {
                    writer.queue(Print("█"))?;
                } else {
                    writer.queue(SetForegroundColor(colors.muted))?;
                    writer.queue(Print("░"))?;
                }
            }
            
            writer.queue(SetForegroundColor(colors.foreground))?;
            writer.queue(Print("┐"))?;

            // Draw temperature value and percentage
            if area.height >= 5 {
                writer.queue(cursor::MoveTo(area.x, area.y + 3))?;
                writer.queue(SetForegroundColor(colors.foreground))?;
                let temp_f = temp * 9.0 / 5.0 + 32.0;
                let temp_text = format!("{:.0}°F ({:.0}%)", temp_f, percentage);
                let centered_x = area.x + (area.width.saturating_sub(temp_text.len() as u16)) / 2;
                writer.queue(cursor::MoveTo(centered_x, area.y + 3))?;
                writer.queue(Print(temp_text))?;
            }

            // Show individual sensor readings if space allows
            if area.height >= 6 && temperatures.len() > 1 {
                let sensors_to_show = (area.height as usize - 5).min(temperatures.len()).min(4);
                
                for (i, temp_info) in temperatures.iter().take(sensors_to_show).enumerate() {
                    writer.queue(cursor::MoveTo(area.x, area.y + 4 + i as u16))?;
                    
                    let sensor_color = if temp_info.temperature >= 80.0 {  // 176°F
                        colors.error
                    } else if temp_info.temperature >= 65.0 {            // 149°F
                        colors.warning
                    } else {
                        colors.muted
                    };
                    
                    writer.queue(SetForegroundColor(sensor_color))?;
                    let temp_f = temp_info.temperature * 9.0 / 5.0 + 32.0;
                    let sensor_text = format!("{}: {:.0}°F", 
                        temp_info.label.chars().take(8).collect::<String>(),
                        temp_f
                    );
                    
                    let truncated = if sensor_text.len() > area.width as usize {
                        format!("{}...", &sensor_text[..area.width as usize - 3])
                    } else {
                        format!("{:width$}", sensor_text, width = area.width as usize)
                    };
                    writer.queue(Print(truncated))?;
                }
            }
        } else if area.height >= 3 {
            // No temperature data available
            writer.queue(cursor::MoveTo(area.x, area.y + 2))?;
            writer.queue(SetForegroundColor(colors.muted))?;
            let no_data_text = "No temperature sensors detected";
            let centered_x = area.x + (area.width.saturating_sub(no_data_text.len() as u16)) / 2;
            writer.queue(cursor::MoveTo(centered_x, area.y + 2))?;
            writer.queue(Print(no_data_text))?;
        }

        Ok(())
    }
}

/// Format bytes in human-readable format
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{}{}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1}{}", size, UNITS[unit_index])
    }
}
