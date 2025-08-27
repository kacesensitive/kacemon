pub mod cpu;
pub mod disk;
pub mod memory;
pub mod network;
pub mod process;
pub mod system;
pub mod temperature;

pub use cpu::CpuCollector;
pub use disk::DiskCollector;
pub use memory::MemoryCollector;
pub use network::NetworkCollector;
pub use process::ProcessCollector;
pub use system::SystemCollector;
pub use temperature::TemperatureCollector;

use crate::{error::Result, model::SystemSnapshot};
use std::time::SystemTime;

/// Main metrics collector that coordinates all sub-collectors
pub struct MetricsCollector {
    system: SystemCollector,
    cpu: CpuCollector,
    memory: MemoryCollector,
    disk: DiskCollector,
    network: NetworkCollector,
    temperature: TemperatureCollector,
    process: ProcessCollector,
}

impl MetricsCollector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            system: SystemCollector::new()?,
            cpu: CpuCollector::new()?,
            memory: MemoryCollector::new()?,
            disk: DiskCollector::new()?,
            network: NetworkCollector::new()?,
            temperature: TemperatureCollector::new()?,
            process: ProcessCollector::new()?,
        })
    }

    pub fn collect(&mut self) -> Result<SystemSnapshot> {
        let timestamp = SystemTime::now();
        
        let system = self.system.collect()?;
        let cpu_cores = self.cpu.collect()?;
        let memory = self.memory.collect()?;
        let disks = self.disk.collect()?;
        let networks = self.network.collect()?;
        let temperatures = self.temperature.collect()?;
        let processes = self.process.collect()?;

        Ok(SystemSnapshot {
            timestamp,
            system,
            cpu_cores,
            memory,
            disks,
            networks,
            temperatures,
            processes,
        })
    }

    /// Initialize the collectors (useful for taking initial baseline measurements)
    pub fn init(&mut self) -> Result<()> {
        self.cpu.init()?;
        self.disk.init()?;
        self.network.init()?;
        self.temperature.init()?;
        self.process.init()?;
        Ok(())
    }
}
