//! `tr-check`: verifies that the Turkish translation under `tr/` keeps everything
//! that must stay identical to the original `book/` and `exercises/` (see `tr/STYLE.md`).
//!
//! Usage: `cargo run -q -p tr-check -- [--root <dir>] [<chapter>...]`

mod issue;
mod md;
mod rs;
mod slug;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use issue::{Issue, Severity};

const USAGE: &str = "\
Usage: tr-check [--root <dir>] [<chapter>...]

Checks translations under <root>/tr against <root>/book/src and <root>/exercises.
Without chapters, every chapter that has something under tr/ is checked.
A chapter is a directory name (e.g. 02_basic_calculator) or the stem of a
markdown file at the book root (e.g. going_further).

Exit status: 0 = no errors, 1 = errors found, 2 = usage error.";

struct Args {
    root: Option<PathBuf>,
    chapters: Vec<String>,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        root: None,
        chapters: Vec::new(),
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            "--root" => {
                let v = it.next().ok_or("--root needs a value")?;
                args.root = Some(PathBuf::from(v));
            }
            _ if a.starts_with("--root=") => {
                args.root = Some(PathBuf::from(&a["--root=".len()..]));
            }
            _ if a.starts_with('-') => return Err(format!("unknown option `{a}`")),
            _ => args.chapters.push(a.trim_end_matches('/').to_string()),
        }
    }
    Ok(args)
}

fn is_repo_root(p: &Path) -> bool {
    p.join("book/src").is_dir() && p.join("exercises").is_dir()
}

fn find_root() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let from_cwd = cwd.ancestors().find(|p| is_repo_root(p));
    let from_manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| is_repo_root(p));
    from_cwd.or(from_manifest).map(Path::to_path_buf)
}

/// All files under `dir`, as `/`-separated paths relative to `dir`, sorted.
/// Directories named `target` are skipped.
fn walk(dir: &Path) -> BTreeSet<String> {
    fn go(base: &Path, dir: &Path, out: &mut BTreeSet<String>) {
        let Ok(rd) = fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_dir() {
                if e.file_name() != "target" {
                    go(base, &p, out);
                }
            } else if let Ok(rel) = p.strip_prefix(base) {
                let rel: Vec<_> = rel.iter().map(|c| c.to_string_lossy()).collect();
                out.insert(rel.join("/"));
            }
        }
    }
    let mut out = BTreeSet::new();
    go(dir, dir, &mut out);
    out
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Markdown,
    Rust,
}

/// One original file and where its translation must live (both relative to the root).
struct Pair {
    orig: String,
    tr: String,
    kind: Kind,
}

struct Report {
    /// Keyed by translation path so output is grouped and sorted.
    issues: BTreeMap<String, Vec<Issue>>,
    checked: usize,
    missing: usize,
}

impl Report {
    fn push(&mut self, path: &str, issue: Issue) {
        self.issues.entry(path.to_string()).or_default().push(issue);
    }
}

struct Repo {
    root: PathBuf,
}

