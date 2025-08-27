pub mod linux;
pub mod macos;
pub mod windows;

use crate::error::Result;

/// Platform-specific functionality trait
pub trait PlatformProvider {
    /// Get enhanced process information specific to this platform
    fn get_process_details(&self, pid: u32) -> Result<PlatformProcessDetails>;
    
    /// Get system-specific performance metrics
    fn get_system_metrics(&self) -> Result<PlatformSystemMetrics>;
    
    /// Check if process termination is supported
    fn supports_process_kill(&self) -> bool;
    
    /// Get platform name
    fn platform_name(&self) -> &'static str;
}

/// Platform-specific process details
#[derive(Debug, Clone, Default)]
pub struct PlatformProcessDetails {
    pub cmdline: Option<String>,
    pub cwd: Option<String>,
    pub environment: Option<Vec<(String, String)>>,
    pub open_files: Option<Vec<String>>,
    pub cgroup: Option<String>,
    pub container_id: Option<String>,
}

/// Platform-specific system metrics
#[derive(Debug, Clone, Default)]
pub struct PlatformSystemMetrics {
    pub context_switches: Option<u64>,
    pub interrupts: Option<u64>,
    pub processes_created: Option<u64>,
    pub processes_running: Option<u64>,
    pub processes_blocked: Option<u64>,
}

/// Get the appropriate platform provider for the current system
pub fn get_platform_provider() -> Box<dyn PlatformProvider> {
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxProvider::new())
    }
    
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacosProvider::new())
    }
    
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsProvider::new())
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Box::new(GenericProvider::new())
    }
}

/// Generic provider for unsupported platforms
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub struct GenericProvider;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
impl GenericProvider {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
impl PlatformProvider for GenericProvider {
    fn get_process_details(&self, _pid: u32) -> Result<PlatformProcessDetails> {
        Ok(PlatformProcessDetails::default())
    }
    
    fn get_system_metrics(&self) -> Result<PlatformSystemMetrics> {
        Ok(PlatformSystemMetrics::default())
    }
    
    fn supports_process_kill(&self) -> bool {
        false
    }
    
    fn platform_name(&self) -> &'static str {
        "generic"
    }
}
