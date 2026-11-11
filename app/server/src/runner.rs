//! Compile + test runner (see README.md "Mimari").
//!
//! 1. `cargo test -p <pkg> --no-run --message-format=json` builds every test binary of the
//!    exercise and yields compiler diagnostics plus the paths of the test executables.
//! 2. Each test executable is listed (`--list --format terse`) and run directly; libtest's
//!    human output (`test <name> ... ok|FAILED|ignored`, `---- <name> stdout ----`) is parsed.
//!
//! The runner sits behind the [`Runner`] trait so a sandboxed implementation can replace it.

use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;
use tokio::process::Command;

use crate::hidden::{headline_only, is_hidden, scrub_locations, scrub_rendered, HiddenLines};
use crate::proc::run_captured;

pub const LOG_LIMIT: usize = 64 * 1024;
const CARGO_STDOUT_CAP: usize = 16 * 1024 * 1024;
const TEST_OUTPUT_CAP: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Passed,
    Failed,
    CompileError,
    Timeout,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Passed => "passed",
            Status::Failed => "failed",
            Status::CompileError => "compile_error",
            Status::Timeout => "timeout",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub file: String,
    /// `None` for diagnostics inside hidden test code.
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
    pub severity: String,
    pub message: String,
    pub rendered: String,
    /// The primary span lies in hidden test code; `rendered` is the short form.
    pub hidden: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunOutcome {
    pub status: Status,
    pub tests: Vec<TestResult>,
    pub tests_passed: usize,
    pub tests_total: usize,
    pub diagnostics: Vec<Diagnostic>,
    pub log: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub workspace_dir: PathBuf,
    /// Absolute path of the exercise crate inside the workspace.
    pub exercise_dir: PathBuf,
    /// `exercises/<chapter>/<exercise>/` (how rustc names files, relative to the workspace).
    pub rel_prefix: String,
    pub package: String,
    /// False for "make it compile" exercises whose original sources contain no tests;
    /// for those a clean build with zero tests counts as passed.
    pub expects_tests: bool,
    /// Hidden test code per exercise-relative file (scrubbed from all output).
    pub hidden: HiddenLines,
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait Runner: Send + Sync {
    fn run<'a>(&'a self, job: &'a Job) -> BoxFuture<'a, RunOutcome>;
}

pub struct CargoRunner {
    pub compile_timeout: Duration,
    pub test_timeout: Duration,
}

impl Runner for CargoRunner {
    fn run<'a>(&'a self, job: &'a Job) -> BoxFuture<'a, RunOutcome> {
        Box::pin(self.run_job(job))
    }
}

// ---------------------------------------------------------------------------------------
// Parsing: cargo JSON messages
// ---------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestArtifact {
    pub executable: PathBuf,
    pub target_name: String,
    /// Cargo target kind: `lib`, `bin`, `test`, ...
    pub kind: String,
}

#[derive(Debug, Default)]
pub struct CargoParse {
    pub diagnostics: Vec<Diagnostic>,
    /// Rendered text of every diagnostic belonging to the exercise (relativized).
    pub rendered: String,
    pub artifacts: Vec<TestArtifact>,
    pub build_success: Option<bool>,
}

/// Rewrites workspace-relative and absolute exercise paths to exercise-relative ones.
pub fn relativize(text: &str, rel_prefix: &str, abs_prefix: &str) -> String {
    let t = if abs_prefix.is_empty() {
        text.to_string()
    } else {
        text.replace(abs_prefix, "")
    };
    t.replace(rel_prefix, "")
}

fn manifest_matches(v: &Value, rel_prefix: &str) -> bool {
    v.get("manifest_path")
        .and_then(Value::as_str)
        .is_some_and(|m| m.ends_with(&format!("/{rel_prefix}Cargo.toml")))
}

