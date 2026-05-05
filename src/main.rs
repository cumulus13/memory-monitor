//! File: src\main.rs
//! Author: Hadi Cahyadi <cumulus13@gmail.com>
//! Date: 2026-05-06
//! Description: 
//! License: MIT

mod config;
mod monitor;
mod notification;
mod cleaner;

use monitor::MemoryMonitor;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use crossterm::terminal;

fn main() {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let width = terminal::size().map(|(w, _)| w as usize).unwrap_or(80);
    
    // Print header
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)).set_bold(true)).ok();
    println!("{}", "=".repeat(width));
    println!("        🚀 Memory Monitor & Auto-Cleaner");
    println!("{}", "=".repeat(width));
    println!();
    stdout.reset().ok();
    
    let mut monitor = MemoryMonitor::new();
    
    // Show current configuration
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan))).ok();
    println!("⚙️  Current Configuration:");
    stdout.reset().ok();
    println!("  📍 Threshold: {}%", monitor.threshold());
    println!("  ⏱️ Check Interval: {}s", monitor.check_interval());
    println!("  🔧 CleanMem Path: {}", monitor.cleanmem_path());
    
    if monitor.has_notifier() {
        println!("  🌐 Growl Host: {}:{}", monitor.growl_host(), monitor.growl_port());
        println!("  📱 App Name: {}", monitor.growl_app_name());
    }
    
    println!();
    
    // Handle Ctrl+C gracefully
    ctrlc::set_handler(move || {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow))).ok();
        println!("\n👋 Monitoring stopped by user");
        stdout.reset().ok();
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");
    
    // Start monitoring
    monitor.monitor();
}