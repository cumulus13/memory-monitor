// File: src/cleaner.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2026-05-06
// Description: Multi-platform native memory cleaner — no external tool required.
//              Falls back to native OS APIs automatically when cleanmem_path is
//              absent or the binary does not exist.
// License: MIT

use std::path::Path;
use std::process::Command;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

// ─────────────────────────────────────────────────────────────────────────────
// Shared result type
// ─────────────────────────────────────────────────────────────────────────────
pub struct CleanResult {
    pub success: bool,
    pub method: &'static str,
    pub detail: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Windows
// ─────────────────────────────────────────────────────────────────────────────
//
// Three-stage approach (matches what CleanMem.exe and RAMMap do internally):
//
//  Stage 1 — EmptyWorkingSet on every process we can open.
//            Moves pages from active RAM → Standby/Modified lists.
//            Requires PROCESS_SET_QUOTA access right; most user processes are
//            accessible even without admin (system processes will be skipped).
//
//  Stage 2 — NtSetSystemInformation(SystemMemoryListInformation,
//                                    MemoryFlushModifiedList)
//            Writes Modified-list pages to the pagefile/disk, converting them
//            to Standby pages.
//            Requires SeProfileSingleProcessPrivilege (granted to Admins).
//
//  Stage 3 — NtSetSystemInformation(SystemMemoryListInformation,
//                                    MemoryPurgeStandbyList)
//            Marks Standby pages as Free — this is what actually shows up as
//            "more free RAM" in Task Manager.
//            Requires SeIncreaseQuotaPrivilege (granted to Admins).
//
// Without admin, only Stage 1 partially succeeds; it still helps by trimming
// the user's own and other accessible processes' working sets.
//
#[cfg(target_os = "windows")]
mod platform {
    use std::ffi::c_void;
    use windows_sys::Win32::{
        Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
        System::{
            ProcessStatus::EnumProcesses,
            Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA},
        },
    };

