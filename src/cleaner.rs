// File: src\cleaner.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2026-05-06
// Description: 
// License: MIT

use std::path::Path;
use std::process::Command;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Memory cleaner handler
pub struct MemoryCleaner {
    cleanmem_path: String,
}

impl MemoryCleaner {
    /// Create a new MemoryCleaner instance
    pub fn new(cleanmem_path: &str) -> Self {
        MemoryCleaner {
            cleanmem_path: cleanmem_path.to_string(),
        }
    }
    
    /// Execute the memory cleaner
    pub fn clean(&self) -> bool {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        if !Path::new(&self.cleanmem_path).exists() {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red))).ok();
            println!("❌ Error: {} not found!", self.cleanmem_path);
            stdout.reset().ok();
            return false;
        }
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow))).ok();
        println!("🧹 Running {}...", self.cleanmem_path);
        stdout.reset().ok();
        
        match Command::new(&self.cleanmem_path).output() {
            Ok(output) => {
                if output.status.success() {
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).ok();
                    println!("✅ CleanMem executed successfully");
                    stdout.reset().ok();
                    true
                } else {
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red))).ok();
                    println!("❌ Error running CleanMem: {}", 
                        String::from_utf8_lossy(&output.stderr));
                    stdout.reset().ok();
                    false
                }
            }
            Err(e) => {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red))).ok();
                println!("❌ Unexpected error: {}", e);
                stdout.reset().ok();
                false
            }
        }
    }
}