/// Maps a rustc span to an exercise-relative file, following macro expansions back to
/// the call site when the primary span points into another crate or std.
fn span_location(span: &Value, rel_prefix: &str, abs_prefix: &str) -> Option<(String, [u32; 4])> {
    let mut cur = Some(span);
    let mut depth = 0;
    while let Some(s) = cur {
        if depth > 32 {
            break;
        }
        depth += 1;
        let file = s.get("file_name").and_then(Value::as_str).unwrap_or("");
        let rel = file.strip_prefix(rel_prefix).or_else(|| {
            (!abs_prefix.is_empty())
                .then(|| file.strip_prefix(abs_prefix))
                .flatten()
        });
        if let Some(rel) = rel {
            let num = |k: &str| s.get(k).and_then(Value::as_u64).unwrap_or(1).max(1) as u32;
            return Some((
                rel.to_string(),
                [
                    num("line_start"),
                    num("column_start"),
                    num("line_end"),
                    num("column_end"),
                ],
            ));
        }
        cur = s.get("expansion").and_then(|e| e.get("span"));
    }
    None
}

fn message_location(msg: &Value, rel_prefix: &str, abs_prefix: &str) -> Option<(String, [u32; 4])> {
    let spans = msg.get("spans")?.as_array()?;
    let primary = spans
        .iter()
        .filter(|s| s.get("is_primary").and_then(Value::as_bool) == Some(true));
    for s in primary {
        if let Some(loc) = span_location(s, rel_prefix, abs_prefix) {
            return Some(loc);
        }
    }
    spans
        .iter()
        .find_map(|s| span_location(s, rel_prefix, abs_prefix))
}

/// Parses `cargo --message-format=json` stdout for one exercise package.
pub fn parse_cargo_messages(
    stdout: &str,
    rel_prefix: &str,
    abs_prefix: &str,
    hidden: &HiddenLines,
) -> CargoParse {
    let mut out = CargoParse::default();
    let mut seen = std::collections::HashSet::new();
    for line in stdout.lines() {
        let line = line.trim();
        if !line.starts_with('{') {
            continue;
        }
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        match v.get("reason").and_then(Value::as_str) {
            Some("compiler-message") if manifest_matches(&v, rel_prefix) => {
                let Some(msg) = v.get("message") else {
                    continue;
                };
                let level = msg.get("level").and_then(Value::as_str).unwrap_or("error");
                let rendered = msg
                    .get("rendered")
                    .and_then(Value::as_str)
                    .map(|r| relativize(r, rel_prefix, abs_prefix))
                    .unwrap_or_default();
                // The same diagnostic is reported once per target (lib, lib test, ...).
                if !seen.insert(rendered.clone()) {
                    continue;
                }
                let location = message_location(msg, rel_prefix, abs_prefix);
                let in_hidden = location
                    .as_ref()
                    .is_some_and(|(f, [l, ..])| is_hidden(hidden, f, *l));
                let rendered = if in_hidden {
                    headline_only(&rendered, hidden)
                } else {
                    scrub_rendered(&rendered, hidden)
                };
                // Shortened hidden diagnostics (one per failing call site) often coincide.
                if in_hidden && !seen.insert(format!("hidden\0{rendered}")) {
                    continue;
                }
                if !rendered.is_empty() {
                    out.rendered.push_str(&rendered);
                    if !rendered.ends_with('\n') {
                        out.rendered.push('\n');
                    }
                }
                let Some((file, [line, column, end_line, end_column])) = location else {
                    continue;
                };
                let severity = match level {
                    l if l.starts_with("error") => "error",
                    "warning" => "warning",
                    "note" | "failure-note" => "note",
                    "help" => "help",
                    _ => "error",
                };
                let pos = |v: u32| (!in_hidden).then_some(v);
                out.diagnostics.push(Diagnostic {
                    file,
                    line: pos(line),
                    column: pos(column),
                    end_line: pos(end_line),
                    end_column: pos(end_column),
                    hidden: in_hidden,
                    severity: severity.to_string(),
                    message: msg
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    rendered,
                });
            }
            Some("compiler-artifact") if manifest_matches(&v, rel_prefix) => {
                let is_test = v
                    .get("profile")
                    .and_then(|p| p.get("test"))
                    .and_then(Value::as_bool)
                    == Some(true);
                let exe = v.get("executable").and_then(Value::as_str);
                if let (true, Some(exe)) = (is_test, exe) {
                    let target = v.get("target");
                    let target_name = target
                        .and_then(|t| t.get("name"))
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let kind = target
                        .and_then(|t| t.get("kind"))
                        .and_then(|k| k.get(0))
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    out.artifacts.push(TestArtifact {
                        executable: PathBuf::from(exe),
                        target_name,
                        kind,
                    });
                }
            }
            Some("build-finished") => {
                out.build_success = v.get("success").and_then(Value::as_bool);
            }
            _ => {}
        }
    }
    out
}

