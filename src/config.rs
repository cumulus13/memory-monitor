//! File: src\config.rs
//! Author: Hadi Cahyadi <cumulus13@gmail.com>
//! Date: 2026-05-06
//! Description: 
//! License: MIT

use configparser::ini::Ini;
use std::path::{Path, PathBuf};

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
    /// Get candidate config filenames to search for
    fn get_candidate_filenames() -> Vec<String> {
        let mut candidates = vec![
            "config.ini".to_string(),
            "memory-monitor.ini".to_string(),
            "memory_monitor.ini".to_string(),
        ];
        
        // Add exe name based candidates
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_name) = exe_path.file_stem().and_then(|n| n.to_str()) {
                candidates.push(format!("{}.ini", exe_name));
                candidates.push(format!("{}.conf", exe_name));
                candidates.push(format!("{}_config.ini", exe_name));
            }
        }
        
        candidates
    }

    /// Get candidate search directories in priority order
    fn get_search_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        
        // 1. Executable directory (highest priority)
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                dirs.push(exe_dir.to_path_buf());
            }
        }
        
        // 2. Current working directory
        if let Ok(cwd) = std::env::current_dir() {
            dirs.push(cwd);
        }
        
        // 3. Platform-specific config directories
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                dirs.push(Path::new(&appdata).join("memory-monitor"));
            }
            if let Ok(programdata) = std::env::var("PROGRAMDATA") {
                dirs.push(Path::new(&programdata).join("memory-monitor"));
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            let xdg_config = std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
                    Path::new(&home).join(".config")
                });
            dirs.push(xdg_config.join("memory-monitor"));
            dirs.push(PathBuf::from("/etc/memory-monitor"));
        }
        
        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                dirs.push(Path::new(&home)
                    .join("Library")
                    .join("Application Support")
                    .join("memory-monitor"));
            }
            dirs.push(PathBuf::from("/Library/Application Support/memory-monitor"));
        }
        
        dirs
    }

    /// Find config file by searching through directories and candidate filenames
    fn find_config_file() -> Option<PathBuf> {
        let dirs = Self::get_search_dirs();
        let filenames = Self::get_candidate_filenames();
        
        for dir in &dirs {
            for filename in &filenames {
                let config_path = dir.join(filename);
                if config_path.exists() {
                    return Some(config_path);
                }
            }
        }
        
        None
    }
    
    /// Get the path where default config should be created
    fn get_default_config_path() -> PathBuf {
        let exe_name = std::env::current_exe()
            .ok()
            .and_then(|p| p.file_stem().and_then(|n| n.to_str()).map(String::from))
            .unwrap_or_else(|| "memory-monitor".to_string());
        
        // Prefer exe directory
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                if let Ok(metadata) = std::fs::metadata(exe_dir) {
                    if !metadata.permissions().readonly() {
                        return exe_dir.join("config.ini");
                    }
                }
            }
        }
        
        // Fallback to platform-specific
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let dir = Path::new(&appdata).join(&exe_name);
                std::fs::create_dir_all(&dir).ok();
                return dir.join("config.ini");
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            let xdg_config = std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
                    Path::new(&home).join(".config")
                });
            
            let dir = xdg_config.join(&exe_name);
            std::fs::create_dir_all(&dir).ok();
            return dir.join("config.ini");
        }
        
        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                let dir = Path::new(&home)
                    .join("Library")
                    .join("Application Support")
                    .join(&exe_name);
                std::fs::create_dir_all(&dir).ok();
                return dir.join("config.ini");
            }
        }
        
        // Last fallback
        PathBuf::from("config.ini")
    }
    
    pub fn load(config_file: &str) -> Self {
        // If user specified an explicit file, try that directly
        if config_file != "config.ini" || std::path::Path::new(config_file).is_absolute() {
            let explicit_path = Path::new(config_file);
            if explicit_path.exists() {
                return Self::load_from_path(explicit_path);
            }
        }
        
        // Search for config file
        let config_path = Self::find_config_file()
            .unwrap_or_else(|| Self::get_default_config_path());
        
        if !config_path.exists() {
            Self::create_default_at_path(&config_path);
        }
        
        Self::load_from_path(&config_path)
    }
    
    fn load_from_path(path: &Path) -> Self {
        let mut config = Ini::new();
        config.load(path).unwrap_or_default();
        
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
    
    fn create_default_at_path(path: &Path) {
        let mut config = Ini::new();
        config.set("Settings", "threshold", Some("98.0".to_string()));
        config.set("Settings", "check_interval", Some("60".to_string()));
        config.set("Settings", "cleanmem_path", Some("cleanmem.exe".to_string()));
        config.set("Settings", "icon_path", Some("memory-monitor.png".to_string()));
        config.set("Growl", "host", Some("localhost".to_string()));
        config.set("Growl", "port", Some("23053".to_string()));
        config.set("Growl", "password", Some(String::new()));
        config.set("Growl", "app_name", Some("Memory Monitor".to_string()));
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        
        config.write(path).expect("Failed to write config file");
        println!("✅ Created default config file: {}", path.display());
    }
    
    // pub fn create_default(config_file: &str) {
    //     let path = Self::get_default_config_path();
    //     Self::create_default_at_path(&path);
    // }
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
    fn test_candidate_filenames() {
        let candidates = MonitorConfig::get_candidate_filenames();
        assert!(candidates.contains(&"config.ini".to_string()));
        assert!(candidates.contains(&"memory-monitor.ini".to_string()));
        assert!(candidates.contains(&"memory_monitor.ini".to_string()));
    }
    
    #[test]
    fn test_create_and_load_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        MonitorConfig::create_default_at_path(path);
        let config = MonitorConfig::load_from_path(path);
        
        assert_eq!(config.threshold, 98.0);
        assert_eq!(config.growl_port, 23053);
    }
}