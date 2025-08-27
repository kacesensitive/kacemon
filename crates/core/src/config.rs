use crate::{error::Result, model::{ProcessColumns, SortKey, Theme}};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, time::Duration};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Refresh interval in milliseconds
    pub refresh_ms: u64,
    
    /// UI theme
    pub theme: Theme,
    
    /// Disable colors
    pub no_color: bool,
    
    /// Initial sort key
    pub initial_sort: SortKey,
    
    /// Visible columns in process table
    pub process_columns: ProcessColumns,
    
    /// Tree view enabled by default
    pub tree_view: bool,
    
    /// Enable Linux procfs features (if available)
    pub use_procfs: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_ms: 2000,
            theme: Theme::Dark,
            no_color: false,
            initial_sort: SortKey::Cpu,
            process_columns: ProcessColumns::default(),
            tree_view: false,
            use_procfs: cfg!(feature = "linux_procfs"),
        }
    }
}

impl Config {
    /// Load configuration from multiple sources in order of preference:
    /// 1. CLI arguments override everything
    /// 2. JSON config file if specified
    /// 3. Default config file locations
    /// 4. Built-in defaults
    pub fn load(cli_config: Option<&CliConfig>, json_path: Option<&PathBuf>) -> Result<Self> {
        let mut config = Self::default();
        
        // Try to load from default config locations
        if let Some(default_config) = Self::load_default_config()? {
            config.merge(default_config);
        }
        
        // Override with JSON config file if specified
        if let Some(path) = json_path {
            let file_config = Self::load_from_file(path)?;
            config.merge(file_config);
        }
        
        // Override with CLI arguments
        if let Some(cli) = cli_config {
            config.apply_cli_overrides(cli);
        }
        
        config.validate()?;
        Ok(config)
    }
    
    /// Load configuration from a specific JSON file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|e| crate::error::CoreError::config(format!("Failed to read config file {}: {}", path.display(), e)))?;
        
        let config: Self = serde_json::from_str(&contents)
            .map_err(|e| crate::error::CoreError::config(format!("Failed to parse config file {}: {}", path.display(), e)))?;
        
        Ok(config)
    }
    
    /// Load configuration from default locations
    fn load_default_config() -> Result<Option<Self>> {
        let config_paths = Self::default_config_paths();
        
        for path in config_paths {
            if path.exists() {
                match Self::load_from_file(&path) {
                    Ok(config) => return Ok(Some(config)),
                    Err(e) => {
                        eprintln!("Warning: Failed to load config from {}: {}", path.display(), e);
                        continue;
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Get default configuration file search paths
    fn default_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        // XDG config directory
        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("srmon").join("config.json"));
        }
        
        // Home directory
        if let Some(home_dir) = dirs::home_dir() {
            paths.push(home_dir.join(".srmon.json"));
        }
        
        // Current directory
        paths.push(PathBuf::from("srmon.json"));
        
        paths
    }
    
    /// Merge another configuration into this one
    fn merge(&mut self, other: Self) {
        // Only override non-default values
        if other.refresh_ms != 250 {
            self.refresh_ms = other.refresh_ms;
        }
        if other.theme != Theme::Dark {
            self.theme = other.theme;
        }
        if other.no_color {
            self.no_color = other.no_color;
        }
        if other.initial_sort != SortKey::Cpu {
            self.initial_sort = other.initial_sort;
        }
        // For process columns, merge individual fields
        // For simplicity, we'll replace the whole struct if any field differs from default
        let default_columns = ProcessColumns::default();
        if other.process_columns.pid != default_columns.pid
            || other.process_columns.name != default_columns.name
            || other.process_columns.user != default_columns.user
            || other.process_columns.cpu_percent != default_columns.cpu_percent
            || other.process_columns.memory_percent != default_columns.memory_percent
            || other.process_columns.memory_rss != default_columns.memory_rss
            || other.process_columns.memory_vsz != default_columns.memory_vsz
            || other.process_columns.threads != default_columns.threads
            || other.process_columns.state != default_columns.state
            || other.process_columns.start_time != default_columns.start_time
        {
            self.process_columns = other.process_columns;
        }
        if other.tree_view {
            self.tree_view = other.tree_view;
        }
        if !other.use_procfs {
            self.use_procfs = other.use_procfs;
        }
    }
    
    /// Apply CLI argument overrides
    fn apply_cli_overrides(&mut self, cli: &CliConfig) {
        if let Some(refresh) = cli.refresh_ms {
            self.refresh_ms = refresh;
        }
        if let Some(theme) = &cli.theme {
            self.theme = theme.clone();
        }
        if cli.no_color {
            self.no_color = true;
        }
    }
    
    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        if self.refresh_ms < 50 {
            return Err(crate::error::CoreError::config(
                "Refresh interval must be at least 50ms".to_string()
            ));
        }
        
        if self.refresh_ms > 10000 {
            return Err(crate::error::CoreError::config(
                "Refresh interval must be at most 10 seconds".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Get refresh interval as Duration
    pub fn refresh_interval(&self) -> Duration {
        Duration::from_millis(self.refresh_ms)
    }
}

/// CLI configuration (temporary struct for CLI parsing)
#[derive(Debug, Clone)]
pub struct CliConfig {
    pub refresh_ms: Option<u64>,
    pub theme: Option<Theme>,
    pub no_color: bool,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            refresh_ms: None,
            theme: None,
            no_color: false,
        }
    }
}

// Add dirs dependency for config directory discovery
// This will be added to Cargo.toml dependencies
