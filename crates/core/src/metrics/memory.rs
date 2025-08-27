use crate::{error::Result, model::MemoryInfo};
use sysinfo::System;

pub struct MemoryCollector {
    sys: System,
}

impl MemoryCollector {
    pub fn new() -> Result<Self> {
        let sys = System::new();
        
        Ok(Self { sys })
    }

    pub fn collect(&mut self) -> Result<MemoryInfo> {
        self.sys.refresh_memory();
        
        let total = self.sys.total_memory();
        let used = self.sys.used_memory();
        let available = self.sys.available_memory();
        let free = self.sys.free_memory();
        
        // Calculate buffers and cached memory
        // Note: sysinfo doesn't provide these separately on all platforms
        let (buffers, cached) = self.get_buffer_cached_memory();
        
        let swap_total = self.sys.total_swap();
        let swap_used = self.sys.used_swap();
        let swap_free = self.sys.free_swap();

        Ok(MemoryInfo {
            total,
            used,
            available,
            free,
            buffers,
            cached,
            swap_total,
            swap_used,
            swap_free,
        })
    }

    #[cfg(target_os = "linux")]
    fn get_buffer_cached_memory(&self) -> (u64, u64) {
        // On Linux, we could parse /proc/meminfo for more detailed information
        // For now, we'll estimate or use sysinfo where available
        #[cfg(feature = "linux_procfs")]
        {
            if let Ok(meminfo) = procfs::Meminfo::new() {
                let buffers = meminfo.buffers.unwrap_or(procfs::Bytes(0)).0;
                let cached = meminfo.cached.unwrap_or(procfs::Bytes(0)).0;
                return (buffers, cached);
            }
        }
        
        // Fallback estimation
        let total = self.sys.total_memory();
        let used = self.sys.used_memory();
        let free = self.sys.free_memory();
        let available = self.sys.available_memory();
        
        // Rough estimation: cached = available - free, buffers = small portion of used
        let cached = if available > free { available - free } else { 0 };
        let buffers = (used as f64 * 0.05) as u64; // Rough estimate: 5% of used memory
        
        (buffers, cached)
    }

    #[cfg(not(target_os = "linux"))]
    fn get_buffer_cached_memory(&self) -> (u64, u64) {
        // On non-Linux systems, these concepts may not apply or be available
        // Return zeros or best estimates
        (0, 0)
    }
}
