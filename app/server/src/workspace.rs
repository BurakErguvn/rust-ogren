//! The user's working copy (`app/workspace/`; layout described in README "Mimari").
//!
//! ```text
//! app/workspace/
//!   Cargo.toml   [workspace] members = ["exercises/*/*", "helpers/*"] + root [profile.*]
//!   Cargo.lock   copied once from the repository root
//!   helpers/     synced from the repository root
//!   exercises/<chapter>/<exercise>/   original crate, .rs files overlaid from tr/exercises,
//!                top-level `#[cfg(test)]` items moved to the end of each file (hidden.rs)
//!   .state/      per-exercise snapshot (`<ex>.json`) and hidden test code (`<ex>.hidden.json`)
//!   target/      shared by the test runner
//!   target-ra/   rust-analyzer's own `cargo check` output
//! ```

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::hidden::HiddenLines;

/// Max size of a single source file accepted from the client.
pub const MAX_FILE_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    Empty,
    Absolute,
    Traversal,
    InvalidChar,
    NotEditableDir,
    NotRustFile,
    TooLong,
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            PathError::Empty => "Dosya yolu boş olamaz.",
            PathError::Absolute => "Mutlak dosya yolları kabul edilmez.",
            PathError::Traversal => "Dosya yolunda `..`, `.` veya boş parça olamaz.",
            PathError::InvalidChar => "Dosya yolunda geçersiz karakter var.",
            PathError::NotEditableDir => {
                "Yalnızca `src/` ve `tests/` altındaki dosyalar düzenlenebilir."
            }
            PathError::NotRustFile => "Yalnızca `.rs` uzantılı dosyalar kaydedilebilir.",
            PathError::TooLong => "Dosya yolu çok uzun.",
        };
        f.write_str(msg)
    }
}

/// Validates an untrusted, `/`-separated path relative to the exercise root.
/// Accepts only `src/**/*.rs` and `tests/**/*.rs`.
pub fn validate_rel_path(raw: &str) -> Result<PathBuf, PathError> {
    if raw.is_empty() {
        return Err(PathError::Empty);
    }
    if raw.len() > 255 {
        return Err(PathError::TooLong);
    }
    if raw.starts_with('/') || raw.starts_with('~') {
        return Err(PathError::Absolute);
    }
    if raw
        .chars()
        .any(|c| c == '\\' || c == ':' || c == '\0' || c.is_control())
    {
        return Err(PathError::InvalidChar);
    }
    let parts: Vec<&str> = raw.split('/').collect();
    if parts
        .iter()
        .any(|p| p.is_empty() || *p == "." || *p == "..")
    {
        return Err(PathError::Traversal);
    }
    if parts.len() < 2 || !(parts[0] == "src" || parts[0] == "tests") {
        return Err(PathError::NotEditableDir);
    }
    let last = parts[parts.len() - 1];
    if !last.ends_with(".rs") || last.len() <= 3 {
        return Err(PathError::NotRustFile);
    }
    let p = PathBuf::from(raw);
    // Belt and braces: the joined path must consist of normal components only.
    if !p.components().all(|c| matches!(c, Component::Normal(_))) {
        return Err(PathError::Traversal);
    }
    Ok(p)
}

/// `file://` URI for an absolute path, percent-encoding everything but unreserved chars and `/`.
pub fn file_uri(path: &Path) -> String {
    let mut s = String::from("file://");
    for b in path.to_string_lossy().bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~' | b'/') {
            s.push(b as char);
        } else {
            s.push_str(&format!("%{b:02X}"));
        }
    }
    s
}

/// Orders files: `src/lib.rs`, `src/main.rs`, then alphabetical.
pub fn sort_files(paths: &mut [String]) {
    fn rank(p: &str) -> u8 {
        match p {
            "src/lib.rs" => 0,
            "src/main.rs" => 1,
            _ => 2,
        }
    }
    paths.sort_by(|a, b| rank(a).cmp(&rank(b)).then_with(|| a.cmp(b)));
}

/// Whether Rust sources contain any test function.
pub fn sources_have_tests(sources: &[String]) -> bool {
    sources
        .iter()
        .any(|s| s.contains("#[test]") || s.contains("#[tokio::test"))
}

/// Exercises whose test code stays visible although no marker was found automatically
/// (the instructions send the learner to the tests, or the required API exists only there).
/// Exercises with `todo!()`, `unimplemented!()`, `TODO` or `__` in their test code are
/// detected automatically by [`plan_hiding`].
pub const ALWAYS_VISIBLE: &[(&str, &str)] = &[
    (
        "01_intro/00_welcome",
        "instructions explain the tests; the expected string is only in the test",
    ),
    (
        "03_ticket_v1/02_validation",
        "\"You can find the needed panic messages in the tests\"",
    ),
    (
        "04_traits/10_assoc_vs_generic",
        "the required `Power` signatures exist only in the tests",
    ),
    (
        "04_traits/12_copy",
        "\"implement the necessary traits to make the test compile\"",
    ),
    (
        "04_traits/13_drop",
        "\"You can see the expected API in the tests below\"",
    ),
    (
        "07_threads/07_ack",
        "\"Refer to the tests to understand the expected schema\"",
    ),
    (
        "08_futures/06_async_aware_primitives",
        "\"you'll have to touch the testing code too\"",
    ),
    (
        "08_futures/07_cancellation",
        "\"fix the `assert_eq` at the end of the tests\"",
    ),
];

/// Path of the workspace root manifest as seen by the client.
pub const WORKSPACE_MANIFEST_PATH: &str = "@workspace/Cargo.toml";
/// The exercise's own manifest as seen by the client.
pub const OWN_MANIFEST_PATH: &str = "Cargo.toml";

/// A client-supplied file path, validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditPath {
    /// `src/**.rs` or `tests/**.rs`; the string is the normalized `/`-separated path.
    Source(String),
    /// The exercise's `Cargo.toml`.
    OwnManifest,
    /// The workspace root `Cargo.toml` (`@workspace/Cargo.toml`).
    WorkspaceManifest,
}

pub fn parse_edit_path(raw: &str) -> Result<EditPath, PathError> {
    match raw {
        OWN_MANIFEST_PATH => Ok(EditPath::OwnManifest),
        WORKSPACE_MANIFEST_PATH => Ok(EditPath::WorkspaceManifest),
        _ => {
            validate_rel_path(raw)?;
            Ok(EditPath::Source(raw.to_string()))
        }
    }
}

/// Which manifests an exercise asks the learner to edit.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CargoEdit {
    pub own: bool,
    pub root: bool,
}

/// Detects manifest-editing instructions in the comments of the original (English)
/// sources: `Cargo.toml` or adding a dependency -> own manifest; "root of the repository"
/// -> also the workspace root manifest.
pub fn detect_cargo_edit(sources: &[String]) -> CargoEdit {
    let comments: String = sources
        .iter()
        .map(|s| crate::hidden::comment_text(s))
        .collect::<Vec<_>>()
        .join("\n");
    let lower = comments.to_lowercase();
    let own = comments.contains("Cargo.toml") || lower.contains("dependenc");
    let root = comments.contains("Cargo.toml") && lower.contains("root of the repository");
    CargoEdit { own, root }
}

/// Per-exercise record of the hidden test code (`.state/<chapter>/<exercise>.hidden.json`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HiddenState {
    /// False if this exercise is an exception and shows its tests.
    pub hide: bool,
    /// Why tests are visible (exceptions only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exception: Option<String>,
    /// Exercise-relative path -> hidden blocks appended to that file on disk.
    #[serde(default)]
    pub files: BTreeMap<String, String>,
}

