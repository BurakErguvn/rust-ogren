//! Child process helpers: every child gets its own process group, which is killed
//! (SIGKILL to the whole group) on timeout, when the owning future is dropped (client
//! disconnect) or at server shutdown.

use std::collections::HashSet;
use std::process::{ExitStatus, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};

fn registry() -> &'static Mutex<HashSet<i32>> {
    static REG: OnceLock<Mutex<HashSet<i32>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashSet::new()))
}

fn kill_group(pgid: i32) {
    if pgid > 1 {
        // SAFETY: plain syscall; a negative pid addresses the process group.
        unsafe {
            libc::kill(-pgid, libc::SIGKILL);
        }
    }
}

/// Kills all live child process groups. Called on server shutdown.
pub fn kill_all() {
    let groups: Vec<i32> = registry()
        .lock()
        .map(|g| g.iter().copied().collect())
        .unwrap_or_default();
    for pgid in groups {
        tracing::info!("shutdown: killing process group {pgid}");
        kill_group(pgid);
    }
}

/// Owns a process group; kills it when dropped.
#[derive(Debug)]
pub struct GroupGuard {
    pgid: i32,
}

impl GroupGuard {
    pub fn new(child: &Child) -> Option<GroupGuard> {
        let pid = child.id()? as i32;
        if let Ok(mut g) = registry().lock() {
            g.insert(pid);
        }
        Some(GroupGuard { pgid: pid })
    }

    pub fn kill(&self) {
        kill_group(self.pgid);
    }
}

impl Drop for GroupGuard {
    fn drop(&mut self) {
        // Always kill: also reaps grandchildren the leader may have left behind.
        kill_group(self.pgid);
        if let Ok(mut g) = registry().lock() {
            g.remove(&self.pgid);
        }
    }
}

/// Configures a command to start in a new process group and be killed if dropped.
pub fn isolate(cmd: &mut Command) -> &mut Command {
    cmd.process_group(0).kill_on_drop(true)
}

#[derive(Debug)]
pub struct Captured {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    /// `None` when the process was killed due to timeout.
    pub status: Option<ExitStatus>,
    pub timed_out: bool,
    /// Output beyond the cap was discarded.
    pub truncated: bool,
}

impl Captured {
    pub fn stdout_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }
    pub fn stderr_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
}

/// Reads everything, keeping at most `cap` bytes.
async fn read_capped<R: AsyncRead + Unpin>(mut r: R, cap: usize) -> (Vec<u8>, bool) {
    let mut out = Vec::new();
    let mut buf = [0u8; 8192];
    let mut truncated = false;
    loop {
        match r.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                let room = cap.saturating_sub(out.len());
                if room < n {
                    truncated = true;
                }
                out.extend_from_slice(&buf[..n.min(room)]);
            }
        }
    }
    (out, truncated)
}

/// Runs `cmd` to completion with a deadline, capturing at most `cap` bytes of each stream.
/// On timeout the whole process group is killed.
pub async fn run_captured(
    mut cmd: Command,
    timeout: Duration,
    cap: usize,
) -> std::io::Result<Captured> {
    isolate(&mut cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn()?;
    let guard = GroupGuard::new(&child);
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let out_task = tokio::spawn(read_capped(stdout, cap));
    let err_task = tokio::spawn(read_capped(stderr, cap));

    let (status, timed_out) = match tokio::time::timeout(timeout, child.wait()).await {
        Ok(status) => (Some(status?), false),
        Err(_) => {
            if let Some(g) = &guard {
                g.kill();
            }
            let _ = child.kill().await;
            (None, true)
        }
    };
    // Kill leftovers in the group so the pipes close.
    if let Some(g) = &guard {
        g.kill();
    }
    drop(guard);

    // A process that escaped the group could keep the pipes open; don't wait forever.
    let grace = Duration::from_secs(2);
    let (stdout, t1) = tokio::time::timeout(grace, out_task)
        .await
        .ok()
        .and_then(|r| r.ok())
        .unwrap_or_default();
    let (stderr, t2) = tokio::time::timeout(grace, err_task)
        .await
        .ok()
        .and_then(|r| r.ok())
        .unwrap_or_default();
    Ok(Captured {
        stdout,
        stderr,
        status,
        timed_out,
        truncated: t1 || t2,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn captures_output() {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("echo hello; echo err >&2; exit 3");
        let c = run_captured(cmd, Duration::from_secs(5), 1024)
            .await
            .unwrap();
        assert_eq!(c.stdout_lossy(), "hello\n");
        assert_eq!(c.stderr_lossy(), "err\n");
        assert_eq!(c.status.unwrap().code(), Some(3));
        assert!(!c.timed_out);
    }

    #[tokio::test]
    async fn kills_group_on_timeout() {
        // The background `sleep` holds the pipe open; it must be killed with the group.
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("sleep 30 & sleep 30; echo never");
        let start = std::time::Instant::now();
        let c = run_captured(cmd, Duration::from_millis(300), 1024)
            .await
            .unwrap();
        assert!(c.timed_out);
        assert!(c.status.is_none());
        assert!(start.elapsed() < Duration::from_secs(2));
    }

    #[tokio::test]
    async fn caps_output() {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("yes | head -c 100000");
        let c = run_captured(cmd, Duration::from_secs(5), 1000)
            .await
            .unwrap();
        assert_eq!(c.stdout.len(), 1000);
        assert!(c.truncated);
    }
}
