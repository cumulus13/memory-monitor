// File: src\monitor.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2026-05-06
// Description: 
// License: MIT

use chrono::Local;
use crossterm::{
    cursor,
    terminal::{self, Clear, ClearType},
    ExecutableCommand,
};
use sysinfo::{System, SystemExt};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::thread;
use std::time::Duration;

use crate::config::MonitorConfig;
use crate::notification::GntpNotifier;
use crate::cleaner::MemoryCleaner;

pub struct MemoryMonitor {
    pub config: MonitorConfig,
    pub notifier: Option<GntpNotifier>,
    cleaner: MemoryCleaner,
    system: System,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        let config = MonitorConfig::load("config.ini");
        let notifier = GntpNotifier::new(
            &config.growl_host,
            config.growl_port,
            &config.growl_password,
            &config.growl_app_name,
            &config.icon_path,
        );
        let cleaner = MemoryCleaner::new(&config.cleanmem_path);
        
        MemoryMonitor {
            config,
            notifier,
            cleaner,
            system: System::new_all(),
        }
    }
    
    #[allow(dead_code)]
    pub fn with_config(config: MonitorConfig) -> Self {
        let notifier = GntpNotifier::new(
            &config.growl_host,
            config.growl_port,
            &config.growl_password,
            &config.growl_app_name,
            &config.icon_path,
        );
        let cleaner = MemoryCleaner::new(&config.cleanmem_path);
        
        MemoryMonitor {
            config,
            notifier,
            cleaner,
            system: System::new_all(),
        }
    }
    
    pub fn get_memory_usage(&mut self) -> f64 {
        self.system.refresh_memory();
        let total = self.system.total_memory() as f64;
        let used = self.system.used_memory() as f64;
        (used / total) * 100.0
    }

    pub fn threshold(&self) -> f64 {
        self.config.threshold
    }

    pub fn check_interval(&self) -> u64 {
        self.config.check_interval
    }

    pub fn cleanmem_path(&self) -> &str {
        &self.config.cleanmem_path
    }

    pub fn growl_host(&self) -> &str {
        &self.config.growl_host
    }

    pub fn growl_port(&self) -> u16 {
        self.config.growl_port
    }

    pub fn growl_app_name(&self) -> &str {
        &self.config.growl_app_name
    }

    pub fn has_notifier(&self) -> bool {
        self.notifier.is_some()
    }
    
    fn display_status(&self, memory_percent: f64) {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        
        let (color, emoji, status) = if memory_percent >= self.config.threshold {
            (Color::Red, "🔴", "CRITICAL")
        } else if memory_percent >= self.config.threshold - 5.0 {
            (Color::Yellow, "🟡", "WARNING")
        } else {
            (Color::Green, "🟢", "OK")
        };
        
        stdout.execute(Clear(ClearType::All)).ok();
        stdout.execute(cursor::MoveTo(0, 0)).ok();
        
        let width = terminal::size().map(|(w, _)| w as usize).unwrap_or(80);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)).set_bold(true)).ok();
        println!("{}", "=".repeat(width));
        println!("        🚀 Memory Monitor & Auto-Cleaner");
        println!("{}", "=".repeat(width));
        stdout.reset().ok();
        
        stdout.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true)).ok();
        println!("\n{} Memory Status: {}", emoji, status);
        stdout.reset().ok();
        
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        stdout.set_color(ColorSpec::new().set_fg(Some(color))).ok();
        println!("📊 Usage: {:.2}%", memory_percent);
        stdout.reset().ok();
        
        println!("🎯 Threshold: {}%", self.config.threshold);
        println!("⏰ Interval : {} {}", self.config.check_interval, 
            if self.config.check_interval < 10 { "second" } else { "seconds" });
        println!("⏰ Time: {}", timestamp);
    }
    
    pub fn monitor(&mut self) {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        let width = terminal::size().map(|(w, _)| w as usize).unwrap_or(80);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true)).ok();
        println!("{}", "─".repeat(width));
        println!("💓 Memory Monitor Started");
        println!("Threshold: {}% | Interval: {}s", self.config.threshold, self.config.check_interval);
        println!("{}", "─".repeat(width));
        stdout.reset().ok();
        
        loop {
            let memory_percent = self.get_memory_usage();
            self.display_status(memory_percent);
            
            if memory_percent >= self.config.threshold {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true)).ok();
                println!("\n⚠️  ALERT: Memory usage at {:.2}%!\n", memory_percent);
                stdout.reset().ok();
                
                let success = self.cleaner.clean();
                
                if let Some(ref notifier) = self.notifier {
                    if success {
                        notifier.notify(
                            "🧹 Memory Cleaned",
                            &format!("Memory was at {:.2}%. CleanMem executed successfully.", memory_percent),
                            0,
                        );
                    } else {
                        notifier.notify(
                            "🔴 Memory Critical",
                            &format!("Memory at {:.2}%. CleanMem execution failed!", memory_percent),
                            2,
                        );
                    }
                }
                
                thread::sleep(Duration::from_secs(30));
                
                let new_memory = self.get_memory_usage();
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan))).ok();
                println!("📉 Memory after cleanup: {:.2}%\n", new_memory);
                stdout.reset().ok();
            }
            
            thread::sleep(Duration::from_secs(self.config.check_interval));
        }
    }
}