/// Result of [`plan_hiding`]: the new hidden state plus files to rewrite on disk.
#[derive(Debug, Default)]
pub struct HidingPlan {
    pub state: HiddenState,
    /// (path, composed on-disk content) for files whose layout changes.
    pub rewrites: Vec<(String, String)>,
    pub notes: Vec<String>,
}

/// Decides what to hide for one exercise. `files` are its `src/**.rs` and `tests/**.rs`
/// files (path, content).
pub fn plan_hiding(id: &str, files: &[(String, String)]) -> HidingPlan {
    let mut plan = HidingPlan::default();
    let mut extracted = Vec::new();
    let mut exception: Option<String> = ALWAYS_VISIBLE
        .iter()
        .find(|(x, _)| *x == id)
        .map(|(_, why)| why.to_string());
    for (path, content) in files {
        if path.starts_with("tests/") {
            if exception.is_none() {
                if let Some(m) = crate::hidden::learner_edit_marker(content) {
                    exception = Some(format!("{path}: {m}"));
                }
            }
            continue;
        }
        let e = crate::hidden::extract(content);
        for n in &e.notes {
            plan.notes.push(format!("{path}: {n}"));
        }
        if e.blocks > 0 {
            if exception.is_none() {
                if let Some(m) = crate::hidden::learner_edit_marker(&e.hidden) {
                    exception = Some(format!("{path}: {m}"));
                }
            }
            extracted.push((path.clone(), content.clone(), e));
        }
    }
    if let Some(why) = exception {
        plan.state = HiddenState {
            hide: false,
            exception: Some(why),
            files: BTreeMap::new(),
        };
        return plan;
    }
    plan.state.hide = true;
    for (path, original, e) in extracted {
        let disk = crate::hidden::compose(&e.visible, &e.hidden);
        if disk != original {
            plan.rewrites.push((path.clone(), disk));
        }
        plan.state.files.insert(path, e.hidden);
    }
    plan
}

/// Reconciles a learner-edited workspace manifest with the generated default: only
/// `[workspace]` (backend-owned) and `[profile.*]` (learner-editable) are allowed.
/// Returns the text to write: the learner's text verbatim if its `[workspace]` table is
/// untouched, otherwise the default with the learner's profiles.
pub fn reconcile_workspace_manifest(learner: &str, default: &str) -> Result<String, String> {
    let l: toml::Table = learner
        .parse()
        .map_err(|e: toml::de::Error| toml_error_message(learner, &e))?;
    if let Some(k) = l.keys().find(|k| *k != "workspace" && *k != "profile") {
        return Err(format!(
            "Kök Cargo.toml'da yalnızca `[profile.*]` bölümleri değiştirilebilir (`{k}` eklenemez)."
        ));
    }
    if l.get("profile").is_some_and(|p| !p.is_table()) {
        return Err("`profile` bir tablo olmalı.".into());
    }
    let d: toml::Table = default.parse().map_err(|e| format!("{e}"))?;
    if l.get("workspace") == d.get("workspace") {
        return Ok(learner.to_string());
    }
    render_manifest(d.get("workspace"), l.get("profile"))
}

/// `[workspace]` first, then `[profile.*]` (a single table would be key-sorted).
fn render_manifest(
    workspace: Option<&toml::Value>,
    profile: Option<&toml::Value>,
) -> Result<String, String> {
    let mut out = String::from(MANIFEST_HEADER);
    for (key, value) in [("workspace", workspace), ("profile", profile)] {
        if let Some(v) = value {
            let mut doc = toml::Table::new();
            doc.insert(key.into(), v.clone());
            if !out.ends_with(MANIFEST_HEADER) {
                out.push('\n');
            }
            out.push_str(&toml::to_string(&doc).map_err(|e| format!("{e}"))?);
        }
    }
    Ok(out)
}

const MANIFEST_HEADER: &str = "# Generated by app/server from the repository root Cargo.toml.\n\
# Only the [profile.*] sections may be edited; [workspace] is managed by the app.\n";

/// `Cargo.toml ayrıştırılamadı (satır L, sütun C): msg` with a 1-based position taken
/// from the error span (column counted in characters). The frontend places an editor
/// marker from the `satır N` part.
fn toml_error_message(text: &str, e: &toml::de::Error) -> String {
    match e.span() {
        Some(span) => {
            let (line, col) = line_col(text, span.start);
            format!(
                "Cargo.toml ayrıştırılamadı (satır {line}, sütun {col}): {}",
                e.message()
            )
        }
        None => format!("Cargo.toml ayrıştırılamadı: {}", e.message()),
    }
}

/// 1-based (line, column) of a byte offset; the offset is clamped to the text and to a
/// char boundary.
fn line_col(text: &str, offset: usize) -> (usize, usize) {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    let before = &text[..offset];
    let line = before.matches('\n').count() + 1;
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    let col = before[line_start..].chars().count() + 1;
    (line, col)
}

fn validate_own_manifest(text: &str) -> Result<(), String> {
    let t: toml::Table = text
        .parse()
        .map_err(|e: toml::de::Error| toml_error_message(text, &e))?;
    if t.get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .is_none()
    {
        return Err("Cargo.toml'da `[package]` bölümü ve `name` alanı bulunmalı.".into());
    }
    Ok(())
}

pub struct Workspace {
    pub dir: PathBuf,
    content: Content,
    lock: Mutex<()>,
}

#[derive(Debug, Clone)]
pub struct ExerciseFile {
    pub path: String,
    pub content: String,
    pub language: &'static str,
    pub has_hidden_tests: bool,
    /// Learner-created `src/**.rs` file (accepted by [`Workspace::delete_file`]).
    pub deletable: bool,
}

#[derive(Debug, Clone)]
pub struct Listing {
    pub files: Vec<ExerciseFile>,
    pub hidden_test_files: usize,
}

#[derive(Debug)]
pub enum WriteError {
    Outside,
    /// 403 with a Turkish message.
    Forbidden(&'static str),
    /// 400 with a Turkish message.
    Invalid(String),
    NotFound,
    Io(std::io::Error),
}

impl From<std::io::Error> for WriteError {
    fn from(e: std::io::Error) -> Self {
        WriteError::Io(e)
    }
}

impl Workspace {
    /// `dir` must be absolute.
    pub fn new(dir: PathBuf, content: Content) -> Self {
        Workspace {
            dir,
            content,
            lock: Mutex::new(()),
        }
    }

    pub fn exercise_dir(&self, chapter: &str, exercise: &str) -> PathBuf {
        self.dir.join("exercises").join(chapter).join(exercise)
    }

    /// Path prefix as seen by rustc (relative to the workspace root).
    pub fn exercise_rel_prefix(chapter: &str, exercise: &str) -> String {
        format!("exercises/{chapter}/{exercise}/")
    }

    fn manifest_edited_marker(&self) -> PathBuf {
        self.dir.join(".state/workspace-manifest-edited")
    }

    fn default_manifest(&self) -> std::io::Result<String> {
        let root = std::fs::read_to_string(self.content.root.join("Cargo.toml"))?;
        workspace_manifest(&root).map_err(std::io::Error::other)
    }

