//! Runtime configuration, read from environment variables.

use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    /// Repository root (contains `book/`, `exercises/`, `helpers/`, `tr/`).
    pub root: PathBuf,
    pub web_dist: PathBuf,
    pub workspace_dir: PathBuf,
    pub data_dir: PathBuf,
    pub compile_timeout: Duration,
    pub test_timeout: Duration,
    /// Path or name of the rust-analyzer binary.
    pub rust_analyzer: PathBuf,
    /// Close an LSP session after this long without client messages.
    pub lsp_idle_timeout: Duration,
    /// Maximum number of concurrent rust-analyzer processes; the oldest is closed beyond this.
    pub lsp_max_sessions: usize,
}

fn env_string(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

fn env_parse<T: std::str::FromStr>(name: &str, default: T) -> Result<T, String> {
    match env_string(name) {
        None => Ok(default),
        Some(v) => v
            .trim()
            .parse()
            .map_err(|_| format!("invalid value for {name}: {v:?}")),
    }
}

fn is_repo_root(p: &Path) -> bool {
    p.join("book/src/SUMMARY.md").is_file() && p.join("exercises").is_dir()
}

/// `RUST_LEARN_ROOT` > compile-time location of this crate (`app/server/../..`) >
/// current directory or one of its ancestors.
fn detect_root() -> Result<PathBuf, String> {
    if let Some(v) = env_string("RUST_LEARN_ROOT") {
        let p = PathBuf::from(v);
        if !is_repo_root(&p) {
            return Err(format!(
                "RUST_LEARN_ROOT={} does not look like the repository root (book/src/SUMMARY.md missing)",
                p.display()
            ));
        }
        return Ok(p);
    }
    let compiled = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    if is_repo_root(&compiled) {
        return Ok(compiled);
    }
    if let Ok(cwd) = std::env::current_dir() {
        for dir in cwd.ancestors() {
            if is_repo_root(dir) {
                return Ok(dir.to_path_buf());
            }
        }
    }
    Err("could not find the repository root; set RUST_LEARN_ROOT".into())
}

impl Config {
    pub fn from_env() -> Result<Config, String> {
        let root = detect_root()?;
        let root = root
            .canonicalize()
            .map_err(|e| format!("canonicalize root: {e}"))?;
        let path_or = |name: &str, default: PathBuf| -> PathBuf {
            env_string(name).map(PathBuf::from).unwrap_or(default)
        };
        let web_dist = path_or("WEB_DIST", root.join("app/web/out"));
        let workspace_dir = path_or("WORKSPACE_DIR", root.join("app/workspace"));
        let data_dir = path_or("DATA_DIR", root.join("app/data"));
        let host: IpAddr = env_parse("HOST", IpAddr::from([127, 0, 0, 1]))?;
        let port: u16 = env_parse("PORT", 3000)?;
        let compile_timeout = Duration::from_secs(env_parse("COMPILE_TIMEOUT_SECS", 120u64)?);
        let test_timeout = Duration::from_secs(env_parse("TEST_TIMEOUT_SECS", 15u64)?);
        let lsp_idle_timeout = Duration::from_secs(env_parse("LSP_IDLE_TIMEOUT_SECS", 1800u64)?);
        let lsp_max_sessions: usize = env_parse("LSP_MAX_SESSIONS", 3usize)?.max(1);
        let rust_analyzer = path_or("RUST_ANALYZER", PathBuf::from("rust-analyzer"));
        Ok(Config {
            host,
            port,
            root,
            web_dist,
            workspace_dir,
            data_dir,
            compile_timeout,
            test_timeout,
            rust_analyzer,
            lsp_idle_timeout,
            lsp_max_sessions,
        })
    }
}
