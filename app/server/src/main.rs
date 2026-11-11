//! Backend for the Turkish interactive edition of "100 Exercises to Learn Rust".
//! See README.md ("Mimari") and app/API.md.

mod api;
mod config;
mod content;
mod db;
mod error;
mod hidden;
mod lsp;
mod proc;
mod runner;
mod workspace;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

fn absolute(p: PathBuf) -> PathBuf {
    if p.is_absolute() {
        p
    } else {
        std::env::current_dir().map(|c| c.join(&p)).unwrap_or(p)
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    let term = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    tokio::select! {
        _ = ctrl_c => {},
        _ = term => {},
    }
    tracing::info!("shutting down");
    proc::kill_all();
    // Open WebSockets would otherwise keep graceful shutdown waiting.
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_secs(3)).await;
        proc::kill_all();
        std::process::exit(0);
    });
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("server=info,tower_http=warn")),
        )
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stdout()))
        .init();

    let mut cfg = match config::Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("configuration error: {e}");
            std::process::exit(2);
        }
    };
    cfg.workspace_dir = absolute(cfg.workspace_dir);
    cfg.data_dir = absolute(cfg.data_dir);
    cfg.web_dist = absolute(cfg.web_dist);

    if let Err(e) = std::fs::create_dir_all(&cfg.workspace_dir)
        .and_then(|_| std::fs::create_dir_all(&cfg.data_dir))
    {
        tracing::error!("cannot create workspace/data directories: {e}");
        std::process::exit(2);
    }
    // Canonical paths keep LSP URIs and rustc's absolute paths consistent.
    cfg.workspace_dir = cfg
        .workspace_dir
        .canonicalize()
        .unwrap_or(cfg.workspace_dir);
    cfg.data_dir = cfg.data_dir.canonicalize().unwrap_or(cfg.data_dir);

    let content = content::Content::new(cfg.root.clone());
    let ws = Arc::new(workspace::Workspace::new(
        cfg.workspace_dir.clone(),
        content.clone(),
    ));
    if let Err(e) = ws.init() {
        tracing::error!(
            "cannot initialise workspace {}: {e}",
            cfg.workspace_dir.display()
        );
        std::process::exit(2);
    }
    let db_path = cfg.data_dir.join("progress.db");
    let db = match db::Db::open(&db_path) {
        Ok(db) => db,
        Err(e) => {
            tracing::error!("cannot open database {}: {e}", db_path.display());
            std::process::exit(2);
        }
    };

    tracing::info!("repository root: {}", cfg.root.display());
    tracing::info!("workspace: {}", cfg.workspace_dir.display());
    tracing::info!("database: {}", db_path.display());
    if cfg.web_dist.is_dir() {
        tracing::info!("serving frontend from {}", cfg.web_dist.display());
    } else {
        tracing::warn!(
            "frontend not built: {} does not exist",
            cfg.web_dist.display()
        );
    }

    let addr = SocketAddr::new(cfg.host, cfg.port);
    let state = api::AppState {
        runner: Arc::new(runner::CargoRunner {
            compile_timeout: cfg.compile_timeout,
            test_timeout: cfg.test_timeout,
        }),
        lsp: lsp::LspSessions::new(cfg.lsp_max_sessions),
        cfg: Arc::new(cfg),
        content,
        workspace: ws,
        db,
        job_lock: Arc::new(tokio::sync::Mutex::new(())),
    };
    let app = api::router(state).layer(TraceLayer::new_for_http());

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("cannot bind {addr}: {e}");
            std::process::exit(2);
        }
    };
    tracing::info!("listening on http://{addr}");
    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        tracing::error!("server error: {e}");
    }
    proc::kill_all();
}
