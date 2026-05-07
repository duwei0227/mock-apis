use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

pub fn pid_path(db: &str) -> PathBuf {
    let path = Path::new(db);
    let stem = path.file_stem().unwrap_or_else(|| OsStr::new("apimock"));
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    dir.join(format!("{}.pid", stem.to_string_lossy()))
}

fn read_pid(db: &str) -> Option<u32> {
    std::fs::read_to_string(pid_path(db))
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

fn write_pid(db: &str, pid: u32) -> std::io::Result<()> {
    std::fs::write(pid_path(db), pid.to_string())
}

pub fn remove_pid(db: &str) {
    let _ = std::fs::remove_file(pid_path(db));
}

pub fn is_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        // kill -0 checks existence without signalling
        Command::new("kill")
            .args(["-0", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
            .unwrap_or(false)
    }
}

/// Returns true if a daemon process *other than the current process* owns the PID file.
/// Prevents TUI/dashboard from calling start_all_enabled when a daemon already manages ports.
pub fn is_external_daemon_running(db: &str) -> bool {
    let my_pid = std::process::id();
    match read_pid(db) {
        Some(pid) if pid != my_pid => is_process_alive(pid),
        _ => false,
    }
}

pub fn start(db: &str, port: u16) -> anyhow::Result<()> {
    if let Some(pid) = read_pid(db) {
        if is_process_alive(pid) {
            println!("Mock server is already running (PID: {}).", pid);
            return Ok(());
        }
        // Stale PID file — clean up before re-spawning.
        remove_pid(db);
    }

    let exe = std::env::current_exe()?;
    let mut cmd = Command::new(&exe);
    cmd.args(["serve", "--db", db, "--port", &port.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // On Unix: create a new session so the daemon survives terminal close.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                extern "C" {
                    fn setsid() -> i32;
                }
                setsid();
                Ok(())
            });
        }
    }

    let child = cmd.spawn()?;
    let pid = child.id();
    // Detach: do not wait for the child — it runs as a daemon.
    std::mem::forget(child);

    write_pid(db, pid)?;
    println!("Mock server started (PID: {}).", pid);
    println!("Dashboard: http://localhost:{}", port);
    println!("Stop with: mock stop");
    Ok(())
}

pub fn stop(db: &str) -> anyhow::Result<()> {
    let pid = match read_pid(db) {
        Some(p) => p,
        None => {
            println!("No running mock server found.");
            return Ok(());
        }
    };

    if !is_process_alive(pid) {
        println!("Mock server (PID: {}) is not running. Cleaning up stale PID file.", pid);
        remove_pid(db);
        return Ok(());
    }

    kill_process(pid)?;

    // Wait up to 5 s for graceful exit.
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(100));
        if !is_process_alive(pid) {
            break;
        }
    }

    remove_pid(db);
    println!("Mock server stopped.");
    Ok(())
}

fn kill_process(pid: u32) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        let ok = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()?
            .success();
        if !ok {
            anyhow::bail!("Failed to send SIGTERM to PID {}", pid);
        }
    }
    #[cfg(windows)]
    {
        Command::new("taskkill")
            .args(["/PID", &pid.to_string()])
            .status()?;
    }
    Ok(())
}

pub fn restart(db: &str, port: u16) -> anyhow::Result<()> {
    stop(db)?;
    start(db, port)
}

pub fn status(db: &str) {
    match read_pid(db) {
        None => println!("Mock server is not running."),
        Some(pid) if !is_process_alive(pid) => {
            println!("Mock server is not running (stale PID file).");
            remove_pid(db);
        }
        Some(pid) => println!("Mock server is running (PID: {}).", pid),
    }
}
