pub mod config;
pub mod error;
pub mod metrics;
pub mod model;
pub mod platform;

pub use config::Config;
pub use error::{CoreError, Result};
pub use metrics::MetricsCollector;
pub use model::*;
pub use platform::PlatformProvider;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.refresh_ms, 2000);
        assert_eq!(config.theme, Theme::Dark);
        assert!(!config.no_color);
    }
    
    #[test]
    fn test_sort_key_cycle() {
        let mut sort = SortKey::Cpu;
        sort = sort.next();
        assert_eq!(sort, SortKey::Memory);
        sort = sort.next();
        assert_eq!(sort, SortKey::Pid);
        sort = sort.next();
        assert_eq!(sort, SortKey::Name);
        sort = sort.next();
        assert_eq!(sort, SortKey::Cpu);
    }
    
    #[test]
    fn test_metrics_collector_creation() {
        let result = MetricsCollector::new();
        assert!(result.is_ok());
    }
    
    #[test] 
    fn test_system_snapshot_serialization() {
        use std::time::SystemTime;
        
        let snapshot = SystemSnapshot {
            timestamp: SystemTime::now(),
            system: SystemInfo {
                hostname: "test".to_string(),
                os_name: "test-os".to_string(),
                os_version: "1.0".to_string(),
                uptime: std::time::Duration::from_secs(3600),
                boot_time: SystemTime::now(),
                load_avg_1: 0.5,
                load_avg_5: 0.4,
                load_avg_15: 0.3,
            },
            cpu_cores: vec![],
            memory: MemoryInfo {
                total: 8_000_000_000,
                used: 4_000_000_000,
                available: 4_000_000_000,
                free: 3_000_000_000,
                buffers: 500_000_000,
                cached: 500_000_000,
                swap_total: 2_000_000_000,
                swap_used: 0,
                swap_free: 2_000_000_000,
            },
            disks: vec![],
            networks: vec![],
            temperatures: vec![],
            processes: vec![],
        };
        
        let json = serde_json::to_string(&snapshot);
        assert!(json.is_ok());
        
        let deserialized: std::result::Result<SystemSnapshot, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }
}