// ---------------------------------------------------------------------------------------
// Parsing: libtest output
// ---------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestOutcome {
    Ok,
    Failed,
    Ignored,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct LibtestRun {
    /// In output order.
    pub results: Vec<(String, TestOutcome)>,
    /// `---- <name> stdout ----` sections.
    pub failures: HashMap<String, String>,
}

/// Parses `<binary> --list --format terse`.
pub fn parse_test_list(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter_map(|l| l.strip_suffix(": test"))
        .map(str::to_string)
        .collect()
}

fn parse_result_line(line: &str) -> Option<(String, TestOutcome)> {
    let rest = line.strip_prefix("test ")?;
    let (name, result) = rest.rsplit_once(" ... ")?;
    let outcome = match result.trim_end() {
        "ok" => TestOutcome::Ok,
        "FAILED" => TestOutcome::Failed,
        r if r.starts_with("ignored") => TestOutcome::Ignored,
        _ => return None,
    };
    let name = name.strip_suffix(" - should panic").unwrap_or(name);
    Some((name.to_string(), outcome))
}

fn section_header(line: &str) -> Option<&str> {
    line.strip_prefix("---- ")?.strip_suffix(" stdout ----")
}

pub fn parse_libtest(stdout: &str) -> LibtestRun {
    let mut run = LibtestRun::default();
    let mut current: Option<(String, Vec<&str>)> = None;
    let flush = |cur: &mut Option<(String, Vec<&str>)>, run: &mut LibtestRun| {
        if let Some((name, lines)) = cur.take() {
            let text = lines.join("\n");
            run.failures
                .insert(name, text.trim_matches('\n').to_string());
        }
    };
    for line in stdout.lines() {
        if let Some(name) = section_header(line) {
            flush(&mut current, &mut run);
            current = Some((name.to_string(), Vec::new()));
            continue;
        }
        if let Some((_, lines)) = current.as_mut() {
            if line == "failures:" || line.starts_with("test result: ") {
                flush(&mut current, &mut run);
            } else {
                lines.push(line);
            }
            continue;
        }
        if let Some(r) = parse_result_line(line) {
            run.results.push(r);
        }
    }
    flush(&mut current, &mut run);
    run
}

/// Final status per app/API.md.
pub fn compute_status(
    build_ok: bool,
    timed_out: bool,
    tests: &[TestResult],
    expects_tests: bool,
) -> Status {
    if timed_out {
        return Status::Timeout;
    }
    if !build_ok {
        return Status::CompileError;
    }
    if tests.is_empty() {
        return if expects_tests {
            Status::Failed
        } else {
            Status::Passed
        };
    }
    if tests.iter().all(|t| t.passed) {
        Status::Passed
    } else {
        Status::Failed
    }
}

/// Truncates to at most `max` bytes keeping head and tail, on char boundaries.
pub fn truncate_log(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let marker = "\n\n… (çıktı kısaltıldı) …\n\n";
    let budget = max.saturating_sub(marker.len());
    let mut head = budget / 2;
    while !s.is_char_boundary(head) {
        head -= 1;
    }
    let mut tail = s.len() - (budget - head);
    while !s.is_char_boundary(tail) {
        tail += 1;
    }
    format!("{}{}{}", &s[..head], marker, &s[tail..])
}

// ---------------------------------------------------------------------------------------
// Execution
// ---------------------------------------------------------------------------------------

/// Drops cargo noise that is an artefact of the workspace layout: the root
/// `[profile.dev.package.copy]` only matches once `04_traits/12_copy` is opened.
/// Also strips the absolute path from status lines such as
/// `   Compiling x v0.1.0 (/abs/path)` so server paths never reach the client.
pub fn clean_cargo_stderr(stderr: &str) -> String {
    stderr
        .lines()
        .filter(|l| {
            !(l.starts_with("warning: profile package spec")
                && l.ends_with("did not match any packages"))
        })
        .map(|l| format!("{}\n", strip_status_path(l)))
        .collect()
}

