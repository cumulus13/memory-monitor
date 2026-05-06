# 🚀 Memory Monitor & Auto-Cleaner

[![Crates.io](https://img.shields.io/crates/v/memory-monitor.svg)](https://crates.io/crates/memory-monitor)
[![Documentation](https://docs.rs/memory-monitor/badge.svg)](https://docs.rs/memory-monitor)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)

A high-performance memory monitoring daemon with automatic cleanup and Growl/GNTP notification support. Written in Rust for maximum reliability and minimal resource usage.

## ✨ Features

- 🔍 **Real-time Memory Monitoring** — Track system memory usage with configurable thresholds
- 🧹 **Automatic Memory Cleanup** — Built-in native OS cleanup engine; no external tools required
- 🌍 **Multi-Platform** — Native cleanup strategies for Windows, Linux, and macOS
- 🔔 **Growl Notifications** — Desktop alerts via GNTP protocol
- 🎨 **Beautiful Terminal UI** — Color-coded status display with emojis
- ⚙️ **Highly Configurable** — All settings adjustable via `config.ini`
- 🚀 **Minimal Footprint** — Written in Rust for optimal performance
- 🔒 **Production Ready** — Error handling, graceful shutdown, and comprehensive logging

## 📦 Installation

### From Crates.io

```bash
cargo install memory-monitor
```

### From Source

```bash
git clone https://github.com/cumulus13/memory-monitor
cd memory-monitor
cargo build --release
# Binary at target/release/memory-monitor
```

## 🏃 Quick Start

1. **Run the monitor:**
```bash
memory-monitor
```

2. **On first run**, a `config.ini` is created with default settings:
```ini
[Settings]
threshold = 98.0
check_interval = 60
cleanmem_path =

[Growl]
host = localhost
port = 23053
password =
app_name = Memory Monitor
```

3. **That's it.** No external tools needed — the built-in cleanup engine activates automatically.

## ⚙️ Configuration

### Memory Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `threshold` | `98.0` | Memory usage percentage that triggers cleanup |
| `check_interval` | `60` | Seconds between memory checks |
| `cleanmem_path` | *(empty)* | Optional path to an external memory cleaner (e.g. `cleanmem.exe`). Leave empty to use the built-in native engine. |

### Growl/GNTP Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `host` | `localhost` | Growl server hostname or IP |
| `port` | `23053` | Growl server port |
| `password` | `""` | Growl server password (if required) |
| `app_name` | `Memory Monitor` | Application name shown in Growl |

## 🧹 Memory Cleanup Engine

### Strategy Selection

The cleaner picks the best available strategy automatically:

1. **If `cleanmem_path` is set and the file exists** → runs the external binary (legacy / Windows-only tool support)
2. **Otherwise** → activates the built-in native OS engine (recommended)

No configuration change is needed when moving between platforms.

### Native Cleanup — How It Works Per Platform

#### 🪟 Windows

A three-stage pipeline that replicates what tools like CleanMem, RAMMap, and Process Hacker do internally — using only Win32/NT APIs (no WMI, no PowerShell, no shell commands):

| Stage | API | Effect | Requires |
|-------|-----|--------|----------|
| 1 | `EnumProcesses` + `EmptyWorkingSet` (psapi.dll) | Trims the working set of every accessible process, moving pages from active RAM into the Standby/Modified lists | Any user (system processes skipped silently) |
| 2 | `NtSetSystemInformation(MemoryFlushModifiedList)` (ntdll.dll) | Writes Modified-list pages to the pagefile, converting them to Standby pages | Administrator |
| 3 | `NtSetSystemInformation(MemoryPurgeStandbyList)` (ntdll.dll) | Converts Standby pages to **Free RAM** — the number you see go up in Task Manager | Administrator |

> **Tip:** Run as Administrator for full effect. Without it, Stage 1 still trims accessible process working sets.

#### 🐧 Linux

| Stage | Method | Effect | Requires |
|-------|--------|--------|----------|
| 1 | `sync(2)` syscall | Flushes dirty filesystem pages to disk, maximising what can be dropped | Any user |
| 2 | `echo 3 > /proc/sys/vm/drop_caches` | Drops pagecache + dentries + inodes (non-destructive — only clean pages are dropped) | root / `CAP_SYS_ADMIN` |
| Fallback | `malloc_trim(0)` | Returns the monitor's own free heap pages to the OS | Any user |

> **Tip:** Run with `sudo memory-monitor` for the full cache-drop effect.

#### 🍎 macOS

| Stage | Method | Effect | Requires |
|-------|--------|--------|----------|
| 1 | `/usr/bin/purge` | Flushes the disk buffer cache (Apple's built-in tool) | root |
| Fallback | `malloc_trim(0)` → `malloc_zone_pressure_relief` | Asks all malloc zones to return free pages to the kernel | Any user |

> **Tip:** Run with `sudo memory-monitor` so `purge` can flush the disk cache.

## 🔔 Notification System

### Visual Indicators

| Memory Usage | Color | Status |
|-------------|-------|--------|
| Below threshold − 5% | 🟢 Green | OK |
| Within 5% of threshold | 🟡 Yellow | WARNING |
| At or above threshold | 🔴 Red | CRITICAL |

### Desktop Notifications

The monitor supports GNTP (Growl Notification Transport Protocol) for desktop notifications:

- **Normal Priority (0)**: Successful cleanup notifications
- **Emergency Priority (2)**: Critical memory alerts when cleanup fails

## 📊 Usage Examples

### Basic Monitoring (native engine)
```bash
# Leave cleanmem_path empty in config.ini — native engine runs automatically
memory-monitor
```

### With Elevated Privileges (recommended for full cleanup)
```bash
# Linux / macOS
sudo memory-monitor

# Windows — run the terminal as Administrator, then:
memory-monitor
```

### Legacy External Tool
```ini
# config.ini — point to an existing binary to use it instead of the native engine
[Settings]
cleanmem_path = C:\Tools\cleanmem.exe
```

### Silent / Background Mode
```bash
# Linux / macOS
nohup sudo memory-monitor &

# Windows (run terminal as Administrator)
start /B memory-monitor
```

## 🔧 Platform Requirements

| Platform | Minimum Privilege | Full Cleanup Privilege |
|----------|------------------|----------------------|
| Windows | Standard user (Stage 1 only) | Administrator |
| Linux | Any user (malloc_trim only) | root / CAP_SYS_ADMIN |
| macOS | Any user (malloc_trim only) | root (sudo) |

No external tools are required on any platform. `cleanmem.exe` remains supported for Windows users who already have it, but is fully optional.

## 📈 Performance

Memory Monitor is designed for minimal resource usage:

- **Memory footprint**: ~5–10 MB
- **CPU usage**: <1% at default 60 s interval
- **Binary size**: ~2–3 MB (release mode)

## 🛠️ Development

### Building
```bash
cargo build
cargo build --release  # Optimised build
```

### Testing
```bash
cargo test
cargo test -- --nocapture  # Show output
```

### Running Examples
```bash
cargo run --example basic
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

---

## 👤 Author
        
[Hadi Cahyadi](mailto:cumulus13@gmail.com)
    

[![Buy Me a Coffee](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/cumulus13)

[![Donate via Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/cumulus13)
 
[Support me on Patreon](https://www.patreon.com/cumulus13)

---

## 🎯 Roadmap

- [x] Multi-platform native memory cleanup (Windows, Linux, macOS)
- [ ] Multiple threshold levels
- [ ] Email notifications
- [ ] Slack/Telegram integration
- [ ] Prometheus metrics endpoint
- [ ] Web UI dashboard
- [ ] Docker support

## ⚡ Changelog

### v0.1.4 (2026-05-06)
- **Multi-platform native memory cleanup** — no external tools required
- Windows: `EmptyWorkingSet` + `NtSetSystemInformation` (MemoryFlushModifiedList + MemoryPurgeStandbyList) via psapi/ntdll
- Linux: `sync(2)` + `/proc/sys/vm/drop_caches=3` + `malloc_trim` fallback
- macOS: `/usr/bin/purge` + `malloc_trim`/`malloc_zone_pressure_relief` fallback
- Auto-fallback: if `cleanmem_path` is unset or binary not found, native engine activates automatically
- `cleanmem_path` remains fully supported for legacy compatibility
- Added `libc` (Linux/macOS) and `windows-sys` (Windows) platform dependencies

### v0.1.3
- Config file search across multiple directories and candidate filenames
- Platform-specific config paths (APPDATA, XDG_CONFIG_HOME, ~/Library)

### v0.1.0 (2025-11-01)
- Initial release
- Basic memory monitoring
- CleanMem integration
- Growl/GNTP notifications
- Config file support

## 🙏 Acknowledgments

- [Growl](http://growl.info/) — Notification system
- [CleanMem](https://www.pcwintech.com/cleanmem) — Memory optimisation tool (optional, legacy)
- Rust community for excellent crates

---

**Made with ❤️ using Rust**
