use super::{PlatformProcessDetails, PlatformProvider, PlatformSystemMetrics};
use crate::error::{CoreError, Result};

pub struct LinuxProvider;

impl LinuxProvider {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformProvider for LinuxProvider {
    fn get_process_details(&self, pid: u32) -> Result<PlatformProcessDetails> {
        #[cfg(feature = "linux_procfs")]
        {
            self.get_process_details_procfs(pid)
        }
        
        #[cfg(not(feature = "linux_procfs"))]
        {
            Ok(PlatformProcessDetails::default())
        }
    }
    
    fn get_system_metrics(&self) -> Result<PlatformSystemMetrics> {
        #[cfg(feature = "linux_procfs")]
        {
            self.get_system_metrics_procfs()
        }
        
        #[cfg(not(feature = "linux_procfs"))]
        {
            Ok(PlatformSystemMetrics::default())
        }
    }
    
    fn supports_process_kill(&self) -> bool {
        true
    }
    
    fn platform_name(&self) -> &'static str {
        "linux"
    }
}

#[cfg(feature = "linux_procfs")]
impl LinuxProvider {
    fn get_process_details_procfs(&self, pid: u32) -> Result<PlatformProcessDetails> {
        let process = procfs::process::Process::new(pid as i32)
            .map_err(|e| CoreError::platform(format!("Failed to read process {}: {}", pid, e)))?;
        
        // Get command line
        let cmdline = process.cmdline()
            .ok()
            .map(|cmd| cmd.join(" "));
        
        // Get current working directory
        let cwd = process.cwd()
            .ok()
            .and_then(|path| path.to_str().map(|s| s.to_string()));
        
        // Get environment variables
        let environment = process.environ()
            .ok()
            .map(|env| env.into_iter().collect());
        
        // Get open file descriptors
        let open_files = process.fd()
            .ok()
            .map(|fds| {
                fds.into_iter()
                    .filter_map(|fd| fd.ok())
                    .filter_map(|fd| fd.target.to_str().map(|s| s.to_string()))
                    .collect()
            });
        
        // Get cgroup information
        let cgroup = process.cgroups()
            .ok()
            .and_then(|cgroups| cgroups.first().map(|cg| cg.pathname.clone()));
        
        // Try to extract container ID from cgroup
        let container_id = cgroup.as_ref().and_then(|cg| {
            if cg.contains("docker") {
                // Extract Docker container ID
                cg.split('/').last().map(|id| {
                    if id.len() >= 12 {
                        id[..12].to_string()
                    } else {
                        id.to_string()
                    }
                })
            } else if cg.contains("kubepods") {
                // Extract Kubernetes pod container ID
                cg.split('/').last().map(|id| {
                    if id.len() >= 12 {
                        id[..12].to_string()
                    } else {
                        id.to_string()
                    }
                })
            } else {
                None
            }
        });
        
        Ok(PlatformProcessDetails {
            cmdline,
            cwd,
            environment,
            open_files,
            cgroup,
            container_id,
        })
    }
    
    fn get_system_metrics_procfs(&self) -> Result<PlatformSystemMetrics> {
        // Read /proc/stat for system-wide statistics
        let stat = procfs::KernelStats::new()
            .map_err(|e| CoreError::platform(format!("Failed to read /proc/stat: {}", e)))?;
        
        // Read /proc/loadavg and other system metrics
        let context_switches = Some(stat.ctxt);
        let interrupts = Some(stat.intr.iter().sum());
        let processes_created = Some(stat.processes);
        let processes_running = Some(stat.procs_running as u64);
        let processes_blocked = Some(stat.procs_blocked as u64);
        
        Ok(PlatformSystemMetrics {
            context_switches,
            interrupts,
            processes_created,
            processes_running,
            processes_blocked,
        })
    }
}

#[cfg(not(feature = "linux_procfs"))]
impl LinuxProvider {
    // Stub implementations when procfs feature is not enabled
}
