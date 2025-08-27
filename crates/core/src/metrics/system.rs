use crate::{error::Result, model::SystemInfo};
use std::time::{Duration, SystemTime};
use sysinfo::System;

pub struct SystemCollector {
    sys: System,
}

impl SystemCollector {
    pub fn new() -> Result<Self> {
        let sys = System::new();
        
        Ok(Self { sys })
    }

    pub fn collect(&mut self) -> Result<SystemInfo> {
        let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
        let os_name = System::name().unwrap_or_else(|| "unknown".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "unknown".to_string());
        
        let uptime = Duration::from_secs(System::uptime());
        let boot_time = SystemTime::now() - uptime;
        
        // Load averages - sysinfo doesn't provide this on all platforms
        // We'll use platform-specific implementations where available
        let (load_avg_1, load_avg_5, load_avg_15) = self.get_load_averages();

        Ok(SystemInfo {
            hostname,
            os_name,
            os_version,
            uptime,
            boot_time,
            load_avg_1,
            load_avg_5,
            load_avg_15,
        })
    }

    #[cfg(unix)]
    fn get_load_averages(&self) -> (f64, f64, f64) {
        let load_avg = System::load_average();
        (load_avg.one, load_avg.five, load_avg.fifteen)
    }

    #[cfg(not(unix))]
    fn get_load_averages(&self) -> (f64, f64, f64) {
        // Windows doesn't have load averages in the traditional sense
        // We could implement something similar using CPU queue length
        (0.0, 0.0, 0.0)
    }
}
