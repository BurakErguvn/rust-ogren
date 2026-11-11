//! HTTP handlers (contract: app/API.md).

use std::sync::Arc;

use axum::body::{Body, Bytes};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::http::{header, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

use crate::config::Config;
use crate::content::{is_valid_segment, ChapterInfo, Content};
use crate::db::{Db, Progress, SubmitRecord};
use crate::error::{ApiError, ApiResult};
use crate::lsp::{self, LspSessions};
use crate::runner::{job_for, RunOutcome, Runner, Status};
use crate::workspace::{file_uri, parse_edit_path, Workspace, WriteError, MAX_FILE_BYTES};

#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<Config>,
    pub content: Content,
    pub workspace: Arc<Workspace>,
    pub db: Db,
    pub runner: Arc<dyn Runner>,
    /// One compile/test job at a time.
    pub job_lock: Arc<tokio::sync::Mutex<()>>,
    pub lsp: Arc<LspSessions>,
}

pub fn router(state: AppState) -> Router {
    let book_assets = ServeDir::new(state.content.root.join("book/src"));
    let api = Router::new()
        .route("/toc", get(toc))
        .route("/pages/{*page_id}", get(page))
        .route("/progress", get(progress))
        .route("/exercises/{chapter}/{exercise}/files", get(files))
        .route(
            "/exercises/{chapter}/{exercise}/files/{*path}",
            put(save_file).delete(delete_file),
        )
        .route("/exercises/{chapter}/{exercise}/test", post(run_test))
        .route("/exercises/{chapter}/{exercise}/submit", post(run_submit))
        .route("/exercises/{chapter}/{exercise}/reset", post(reset))
        .route("/lsp/{chapter}/{exercise}", get(lsp_ws))
        .nest_service("/book-assets", book_assets)
        .fallback(api_not_found);
    Router::new()
        .nest("/api", api)
        .fallback(static_files)
        .with_state(state)
}

async fn blocking<T, F>(f: F) -> ApiResult<T>
where
    T: Send + 'static,
    F: FnOnce() -> ApiResult<T> + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| ApiError::internal("blocking task panicked", e))?
}

async fn api_not_found() -> ApiError {
    ApiError::not_found("Böyle bir API ucu yok.")
}

// ------------------------------------------------------------------------------------
// Content
// ------------------------------------------------------------------------------------

fn exercise_json(has_exercise: bool, p: Option<&Progress>) -> Value {
    if has_exercise {
        json!(p.copied().unwrap_or_default())
    } else {
        Value::Null
    }
}

fn load_toc(
    state: &AppState,
) -> ApiResult<(
    Vec<ChapterInfo>,
    std::collections::HashMap<String, Progress>,
)> {
    let toc = state
        .content
        .toc()
        .map_err(|e| ApiError::internal("reading SUMMARY.md", e))?;
    let progress = state
        .db
        .all()
        .map_err(|e| ApiError::internal("reading progress", e))?;
    Ok((toc, progress))
}

#[derive(Serialize, Default, Clone, Copy)]
struct Totals {
    completed: usize,
    total: usize,
    score_sum: i64,
}

fn chapter_totals(
    ch: &ChapterInfo,
    progress: &std::collections::HashMap<String, Progress>,
) -> Totals {
    let mut t = Totals::default();
    for p in ch.pages.iter().filter(|p| p.has_exercise) {
        t.total += 1;
        if let Some(pr) = progress.get(&p.id).filter(|pr| pr.completed) {
            t.completed += 1;
            t.score_sum += pr.score.unwrap_or(0);
        }
    }
    t
}

fn sum_totals(it: impl Iterator<Item = Totals>) -> Totals {
    it.fold(Totals::default(), |a, b| Totals {
        completed: a.completed + b.completed,
        total: a.total + b.total,
        score_sum: a.score_sum + b.score_sum,
    })
}

