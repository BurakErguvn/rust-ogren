//! rust-analyzer stdio <-> WebSocket bridge.
//!
//! One WebSocket text frame = one JSON-RPC message without `Content-Length` header.
//! The server adds/strips the LSP base-protocol framing on the stdio side.

use std::collections::VecDeque;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader,
};
use tokio::process::{Child, Command};
use tokio::sync::Notify;

use crate::proc::{isolate, GroupGuard};

const MAX_MESSAGE_BYTES: usize = 64 * 1024 * 1024;

/// Tracks live sessions; beyond `max` the oldest one is asked to close.
pub struct LspSessions {
    max: usize,
    next_id: AtomicU64,
    live: Mutex<VecDeque<(u64, Arc<Notify>)>>,
}

pub struct SessionTicket {
    id: u64,
    pub cancel: Arc<Notify>,
    sessions: Arc<LspSessions>,
}

impl Drop for SessionTicket {
    fn drop(&mut self) {
        if let Ok(mut live) = self.sessions.live.lock() {
            live.retain(|(id, _)| *id != self.id);
        }
    }
}

impl LspSessions {
    pub fn new(max: usize) -> Arc<Self> {
        Arc::new(LspSessions {
            max,
            next_id: AtomicU64::new(1),
            live: Mutex::new(VecDeque::new()),
        })
    }

    pub fn register(self: &Arc<Self>) -> SessionTicket {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let cancel = Arc::new(Notify::new());
        if let Ok(mut live) = self.live.lock() {
            live.push_back((id, cancel.clone()));
            while live.len() > self.max {
                if let Some((old, n)) = live.pop_front() {
                    tracing::info!("lsp: closing oldest session {old} (limit {})", self.max);
                    n.notify_one();
                }
            }
        }
        SessionTicket {
            id,
            cancel,
            sessions: self.clone(),
        }
    }
}

/// Spawns rust-analyzer in the workspace with its own target dir so its `cargo check`
/// never contends with the test runner's `target/` lock.
pub fn spawn_rust_analyzer(
    binary: &Path,
    workspace: &Path,
) -> std::io::Result<(Child, Option<GroupGuard>)> {
    let mut cmd = Command::new(binary);
    isolate(&mut cmd)
        .current_dir(workspace)
        .env("CARGO_TARGET_DIR", workspace.join("target-ra"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = cmd.spawn()?;
    let guard = GroupGuard::new(&child);
    Ok((child, guard))
}

/// Reads one LSP message body. `Ok(None)` on clean EOF.
pub async fn read_lsp_message<R: AsyncBufRead + Unpin>(
    r: &mut R,
) -> std::io::Result<Option<Vec<u8>>> {
    let mut content_length: Option<usize> = None;
    let mut line = String::new();
    let mut saw_any = false;
    loop {
        line.clear();
        let n = r.read_line(&mut line).await?;
        if n == 0 {
            return if saw_any {
                Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "eof in LSP header",
                ))
            } else {
                Ok(None)
            };
        }
        saw_any = true;
        let l = line.trim_end_matches(['\r', '\n']);
        if l.is_empty() {
            if content_length.is_some() {
                break;
            }
            continue; // tolerate stray blank lines between messages
        }
        if let Some((k, v)) = l.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-length") {
                content_length = v.trim().parse().ok();
            }
        }
    }
    let len = content_length.unwrap_or(0);
    if len > MAX_MESSAGE_BYTES {
        return Err(std::io::Error::other(format!(
            "LSP message too large: {len}"
        )));
    }
    let mut body = vec![0u8; len];
    r.read_exact(&mut body).await?;
    Ok(Some(body))
}

pub async fn write_lsp_message<W: AsyncWrite + Unpin>(
    w: &mut W,
    body: &[u8],
) -> std::io::Result<()> {
    w.write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())
        .await?;
    w.write_all(body).await?;
    w.flush().await
}

pub async fn bridge(
    socket: WebSocket,
    mut child: Child,
    guard: Option<GroupGuard>,
    ticket: SessionTicket,
    idle_timeout: Duration,
    label: String,
) {
    tracing::info!("lsp[{label}]: session started (pid {:?})", child.id());
    let mut stdin = child.stdin.take().expect("piped stdin");
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");

    let stderr_label = label.clone();
    let stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(l)) = lines.next_line().await {
            tracing::debug!("rust-analyzer[{stderr_label}]: {l}");
        }
    });

    let (mut ws_tx, mut ws_rx) = socket.split();
    let mut reader = BufReader::new(stdout);

    let to_client = async {
        loop {
            match read_lsp_message(&mut reader).await {
                Ok(Some(body)) => {
                    let text = String::from_utf8_lossy(&body).into_owned();
                    if ws_tx.send(Message::Text(text.into())).await.is_err() {
                        return "client send failed";
                    }
                }
                Ok(None) => return "rust-analyzer exited",
                Err(e) => {
                    tracing::warn!("lsp[{label}]: bad message from rust-analyzer: {e}");
                    return "rust-analyzer stream error";
                }
            }
        }
    };

    let from_client = async {
        loop {
            let next = match tokio::time::timeout(idle_timeout, ws_rx.next()).await {
                Ok(n) => n,
                Err(_) => return "idle timeout",
            };
            let body = match next {
                Some(Ok(Message::Text(t))) => t.as_str().as_bytes().to_vec(),
                Some(Ok(Message::Binary(b))) => b.to_vec(),
                Some(Ok(Message::Ping(_) | Message::Pong(_))) => continue,
                Some(Ok(Message::Close(_))) | None => return "client closed",
                Some(Err(_)) => return "client error",
            };
            if write_lsp_message(&mut stdin, &body).await.is_err() {
                return "rust-analyzer stdin closed";
            }
        }
    };

    let reason = tokio::select! {
        r = to_client => r,
        r = from_client => r,
        _ = ticket.cancel.notified() => "replaced by a newer session",
    };
    tracing::info!("lsp[{label}]: session ended: {reason}");
    drop(guard); // kills the rust-analyzer process group
    let _ = child.kill().await;
    let _ = ws_tx.send(Message::Close(None)).await;
    stderr_task.abort();
    drop(ticket);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn framing_roundtrip() {
        let mut buf: Vec<u8> = Vec::new();
        write_lsp_message(&mut buf, br#"{"jsonrpc":"2.0","id":1}"#)
            .await
            .unwrap();
        write_lsp_message(&mut buf, "{\"m\":\"ç\"}".as_bytes())
            .await
            .unwrap();
        assert!(buf.starts_with(b"Content-Length: 24\r\n\r\n{"));
        let mut r = BufReader::new(&buf[..]);
        assert_eq!(
            read_lsp_message(&mut r).await.unwrap().unwrap(),
            br#"{"jsonrpc":"2.0","id":1}"#
        );
        assert_eq!(
            read_lsp_message(&mut r).await.unwrap().unwrap(),
            "{\"m\":\"ç\"}".as_bytes()
        );
        assert!(read_lsp_message(&mut r).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn tolerates_extra_headers() {
        let raw = b"Content-Type: application/vscode-jsonrpc; charset=utf-8\r\ncontent-length: 2\r\n\r\n{}";
        let mut r = BufReader::new(&raw[..]);
        assert_eq!(read_lsp_message(&mut r).await.unwrap().unwrap(), b"{}");
    }

    #[test]
    fn session_limit_cancels_oldest() {
        let s = LspSessions::new(1);
        let a = s.register();
        let _b = s.register();
        // `a` got a stored permit.
        let fut = a.cancel.notified();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            tokio::time::timeout(Duration::from_millis(100), fut)
                .await
                .expect("notified");
        });
    }
}
