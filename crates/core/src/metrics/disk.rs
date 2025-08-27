use crate::{error::Result, model::DiskInfo};
use std::collections::HashMap;
use sysinfo::{Disks};

pub struct DiskCollector {
    disks: Disks,
    previous_stats: HashMap<String, (u64, u64)>, // (read_bytes, write_bytes)
}

impl DiskCollector {
    pub fn new() -> Result<Self> {
        let disks = Disks::new_with_refreshed_list();
        
        Ok(Self {
            disks,
            previous_stats: HashMap::new(),
        })
    }

    pub fn init(&mut self) -> Result<()> {
        self.disks.refresh_list();
        self.disks.refresh();
        
        // Store initial I/O stats for delta calculation
        for disk in &self.disks {
            let name = disk.name().to_string_lossy().to_string();
            // Note: sysinfo doesn't provide disk I/O stats on all platforms
            // We'll implement platform-specific collection where possible
            let (read_bytes, write_bytes) = self.get_disk_io_stats(&name);
            self.previous_stats.insert(name, (read_bytes, write_bytes));
        }
        
        Ok(())
    }

    pub fn collect(&mut self) -> Result<Vec<DiskInfo>> {
        self.disks.refresh();
        
        let mut disks = Vec::new();
        
        for disk in &self.disks {
            let name = disk.name().to_string_lossy().to_string();
            let mount_point = disk.mount_point().to_string_lossy().to_string();
            let file_system = disk.file_system().to_string_lossy().to_string();
            let total_space = disk.total_space();
            let available_space = disk.available_space();
            let used_space = total_space.saturating_sub(available_space);
            
            // Get current I/O stats
            let (read_bytes, write_bytes) = self.get_disk_io_stats(&name);
            
            // Calculate deltas
            let (read_bytes_delta, write_bytes_delta) = if let Some((prev_read, prev_write)) = self.previous_stats.get(&name) {
                (
                    read_bytes.saturating_sub(*prev_read),
                    write_bytes.saturating_sub(*prev_write),
                )
            } else {
                (0, 0)
            };
            
            // Update previous stats
            self.previous_stats.insert(name.clone(), (read_bytes, write_bytes));
            
            disks.push(DiskInfo {
                name,
                mount_point,
                file_system,
                total_space,
                used_space,
                available_space,
                read_bytes,
                write_bytes,
                read_bytes_delta,
                write_bytes_delta,
            });
        }
        
        Ok(disks)
    }

    #[cfg(target_os = "linux")]
    fn get_disk_io_stats(&self, disk_name: &str) -> (u64, u64) {
        #[cfg(feature = "linux_procfs")]
        {
            if let Ok(diskstats) = procfs::diskstats() {
                // Extract device name from path (e.g., "/dev/sda1" -> "sda1")
                let device_name = disk_name
                    .strip_prefix("/dev/")
                    .unwrap_or(disk_name);
                
                for stat in diskstats {
                    if stat.name == device_name {
                        // Convert sectors to bytes (assuming 512 bytes per sector)
                        let read_bytes = stat.sectors_read * 512;
                        let write_bytes = stat.sectors_written * 512;
                        return (read_bytes, write_bytes);
                    }
                }
            }
        }
        
        // Fallback: return zeros if procfs is not available
        (0, 0)
    }

    #[cfg(target_os = "macos")]
    fn get_disk_io_stats(&self, _disk_name: &str) -> (u64, u64) {
        // macOS implementation would use IOKit
        // For now, return zeros as a placeholder
        (0, 0)
    }

    #[cfg(target_os = "windows")]
    fn get_disk_io_stats(&self, _disk_name: &str) -> (u64, u64) {
        // Windows implementation would use Performance Counters or WMI
        // For now, return zeros as a placeholder
        (0, 0)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    fn get_disk_io_stats(&self, _disk_name: &str) -> (u64, u64) {
        (0, 0)
    }
}