async fn toc(State(state): State<AppState>) -> ApiResult<Json<Value>> {
    blocking(move || {
        let (toc, progress) = load_toc(&state)?;
        let chapters: Vec<Value> = toc
            .iter()
            .map(|ch| {
                let pages: Vec<Value> = ch
                    .pages
                    .iter()
                    .map(|p| {
                        json!({
                            "id": p.id,
                            "title": p.title,
                            "translated": p.translated,
                            "exercise": exercise_json(p.has_exercise, progress.get(&p.id)),
                        })
                    })
                    .collect();
                json!({ "id": ch.id, "title": ch.title, "pages": pages })
            })
            .collect();
        let totals = sum_totals(toc.iter().map(|c| chapter_totals(c, &progress)));
        Ok(Json(json!({ "chapters": chapters, "progress": totals })))
    })
    .await
}

async fn progress(State(state): State<AppState>) -> ApiResult<Json<Value>> {
    blocking(move || {
        let (toc, progress) = load_toc(&state)?;
        let per: Vec<(String, Totals)> = toc
            .iter()
            .map(|c| (c.id.clone(), chapter_totals(c, &progress)))
            .filter(|(_, t)| t.total > 0)
            .collect();
        let totals = sum_totals(per.iter().map(|(_, t)| *t));
        let chapters: Vec<Value> = per
            .iter()
            .map(|(id, t)| json!({ "id": id, "completed": t.completed, "total": t.total, "score_sum": t.score_sum }))
            .collect();
        Ok(Json(json!({
            "completed": totals.completed,
            "total": totals.total,
            "score_sum": totals.score_sum,
            "chapters": chapters,
        })))
    })
    .await
}

async fn page(
    State(state): State<AppState>,
    Path(page_id): Path<String>,
) -> ApiResult<Json<Value>> {
    let page_id = page_id.trim_end_matches('/').to_string();
    let segs: Vec<&str> = page_id.split('/').collect();
    if segs.is_empty() || segs.len() > 2 || !segs.iter().all(|s| is_valid_segment(s)) {
        return Err(ApiError::bad_request("Geçersiz sayfa kimliği."));
    }
    blocking(move || {
        let (toc, progress) = load_toc(&state)?;
        let flat: Vec<_> = toc.iter().flat_map(|c| c.pages.iter()).collect();
        let idx = flat
            .iter()
            .position(|p| p.id == page_id)
            .ok_or_else(|| ApiError::not_found("Sayfa bulunamadı."))?;
        let p = flat[idx];
        let (markdown, translated) = state
            .content
            .page_markdown(&p.id)
            .map_err(|e| ApiError::internal("reading page markdown", e))?;
        let notes = state.content.page_notes(&p.id);
        Ok(Json(json!({
            "id": p.id,
            "title": p.title,
            "translated": translated,
            "markdown": markdown,
            "prev": idx.checked_sub(1).map(|i| flat[i].id.clone()),
            "next": flat.get(idx + 1).map(|n| n.id.clone()),
            "exercise": exercise_json(p.has_exercise, progress.get(&p.id)),
            "notes": notes,
        })))
    })
    .await
}

// ------------------------------------------------------------------------------------
// Exercises
// ------------------------------------------------------------------------------------

fn check_exercise(state: &AppState, chapter: &str, exercise: &str) -> ApiResult<String> {
    if !state.content.exercise_exists(chapter, exercise) {
        return Err(ApiError::not_found("Egzersiz bulunamadı."));
    }
    Ok(format!("{chapter}/{exercise}"))
}

fn ensure(state: &AppState, chapter: &str, exercise: &str) -> ApiResult<()> {
    state
        .workspace
        .ensure_exercise(chapter, exercise)
        .map_err(|e| ApiError::internal("creating working copy", e))
}

fn files_response(
    state: &AppState,
    chapter: &str,
    exercise: &str,
    id: &str,
) -> ApiResult<Json<Value>> {
    let listing = state
        .workspace
        .list_files(chapter, exercise)
        .map_err(|e| ApiError::internal("listing exercise files", e))?;
    let progress = state
        .db
        .get(id)
        .map_err(|e| ApiError::internal("reading progress", e))?;
    let ws_root = &state.workspace.dir;
    Ok(Json(json!({
        "id": id,
        "workspace_root_uri": file_uri(ws_root),
        "exercise_root_uri": file_uri(&state.workspace.exercise_dir(chapter, exercise)),
        "files": listing
            .files
            .iter()
            .map(|f| json!({
                "path": f.path,
                "content": f.content,
                "language": f.language,
                "editable": true,
                "has_hidden_tests": f.has_hidden_tests,
                "deletable": f.deletable,
            }))
            .collect::<Vec<_>>(),
        "hidden_test_files": listing.hidden_test_files,
        "progress": progress,
    })))
}

