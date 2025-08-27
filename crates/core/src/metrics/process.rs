use crate::{error::Result, model::{ProcessInfo, ProcessState, SortKey}};
use std::{collections::HashMap, time::SystemTime};
use sysinfo::{Pid, Process, System};

pub struct ProcessCollector {
    sys: System,
    previous_cpu_times: HashMap<Pid, u64>,
}

impl ProcessCollector {
    pub fn new() -> Result<Self> {
        let mut sys = System::new_all();
        sys.refresh_processes();
        
        Ok(Self {
            sys,
            previous_cpu_times: HashMap::new(),
        })
    }

    pub fn init(&mut self) -> Result<()> {
        self.sys.refresh_processes();
        
        // Store initial CPU times for delta calculation
        for (pid, process) in self.sys.processes() {
            self.previous_cpu_times.insert(*pid, process.cpu_usage() as u64);
        }
        
        Ok(())
    }

    pub fn collect(&mut self) -> Result<Vec<ProcessInfo>> {
        self.sys.refresh_processes();
        
        let mut processes = Vec::new();
        let total_memory = self.sys.total_memory();
        
        for (pid, process) in self.sys.processes() {
            let process_info = self.process_to_info(*pid, process, total_memory)?;
            processes.push(process_info);
        }
        
        Ok(processes)
    }