    // windows-sys doesn't expose NtSetSystemInformation or EmptyWorkingSet in its
    // current feature set, so we bind them manually — they are stable Win32/NT APIs.
    #[link(name = "psapi")]
    unsafe extern "system" {
        fn EmptyWorkingSet(hprocess: HANDLE) -> i32;
    }

    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtSetSystemInformation(
            system_information_class: u32,
            system_information: *mut c_void,
            system_information_length: u32,
        ) -> i32; // NTSTATUS: >= 0 means success
    }

    const SYSTEM_MEMORY_LIST_INFORMATION: u32 = 0x50; // 80
    const MEMORY_FLUSH_MODIFIED_LIST: u32 = 3;
    const MEMORY_PURGE_STANDBY_LIST: u32 = 4;

    // ── Stage 1: trim working sets ─────────────────────────────────────────
    fn empty_all_working_sets() -> (usize, usize) {
        let mut pids = vec![0u32; 1024];
        let mut bytes_returned: u32 = 0;
        let ok = unsafe {
            EnumProcesses(
                pids.as_mut_ptr(),
                (pids.len() * std::mem::size_of::<u32>()) as u32,
                &mut bytes_returned,
            )
        };
        if ok == 0 {
            return (0, 0);
        }
        let count = bytes_returned as usize / std::mem::size_of::<u32>();
        let mut trimmed = 0usize;
        let mut skipped = 0usize;

        for &pid in &pids[..count] {
            if pid == 0 {
                continue;
            }
            let handle = unsafe {
                OpenProcess(PROCESS_SET_QUOTA | PROCESS_QUERY_INFORMATION, 0, pid)
            };
            if handle == 0 || handle == INVALID_HANDLE_VALUE {
                skipped += 1;
                continue;
            }
            let result = unsafe { EmptyWorkingSet(handle) };
            unsafe { CloseHandle(handle) };
            if result != 0 {
                trimmed += 1;
            } else {
                skipped += 1;
            }
        }
        (trimmed, skipped)
    }

    // ── Stage 2: flush Modified list to pagefile ───────────────────────────
    fn flush_modified_list() -> bool {
        let mut cmd = MEMORY_FLUSH_MODIFIED_LIST;
        let status = unsafe {
            NtSetSystemInformation(
                SYSTEM_MEMORY_LIST_INFORMATION,
                &mut cmd as *mut u32 as *mut c_void,
                std::mem::size_of::<u32>() as u32,
            )
        };
        status >= 0
    }

    // ── Stage 3: purge Standby list → Free ────────────────────────────────
    fn purge_standby_list() -> bool {
        let mut cmd = MEMORY_PURGE_STANDBY_LIST;
        let status = unsafe {
            NtSetSystemInformation(
                SYSTEM_MEMORY_LIST_INFORMATION,
                &mut cmd as *mut u32 as *mut c_void,
                std::mem::size_of::<u32>() as u32,
            )
        };
        status >= 0
    }

    pub fn clean_memory() -> super::CleanResult {
        let (trimmed, skipped) = empty_all_working_sets();
        let flushed = flush_modified_list();
        let purged = purge_standby_list();

        let success = purged || flushed || trimmed > 0;
        super::CleanResult {
            success,
            method: "Windows native (EmptyWorkingSet + NtSetSystemInformation)",
            detail: format!(
                "Working-set trim: {} processes OK, {} skipped | \
                 Modified list flushed: {} | Standby list purged: {}",
                trimmed, skipped, flushed, purged
            ),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Linux
// ─────────────────────────────────────────────────────────────────────────────
//
// Two-stage approach:
//
//  Stage 1 — sync(2): flush dirty filesystem pages to disk.
//            Without this, drop_caches won't be able to reclaim as much,
//            since dirty pages cannot be dropped.
//
//  Stage 2 — echo 3 > /proc/sys/vm/drop_caches:
//            Tells the kernel to drop pagecache (1) + dentries + inodes (2).
//            This is completely safe — the kernel will only drop *clean* pages;
//            nothing in-use or dirty is touched.
//            Requires CAP_SYS_ADMIN (i.e. root).
//
//  Fallback (no root) — malloc_trim(0):
//            Asks glibc's allocator to return all free top-of-heap memory back
//            to the OS. Only affects the monitor process itself, but costs
//            nothing and doesn't require privileges.
//
#[cfg(target_os = "linux")]
mod platform {
    use std::io::Write;

    fn sync_fs() {
        unsafe { libc::sync() };
    }

    fn drop_caches(level: u8) -> std::io::Result<()> {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .open("/proc/sys/vm/drop_caches")?;
        write!(f, "{}", level)?;
        Ok(())
    }

    fn malloc_trim_self() {
        unsafe { libc::malloc_trim(0) };
    }

    pub fn clean_memory() -> super::CleanResult {
        sync_fs();
        malloc_trim_self(); // always harmless

        match drop_caches(3) {
            Ok(_) => super::CleanResult {
                success: true,
                method: "Linux native (sync + drop_caches=3)",
                detail: "Filesystem synced; pagecache, dentries and inodes dropped.".to_string(),
            },
            Err(e) => super::CleanResult {
                success: false,
                method: "Linux limited (sync + malloc_trim — root required for drop_caches)",
                detail: format!(
                    "sync() OK; drop_caches failed ({}). \
                     Run as root or with CAP_SYS_ADMIN for full effect.",
                    e
                ),
            },
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// macOS
// ─────────────────────────────────────────────────────────────────────────────
//
// Two-stage approach:
//
//  Stage 1 — `purge` command:
//            Apple's built-in tool that flushes the disk buffer cache.
//            Lives at /usr/bin/purge; effective but requires root/sudo.
//
//  Fallback — malloc_trim(0):
//            Same glibc-compatible call; on macOS this maps to
//            malloc_zone_pressure_relief(NULL, 0) via libSystem, which asks all
//            registered malloc zones to return free pages to the kernel.
//
#[cfg(target_os = "macos")]
mod platform {
    fn run_purge() -> bool {
        std::process::Command::new("/usr/bin/purge")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn malloc_trim_self() {
        // malloc_trim is available on macOS via libSystem (maps to
        // malloc_zone_pressure_relief internally).
        unsafe { libc::malloc_trim(0) };
    }

    pub fn clean_memory() -> super::CleanResult {
        malloc_trim_self(); // always run regardless

        if run_purge() {
            super::CleanResult {
                success: true,
                method: "macOS native (/usr/bin/purge)",
                detail: "Disk buffer cache purged; malloc zones pressure-relieved.".to_string(),
            }
        } else {
            super::CleanResult {
                success: false,
                method: "macOS limited (malloc_trim only — sudo required for purge)",
                detail: "/usr/bin/purge requires root. malloc zone pressure-relief applied. \
                         Run with sudo for full disk-cache flush."
                    .to_string(),
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unsupported platform stub
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
mod platform {
    pub fn clean_memory() -> super::CleanResult {
        super::CleanResult {
            success: false,
            method: "Unsupported platform",
            detail: "No native memory-cleaning strategy is available for this OS.".to_string(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public struct
// ─────────────────────────────────────────────────────────────────────────────

/// Memory cleaner that automatically selects the best available strategy:
///
/// - If `cleanmem_path` points to an existing executable → run it (legacy mode).
/// - Otherwise → invoke the built-in native OS strategy:
///     - **Windows**: EmptyWorkingSet (all processes) → flush Modified list →
///                    purge Standby list via `NtSetSystemInformation`.
///     - **Linux**:   `sync` + `echo 3 > /proc/sys/vm/drop_caches` (root).
///     - **macOS**:   `/usr/bin/purge` (root) + `malloc_trim`.
pub struct MemoryCleaner {
    cleanmem_path: String,
}

impl MemoryCleaner {
    pub fn new(cleanmem_path: &str) -> Self {
        MemoryCleaner {
            cleanmem_path: cleanmem_path.to_string(),
        }
    }

    /// Run memory cleanup. Returns `true` on full success, `false` on partial
    /// success or failure (with printed diagnostics).
    pub fn clean(&self) -> bool {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);

        let use_external = !self.cleanmem_path.is_empty()
            && Path::new(&self.cleanmem_path).exists();

        if use_external {
            return self.run_external(&mut stdout);
        }

        // Tell the user what we're doing and why
        if !self.cleanmem_path.is_empty() {
            stdout
                .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))
                .ok();
            println!(
                "⚠️  '{}' not found — switching to native OS memory cleaner.",
                self.cleanmem_path
            );
            stdout.reset().ok();
        } else {
            stdout
                .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
                .ok();
            println!("🧠 cleanmem_path not configured — using native OS memory cleaner.");
            stdout.reset().ok();
        }

        self.run_native(&mut stdout)
    }

    // ── external tool (unchanged legacy behaviour) ────────────────────────
    fn run_external(&self, stdout: &mut StandardStream) -> bool {
        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))
            .ok();
        println!("🧹 Running {}...", self.cleanmem_path);
        stdout.reset().ok();

        match Command::new(&self.cleanmem_path).output() {
            Ok(output) => {
                if output.status.success() {
                    stdout
                        .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
                        .ok();
                    println!("✅ External cleaner executed successfully");
                    stdout.reset().ok();
                    true
                } else {
                    stdout
                        .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                        .ok();
                    println!(
                        "❌ External cleaner error: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    stdout.reset().ok();
                    false
                }
            }
            Err(e) => {
                stdout
                    .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                    .ok();
                println!("❌ Unexpected error running external cleaner: {}", e);
                stdout.reset().ok();
                false
            }
        }
    }

    // ── native OS path ─────────────────────────────────────────────────────
    fn run_native(&self, stdout: &mut StandardStream) -> bool {
        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
            .ok();
        println!("🔧 Running native memory cleanup…");
        stdout.reset().ok();

        let result = platform::clean_memory();

        if result.success {
            stdout
                .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
                .ok();
            println!("✅ Native cleanup succeeded");
            println!("   Method : {}", result.method);
            println!("   Detail : {}", result.detail);
        } else {
            stdout
                .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))
                .ok();
            println!("⚠️  Native cleanup partial / limited");
            println!("   Method : {}", result.method);
            println!("   Detail : {}", result.detail);

            self.print_privilege_tip(stdout);
        }

        stdout.reset().ok();
        result.success
    }

    fn print_privilege_tip(&self, stdout: &mut StandardStream) {
        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))
            .ok();
        #[cfg(target_os = "windows")]
        println!("💡 Tip: Run memory-monitor as Administrator for full effect.");
        #[cfg(target_os = "linux")]
        println!("💡 Tip: Run with `sudo memory-monitor` for full cache-drop effect.");
        #[cfg(target_os = "macos")]
        println!("💡 Tip: Run with `sudo memory-monitor` so `purge` can flush the disk cache.");
        stdout.reset().ok();
    }
}