impl Repo {
    fn book_dir_chapters(&self) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        for (dir, root_md) in [("book/src", true), ("exercises", false)] {
            let Ok(rd) = fs::read_dir(self.root.join(dir)) else {
                continue;
            };
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                let p = e.path();
                if p.is_dir() {
                    out.insert(name);
                } else if root_md && name.ends_with(".md") {
                    out.insert(name.trim_end_matches(".md").to_string());
                }
            }
        }
        out
    }

    /// Chapter names that appear under tr/book or tr/exercises.
    fn tr_chapters(&self) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        for dir in ["tr/book", "tr/exercises"] {
            let Ok(rd) = fs::read_dir(self.root.join(dir)) else {
                continue;
            };
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if e.path().is_dir() {
                    out.insert(name);
                } else if dir == "tr/book" && name.ends_with(".md") {
                    out.insert(name.trim_end_matches(".md").to_string());
                } else {
                    out.insert(name);
                }
            }
        }
        out
    }

    /// Expected translation files of a chapter, plus translated files with no original.
    fn chapter_pairs(&self, ch: &str) -> (Vec<Pair>, Vec<String>) {
        let mut pairs = Vec::new();
        let mut extra = Vec::new();

        let book_dir = self.root.join("book/src").join(ch);
        let book_md = self.root.join("book/src").join(format!("{ch}.md"));
        if book_dir.is_dir() {
            let origs: BTreeSet<String> = walk(&book_dir)
                .into_iter()
                .filter(|f| f.ends_with(".md"))
                .collect();
            for f in &origs {
                pairs.push(Pair {
                    orig: format!("book/src/{ch}/{f}"),
                    tr: format!("tr/book/{ch}/{f}"),
                    kind: Kind::Markdown,
                });
            }
            for f in walk(&self.root.join("tr/book").join(ch)) {
                if !origs.contains(&f) {
                    extra.push(format!("tr/book/{ch}/{f}"));
                }
            }
        } else if self.root.join("tr/book").join(ch).is_dir() {
            for f in walk(&self.root.join("tr/book").join(ch)) {
                extra.push(format!("tr/book/{ch}/{f}"));
            }
        }
        if book_md.is_file() {
            pairs.push(Pair {
                orig: format!("book/src/{ch}.md"),
                tr: format!("tr/book/{ch}.md"),
                kind: Kind::Markdown,
            });
        } else if self.root.join("tr/book").join(format!("{ch}.md")).is_file() {
            extra.push(format!("tr/book/{ch}.md"));
        }

        let ex_dir = self.root.join("exercises").join(ch);
        let origs: BTreeSet<String> = if ex_dir.is_dir() {
            walk(&ex_dir)
                .into_iter()
                .filter(|f| f.ends_with(".rs"))
                .collect()
        } else {
            BTreeSet::new()
        };
        for f in &origs {
            pairs.push(Pair {
                orig: format!("exercises/{ch}/{f}"),
                tr: format!("tr/exercises/{ch}/{f}"),
                kind: Kind::Rust,
            });
        }
        let tr_ex = self.root.join("tr/exercises").join(ch);
        if tr_ex.is_file() {
            extra.push(format!("tr/exercises/{ch}"));
        }
        for f in walk(&tr_ex) {
            if !origs.contains(&f) {
                extra.push(format!("tr/exercises/{ch}/{f}"));
            }
        }
        (pairs, extra)
    }

    fn read(&self, rel: &str) -> Result<String, String> {
        fs::read_to_string(self.root.join(rel)).map_err(|e| format!("cannot read {rel}: {e}"))
    }

    fn check_chapter(&self, ch: &str, report: &mut Report) {
        let (pairs, extra) = self.chapter_pairs(ch);
        for e in extra {
            report.push(
                &e,
                Issue::error(
                    None,
                    "translated file has no original (wrong name or path?)",
                ),
            );
        }
        for p in pairs {
            if !self.root.join(&p.tr).is_file() {
                report.missing += 1;
                report.push(
                    &p.tr,
                    Issue::error(None, format!("missing translation of {}", p.orig)),
                );
                continue;
            }
            let (orig, tr) = match (self.read(&p.orig), self.read(&p.tr)) {
                (Ok(o), Ok(t)) => (o, t),
                (Err(e), _) | (_, Err(e)) => {
                    report.push(&p.tr, Issue::error(None, e));
                    continue;
                }
            };
            report.checked += 1;
            let issues = match p.kind {
                Kind::Markdown => md::check(&orig, &tr),
                Kind::Rust => rs::check(&orig, &tr),
            };
            for i in issues {
                report.push(&p.tr, i);
            }
        }
    }
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("tr-check: {e}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let root = match args.root.or_else(find_root) {
        Some(r) if is_repo_root(&r) => r,
        Some(r) => {
            eprintln!(
                "tr-check: {} does not look like the repo root (no book/src or exercises/)",
                r.display()
            );
            return ExitCode::from(2);
        }
        None => {
            eprintln!("tr-check: cannot find the repo root; pass --root <dir>");
            return ExitCode::from(2);
        }
    };
    let repo = Repo { root };
    let known = repo.book_dir_chapters();
    let mut report = Report {
        issues: BTreeMap::new(),
        checked: 0,
        missing: 0,
    };

    let chapters: Vec<String> = if args.chapters.is_empty() {
        let mut v = Vec::new();
        for ch in repo.tr_chapters() {
            if known.contains(&ch) {
                v.push(ch);
            } else {
                let path = if repo.root.join("tr/book").join(&ch).exists() {
                    format!("tr/book/{ch}")
                } else {
                    format!("tr/exercises/{ch}")
                };
                report.push(
                    &path,
                    Issue::error(None, "no chapter with this name in book/src or exercises/"),
                );
            }
        }
        v
    } else {
        for ch in &args.chapters {
            if !known.contains(ch) {
                eprintln!("tr-check: unknown chapter `{ch}`");
                return ExitCode::from(2);
            }
        }
        args.chapters.clone()
    };

    for ch in &chapters {
        repo.check_chapter(ch, &mut report);
    }

    let (mut errors, mut warnings) = (0, 0);
    for (path, issues) in &report.issues {
        for i in issues {
            match i.severity {
                Severity::Error => errors += 1,
                Severity::Warning => warnings += 1,
            }
            match i.line {
                Some(l) => println!("{}: {path}:{l}: {}", i.severity, i.msg),
                None => println!("{}: {path}: {}", i.severity, i.msg),
            }
        }
    }
    println!(
        "tr-check: {} chapter(s), {} file(s) checked, {} missing, {errors} error(s), {warnings} warning(s)",
        chapters.len(),
        report.checked,
        report.missing,
    );
    if errors > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
