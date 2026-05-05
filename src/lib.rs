//! File: src\lib.rs
//! Author: Hadi Cahyadi <cumulus13@gmail.com>
//! Date: 2026-05-06
//! Description: 
//! License: MIT

//! Memory Monitor & Auto-Cleaner
//!
//! A high-performance memory monitoring daemon with automatic cleanup
//! and Growl/GNTP notification support.
//!
//! # Quick Start
//!
//! ```rust
//! use memory_monitor::MemoryMonitor;
//!
//! let monitor = MemoryMonitor::new();
//! monitor.monitor();
//! ```
//!
//! # Features
//!
//! - Real-time memory usage tracking
//! - Automatic memory cleanup when threshold exceeded
//! - GNTP/Growl desktop notifications
//! - Configurable thresholds and intervals
//! - Color-coded terminal interface

pub mod config;
pub mod monitor;
pub mod notification;
pub mod cleaner;

pub use monitor::MemoryMonitor;
pub use config::MonitorConfig;
