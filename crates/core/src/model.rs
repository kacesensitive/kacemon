use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Sort key for process table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortKey {
    Cpu,
    Memory,
    Pid,
    Name,
}

impl Default for SortKey {
    fn default() -> Self {
        Self::Cpu
    }
}

impl SortKey {
    pub fn next(self) -> Self {
        match self {
            Self::Cpu => Self::Memory,
            Self::Memory => Self::Pid,
            Self::Pid => Self::Name,
            Self::Name => Self::Cpu,
        }
    }
}

/// Process state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessState {
    Running,
    Sleeping,
    Waiting,
    Zombie,
    Stopped,
    Paging,
    Dead,
    Unknown,
}

impl Default for ProcessState {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cmd: Vec<String>,
    pub user: String,
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub memory_rss: u64,  // Resident Set Size in bytes
    pub memory_vsz: u64,  // Virtual Size in bytes
    pub threads: u64,
    pub state: ProcessState,
    pub start_time: SystemTime,
    pub parent_pid: Option<u32>,
    pub cgroup: Option<String>,  // Linux only
}

/// System information snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub uptime: Duration,
    pub boot_time: SystemTime,
    pub load_avg_1: f64,
    pub load_avg_5: f64,
    pub load_avg_15: f64,
}

/// CPU core information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCore {
    pub id: usize,
    pub name: String,
    pub usage_percent: f32,
    pub frequency: u64,  // MHz
}

/// Memory information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub free: u64,
    pub buffers: u64,
    pub cached: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub swap_free: u64,
}

/// Disk information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_space: u64,
    pub used_space: u64,
    pub available_space: u64,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub read_bytes_delta: u64,   // since last snapshot
    pub write_bytes_delta: u64,  // since last snapshot
}

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interface_name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_bytes_delta: u64,     // since last snapshot
    pub tx_bytes_delta: u64,     // since last snapshot
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

/// Temperature sensor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureInfo {
    pub label: String,
    pub temperature: f32,        // Current temperature in Celsius
    pub critical: Option<f32>,   // Critical temperature threshold
    pub max: Option<f32>,        // Maximum temperature threshold
}

/// Complete system metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub timestamp: SystemTime,
    pub system: SystemInfo,
    pub cpu_cores: Vec<CpuCore>,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub networks: Vec<NetworkInfo>,
    pub temperatures: Vec<TemperatureInfo>,
    pub processes: Vec<ProcessInfo>,
}

/// Theme configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Dark
    }
}

/// Visible columns in process table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessColumns {
    pub pid: bool,
    pub name: bool,
    pub user: bool,
    pub cpu_percent: bool,
    pub memory_percent: bool,
    pub memory_rss: bool,
    pub memory_vsz: bool,
    pub threads: bool,
    pub state: bool,
    pub start_time: bool,
}

impl Default for ProcessColumns {
    fn default() -> Self {
        Self {
            pid: true,
            name: true,
            user: true,
            cpu_percent: true,
            memory_percent: true,
            memory_rss: true,
            memory_vsz: false,
            threads: false,
            state: true,
            start_time: false,
        }
    }
}