    /// Writes the workspace manifest: the generated default, or — once the learner edited
    /// it — their file with the backend-owned `[workspace]` table enforced.
    fn sync_manifest(&self) -> std::io::Result<()> {
        let default = self.default_manifest()?;
        let path = self.dir.join("Cargo.toml");
        if self.manifest_edited_marker().is_file() {
            if let Ok(current) = std::fs::read_to_string(&path) {
                match reconcile_workspace_manifest(&current, &default) {
                    Ok(text) => return write_if_changed(&path, text.as_bytes()),
                    Err(e) => tracing::warn!("workspace Cargo.toml reset to default: {e}"),
                }
            }
            let _ = std::fs::remove_file(self.manifest_edited_marker());
        }
        write_if_changed(&path, default.as_bytes())
    }

    /// Creates/refreshes the workspace skeleton. Idempotent; unchanged files are not
    /// rewritten so cargo fingerprints stay valid.
    pub fn init(&self) -> std::io::Result<()> {
        let _g = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        std::fs::create_dir_all(self.dir.join("exercises"))?;
        std::fs::create_dir_all(self.dir.join(".tmp"))?;
        std::fs::create_dir_all(self.dir.join(".state"))?;
        let root = &self.content.root;

        self.sync_manifest()?;

        let lock = self.dir.join("Cargo.lock");
        if !lock.exists() {
            std::fs::copy(root.join("Cargo.lock"), &lock)?;
        }

        // Only crate directories: a non-crate entry under helpers/ would break the glob.
        for entry in std::fs::read_dir(root.join("helpers"))? {
            let entry = entry?;
            let src = entry.path();
            if src.is_dir() && src.join("Cargo.toml").is_file() {
                let dst = self.dir.join("helpers").join(entry.file_name());
                sync_dir(&src, &dst)?;
            }
        }
        Ok(())
    }

    pub fn exercise_present(&self, chapter: &str, exercise: &str) -> bool {
        self.exercise_dir(chapter, exercise)
            .join("Cargo.toml")
            .is_file()
    }

    /// Creates the exercise copy if missing; migrates copies created before test code
    /// was hidden.
    pub fn ensure_exercise(&self, chapter: &str, exercise: &str) -> std::io::Result<()> {
        if self.exercise_present(chapter, exercise) {
            if self.hidden_state_path(chapter, exercise).is_file() {
                return Ok(());
            }
            let _g = self.lock.lock().unwrap_or_else(|e| e.into_inner());
            return self.migrate(chapter, exercise);
        }
        self.init()?;
        let _g = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        if self.exercise_present(chapter, exercise) {
            return Ok(());
        }
        let (staged, hidden) = self.stage_exercise(chapter, exercise)?;
        self.install(chapter, exercise, &staged, &hidden)?;
        tracing::info!("created working copy for {chapter}/{exercise}");
        Ok(())
    }

    /// Old working copy without hidden-test state: an untouched copy is recreated, an
    /// edited one is converted in place (tests moved to the end of each file).
    /// Caller holds `self.lock`.
    fn migrate(&self, chapter: &str, exercise: &str) -> std::io::Result<()> {
        if self.hidden_state_path(chapter, exercise).is_file() {
            return Ok(());
        }
        let dir = self.exercise_dir(chapter, exercise);
        let untouched = match self.read_snapshot(chapter, exercise) {
            Some(recorded) => {
                let mut current = snapshot(&dir)?;
                if !recorded.contains_key(OWN_MANIFEST_PATH) {
                    current.remove(OWN_MANIFEST_PATH);
                }
                current == recorded
            }
            None => false,
        };
        if untouched {
            let (staged, hidden) = self.stage_exercise(chapter, exercise)?;
            self.install(chapter, exercise, &staged, &hidden)?;
            tracing::info!(
                "recreated untouched working copy for {chapter}/{exercise} (hidden tests)"
            );
        } else {
            let id = format!("{chapter}/{exercise}");
            let state = apply_hiding(&id, &dir)?;
            self.write_hidden_state(chapter, exercise, &state)?;
            tracing::info!("converted edited working copy for {id} (hidden tests)");
        }
        Ok(())
    }

    /// Recreates the exercise copy from the original + Turkish overlay. Learner-created
    /// files disappear; for exercises that edit the workspace manifest, it is reset too.
    pub fn reset_exercise(&self, chapter: &str, exercise: &str) -> std::io::Result<()> {
        if self.cargo_edit(chapter, exercise).root {
            let _ = std::fs::remove_file(self.manifest_edited_marker());
        }
        self.init()?;
        let _g = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let (staged, hidden) = self.stage_exercise(chapter, exercise)?;
        self.install(chapter, exercise, &staged, &hidden)?;
        tracing::info!("reset working copy for {chapter}/{exercise}");
        Ok(())
    }

    /// If the user never changed the working copy (its editable files still match the
    /// snapshot taken when it was created) but the sources changed since — typically a
    /// Turkish translation that landed later — recreate it. Edited copies (including ones
    /// with learner-created files) are left alone. Returns whether the copy was refreshed.
    pub fn refresh_if_pristine(&self, chapter: &str, exercise: &str) -> std::io::Result<bool> {
        if !self.exercise_present(chapter, exercise) {
            return Ok(false);
        }
        let _g = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let Some(recorded) = self.read_snapshot(chapter, exercise) else {
            return Ok(false);
        };
        if snapshot(&self.exercise_dir(chapter, exercise))? != recorded {
            return Ok(false);
        }
        let (staged, hidden) = self.stage_exercise(chapter, exercise)?;
        if snapshot(&staged)? == recorded
            && self.read_hidden_state(chapter, exercise).as_ref() == Some(&hidden)
        {
            let _ = std::fs::remove_dir_all(&staged);
            return Ok(false);
        }
        self.install(chapter, exercise, &staged, &hidden)?;
        tracing::info!(
            "refreshed untouched working copy for {chapter}/{exercise} (sources changed)"
        );
        Ok(true)
    }

    fn state_dir(&self, chapter: &str) -> PathBuf {
        self.dir.join(".state").join(chapter)
    }

    fn snapshot_path(&self, chapter: &str, exercise: &str) -> PathBuf {
        self.state_dir(chapter).join(format!("{exercise}.json"))
    }

    fn hidden_state_path(&self, chapter: &str, exercise: &str) -> PathBuf {
        self.state_dir(chapter)
            .join(format!("{exercise}.hidden.json"))
    }

    fn read_snapshot(&self, chapter: &str, exercise: &str) -> Option<Snapshot> {
        let text = std::fs::read_to_string(self.snapshot_path(chapter, exercise)).ok()?;
        serde_json::from_str(&text).ok()
    }

    pub fn read_hidden_state(&self, chapter: &str, exercise: &str) -> Option<HiddenState> {
        let text = std::fs::read_to_string(self.hidden_state_path(chapter, exercise)).ok()?;
        serde_json::from_str(&text).ok()
    }

    fn write_hidden_state(
        &self,
        chapter: &str,
        exercise: &str,
        state: &HiddenState,
    ) -> std::io::Result<()> {
        std::fs::create_dir_all(self.state_dir(chapter))?;
        std::fs::write(
            self.hidden_state_path(chapter, exercise),
            serde_json::to_vec_pretty(state).map_err(std::io::Error::other)?,
        )
    }

