use crate::{error::Result, model::TemperatureInfo};
use sysinfo::Components;

pub struct TemperatureCollector {
    components: Components,
}

impl TemperatureCollector {
    pub fn new() -> Result<Self> {
        let components = Components::new_with_refreshed_list();
        
        Ok(Self {
            components,
        })
    }

    pub fn init(&mut self) -> Result<()> {
        self.components.refresh_list();
        self.components.refresh();
        Ok(())
    }

    pub fn collect(&mut self) -> Result<Vec<TemperatureInfo>> {
        self.components.refresh();
        
        let mut temperatures = Vec::new();
        
        for component in &self.components {
            let temp = component.temperature();
            let critical_temp = component.critical();
            let max_temp = component.max();
            
            // Only include components with valid temperature readings
            if temp > 0.0 {
                temperatures.push(TemperatureInfo {
                    label: component.label().to_string(),
                    temperature: temp,
                    critical: critical_temp,
                    max: if max_temp > 0.0 { Some(max_temp) } else { None },
                });
            }
        }
        
        // If no hardware sensors available, try to get CPU temp from system
        if temperatures.is_empty() {
            // Add a synthetic CPU temperature reading if available
            if let Some(cpu_temp) = self.get_synthetic_cpu_temp() {
                temperatures.push(TemperatureInfo {
                    label: "CPU".to_string(),
                    temperature: cpu_temp,
                    critical: Some(85.0), // Typical CPU critical temp
                    max: Some(100.0),     // Typical CPU max temp
                });
            }
        }
        
        Ok(temperatures)
    }

    /// Get overall system temperature (average of all sensors)
    pub fn get_system_temperature(&mut self) -> Result<Option<f32>> {
        let temps = self.collect()?;
        
        if temps.is_empty() {
            return Ok(None);
        }
        
        let total: f32 = temps.iter().map(|t| t.temperature).sum();
        let average = total / temps.len() as f32;
        
        Ok(Some(average))
    }

    /// Get the highest temperature reading
    pub fn get_max_temperature(&mut self) -> Result<Option<f32>> {
        let temps = self.collect()?;
        
        let max_temp = temps.iter()
            .map(|t| t.temperature)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            
        Ok(max_temp)
    }

    /// Try to get a synthetic CPU temperature when hardware sensors aren't available
    fn get_synthetic_cpu_temp(&self) -> Option<f32> {
        // On some systems, we might estimate temperature based on CPU usage
        // This is a fallback when no real temperature sensors are available
        // For now, return None - real implementations would need platform-specific code
        None
    }
}

/// Get temperature status color based on temperature level
impl TemperatureInfo {
    pub fn get_status(&self) -> TemperatureStatus {
        let temp = self.temperature;
        
        // Use critical temperature if available, otherwise use common thresholds
        let critical_threshold = self.critical.unwrap_or(80.0);
        let warning_threshold = critical_threshold * 0.8; // 80% of critical
        let safe_threshold = critical_threshold * 0.6;    // 60% of critical
        
        if temp >= critical_threshold {
            TemperatureStatus::Critical
        } else if temp >= warning_threshold {
            TemperatureStatus::Warning
        } else if temp >= safe_threshold {
            TemperatureStatus::Warm
        } else {
            TemperatureStatus::Cool
        }
    }
    
    pub fn get_percentage(&self) -> f32 {
        let max_temp = self.max.or(self.critical).unwrap_or(100.0);
        (self.temperature / max_temp * 100.0).min(100.0).max(0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemperatureStatus {
    Cool,     // < 60% of critical
    Warm,     // 60-80% of critical  
    Warning,  // 80-100% of critical
    Critical, // >= critical temp
}
