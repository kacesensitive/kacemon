use crate::{error::Result, model::CpuCore};
use std::collections::HashMap;
use sysinfo::System;

pub struct CpuCollector {
    sys: System,
    previous_usage: HashMap<usize, f32>,
}

impl CpuCollector {
    pub fn new() -> Result<Self> {
        let mut sys = System::new();
        sys.refresh_cpu();
        
        Ok(Self {
            sys,
            previous_usage: HashMap::new(),
        })
    }

    pub fn init(&mut self) -> Result<()> {
        // Take initial measurement for baseline
        self.sys.refresh_cpu();
        
        // Store initial usage values
        for (id, cpu) in self.sys.cpus().iter().enumerate() {
            self.previous_usage.insert(id, cpu.cpu_usage());
        }
        
        Ok(())
    }

    pub fn collect(&mut self) -> Result<Vec<CpuCore>> {
        self.sys.refresh_cpu();
        
        let mut cores = Vec::new();
        
        for (id, cpu) in self.sys.cpus().iter().enumerate() {
            let usage_percent = cpu.cpu_usage();
            let frequency = cpu.frequency();
            let name = cpu.name().to_string();
            
            cores.push(CpuCore {
                id,
                name,
                usage_percent,
                frequency,
            });
            
            // Update previous usage for next calculation
            self.previous_usage.insert(id, usage_percent);
        }
        
        Ok(cores)
    }

    /// Get overall CPU usage across all cores
    pub fn get_overall_usage(&self) -> f32 {
        if self.sys.cpus().is_empty() {
            return 0.0;
        }
        
        let total: f32 = self.sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
        total / self.sys.cpus().len() as f32
    }
}