    /// Moves a staged copy into place (replacing any existing one) and records its
    /// snapshot and hidden test code. Caller holds `self.lock`.
    fn install(
        &self,
        chapter: &str,
        exercise: &str,
        staged: &Path,
        hidden: &HiddenState,
    ) -> std::io::Result<()> {
        let snap = snapshot(staged)?;
        let dst = self.exercise_dir(chapter, exercise);
        std::fs::create_dir_all(dst.parent().expect("has parent"))?;
        if dst.exists() {
            let trash = self.tmp_path("trash");
            std::fs::rename(&dst, &trash)?;
            std::fs::rename(staged, &dst)?;
            if let Err(e) = std::fs::remove_dir_all(&trash) {
                tracing::warn!("could not remove {}: {e}", trash.display());
            }
        } else {
            std::fs::rename(staged, &dst)?;
        }
        std::fs::create_dir_all(self.state_dir(chapter))?;
        std::fs::write(
            self.snapshot_path(chapter, exercise),
            serde_json::to_vec(&snap).map_err(std::io::Error::other)?,
        )?;
        self.write_hidden_state(chapter, exercise, hidden)
    }

    fn tmp_path(&self, what: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        self.dir
            .join(".tmp")
            .join(format!("{what}-{}-{nanos}-{n}", std::process::id()))
    }

    /// Builds a fresh copy in `.tmp/` (outside the member glob) so cargo/rust-analyzer
    /// never see a half-copied crate. Test code is moved to the end of each file.
    fn stage_exercise(
        &self,
        chapter: &str,
        exercise: &str,
    ) -> std::io::Result<(PathBuf, HiddenState)> {
        let staged = self.tmp_path("stage");
        let src = self.content.exercise_source_dir(chapter, exercise);
        copy_dir(&src, &staged, true)?;
        let tr = self.content.exercise_tr_dir(chapter, exercise);
        if tr.is_dir() {
            overlay_rs(&tr, &tr, &staged)?;
        }
        let hidden = apply_hiding(&format!("{chapter}/{exercise}"), &staged)?;
        Ok((staged, hidden))
    }

    /// Manifests this exercise lets the learner edit (from the original sources).
    pub fn cargo_edit(&self, chapter: &str, exercise: &str) -> CargoEdit {
        detect_cargo_edit(&self.original_sources(chapter, exercise))
    }

    fn original_sources(&self, chapter: &str, exercise: &str) -> Vec<String> {
        let dir = self.content.exercise_source_dir(chapter, exercise);
        let mut paths = Vec::new();
        for top in ["src", "tests"] {
            let _ = collect_rs(&dir, &dir.join(top), &mut paths);
        }
        paths
            .iter()
            .filter_map(|p| std::fs::read_to_string(dir.join(p)).ok())
            .collect()
    }

    /// Files shown to the learner: visible parts of `src/**.rs` (and `tests/**.rs` only if
    /// this exercise shows its tests), then editable manifests.
    pub fn list_files(&self, chapter: &str, exercise: &str) -> std::io::Result<Listing> {
        let root = self.exercise_dir(chapter, exercise);
        let state = self
            .read_hidden_state(chapter, exercise)
            .unwrap_or_default();
        let mut paths = Vec::new();
        collect_rs(&root, &root.join("src"), &mut paths)?;
        let mut test_paths = Vec::new();
        collect_rs(&root, &root.join("tests"), &mut test_paths)?;
        let hidden_test_files = if state.hide {
            test_paths.len()
        } else {
            paths.extend(test_paths);
            0
        };
        sort_files(&mut paths);
        let mut files = Vec::with_capacity(paths.len() + 2);
        for p in paths {
            let bytes = std::fs::read(root.join(&p))?;
            let disk = String::from_utf8_lossy(&bytes).into_owned();
            let (content, has_hidden_tests) = match state.files.get(&p) {
                Some(h) if state.hide => match crate::hidden::split_disk(&disk, h) {
                    Some(v) => (v.to_string(), true),
                    None => {
                        tracing::warn!(
                            "{chapter}/{exercise}/{p}: hidden test block not found at end of file; showing it in full"
                        );
                        (disk, false)
                    }
                },
                _ => (disk, false),
            };
            let deletable = self.is_learner_file(chapter, exercise, &p);
            files.push(ExerciseFile {
                path: p,
                content,
                language: "rust",
                has_hidden_tests,
                deletable,
            });
        }
        let edit = self.cargo_edit(chapter, exercise);
        if edit.own {
            files.push(ExerciseFile {
                path: OWN_MANIFEST_PATH.into(),
                content: std::fs::read_to_string(root.join("Cargo.toml"))?,
                language: "toml",
                has_hidden_tests: false,
                deletable: false,
            });
        }
        if edit.root {
            files.push(ExerciseFile {
                path: WORKSPACE_MANIFEST_PATH.into(),
                content: std::fs::read_to_string(self.dir.join("Cargo.toml"))?,
                language: "toml",
                has_hidden_tests: false,
                deletable: false,
            });
        }
        Ok(Listing {
            files,
            hidden_test_files,
        })
    }

    /// File -> first hidden line, for scrubbing compiler and test output.
    pub fn hidden_lines(&self, chapter: &str, exercise: &str) -> HiddenLines {
        let mut map = HiddenLines::new();
        let Some(state) = self.read_hidden_state(chapter, exercise) else {
            return map;
        };
        if !state.hide {
            return map;
        }
        let root = self.exercise_dir(chapter, exercise);
        for (path, hidden) in &state.files {
            if let Ok(disk) = std::fs::read_to_string(root.join(path)) {
                if let Some(v) = crate::hidden::split_disk(&disk, hidden) {
                    map.insert(path.clone(), crate::hidden::first_hidden_line(v));
                }
            }
        }
        let mut tests = Vec::new();
        let _ = collect_rs(&root, &root.join("tests"), &mut tests);
        for t in tests {
            map.insert(t, 1);
        }
        map
    }

    /// Checks that `rel` stays inside the exercise (no symlink escapes) and returns the
    /// absolute target path.
    fn checked_target(
        &self,
        root: &Path,
        rel: &str,
        create_parent: bool,
    ) -> Result<PathBuf, WriteError> {
        let target = root.join(rel);
        let parent = target.parent().ok_or(WriteError::Outside)?;
        if create_parent {
            std::fs::create_dir_all(parent)?;
        } else if !parent.is_dir() {
            return Err(WriteError::NotFound);
        }
        let canon_root = root.canonicalize()?;
        let canon_parent = parent.canonicalize()?;
        if !canon_parent.starts_with(&canon_root) {
            return Err(WriteError::Outside);
        }
        if let Ok(meta) = std::fs::symlink_metadata(&target) {
            if !meta.file_type().is_file() {
                return Err(WriteError::Outside);
            }
        }
        Ok(target)
    }