/// `   Compiling x v0.1.0 (/abs/path)` -> `   Compiling x v0.1.0`; other lines unchanged.
fn strip_status_path(line: &str) -> &str {
    const STATUS: [&str; 6] = [
        "Compiling",
        "Checking",
        "Executable",
        "Running",
        "Fresh",
        "Documenting",
    ];
    let first = line.split_whitespace().next().unwrap_or("");
    if !STATUS.contains(&first) || !line.ends_with(')') {
        return line;
    }
    match line.rfind(" (/") {
        Some(pos) => &line[..pos],
        None => line,
    }
}

fn display_name(artifact: &TestArtifact, test: &str) -> String {
    // Integration test binaries (tests/*.rs) get their target name as prefix so names
    // stay unique across binaries.
    if artifact.kind == "test" {
        format!("{}::{}", artifact.target_name, test)
    } else {
        test.to_string()
    }
}

fn tail_lines(s: &str, n: usize) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].join("\n")
}

fn describe_exit(status: Option<std::process::ExitStatus>) -> String {
    use std::os::unix::process::ExitStatusExt;
    match status {
        Some(s) => match (s.code(), s.signal()) {
            (Some(c), _) => format!("çıkış kodu {c}"),
            (None, Some(sig)) => format!("sinyal {sig}"),
            _ => "bilinmeyen durum".into(),
        },
        None => "öldürüldü".into(),
    }
}

