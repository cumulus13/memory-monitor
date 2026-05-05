/// Basic example showing how to use MemoryMonitor programmatically
use memory_monitor::{MemoryMonitor, MonitorConfig};

fn main() {
    // Create custom configuration
    let config = MonitorConfig {
        threshold: 90.0,       // Trigger at 90% memory usage
        check_interval: 30,    // Check every 30 seconds
        cleanmem_path: "cleanmem.exe".to_string(),
        ..MonitorConfig::default()
    };
    
    // Create monitor with custom config
    let mut monitor = MemoryMonitor::with_config(config);
    
    // Start monitoring
    monitor.monitor();
}