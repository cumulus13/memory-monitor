//! File: src\notification.rs
//! Author: Hadi Cahyadi <cumulus13@gmail.com>
//! Date: 2026-05-06
//! Description: 
//! License: MIT

use gntp::{GntpClient, NotificationType, Resource};
use std::path::PathBuf;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct GntpNotifier {
    client: GntpClient,
    host: String,
    port: u16,
}

impl GntpNotifier {
    pub fn new(host: &str, port: u16, password: &str, app_name: &str, icon_path: &str) -> Option<Self> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        println!("\n📬 Checking notification libraries...");
        
        // Find icon resource
        let icon = Self::find_icon(icon_path);
        
        let mut client = if password.is_empty() {
            GntpClient::new(app_name)
                .with_host(host)
                .with_port(port)
        } else {
            GntpClient::new(app_name)
                .with_host(host)
                .with_port(port)
                .with_password(password)
        };
        
        if let Some(ref icon_res) = icon {
            client = client.with_icon(icon_res.clone());
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).ok();
            println!("  🖼️  Icon loaded: {}", icon_path);
            stdout.reset().ok();
        } else {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow))).ok();
            println!("  ⚠️  Icon not found: {}", icon_path);
            stdout.reset().ok();
        }
        
        // Create notification type with icon
        let mut notification = NotificationType::new("Memory Alert")
            .with_display_name("Memory Alert");
        
        // Add icon to notification type if available
        if let Some(icon_res) = icon {
            notification = notification.with_icon(icon_res);
        }
        
        match client.register(vec![notification]) {
            Ok(_) => {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).ok();
                println!("  ✅ GNTP found - Growl notifications enabled");
                stdout.reset().ok();
                
                Some(GntpNotifier {
                    client,
                    host: host.to_string(),
                    port,
                })
            }
            Err(e) => {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow))).ok();
                println!("  ⚠️  GNTP initialization failed: {:?}", e);
                println!("  💡 Make sure Growl is running on {}:{}", host, port);
                stdout.reset().ok();
                None
            }
        }
    }
    
    fn find_icon(icon_path: &str) -> Option<Resource> {
        // Try exact path first
        if let Ok(icon) = Resource::from_file(icon_path) {
            return Some(icon);
        }
        
        // Try relative to executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let icon_in_exe_dir = exe_dir.join(icon_path);
                if let Ok(icon) = Resource::from_file(&icon_in_exe_dir) {
                    return Some(icon);
                }
            }
        }
        
        // Try current directory
        if let Ok(cwd) = std::env::current_dir() {
            let icon_in_cwd = cwd.join(icon_path);
            if let Ok(icon) = Resource::from_file(&icon_in_cwd) {
                return Some(icon);
            }
        }
        
        None
    }
    
    pub fn notify(&self, title: &str, message: &str, _priority: i8) -> bool {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        println!("\n📤 Sending notifications...");
        
        match self.client.notify("Memory Alert", title, message) {
            Ok(_) => {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).ok();
                println!("📬 ✅ Growl notification sent to {}:{}", self.host, self.port);
                stdout.reset().ok();
                true
            }
            Err(e) => {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red))).ok();
                println!("❌ Growl notification error: {:?}", e);
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow))).ok();
                println!("💡 Make sure Growl is running on {}:{}", self.host, self.port);
                stdout.reset().ok();
                false
            }
        }
    }
}