impl CargoRunner {
    async fn run_job(&self, job: &Job) -> RunOutcome {
        let started = Instant::now();
        let abs_prefix = format!("{}/", job.exercise_dir.display());
        let rel = |s: &str| relativize(s, &job.rel_prefix, &abs_prefix);
        let mut log = String::new();

        // --- Compile -------------------------------------------------------------------
        let mut cmd = Command::new("cargo");
        cmd.args([
            "test",
            "-p",
            &job.package,
            "--no-run",
            "--message-format=json",
            "--color=never",
        ])
        .current_dir(&job.workspace_dir)
        .env("CARGO_TARGET_DIR", job.workspace_dir.join("target"))
        .env("CARGO_TERM_COLOR", "never")
        .env("CARGO_TERM_PROGRESS_WHEN", "never");
        tracing::info!("compiling {} ({})", job.rel_prefix, job.package);
        let compiled = match run_captured(cmd, self.compile_timeout, CARGO_STDOUT_CAP).await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("failed to spawn cargo: {e}");
                return RunOutcome {
                    status: Status::CompileError,
                    tests: vec![],
                    tests_passed: 0,
                    tests_total: 0,
                    diagnostics: vec![],
                    log: format!("`cargo` çalıştırılamadı: {e}"),
                    duration_ms: started.elapsed().as_millis() as u64,
                };
            }
        };
        let parsed = parse_cargo_messages(
            &compiled.stdout_lossy(),
            &job.rel_prefix,
            &abs_prefix,
            &job.hidden,
        );
        log.push_str(&parsed.rendered);
        log.push_str(&rel(&clean_cargo_stderr(&compiled.stderr_lossy())));

        if compiled.timed_out {
            tracing::warn!("compile timeout for {}", job.rel_prefix);
            log.push_str(&format!(
                "\nDerleme {} saniye içinde bitmedi ve durduruldu.\n",
                self.compile_timeout.as_secs()
            ));
            return self.finish(started, Status::Timeout, vec![], parsed.diagnostics, log);
        }
        let build_ok =
            compiled.status.is_some_and(|s| s.success()) && parsed.build_success != Some(false);
        if !build_ok {
            return self.finish(
                started,
                Status::CompileError,
                vec![],
                parsed.diagnostics,
                log,
            );
        }

        // --- Test ----------------------------------------------------------------------
        let deadline = Instant::now() + self.test_timeout;
        let mut tests = Vec::new();
        let mut timed_out = false;
        for artifact in &parsed.artifacts {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                timed_out = true;
                break;
            }
            let (results, binary_timed_out) = self
                .run_binary(job, artifact, remaining, &mut log, &rel)
                .await;
            tests.extend(results);
            if binary_timed_out {
                timed_out = true;
                break;
            }
        }
        if timed_out {
            log.push_str(&format!(
                "\nTestler {} saniye içinde bitmedi ve durduruldu (sonsuz döngü olabilir mi?).\n",
                self.test_timeout.as_secs()
            ));
        }
        let status = compute_status(true, timed_out, &tests, job.expects_tests);
        self.finish(started, status, tests, parsed.diagnostics, log)
    }

    async fn run_binary(
        &self,
        job: &Job,
        artifact: &TestArtifact,
        timeout: Duration,
        log: &mut String,
        rel: &impl Fn(&str) -> String,
    ) -> (Vec<TestResult>, bool) {
        let start = Instant::now();
        let make_cmd = |args: &[&str]| {
            let mut c = Command::new(&artifact.executable);
            c.args(args)
                .current_dir(&job.exercise_dir)
                .env("CARGO_MANIFEST_DIR", &job.exercise_dir)
                .env("RUST_BACKTRACE", "0");
            c
        };
        let label = if artifact.kind == "test" {
            format!("tests/{}.rs", artifact.target_name)
        } else {
            format!("{} ({})", artifact.target_name, artifact.kind)
        };
        log.push_str(&format!("\n── {label} ──\n"));

        let listed = match run_captured(
            make_cmd(&["--list", "--format", "terse"]),
            timeout,
            TEST_OUTPUT_CAP,
        )
        .await
        {
            Ok(c) if c.status.is_some_and(|s| s.success()) => parse_test_list(&c.stdout_lossy()),
            Ok(c) if c.timed_out => {
                log.push_str("Test listesi alınamadı: zaman aşımı.\n");
                return (vec![], true);
            }
            Ok(c) => {
                log.push_str(&scrub_locations(&rel(&c.stderr_lossy()), &job.hidden));
                Vec::new()
            }
            Err(e) => {
                tracing::error!("failed to spawn {}: {e}", artifact.executable.display());
                log.push_str(&format!("Test dosyası çalıştırılamadı: {e}\n"));
                return (vec![], false);
            }
        };

        let remaining = timeout.saturating_sub(start.elapsed());
        let run =
            match run_captured(make_cmd(&["--color", "never"]), remaining, TEST_OUTPUT_CAP).await {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("failed to spawn {}: {e}", artifact.executable.display());
                    log.push_str(&format!("Test dosyası çalıştırılamadı: {e}\n"));
                    return (vec![], false);
                }
            };
        let stdout = scrub_locations(&rel(&run.stdout_lossy()), &job.hidden);
        let stderr = scrub_locations(&rel(&run.stderr_lossy()), &job.hidden);
        log.push_str(&stdout);
        if !stderr.is_empty() {
            log.push_str(&stderr);
        }
        if run.truncated {
            log.push_str("\n(Test çıktısı 1 MB sınırını aştı; fazlası atıldı.)\n");
        }
        let parsed = parse_libtest(&stdout);
        let by_name: HashMap<&str, TestOutcome> = parsed
            .results
            .iter()
            .map(|(n, o)| (n.as_str(), *o))
            .collect();

        // Prefer the list (knows tests that never reported); fall back to reported results.
        let names: Vec<String> = if listed.is_empty() {
            parsed.results.iter().map(|(n, _)| n.clone()).collect()
        } else {
            listed
        };
        let mut results = Vec::new();
        for name in names {
            let (passed, output) = match by_name.get(name.as_str()) {
                Some(TestOutcome::Ignored) => continue,
                Some(TestOutcome::Ok) => (true, String::new()),
                Some(TestOutcome::Failed) => (
                    false,
                    parsed
                        .failures
                        .get(&name)
                        .cloned()
                        .unwrap_or_else(|| "Test başarısız oldu.".into()),
                ),
                None if run.timed_out => (
                    false,
                    format!(
                        "Zaman aşımı: test {} saniye içinde bitmedi ve durduruldu.",
                        self.test_timeout.as_secs()
                    ),
                ),
                None => (
                    false,
                    format!(
                        "Test süreci beklenmedik şekilde sonlandı ({}).\n{}",
                        describe_exit(run.status),
                        tail_lines(&stderr, 40)
                    )
                    .trim_end()
                    .to_string(),
                ),
            };
            results.push(TestResult {
                name: display_name(artifact, &name),
                passed,
                output,
            });
        }
        (results, run.timed_out)
    }

    fn finish(
        &self,
        started: Instant,
        status: Status,
        tests: Vec<TestResult>,
        diagnostics: Vec<Diagnostic>,
        log: String,
    ) -> RunOutcome {
        let tests_passed = tests.iter().filter(|t| t.passed).count();
        let tests_total = tests.len();
        RunOutcome {
            status,
            tests,
            tests_passed,
            tests_total,
            diagnostics,
            log: truncate_log(&log, LOG_LIMIT),
            duration_ms: started.elapsed().as_millis() as u64,
        }
    }
}

