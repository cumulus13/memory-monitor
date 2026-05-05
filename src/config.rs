//! File: src\config.rs
//! Author: Hadi Cahyadi <cumulus13@gmail.com>
//! Date: 2026-05-06
//! Description: 
//! License: MIT

use configparser::ini::Ini;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub threshold: f64,
    pub check_interval: u64,
    pub cleanmem_path: String,
    pub growl_host: String,
    pub growl_port: u16,
    pub growl_password: String,
    pub growl_app_name: String,
    pub icon_path: String,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        MonitorConfig {
            threshold: 98.0,
            check_interval: 60,
            cleanmem_path: "cleanmem.exe".to_string(),
            growl_host: "localhost".to_string(),
            growl_port: 23053,
            growl_password: String::new(),
            growl_app_name: "Memory Monitor".to_string(),
            icon_path: "memory-monitor.png".to_string(),
        }
    }
}

impl MonitorConfig {
    pub fn load(config_file: &str) -> Self {
        if !Path::new(config_file).exists() {
            Self::create_default(config_file);
        }
        
        let mut config = Ini::new();
        config.load(config_file).unwrap_or_default();
        
        MonitorConfig {
            threshold: config
                .get("Settings", "threshold")
                .and_then(|v| v.parse().ok())
                .unwrap_or(98.0),
            
            check_interval: config
                .get("Settings", "check_interval")
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            
            cleanmem_path: config
                .get("Settings", "cleanmem_path")
                .unwrap_or_else(|| "cleanmem.exe".to_string()),
            
            icon_path: config
                .get("Settings", "icon_path")
                .unwrap_or_else(|| "memory-monitor.png".to_string()),
            
            growl_host: config
                .get("Growl", "host")
                .unwrap_or_else(|| "localhost".to_string()),
            
            growl_port: config
                .get("Growl", "port")
                .and_then(|v| v.parse().ok())
                .unwrap_or(23053),
            
            growl_password: config
                .get("Growl", "password")
                .unwrap_or_default(),
            
            growl_app_name: config
                .get("Growl", "app_name")
                .unwrap_or_else(|| "Memory Monitor".to_string()),
        }
    }
    
    pub fn create_default(config_file: &str) {
        let mut config = Ini::new();
        config.set("Settings", "threshold", Some("98.0".to_string()));
        config.set("Settings", "check_interval", Some("60".to_string()));
        config.set("Settings", "cleanmem_path", Some("cleanmem.exe".to_string()));
        config.set("Settings", "icon_path", Some("memory-monitor.png".to_string()));
        config.set("Growl", "host", Some("localhost".to_string()));
        config.set("Growl", "port", Some("23053".to_string()));
        config.set("Growl", "password", Some(String::new()));
        config.set("Growl", "app_name", Some("Memory Monitor".to_string()));
        config.write(config_file).expect("Failed to write config file");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = MonitorConfig::default();
        assert_eq!(config.threshold, 98.0);
        assert_eq!(config.check_interval, 60);
        assert_eq!(config.icon_path, "memory-monitor.png");
    }
    
    #[test]
    fn test_create_and_load_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        
        MonitorConfig::create_default(path);
        let config = MonitorConfig::load(path);
        
        assert_eq!(config.threshold, 98.0);
        assert_eq!(config.growl_port, 23053);
    }
}