async fn files(
    State(state): State<AppState>,
    Path((chapter, exercise)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    let id = check_exercise(&state, &chapter, &exercise)?;
    blocking(move || {
        ensure(&state, &chapter, &exercise)?;
        if let Err(e) = state.workspace.refresh_if_pristine(&chapter, &exercise) {
            tracing::warn!("refresh check for {id} failed: {e}");
        }
        files_response(&state, &chapter, &exercise, &id)
    })
    .await
}

async fn reset(
    State(state): State<AppState>,
    Path((chapter, exercise)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    let id = check_exercise(&state, &chapter, &exercise)?;
    blocking(move || {
        state
            .workspace
            .reset_exercise(&chapter, &exercise)
            .map_err(|e| ApiError::internal("resetting working copy", e))?;
        files_response(&state, &chapter, &exercise, &id)
    })
    .await
}

#[derive(Deserialize)]
struct SaveBody {
    content: String,
}

async fn save_file(
    State(state): State<AppState>,
    Path((chapter, exercise, path)): Path<(String, String, String)>,
    body: Bytes,
) -> ApiResult<StatusCode> {
    check_exercise(&state, &chapter, &exercise)?;
    let rel = parse_edit_path(&path).map_err(|e| ApiError::bad_request(e.to_string()))?;
    let parsed: SaveBody = serde_json::from_slice(&body).map_err(|_| {
        ApiError::bad_request("Geçersiz istek gövdesi: `{ \"content\": \"...\" }` bekleniyor.")
    })?;
    if parsed.content.len() > MAX_FILE_BYTES {
        return Err(ApiError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "Dosya çok büyük (en fazla 512 KB).",
        ));
    }
    blocking(move || {
        ensure(&state, &chapter, &exercise)?;
        state
            .workspace
            .write_file(&chapter, &exercise, &rel, &parsed.content)
            .map(|()| StatusCode::NO_CONTENT)
            .map_err(|e| write_error(e, "writing exercise file"))
    })
    .await
}

fn write_error(e: WriteError, context: &str) -> ApiError {
    match e {
        WriteError::Outside => {
            ApiError::bad_request("Bu dosya yolu egzersiz klasörünün dışına çıkıyor.")
        }
        WriteError::Forbidden(m) => ApiError::new(StatusCode::FORBIDDEN, m),
        WriteError::Invalid(m) => ApiError::bad_request(m),
        WriteError::NotFound => ApiError::not_found("Dosya bulunamadı."),
        WriteError::Io(e) => ApiError::internal(context, e),
    }
}

async fn delete_file(
    State(state): State<AppState>,
    Path((chapter, exercise, path)): Path<(String, String, String)>,
) -> ApiResult<StatusCode> {
    check_exercise(&state, &chapter, &exercise)?;
    let rel = parse_edit_path(&path).map_err(|e| ApiError::bad_request(e.to_string()))?;
    blocking(move || {
        ensure(&state, &chapter, &exercise)?;
        state
            .workspace
            .delete_file(&chapter, &exercise, &rel)
            .map(|()| StatusCode::NO_CONTENT)
            .map_err(|e| write_error(e, "deleting exercise file"))
    })
    .await
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Test,
    Submit,
}

#[derive(Serialize)]
struct RunResponse {
    kind: &'static str,
    #[serde(flatten)]
    outcome: RunOutcome,
    progress: Progress,
}

async fn run_test(
    state: State<AppState>,
    path: Path<(String, String)>,
) -> ApiResult<Json<RunResponse>> {
    run(state, path, Kind::Test).await
}

async fn run_submit(
    state: State<AppState>,
    path: Path<(String, String)>,
) -> ApiResult<Json<RunResponse>> {
    run(state, path, Kind::Submit).await
}

async fn run(
    State(state): State<AppState>,
    Path((chapter, exercise)): Path<(String, String)>,
    kind: Kind,
) -> ApiResult<Json<RunResponse>> {
    let started = std::time::Instant::now();
    let id = check_exercise(&state, &chapter, &exercise)?;
    let Ok(_permit) = state.job_lock.clone().try_lock_owned() else {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "Başka bir test çalışıyor, lütfen bekle.",
        ));
    };
    let job = {
        let state = state.clone();
        let (chapter, exercise) = (chapter.clone(), exercise.clone());
        blocking(move || {
            ensure(&state, &chapter, &exercise)?;
            let ws = &state.workspace;
            let package = ws
                .package_name(&chapter, &exercise)
                .map_err(|e| ApiError::internal("reading exercise Cargo.toml", e))?;
            Ok(job_for(
                &ws.dir,
                ws.exercise_dir(&chapter, &exercise),
                Workspace::exercise_rel_prefix(&chapter, &exercise),
                package,
                ws.original_has_tests(&chapter, &exercise),
                ws.hidden_lines(&chapter, &exercise),
            ))
        })
        .await?
    };
    // If the client disconnects, this future is dropped and the process-group guards
    // inside the runner kill cargo / the test binary.
    let mut outcome = state.runner.run(&job).await;
    // Wall-clock time of the whole request: preparation, compilation and tests.
    outcome.duration_ms = started.elapsed().as_millis() as u64;
    tracing::info!(
        "{} {id}: {} ({}/{}) in {} ms",
        if kind == Kind::Submit {
            "submit"
        } else {
            "test"
        },
        outcome.status.as_str(),
        outcome.tests_passed,
        outcome.tests_total,
        outcome.duration_ms
    );
    let db = state.db.clone();
    let progress = {
        let id = id.clone();
        let rec = (
            outcome.status == Status::Passed,
            outcome.status.as_str(),
            outcome.tests_passed,
            outcome.tests_total,
        );
        blocking(move || {
            let r = if kind == Kind::Submit {
                db.record_submit(&SubmitRecord {
                    exercise_id: &id,
                    passed: rec.0,
                    status: rec.1,
                    tests_passed: rec.2,
                    tests_total: rec.3,
                })
            } else {
                db.get(&id)
            };
            r.map_err(|e| ApiError::internal("updating progress", e))
        })
        .await?
    };
    Ok(Json(RunResponse {
        kind: if kind == Kind::Submit {
            "submit"
        } else {
            "test"
        },
        outcome,
        progress,
    }))
}

