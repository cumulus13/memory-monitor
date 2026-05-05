# 🚀 Memory Monitor & Auto-Cleaner

[![Crates.io](https://img.shields.io/crates/v/memory-monitor.svg)](https://crates.io/crates/memory-monitor)
[![Documentation](https://docs.rs/memory-monitor/badge.svg)](https://docs.rs/memory-monitor)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)

A high-performance memory monitoring daemon with automatic cleanup and Growl/GNTP notification support. Written in Rust for maximum reliability and minimal resource usage.

## ✨ Features

- 🔍 **Real-time Memory Monitoring** - Track system memory usage with configurable thresholds
- 🧹 **Automatic Memory Cleanup** - Execute cleanmem.exe when memory exceeds threshold
- 🔔 **Growl Notifications** - Desktop alerts via GNTP protocol
- 🎨 **Beautiful Terminal UI** - Color-coded status display with emojis
- ⚙️ **Highly Configurable** - All settings adjustable via config.ini
- 🚀 **Minimal Footprint** - Written in Rust for optimal performance
- 🔒 **Production Ready** - Error handling, graceful shutdown, and comprehensive logging

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

2. **On first run**, it creates a `config.ini` with default settings:
```ini
[Settings]
threshold = 98.0
check_interval = 60
cleanmem_path = cleanmem.exe

[Growl]
host = localhost
port = 23053
password = 
app_name = Memory Monitor
```

3. **Install cleanmem** if using the default configuration or point `cleanmem_path` to your preferred memory cleaner.

## ⚙️ Configuration

### Memory Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `threshold` | `98.0` | Memory usage percentage that triggers cleanup |
| `check_interval` | `60` | Seconds between memory checks |
| `cleanmem_path` | `cleanmem.exe` | Path to memory cleaning executable |

### Growl/GNTP Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `host` | `localhost` | Growl server hostname or IP |
| `port` | `23053` | Growl server port |
| `password` | `""` | Growl server password (if required) |
| `app_name` | `Memory Monitor` | Application name shown in Growl |

## 🔔 Notification System

### Visual Indicators

| Memory Usage | Color | Status |
|-------------|-------|--------|
| Below threshold - 5% | 🟢 Green | OK |
| Within 5% of threshold | 🟡 Yellow | WARNING |
| At or above threshold | 🔴 Red | CRITICAL |

### Desktop Notifications

The monitor supports GNTP (Growl Notification Transport Protocol) for desktop notifications:

- **Normal Priority (0)**: Successful cleanup notifications
- **Emergency Priority (2)**: Critical memory alerts when cleanup fails

## 📊 Usage Examples

### Basic Monitoring
```bash
# Run with default settings
memory-monitor
```

### Custom Configuration
```bash
# Create custom config
memory-monitor --config custom-config.ini
```

### Silent Mode (background)
```bash
# Run in background (Linux/macOS)
nohup memory-monitor &

# Run in background (Windows)
start /B memory-monitor
```

## 🔧 Requirements

- **Windows**: cleanmem.exe (or compatible memory cleaner)
- **Growl**: Optional, for desktop notifications
  - [Growl for Windows](http://www.growlforwindows.com/)
  - [Growl for macOS](https://growl.github.io/growl/)

## 📈 Performance

Memory Monitor is designed for minimal resource usage:

- **Memory footprint**: ~5-10MB
- **CPU usage**: <1% at default 60s interval
- **Binary size**: ~2-3MB (release mode)

## 🛠️ Development

### Building
```bash
cargo build
cargo build --release  # Optimized build
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

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👤 Author
        
[Hadi Cahyadi](mailto:cumulus13@gmail.com)
    

[![Buy Me a Coffee](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/cumulus13)

[![Donate via Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/cumulus13)
 
[Support me on Patreon](https://www.patreon.com/cumulus13)

## 🎯 Roadmap

- [ ] Multiple threshold levels
- [ ] Email notifications
- [ ] Slack/Telegram integration
- [ ] Prometheus metrics endpoint
- [ ] Web UI dashboard
- [ ] Docker support

## ⚡ Changelog

### v0.1.0 (2025-11-01)
- Initial release
- Basic memory monitoring
- CleanMem integration
- Growl/GNTP notifications
- Config file support

## 🙏 Acknowledgments

- [Growl](http://growl.info/) - Notification system
- [CleanMem](https://www.pcwintech.com/cleanmem) - Memory optimization tool
- Rust community for excellent crates

---

**Made with ❤️ using Rust**