    /// Saves a file sent by the client. For sources the stored hidden test blocks are
    /// re-appended; manifests are validated so a typo cannot break the whole workspace.
    pub fn write_file(
        &self,
        chapter: &str,
        exercise: &str,
        path: &EditPath,
        content: &str,
    ) -> Result<(), WriteError> {
        let root = self.exercise_dir(chapter, exercise);
        match path {
            EditPath::OwnManifest => {
                if !self.cargo_edit(chapter, exercise).own {
                    return Err(WriteError::Forbidden(
                        "Bu egzersizde Cargo.toml düzenlenemez.",
                    ));
                }
                validate_own_manifest(content).map_err(WriteError::Invalid)?;
                let target = self.checked_target(&root, OWN_MANIFEST_PATH, false)?;
                std::fs::write(target, content)?;
                Ok(())
            }
            EditPath::WorkspaceManifest => {
                if !self.cargo_edit(chapter, exercise).root {
                    return Err(WriteError::Forbidden(
                        "Bu egzersizde kök Cargo.toml düzenlenemez.",
                    ));
                }
                let _g = self.lock.lock().unwrap_or_else(|e| e.into_inner());
                let default = self.default_manifest()?;
                let text =
                    reconcile_workspace_manifest(content, &default).map_err(WriteError::Invalid)?;
                std::fs::create_dir_all(self.dir.join(".state"))?;
                std::fs::write(self.manifest_edited_marker(), b"")?;
                write_if_changed(&self.dir.join("Cargo.toml"), text.as_bytes())?;
                Ok(())
            }
            EditPath::Source(rel) => {
                let _g = self.lock.lock().unwrap_or_else(|e| e.into_inner());
                let state = self
                    .read_hidden_state(chapter, exercise)
                    .unwrap_or_default();
                if rel.starts_with("tests/") && state.hide {
                    return Err(WriteError::Forbidden(
                        "`tests/` altındaki test dosyaları gizlidir ve düzenlenemez.",
                    ));
                }
                let target = self.checked_target(&root, rel, true)?;
                let disk = match state.files.get(rel) {
                    Some(h) if state.hide => crate::hidden::compose(content, h),
                    _ => content.to_string(),
                };
                std::fs::write(target, disk)?;
                Ok(())
            }
        }
    }

    /// Deletes a learner-created `src/**.rs` file (one that is not part of the original
    /// exercise or its translation).
    pub fn delete_file(
        &self,
        chapter: &str,
        exercise: &str,
        path: &EditPath,
    ) -> Result<(), WriteError> {
        const NOT_DELETABLE: &str = "Yalnızca senin oluşturduğun `src/` dosyaları silinebilir.";
        let EditPath::Source(rel) = path else {
            return Err(WriteError::Forbidden(NOT_DELETABLE));
        };
        if !self.is_learner_file(chapter, exercise, rel) {
            return Err(WriteError::Forbidden(NOT_DELETABLE));
        }
        let _g = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let root = self.exercise_dir(chapter, exercise);
        let target = self.checked_target(&root, rel, false)?;
        if !target.is_file() {
            return Err(WriteError::NotFound);
        }
        std::fs::remove_file(&target)?;
        // Drop now-empty directories below src/.
        let src = root.join("src");
        let mut dir = target.parent().map(Path::to_path_buf);
        while let Some(d) = dir {
            if d == src || !d.starts_with(&src) || std::fs::remove_dir(&d).is_err() {
                break;
            }
            dir = d.parent().map(Path::to_path_buf);
        }
        Ok(())
    }

    /// A `src/**.rs` path that is neither in the original exercise nor in its translation.
    pub fn is_learner_file(&self, chapter: &str, exercise: &str, rel: &str) -> bool {
        rel.starts_with("src/")
            && rel.ends_with(".rs")
            && !self
                .content
                .exercise_source_dir(chapter, exercise)
                .join(rel)
                .exists()
            && !self
                .content
                .exercise_tr_dir(chapter, exercise)
                .join(rel)
                .exists()
    }

    /// Package name from the working copy's Cargo.toml.
    pub fn package_name(&self, chapter: &str, exercise: &str) -> std::io::Result<String> {
        let manifest = self.exercise_dir(chapter, exercise).join("Cargo.toml");
        let text = std::fs::read_to_string(&manifest)?;
        let table: toml::Table = text.parse().map_err(std::io::Error::other)?;
        table
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .map(str::to_string)
            .ok_or_else(|| {
                std::io::Error::other(format!("no package.name in {}", manifest.display()))
            })
    }

    /// Exercises without any test in the original sources are "make it compile" exercises.
    pub fn original_has_tests(&self, chapter: &str, exercise: &str) -> bool {
        sources_have_tests(&self.original_sources(chapter, exercise))
    }
}

/// Reads an exercise directory's `src/**.rs` and `tests/**.rs`, decides what to hide and
/// rewrites the files into the "visible + hidden at the end" layout.
fn apply_hiding(id: &str, dir: &Path) -> std::io::Result<HiddenState> {
    let mut paths = Vec::new();
    for top in ["src", "tests"] {
        collect_rs(dir, &dir.join(top), &mut paths)?;
    }
    let mut files = Vec::with_capacity(paths.len());
    for p in paths {
        let text = std::fs::read_to_string(dir.join(&p))?;
        files.push((p, text));
    }
    let plan = plan_hiding(id, &files);
    for n in &plan.notes {
        tracing::warn!("hidden tests, {id}: {n}");
    }
    if let Some(why) = &plan.state.exception {
        tracing::info!("hidden tests, {id}: tests stay visible ({why})");
    }
    for (p, disk) in &plan.rewrites {
        std::fs::write(dir.join(p), disk)?;
    }
    Ok(plan.state)
}

/// Path -> FNV-1a hash of every editable file (`src/**.rs`, `tests/**.rs`, `Cargo.toml`).
type Snapshot = BTreeMap<String, String>;

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn hash_entry(bytes: &[u8]) -> String {
    format!("{:016x}-{}", fnv1a(bytes), bytes.len())
}

fn snapshot(dir: &Path) -> std::io::Result<Snapshot> {
    let mut paths = Vec::new();
    for top in ["src", "tests"] {
        collect_rs(dir, &dir.join(top), &mut paths)?;
    }
    let mut snap = Snapshot::new();
    for p in paths {
        let bytes = std::fs::read(dir.join(&p))?;
        snap.insert(p, hash_entry(&bytes));
    }
    if let Ok(bytes) = std::fs::read(dir.join(OWN_MANIFEST_PATH)) {
        snap.insert(OWN_MANIFEST_PATH.into(), hash_entry(&bytes));
    }
    Ok(snap)
}

/// Generates the workspace manifest, copying every `[profile.*]` table from the root.
pub fn workspace_manifest(root_manifest: &str) -> Result<String, String> {
    let root: toml::Table = root_manifest.parse().map_err(|e| format!("{e}"))?;
    let mut ws = toml::Table::new();
    ws.insert(
        "members".into(),
        toml::Value::Array(vec!["exercises/*/*".into(), "helpers/*".into()]),
    );
    ws.insert("resolver".into(), "2".into());
    render_manifest(Some(&toml::Value::Table(ws)), root.get("profile"))
}

fn write_if_changed(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Ok(existing) = std::fs::read(path) {
        if existing == bytes {
            return Ok(());
        }
    }
    std::fs::write(path, bytes)
}

fn copy_dir(src: &Path, dst: &Path, skip_target: bool) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let ft = entry.file_type()?;
        if skip_target && name == "target" {
            continue;
        }
        let s = entry.path();
        let d = dst.join(&name);
        if ft.is_dir() {
            copy_dir(&s, &d, false)?;
        } else if ft.is_file() {
            std::fs::copy(&s, &d)?;
        }
    }
    Ok(())
}

fn sync_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == "target" {
            continue;
        }
        let ft = entry.file_type()?;
        let s = entry.path();
        let d = dst.join(&name);
        if ft.is_dir() {
            sync_dir(&s, &d)?;
        } else if ft.is_file() {
            write_if_changed(&d, &std::fs::read(&s)?)?;
        }
    }
    Ok(())
}

fn overlay_rs(base: &Path, dir: &Path, dst_root: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        let p = entry.path();
        if ft.is_dir() {
            overlay_rs(base, &p, dst_root)?;
        } else if ft.is_file() && p.extension().is_some_and(|e| e == "rs") {
            let rel = p.strip_prefix(base).expect("under base");
            let d = dst_root.join(rel);
            if let Some(parent) = d.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&p, &d)?;
        }
    }
    Ok(())
}