/// Builds a [`Job`] for an exercise in the workspace.
pub fn job_for(
    workspace_dir: &Path,
    exercise_dir: PathBuf,
    rel_prefix: String,
    package: String,
    expects_tests: bool,
    hidden: HiddenLines,
) -> Job {
    Job {
        workspace_dir: workspace_dir.to_path_buf(),
        exercise_dir,
        rel_prefix,
        package,
        expects_tests,
        hidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIBTEST_FAIL: &str = "\nrunning 3 tests\ntest tests::by_zero - should panic ... FAILED\n\
test tests::case1 ... ok\ntest tests::skipped ... ignored, slow\ntest tests::other ... FAILED\n\nfailures:\n\n\
---- tests::by_zero stdout ----\n\nthread 'tests::by_zero' (16680) panicked at src/lib.rs:6:5:\nattempt to divide by zero\n\
note: panic did not contain expected string\n      panic message: \"attempt to divide by zero\"\n\
---- tests::other stdout ----\nnote: test did not panic as expected at src/lib.rs:20:8\n\n\
failures:\n    tests::by_zero\n    tests::other\n\n\
test result: FAILED. 1 passed; 2 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s\n\n";

    #[test]
    fn parses_libtest_results_and_failures() {
        let run = parse_libtest(LIBTEST_FAIL);
        assert_eq!(
            run.results,
            vec![
                ("tests::by_zero".to_string(), TestOutcome::Failed),
                ("tests::case1".to_string(), TestOutcome::Ok),
                ("tests::skipped".to_string(), TestOutcome::Ignored),
                ("tests::other".to_string(), TestOutcome::Failed),
            ]
        );
        let by_zero = &run.failures["tests::by_zero"];
        assert!(by_zero.starts_with("thread 'tests::by_zero'"));
        assert!(by_zero.ends_with("panic message: \"attempt to divide by zero\""));
        assert_eq!(
            run.failures["tests::other"],
            "note: test did not panic as expected at src/lib.rs:20:8"
        );
        assert_eq!(run.failures.len(), 2);
    }

    #[test]
    fn parses_passing_and_unfinished_runs() {
        let ok = "running 1 test\ntest t ... ok\n\ntest result: ok. 1 passed; 0 failed\n";
        assert_eq!(
            parse_libtest(ok).results,
            vec![("t".into(), TestOutcome::Ok)]
        );
        // Single-threaded run killed mid-test: dangling "test x ... " line.
        let hung = "running 2 tests\ntest a ... ok\ntest b ... ";
        assert_eq!(
            parse_libtest(hung).results,
            vec![("a".into(), TestOutcome::Ok)]
        );
    }

    #[test]
    fn parses_test_list() {
        let l = "tests::by_zero: test\ntests::case1: test\nbench_x: benchmark\n";
        assert_eq!(parse_test_list(l), vec!["tests::by_zero", "tests::case1"]);
    }

    fn msg(manifest: &str, level: &str, file: &str, expansion: &str) -> String {
        format!(
            r#"{{"reason":"compiler-message","manifest_path":"{manifest}","message":{{"rendered":"{level}: boom\n --> {file}:6:18\n","level":"{level}","message":"boom","spans":[{{"file_name":"{file}","is_primary":true,"line_start":6,"line_end":6,"column_start":18,"column_end":21,"expansion":{expansion}}}]}}}}"#
        )
    }

    #[test]
    fn maps_diagnostics_to_exercise_files() {
        let pfx = "exercises/02_basic_calculator/04_panics/";
        let abs = "/ws/exercises/02_basic_calculator/04_panics/";
        let mine = "/ws/exercises/02_basic_calculator/04_panics/Cargo.toml";
        let other = "/ws/helpers/common/Cargo.toml";
        let macro_exp = r#"{"span":{"file_name":"exercises/02_basic_calculator/04_panics/tests/it.rs","is_primary":true,"line_start":3,"line_end":3,"column_start":5,"column_end":9,"expansion":null}}"#;
        let stdout = [
            msg(mine, "error", "exercises/02_basic_calculator/04_panics/src/lib.rs", "null"),
            // duplicate from the `lib test` target
            msg(mine, "error", "exercises/02_basic_calculator/04_panics/src/lib.rs", "null"),
            msg(other, "warning", "helpers/common/src/lib.rs", "null"),
            msg(mine, "warning", "/rustc/abc/library/core/src/macros.rs", macro_exp),
            r#"{"reason":"compiler-message","manifest_path":"/ws/exercises/02_basic_calculator/04_panics/Cargo.toml","message":{"rendered":"error: aborting due to 1 previous error\n","level":"error","message":"aborting","spans":[]}}"#.to_string(),
            r#"{"reason":"compiler-artifact","manifest_path":"/ws/exercises/02_basic_calculator/04_panics/Cargo.toml","target":{"kind":["lib"],"name":"panics"},"profile":{"test":false},"executable":null}"#.to_string(),
            r#"{"reason":"compiler-artifact","manifest_path":"/ws/exercises/02_basic_calculator/04_panics/Cargo.toml","target":{"kind":["lib"],"name":"panics"},"profile":{"test":true},"executable":"/ws/target/debug/deps/panics-123"}"#.to_string(),
            r#"{"reason":"compiler-artifact","manifest_path":"/ws/exercises/02_basic_calculator/04_panics/Cargo.toml","target":{"kind":["test"],"name":"it"},"profile":{"test":true},"executable":"/ws/target/debug/deps/it-456"}"#.to_string(),
            r#"{"reason":"build-finished","success":false}"#.to_string(),
        ]
        .join("\n");
        let p = parse_cargo_messages(&stdout, pfx, abs, &HiddenLines::new());
        assert_eq!(p.build_success, Some(false));
        assert_eq!(p.diagnostics.len(), 2, "{:?}", p.diagnostics);
        let d = &p.diagnostics[0];
        assert_eq!(d.file, "src/lib.rs");
        assert_eq!(
            (d.line, d.column, d.end_line, d.end_column),
            (Some(6), Some(18), Some(6), Some(21))
        );
        assert!(!d.hidden);
        assert_eq!(d.severity, "error");
        assert_eq!(d.rendered, "error: boom\n --> src/lib.rs:6:18\n");
        let w = &p.diagnostics[1];
        assert_eq!(w.file, "tests/it.rs");
        assert_eq!((w.line, w.column), (Some(3), Some(5)));
        assert_eq!(w.severity, "warning");
        assert!(p.rendered.contains("aborting due to"));
        assert!(!p.rendered.contains("helpers"));
        assert_eq!(
            p.artifacts,
            vec![
                TestArtifact {
                    executable: "/ws/target/debug/deps/panics-123".into(),
                    target_name: "panics".into(),
                    kind: "lib".into()
                },
                TestArtifact {
                    executable: "/ws/target/debug/deps/it-456".into(),
                    target_name: "it".into(),
                    kind: "test".into()
                },
            ]
        );
    }

    #[test]
    fn maps_hidden_diagnostics() {
        let pfx = "exercises/02_basic_calculator/04_panics/";
        let abs = "/ws/exercises/02_basic_calculator/04_panics/";
        let mine = "/ws/exercises/02_basic_calculator/04_panics/Cargo.toml";
        let in_test = r#"{"reason":"compiler-message","manifest_path":"MANIFEST","message":{"rendered":"error[E0308]: mismatched types\n  --> exercises/02_basic_calculator/04_panics/src/lib.rs:30:20\n   |\n30 |         assert_eq!(speed(1, 2, 3), 4);\n   |                    ^^^^^ expected `u32`\n   |\n   = note: expected type `u32`\n              found type `i8`\n\n","level":"error","message":"mismatched types","spans":[{"file_name":"exercises/02_basic_calculator/04_panics/src/lib.rs","is_primary":true,"line_start":30,"line_end":30,"column_start":20,"column_end":25,"expansion":null}]}}"#.replace("MANIFEST", mine);
        let visible_with_hidden_note = r#"{"reason":"compiler-message","manifest_path":"MANIFEST","message":{"rendered":"error[E0061]: wrong args\n --> exercises/02_basic_calculator/04_panics/src/lib.rs:3:5\n  |\n3 |     f(1)\n  |     ^\n  |\nnote: called here\n --> exercises/02_basic_calculator/04_panics/src/lib.rs:31:9\n   |\n31 |         f(1, 2)\n   |         ^^^^\n","level":"error","message":"wrong args","spans":[{"file_name":"exercises/02_basic_calculator/04_panics/src/lib.rs","is_primary":true,"line_start":3,"line_end":3,"column_start":5,"column_end":6,"expansion":null}]}}"#.replace("MANIFEST", mine);
        let hidden = HiddenLines::from([("src/lib.rs".to_string(), 20)]);
        let in_test_again = in_test.replace("30:20", "31:20").replace(
            "\"line_start\":30,\"line_end\":30",
            "\"line_start\":31,\"line_end\":31",
        );
        let p = parse_cargo_messages(
            &[in_test.as_str(), &in_test_again, &visible_with_hidden_note].join("\n"),
            pfx,
            abs,
            &hidden,
        );
        assert_eq!(p.diagnostics.len(), 2);
        let h = &p.diagnostics[0];
        assert!(h.hidden);
        assert_eq!(h.file, "src/lib.rs");
        assert_eq!(
            (h.line, h.column, h.end_line, h.end_column),
            (None, None, None, None)
        );
        assert_eq!(
            h.rendered,
            "error[E0308]: mismatched types\n   = note: expected type `u32`\n              found type `i8`\n"
        );
        let v = &p.diagnostics[1];
        assert!(!v.hidden);
        assert_eq!(v.line, Some(3));
        assert!(v.rendered.contains("3 |     f(1)"), "{}", v.rendered);
        assert!(!v.rendered.contains("f(1, 2)"), "{}", v.rendered);
        assert!(
            v.rendered.contains("--> (gizli test kodu)"),
            "{}",
            v.rendered
        );
        assert!(!p.rendered.contains("speed(1, 2, 3)"));
        // Serialized form: null positions, `hidden` flag.
        let j = serde_json::to_value(h).unwrap();
        assert!(j["line"].is_null() && j["hidden"] == true);
    }

    fn t(passed: bool) -> TestResult {
        TestResult {
            name: "x".into(),
            passed,
            output: String::new(),
        }
    }

    #[test]
    fn status_rules() {
        assert_eq!(
            compute_status(true, true, &[t(true)], true),
            Status::Timeout
        );
        assert_eq!(
            compute_status(false, false, &[], true),
            Status::CompileError
        );
        assert_eq!(compute_status(true, false, &[], true), Status::Failed);
        assert_eq!(compute_status(true, false, &[], false), Status::Passed);
        assert_eq!(
            compute_status(true, false, &[t(true), t(true)], true),
            Status::Passed
        );
        assert_eq!(
            compute_status(true, false, &[t(true), t(false)], true),
            Status::Failed
        );
        assert_eq!(
            compute_status(true, false, &[t(false)], false),
            Status::Failed
        );
    }

    #[test]
    fn log_truncation() {
        let s = "ç".repeat(50_000); // 100 KB
        let out = truncate_log(&s, LOG_LIMIT);
        assert!(out.len() <= LOG_LIMIT);
        assert!(out.contains("kısaltıldı"));
        assert_eq!(truncate_log("short", 10), "short");
    }

    #[test]
    fn filters_profile_warning() {
        let e = "warning: profile package spec `copy` in profile `dev` did not match any packages\n   Compiling x v0.1.0\n";
        assert_eq!(clean_cargo_stderr(e), "   Compiling x v0.1.0\n");
    }

    #[test]
    fn strips_absolute_paths_from_status_lines() {
        let e = "   Compiling ticket_fields v0.1.0 (/repo/helpers/ticket_fields)\n\
                 \x20  Compiling panics v0.1.0 (/ws/exercises/02/04_panics)\n\
                 \x20 Executable unittests src/lib.rs (/ws/target/debug/deps/panics-abc)\n\
                 error: value (/not a status line)\n";
        assert_eq!(
            clean_cargo_stderr(e),
            "   Compiling ticket_fields v0.1.0\n   Compiling panics v0.1.0\n  \
             Executable unittests src/lib.rs\nerror: value (/not a status line)\n"
        );
    }

    #[test]
    fn relativizes_paths() {
        assert_eq!(
            relativize(
                "panicked at exercises/a/b/src/lib.rs:1:1 and /ws/exercises/a/b/tests/x.rs",
                "exercises/a/b/",
                "/ws/exercises/a/b/"
            ),
            "panicked at src/lib.rs:1:1 and tests/x.rs"
        );
    }
}