// ------------------------------------------------------------------------------------
// LSP
// ------------------------------------------------------------------------------------

async fn lsp_ws(
    State(state): State<AppState>,
    Path((chapter, exercise)): Path<(String, String)>,
    ws: WebSocketUpgrade,
) -> ApiResult<Response> {
    let id = check_exercise(&state, &chapter, &exercise)?;
    {
        let state = state.clone();
        blocking(move || ensure(&state, &chapter, &exercise)).await?;
    }
    let (child, guard) = lsp::spawn_rust_analyzer(&state.cfg.rust_analyzer, &state.workspace.dir)
        .map_err(|e| {
            tracing::error!(
                "failed to start rust-analyzer ({}): {e}",
                state.cfg.rust_analyzer.display()
            );
            ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "rust-analyzer başlatılamadı. `rustup component add rust-analyzer` ile kurabilir veya RUST_ANALYZER ile yolunu belirtebilirsin.",
            )
        })?;
    let ticket = state.lsp.register();
    let idle = state.cfg.lsp_idle_timeout;
    Ok(ws
        .max_message_size(64 * 1024 * 1024)
        .on_upgrade(move |socket| lsp::bridge(socket, child, guard, ticket, idle, id)))
}

// ------------------------------------------------------------------------------------
// Static frontend
// ------------------------------------------------------------------------------------

async fn static_files(State(state): State<AppState>, req: Request<Body>) -> Response {
    let dist = &state.cfg.web_dist;
    if !dist.is_dir() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            format!(
                "Arayüz henüz derlenmemiş: {} bulunamadı.\n\
                 `cd app/web && npm ci && npm run build` ile derleyebilirsin. API /api altında çalışıyor.\n",
                dist.display()
            ),
        )
            .into_response();
    }
    let svc = ServeDir::new(dist)
        .append_index_html_on_directories(true)
        .not_found_service(ServeFile::new(dist.join("404.html")));
    match svc.oneshot(req).await {
        Ok(r) => r.into_response(),
        Err(e) => ApiError::internal("serving static file", e).into_response(),
    }
}