fn collect_rs(root: &Path, dir: &Path, out: &mut Vec<String>) -> std::io::Result<()> {
    let rd = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    };
    for entry in rd {
        let entry = entry?;
        let ft = entry.file_type()?; // does not follow symlinks
        let p = entry.path();
        if ft.is_dir() {
            collect_rs(root, &p, out)?;
        } else if ft.is_file() && p.extension().is_some_and(|e| e == "rs") {
            if let Ok(rel) = p.strip_prefix(root) {
                let rel = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                out.push(rel);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_errors_carry_position() {
        let bad = "[package]\nname = \"x\"\n\n[dependencies\n";
        let err = validate_own_manifest(bad).unwrap_err();
        assert!(
            err.starts_with("Cargo.toml ayrıştırılamadı (satır 4, sütun "),
            "{err}"
        );
        assert!(err.contains("): "), "{err}");

        let ws = "[profile.dev]\nopt-level = = 1\n";
        let err = reconcile_workspace_manifest(ws, "[workspace]\n").unwrap_err();
        assert!(
            err.starts_with("Cargo.toml ayrıştırılamadı (satır 2, sütun "),
            "{err}"
        );

        assert_eq!(line_col("ab\ncd", 0), (1, 1));
        assert_eq!(line_col("ab\ncd", 4), (2, 2));
        assert_eq!(line_col("çç\nx", 2), (1, 2)); // byte offset -> char column
        assert_eq!(line_col("ab", 99), (1, 3));
        assert_eq!(line_col("ç", 1), (1, 1)); // inside a char -> clamped
    }

    #[test]
    fn accepts_valid_paths() {
        assert!(validate_rel_path("src/lib.rs").is_ok());
        assert!(validate_rel_path("src/a/b/c.rs").is_ok());
        assert!(validate_rel_path("tests/integration.rs").is_ok());
    }

    #[test]
    fn rejects_bad_paths() {
        use PathError::*;
        let cases = [
            ("", Empty),
            ("/etc/passwd", Absolute),
            ("~/x.rs", Absolute),
            ("src/../Cargo.toml", Traversal),
            ("src/../../x.rs", Traversal),
            ("src/./lib.rs", Traversal),
            ("src//lib.rs", Traversal),
            ("src/", Traversal),
            ("..", Traversal),
            ("src\\lib.rs", InvalidChar),
            ("C:/src/lib.rs", InvalidChar),
            ("src/li\0b.rs", InvalidChar),
            ("Cargo.toml", NotEditableDir),
            ("lib.rs", NotEditableDir),
            ("target/x.rs", NotEditableDir),
            ("src/lib.txt", NotRustFile),
            ("src/.rs", NotRustFile),
        ];
        for (p, want) in cases {
            assert_eq!(validate_rel_path(p), Err(want), "{p:?}");
        }
        assert_eq!(validate_rel_path(&"a".repeat(300)), Err(TooLong));
    }

    #[test]
    fn sorts_files() {
        let mut v = vec![
            "tests/a.rs".to_string(),
            "src/z.rs".into(),
            "src/main.rs".into(),
            "src/lib.rs".into(),
            "src/a.rs".into(),
        ];
        sort_files(&mut v);
        assert_eq!(
            v,
            [
                "src/lib.rs",
                "src/main.rs",
                "src/a.rs",
                "src/z.rs",
                "tests/a.rs"
            ]
        );
    }

    #[test]
    fn uri_encoding() {
        assert_eq!(
            file_uri(Path::new("/home/a b/ç")),
            "file:///home/a%20b/%C3%A7"
        );
    }

    #[test]
    fn manifest_keeps_profile() {
        let root =
            "[workspace]\nmembers=[\"x\"]\n[profile.dev.package.copy]\noverflow-checks = true\n";
        let m = workspace_manifest(root).unwrap();
        let t: toml::Table = m.parse().unwrap();
        assert_eq!(
            t["profile"]["dev"]["package"]["copy"]["overflow-checks"].as_bool(),
            Some(true)
        );
        assert_eq!(t["workspace"]["members"][0].as_str(), Some("exercises/*/*"));
        assert_eq!(t["workspace"]["resolver"].as_str(), Some("2"));
    }

    #[test]
    fn refreshes_only_untouched_copies() {
        let tmp =
            std::env::temp_dir().join(format!("ws-test-{}-{}", std::process::id(), fnv1a(b"x")));
        let _ = std::fs::remove_dir_all(&tmp);
        let root = tmp.join("repo");
        let ex = root.join("exercises/ch/ex");
        std::fs::create_dir_all(ex.join("src")).unwrap();
        std::fs::create_dir_all(root.join("helpers/h/src")).unwrap();
        std::fs::write(root.join("helpers/h/Cargo.toml"), "[package]\nname=\"h\"\n").unwrap();
        std::fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
        std::fs::write(root.join("Cargo.lock"), "").unwrap();
        std::fs::write(ex.join("Cargo.toml"), "[package]\nname = \"ex\"\n").unwrap();
        std::fs::write(ex.join("src/lib.rs"), "// english\n").unwrap();
        let ws = Workspace::new(tmp.join("ws"), Content::new(root.clone()));
        ws.ensure_exercise("ch", "ex").unwrap();
        let lib = ws.exercise_dir("ch", "ex").join("src/lib.rs");
        assert_eq!(std::fs::read_to_string(&lib).unwrap(), "// english\n");
        assert!(!ws.refresh_if_pristine("ch", "ex").unwrap());

        // Translation appears -> untouched copy is refreshed.
        let tr = root.join("tr/exercises/ch/ex/src");
        std::fs::create_dir_all(&tr).unwrap();
        std::fs::write(tr.join("lib.rs"), "// türkçe\n").unwrap();
        assert!(ws.refresh_if_pristine("ch", "ex").unwrap());
        assert_eq!(std::fs::read_to_string(&lib).unwrap(), "// türkçe\n");

        // User edit -> never overwritten.
        std::fs::write(&lib, "// my code\n").unwrap();
        std::fs::write(tr.join("lib.rs"), "// türkçe v2\n").unwrap();
        assert!(!ws.refresh_if_pristine("ch", "ex").unwrap());
        assert_eq!(std::fs::read_to_string(&lib).unwrap(), "// my code\n");
        assert_eq!(ws.package_name("ch", "ex").unwrap(), "ex");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    fn src(path: &str, text: &str) -> (String, String) {
        (path.to_string(), text.to_string())
    }

    const LIB: &str = "pub fn f() -> u32 {\n    1\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        assert_eq!(super::f(), 1);\n    }\n}\n\npub fn g() {}\n";

    /// Minimal repository with one exercise; returns (tmp dir, workspace).
    fn fixture(name: &str, lib: &str, tests: Option<&str>) -> (PathBuf, Workspace) {
        let tmp = std::env::temp_dir().join(format!("ws-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let root = tmp.join("repo");
        let ex = root.join("exercises/ch/ex");
        std::fs::create_dir_all(ex.join("src")).unwrap();
        std::fs::create_dir_all(root.join("helpers/h/src")).unwrap();
        std::fs::write(root.join("helpers/h/Cargo.toml"), "[package]\nname=\"h\"\n").unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[workspace]\n[profile.dev.package.copy]\noverflow-checks = true\n",
        )
        .unwrap();
        std::fs::write(root.join("Cargo.lock"), "").unwrap();
        std::fs::write(ex.join("Cargo.toml"), "[package]\nname = \"ex\"\n").unwrap();
        std::fs::write(ex.join("src/lib.rs"), lib).unwrap();
        if let Some(t) = tests {
            std::fs::create_dir_all(ex.join("tests")).unwrap();
            std::fs::write(ex.join("tests/it.rs"), t).unwrap();
        }
        let ws = Workspace::new(tmp.join("ws"), Content::new(root));
        ws.ensure_exercise("ch", "ex").unwrap();
        (tmp, ws)
    }

    #[test]
    fn hides_tests_and_merges_on_put() {
        let (tmp, ws) = fixture("put", LIB, Some("#[test]\nfn it() {}\n"));
        let disk_path = ws.exercise_dir("ch", "ex").join("src/lib.rs");
        let listing = ws.list_files("ch", "ex").unwrap();
        assert_eq!(listing.hidden_test_files, 1);
        assert_eq!(listing.files.len(), 1, "tests/ is not listed");
        let lib = &listing.files[0];
        assert_eq!(lib.path, "src/lib.rs");
        assert!(lib.has_hidden_tests);
        assert_eq!(lib.language, "rust");
        assert_eq!(
            lib.content,
            "pub fn f() -> u32 {\n    1\n}\n\npub fn g() {}\n"
        );
        // On disk: visible + "\n" + hidden; visible lines keep their numbers.
        let disk = std::fs::read_to_string(&disk_path).unwrap();
        assert!(disk.starts_with(&lib.content));
        assert!(disk.ends_with("#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        assert_eq!(super::f(), 1);\n    }\n}\n"));
        let hl = ws.hidden_lines("ch", "ex");
        assert_eq!(hl["src/lib.rs"], 7);
        assert_eq!(hl["tests/it.rs"], 1);

        // PUT: the client's text + stored hidden block; client cfg(test) code is kept.
        let mine = "pub fn f() -> u32 { 2 }\n#[cfg(test)]\nmod mine {}\n";
        ws.write_file("ch", "ex", &EditPath::Source("src/lib.rs".into()), mine)
            .unwrap();
        let disk = std::fs::read_to_string(&disk_path).unwrap();
        assert!(disk.starts_with(mine));
        assert!(disk.ends_with("assert_eq!(super::f(), 1);\n    }\n}\n"));
        let listing = ws.list_files("ch", "ex").unwrap();
        assert_eq!(listing.files[0].content, mine);
        assert_eq!(ws.hidden_lines("ch", "ex")["src/lib.rs"], 5);

        // tests/ is read-only while hidden.
        assert!(matches!(
            ws.write_file("ch", "ex", &EditPath::Source("tests/it.rs".into()), "x"),
            Err(WriteError::Forbidden(_))
        ));
        // No manifest editing in this exercise.
        assert!(matches!(
            ws.write_file(
                "ch",
                "ex",
                &EditPath::OwnManifest,
                "[package]\nname=\"ex\"\n"
            ),
            Err(WriteError::Forbidden(_))
        ));

        // Edited copy is never refreshed; reset restores the original layout.
        assert!(!ws.refresh_if_pristine("ch", "ex").unwrap());
        ws.reset_exercise("ch", "ex").unwrap();
        assert_eq!(
            ws.list_files("ch", "ex").unwrap().files[0].content,
            "pub fn f() -> u32 {\n    1\n}\n\npub fn g() {}\n"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn learner_created_files() {
        let (tmp, ws) = fixture("new", LIB, None);
        let p = EditPath::Source("src/extra/mod2.rs".into());
        // Original files cannot be deleted; missing ones are 404.
        assert!(matches!(
            ws.delete_file("ch", "ex", &EditPath::Source("src/lib.rs".into())),
            Err(WriteError::Forbidden(_))
        ));
        assert!(matches!(
            ws.delete_file("ch", "ex", &EditPath::Source("src/nope.rs".into())),
            Err(WriteError::NotFound)
        ));
        assert!(matches!(
            ws.delete_file("ch", "ex", &EditPath::OwnManifest),
            Err(WriteError::Forbidden(_))
        ));
        ws.write_file("ch", "ex", &p, "pub fn x() {}\n").unwrap();
        let names: Vec<String> = ws
            .list_files("ch", "ex")
            .unwrap()
            .files
            .into_iter()
            .map(|f| f.path)
            .collect();
        assert_eq!(names, ["src/lib.rs", "src/extra/mod2.rs"]);
        let deletable: Vec<bool> = ws
            .list_files("ch", "ex")
            .unwrap()
            .files
            .iter()
            .map(|f| f.deletable)
            .collect();
        assert_eq!(deletable, [false, true]);
        // A new translation does not wipe a copy with learner-created files.
        let tr = ws.content.root.join("tr/exercises/ch/ex/src");
        std::fs::create_dir_all(&tr).unwrap();
        std::fs::write(
            tr.join("lib.rs"),
            LIB.replace("pub fn g", "// tr\npub fn g"),
        )
        .unwrap();
        assert!(!ws.refresh_if_pristine("ch", "ex").unwrap());
        assert!(ws
            .exercise_dir("ch", "ex")
            .join("src/extra/mod2.rs")
            .is_file());
        ws.delete_file("ch", "ex", &p).unwrap();
        assert!(!ws.exercise_dir("ch", "ex").join("src/extra").exists());
        // Back to untouched -> refreshed with the translation.
        assert!(ws.refresh_if_pristine("ch", "ex").unwrap());
        // Reset removes learner-created files.
        ws.write_file("ch", "ex", &p, "pub fn x() {}\n").unwrap();
        ws.reset_exercise("ch", "ex").unwrap();
        assert!(!ws
            .exercise_dir("ch", "ex")
            .join("src/extra/mod2.rs")
            .exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn manifest_editing() {
        let lib = "// Customize the `dev` profile, in the `Cargo.toml`\n// at the root of the repository.\npub fn f() {}\n";
        let (tmp, ws) = fixture("manifest", lib, None);
        assert_eq!(
            ws.cargo_edit("ch", "ex"),
            CargoEdit {
                own: true,
                root: true
            }
        );
        let listing = ws.list_files("ch", "ex").unwrap();
        let paths: Vec<_> = listing
            .files
            .iter()
            .map(|f| (f.path.as_str(), f.language))
            .collect();
        assert_eq!(
            paths,
            [
                ("src/lib.rs", "rust"),
                ("Cargo.toml", "toml"),
                ("@workspace/Cargo.toml", "toml")
            ]
        );
        let ws_manifest = ws.dir.join("Cargo.toml");
        let default = std::fs::read_to_string(&ws_manifest).unwrap();

        // Learner adds a profile; text is kept verbatim and survives init().
        let edited = format!("{default}\n[profile.dev]\noverflow-checks = false # mine\n");
        ws.write_file("ch", "ex", &EditPath::WorkspaceManifest, &edited)
            .unwrap();
        ws.init().unwrap();
        assert_eq!(std::fs::read_to_string(&ws_manifest).unwrap(), edited);

        // Tampering with [workspace] is undone, profiles kept.
        let tampered = "[workspace]\nmembers = []\n[profile.dev]\noverflow-checks = false\n";
        ws.write_file("ch", "ex", &EditPath::WorkspaceManifest, tampered)
            .unwrap();
        let t: toml::Table = std::fs::read_to_string(&ws_manifest)
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(t["workspace"]["members"][0].as_str(), Some("exercises/*/*"));
        assert_eq!(
            t["profile"]["dev"]["overflow-checks"].as_bool(),
            Some(false)
        );
        assert!(t["profile"]["dev"].get("package").is_none());

        // Invalid input is rejected.
        for bad in ["[profile", "[dependencies]\nx = \"1\"\n", "profile = 1\n"] {
            assert!(
                matches!(
                    ws.write_file("ch", "ex", &EditPath::WorkspaceManifest, bad),
                    Err(WriteError::Invalid(_))
                ),
                "{bad}"
            );
        }
        assert!(matches!(
            ws.write_file("ch", "ex", &EditPath::OwnManifest, "[dependencies]\n"),
            Err(WriteError::Invalid(_))
        ));
        let own = "[package]\nname = \"ex\"\n\n[dependencies]\nanyhow = \"1\"\n";
        ws.write_file("ch", "ex", &EditPath::OwnManifest, own)
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(ws.exercise_dir("ch", "ex").join("Cargo.toml")).unwrap(),
            own
        );
        // An edited manifest makes the copy "edited".
        assert!(!ws.refresh_if_pristine("ch", "ex").unwrap());

        // Reset restores both manifests.
        ws.reset_exercise("ch", "ex").unwrap();
        assert_eq!(std::fs::read_to_string(&ws_manifest).unwrap(), default);
        assert_eq!(
            std::fs::read_to_string(ws.exercise_dir("ch", "ex").join("Cargo.toml")).unwrap(),
            "[package]\nname = \"ex\"\n"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn migrates_old_layout() {
        let (tmp, ws) = fixture("migrate", LIB, None);
        let dir = ws.exercise_dir("ch", "ex");
        // Simulate a copy made by the previous version: original layout, no hidden state,
        // snapshot without Cargo.toml.
        std::fs::write(dir.join("src/lib.rs"), LIB).unwrap();
        let mut snap = snapshot(&dir).unwrap();
        snap.remove(OWN_MANIFEST_PATH);
        std::fs::write(
            ws.snapshot_path("ch", "ex"),
            serde_json::to_vec(&snap).unwrap(),
        )
        .unwrap();
        std::fs::remove_file(ws.hidden_state_path("ch", "ex")).unwrap();
        ws.ensure_exercise("ch", "ex").unwrap();
        assert!(ws.list_files("ch", "ex").unwrap().files[0].has_hidden_tests);
        assert!(
            !ws.refresh_if_pristine("ch", "ex").unwrap(),
            "recreated as pristine"
        );

        // Edited old copy: converted in place.
        let edited = LIB.replace("1\n}", "2\n}");
        std::fs::write(dir.join("src/lib.rs"), &edited).unwrap();
        std::fs::remove_file(ws.hidden_state_path("ch", "ex")).unwrap();
        ws.ensure_exercise("ch", "ex").unwrap();
        let f = &ws.list_files("ch", "ex").unwrap().files[0];
        assert_eq!(
            f.content,
            "pub fn f() -> u32 {\n    2\n}\n\npub fn g() {}\n"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn exception_keeps_tests_visible() {
        let lib =
            "pub fn f() {}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() { todo!() }\n}\n";
        let (tmp, ws) = fixture("exception", lib, Some("#[test]\nfn it() {}\n"));
        let l = ws.list_files("ch", "ex").unwrap();
        assert_eq!(l.hidden_test_files, 0);
        assert_eq!(l.files.len(), 2);
        assert_eq!(l.files[0].content, lib);
        assert!(!l.files[0].has_hidden_tests);
        assert!(ws.hidden_lines("ch", "ex").is_empty());
        ws.write_file(
            "ch",
            "ex",
            &EditPath::Source("tests/it.rs".into()),
            "// mine\n",
        )
        .unwrap();
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn plan_marks_exceptions_from_test_files() {
        let files = [
            src("src/lib.rs", LIB),
            src("tests/it.rs", "#[test]\nfn it() { assert_eq!(1, __); }\n"),
        ];
        let plan = plan_hiding("x/y", &files);
        assert!(!plan.state.hide);
        assert!(plan.rewrites.is_empty());
        let plan = plan_hiding("x/y", &files[..1]);
        assert!(plan.state.hide);
        assert_eq!(
            plan.rewrites.len(),
            1,
            "test module in the middle moves to the end"
        );
    }

    /// Scans every exercise of the repository (original + Turkish overlay) and checks the
    /// extraction invariants. Run with `--nocapture` to see exceptions / notes.
    #[test]
    fn scan_all_exercises() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut n = 0;
        for ch in std::fs::read_dir(root.join("exercises")).unwrap() {
            let ch = ch.unwrap().path();
            if !ch.is_dir() {
                continue;
            }
            for ex in std::fs::read_dir(&ch).unwrap() {
                let ex = ex.unwrap().path();
                if !ex.join("Cargo.toml").is_file() {
                    continue;
                }
                n += 1;
                let id = format!(
                    "{}/{}",
                    ch.file_name().unwrap().to_string_lossy(),
                    ex.file_name().unwrap().to_string_lossy()
                );
                let tr = root.join("tr/exercises").join(&id);
                let mut paths = Vec::new();
                for top in ["src", "tests"] {
                    collect_rs(&ex, &ex.join(top), &mut paths).unwrap();
                }
                let files: Vec<(String, String)> = paths
                    .iter()
                    .map(|p| {
                        let t = tr.join(p);
                        let f = if t.is_file() { t } else { ex.join(p) };
                        (p.clone(), std::fs::read_to_string(f).unwrap())
                    })
                    .collect();
                let plan = plan_hiding(&id, &files);
                for note in &plan.notes {
                    println!("NOTE {id}: {note}");
                }
                if let Some(why) = &plan.state.exception {
                    println!("EXCEPTION {id}: {why}");
                }
                let edit = detect_cargo_edit(
                    &paths
                        .iter()
                        .map(|p| std::fs::read_to_string(ex.join(p)).unwrap())
                        .collect::<Vec<_>>(),
                );
                if edit.own {
                    println!("CARGO_EDIT {id}: {edit:?}");
                }
                let moved: Vec<&str> = plan.rewrites.iter().map(|(p, _)| p.as_str()).collect();
                if !moved.is_empty() {
                    println!("MOVED {id}: {moved:?}");
                }
                for (path, content) in &files {
                    let Some(hidden) = plan.state.files.get(path) else {
                        continue;
                    };
                    let disk = plan
                        .rewrites
                        .iter()
                        .find(|(p, _)| p == path)
                        .map(|(_, d)| d.as_str())
                        .unwrap_or(content);
                    let visible = crate::hidden::split_disk(disk, hidden).expect("split");
                    let again = crate::hidden::extract(visible);
                    assert_eq!(
                        again.blocks, 0,
                        "{id}/{path}: test code left in visible part"
                    );
                    assert!(!visible.contains("#[test]"), "{id}/{path}");
                    assert!(hidden.contains("#[cfg(test)]"), "{id}/{path}");
                }
            }
        }
        assert_eq!(n, 98);
    }

    #[test]
    fn detects_tests() {
        assert!(sources_have_tests(&["#[test]\nfn a(){}".into()]));
        assert!(sources_have_tests(&[
            "#[tokio::test]\nasync fn a(){}".into()
        ]));
        assert!(!sources_have_tests(&["fn main(){}".into()]));
    }
}