    pub fn collect_filtered(&mut self, filter: &str) -> Result<Vec<ProcessInfo>> {
        let processes = self.collect()?;
        
        if filter.is_empty() {
            return Ok(processes);
        }
        
        let filter_lower = filter.to_lowercase();
        Ok(processes
            .into_iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&filter_lower)
                    || p.cmd.join(" ").to_lowercase().contains(&filter_lower)
                    || p.user.to_lowercase().contains(&filter_lower)
                    || p.pid.to_string().contains(&filter_lower)
            })
            .collect())
    }

    pub fn collect_sorted(&mut self, sort_key: SortKey, reverse: bool) -> Result<Vec<ProcessInfo>> {
        let mut processes = self.collect()?;
        self.sort_processes(&mut processes, sort_key, reverse);
        Ok(processes)
    }

    pub fn collect_sorted_filtered(
        &mut self,
        sort_key: SortKey,
        reverse: bool,
        filter: &str,
    ) -> Result<Vec<ProcessInfo>> {
        let mut processes = self.collect_filtered(filter)?;
        self.sort_processes(&mut processes, sort_key, reverse);
        Ok(processes)
    }

    fn sort_processes(&self, processes: &mut [ProcessInfo], sort_key: SortKey, reverse: bool) {
        match sort_key {
            SortKey::Cpu => {
                processes.sort_by(|a, b| {
                    if reverse {
                        b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        a.cpu_percent.partial_cmp(&b.cpu_percent).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            }
            SortKey::Memory => {
                processes.sort_by(|a, b| {
                    if reverse {
                        b.memory_percent.partial_cmp(&a.memory_percent).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        a.memory_percent.partial_cmp(&b.memory_percent).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            }
            SortKey::Pid => {
                processes.sort_by(|a, b| {
                    if reverse {
                        b.pid.cmp(&a.pid)
                    } else {
                        a.pid.cmp(&b.pid)
                    }
                });
            }
            SortKey::Name => {
                processes.sort_by(|a, b| {
                    if reverse {
                        b.name.cmp(&a.name)
                    } else {
                        a.name.cmp(&b.name)
                    }
                });
            }
        }
    }

    fn process_to_info(&self, pid: Pid, process: &Process, total_memory: u64) -> Result<ProcessInfo> {
        let pid_u32 = pid.as_u32();
        let name = process.name().to_string();
        let cmd = process.cmd().to_vec();
        let cpu_percent = process.cpu_usage();
        let memory_percent = process.memory() as f32 / (total_memory as f32) * 100.0;
        let memory_rss = process.memory() * 1024; // sysinfo returns KB, convert to bytes
        let memory_vsz = process.virtual_memory() * 1024;
        let threads = 1; // Threads not available in newer sysinfo API
        let state = self.convert_process_status(process.status());
        
        // Convert start time from sysinfo's u64 (seconds since epoch) to SystemTime
        let start_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(process.start_time());
        
        let parent_pid = process.parent().map(|p| p.as_u32());
        
        // Get user information
        let user = self.get_process_user(pid, process);
        
        // Get cgroup information (Linux only)
        let cgroup = self.get_process_cgroup(pid);

        Ok(ProcessInfo {
            pid: pid_u32,
            name,
            cmd,
            user,
            cpu_percent,
            memory_percent,
            memory_rss,
            memory_vsz,
            threads,
            state,
            start_time,
            parent_pid,
            cgroup,
        })
    }

    fn convert_process_status(&self, status: sysinfo::ProcessStatus) -> ProcessState {
        match status {
            sysinfo::ProcessStatus::Idle => ProcessState::Sleeping,
            sysinfo::ProcessStatus::Run => ProcessState::Running,
            sysinfo::ProcessStatus::Sleep => ProcessState::Sleeping,
            sysinfo::ProcessStatus::Stop => ProcessState::Stopped,
            sysinfo::ProcessStatus::Zombie => ProcessState::Zombie,
            sysinfo::ProcessStatus::Tracing => ProcessState::Waiting,
            sysinfo::ProcessStatus::Dead => ProcessState::Dead,
            sysinfo::ProcessStatus::Wakekill => ProcessState::Sleeping,
            sysinfo::ProcessStatus::Waking => ProcessState::Waiting,
            sysinfo::ProcessStatus::Parked => ProcessState::Waiting,
            sysinfo::ProcessStatus::LockBlocked => ProcessState::Waiting,
            sysinfo::ProcessStatus::UninterruptibleDiskSleep => ProcessState::Waiting,
            _ => ProcessState::Unknown,
        }
    }

    #[cfg(unix)]
    fn get_process_user(&self, _pid: Pid, process: &Process) -> String {
        // On Unix systems, we can get user information
        if let Some(uid) = process.user_id() {
            // Try to get username from UID
            #[cfg(feature = "linux_procfs")]
            {
                if let Ok(users) = procfs::net::unix_users() {
                    if let Some(user) = users.get(uid) {
                        return user.clone();
                    }
                }
            }
            
            // Fallback to UID string
            uid.to_string()
        } else {
            "unknown".to_string()
        }
    }

    #[cfg(not(unix))]
    fn get_process_user(&self, _pid: Pid, _process: &Process) -> String {
        // On non-Unix systems (Windows), this is more complex
        // For now, return a placeholder
        "user".to_string()
    }

    #[cfg(all(target_os = "linux", feature = "linux_procfs"))]
    fn get_process_cgroup(&self, pid: Pid) -> Option<String> {
        if let Ok(cgroup) = procfs::process::Process::new(pid.as_u32() as i32)
            .and_then(|p| p.cgroups()) {
            // Get the first cgroup or the one we're most interested in
            cgroup.first().map(|cg| cg.pathname.clone())
        } else {
            None
        }
    }

    #[cfg(not(all(target_os = "linux", feature = "linux_procfs")))]
    fn get_process_cgroup(&self, _pid: Pid) -> Option<String> {
        None
    }

    /// Kill a process by PID (Unix only)
    #[cfg(unix)]
    pub fn kill_process(&self, pid: u32) -> Result<()> {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid as NixPid;
        
        let nix_pid = NixPid::from_raw(pid as i32);
        kill(nix_pid, Signal::SIGTERM)
            .map_err(|e| crate::error::CoreError::platform(format!("Failed to kill process {}: {}", pid, e)))?;
        
        Ok(())
    }

    #[cfg(not(unix))]
    pub fn kill_process(&self, _pid: u32) -> Result<()> {
        Err(crate::error::CoreError::unsupported_platform(
            "Process termination not supported on this platform".to_string()
        ))
    }
}
