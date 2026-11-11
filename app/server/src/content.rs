//! Book content: table of contents from `book/src/SUMMARY.md`, Turkish/English page lookup.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// One entry of SUMMARY.md.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryPage {
    /// Path under `book/src` without `.md`, e.g. `02_basic_calculator/04_panics`.
    pub id: String,
    /// English title as written in SUMMARY.md.
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryChapter {
    /// Directory of the first page (`02_basic_calculator`), or the page id for
    /// root-level pages (`going_further`).
    pub id: String,
    pub pages: Vec<SummaryPage>,
}

/// Parses mdBook's SUMMARY.md. A list item without indentation starts a new chapter;
/// indented items are pages of the current chapter.
pub fn parse_summary(text: &str) -> Vec<SummaryChapter> {
    let mut chapters: Vec<SummaryChapter> = Vec::new();
    for line in text.lines() {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim_start();
        let rest = match trimmed
            .strip_prefix("- [")
            .or_else(|| trimmed.strip_prefix("* ["))
        {
            Some(r) => r,
            None => continue,
        };
        let Some(close) = rest.find("](") else {
            continue;
        };
        let title = rest[..close].to_string();
        let after = &rest[close + 2..];
        let Some(end) = after.find(')') else { continue };
        let link = after[..end].trim();
        let Some(id) = link.strip_suffix(".md") else {
            continue;
        };
        let id = id.trim_start_matches("./").to_string();
        if id.is_empty() {
            continue;
        }
        let page = SummaryPage { id, title };
        if indent == 0 || chapters.is_empty() {
            let chapter_id = match page.id.rsplit_once('/') {
                Some((dir, _)) => dir.to_string(),
                None => page.id.clone(),
            };
            chapters.push(SummaryChapter {
                id: chapter_id,
                pages: vec![page],
            });
        } else {
            chapters.last_mut().expect("non-empty").pages.push(page);
        }
    }
    chapters
}

/// Extracts the first `# ` heading, dropping a trailing `{#slug}` attribute.
/// Backticks are preserved.
pub fn first_heading(markdown: &str) -> Option<String> {
    let mut in_fence = false;
    for line in markdown.lines() {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some(h) = line.strip_prefix("# ") {
            let mut h = h.trim();
            if h.ends_with('}') {
                if let Some(pos) = h.rfind("{#") {
                    h = h[..pos].trim_end();
                }
            }
            if !h.is_empty() {
                return Some(h.to_string());
            }
        }
    }
    None
}

/// A page or exercise id segment: ASCII letters, digits, `_` and `-` only.
pub fn is_valid_segment(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 100
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

#[derive(Debug, Clone)]
pub struct Content {
    pub root: PathBuf,
    /// `tr/removed-exercises.json`, re-read when its mtime changes.
    removed: Arc<Mutex<RemovedCache>>,
}

#[derive(Debug, Default)]
struct RemovedCache {
    mtime: Option<SystemTime>,
    ids: Arc<HashSet<String>>,
}

/// Parses `{ "removed": ["01_intro/00_welcome", ...] }`.
pub fn parse_removed(text: &str) -> Result<HashSet<String>, String> {
    #[derive(serde::Deserialize)]
    struct File {
        #[serde(default)]
        removed: Vec<String>,
    }
    let f: File = serde_json::from_str(text).map_err(|e| e.to_string())?;
    Ok(f.removed
        .into_iter()
        .map(|s| s.trim().trim_matches('/').to_string())
        .collect())
}

/// An app-specific note shown on a page (`tr/notes/<page_id>.json`).
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct PageNote {
    /// Heading id the note is placed before; `None` = top of the page (after the H1).
    #[serde(default)]
    pub before: Option<String>,
    pub markdown: String,
}

/// Parses `{ "notes": [ { "before": "<heading id>" | null, "markdown": "..." } ] }`.
pub fn parse_notes(text: &str) -> Result<Vec<PageNote>, String> {
    #[derive(serde::Deserialize)]
    struct File {
        #[serde(default)]
        notes: Vec<PageNote>,
    }
    let f: File = serde_json::from_str(text).map_err(|e| e.to_string())?;
    Ok(f.notes)
}

#[derive(Debug, Clone)]
pub struct PageInfo {
    pub id: String,
    pub title: String,
    pub translated: bool,
    pub has_exercise: bool,
}

#[derive(Debug, Clone)]
pub struct ChapterInfo {
    pub id: String,
    pub title: String,
    pub pages: Vec<PageInfo>,
}

impl Content {
    pub fn new(root: PathBuf) -> Self {
        Content {
            root,
            removed: Arc::default(),
        }
    }

    /// Exercise ids listed in `tr/removed-exercises.json` (missing file = none).
    pub fn removed(&self) -> Arc<HashSet<String>> {
        let path = self.root.join("tr/removed-exercises.json");
        let mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        let mut cache = self.removed.lock().unwrap_or_else(|e| e.into_inner());
        if mtime.is_none() {
            cache.mtime = None;
            cache.ids = Arc::default();
            return cache.ids.clone();
        }
        if cache.mtime != mtime {
            let ids = match std::fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|t| parse_removed(&t))
            {
                Ok(ids) => {
                    tracing::info!("{} removed exercise(s) in {}", ids.len(), path.display());
                    ids
                }
                Err(e) => {
                    tracing::warn!("ignoring invalid {}: {e}", path.display());
                    HashSet::new()
                }
            };
            cache.mtime = mtime;
            cache.ids = Arc::new(ids);
        }
        cache.ids.clone()
    }

    pub fn tr_page_path(&self, page_id: &str) -> PathBuf {
        self.root.join("tr/book").join(format!("{page_id}.md"))
    }

    pub fn en_page_path(&self, page_id: &str) -> PathBuf {
        self.root.join("book/src").join(format!("{page_id}.md"))
    }

    pub fn notes_path(&self, page_id: &str) -> PathBuf {
        self.root.join("tr/notes").join(format!("{page_id}.json"))
    }

    /// App notes for a page, read on every call. Missing file = none; an unreadable or
    /// invalid file is logged and treated as none so the page still loads.
    pub fn page_notes(&self, page_id: &str) -> Vec<PageNote> {
        let path = self.notes_path(page_id);
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return vec![],
            Err(e) => {
                tracing::warn!("cannot read {}: {e}", path.display());
                return vec![];
            }
        };
        parse_notes(&text).unwrap_or_else(|e| {
            tracing::warn!("ignoring invalid {}: {e}", path.display());
            vec![]
        })
    }

    pub fn exercise_source_dir(&self, chapter: &str, exercise: &str) -> PathBuf {
        self.root.join("exercises").join(chapter).join(exercise)
    }

    pub fn exercise_tr_dir(&self, chapter: &str, exercise: &str) -> PathBuf {
        self.root.join("tr/exercises").join(chapter).join(exercise)
    }

    /// Whether the exercise crate exists in `exercises/` (removed or not).
    pub fn exercise_source_exists(&self, chapter: &str, exercise: &str) -> bool {
        is_valid_segment(chapter)
            && is_valid_segment(exercise)
            && self
                .exercise_source_dir(chapter, exercise)
                .join("Cargo.toml")
                .is_file()
    }

    /// Whether the exercise is available in the app (exists and is not removed).
    pub fn exercise_exists(&self, chapter: &str, exercise: &str) -> bool {
        self.exercise_source_exists(chapter, exercise)
            && !self.removed().contains(&format!("{chapter}/{exercise}"))
    }

    fn page_has_exercise(&self, page_id: &str, removed: &HashSet<String>) -> bool {
        match page_id.split_once('/') {
            Some((ch, ex)) => self.exercise_source_exists(ch, ex) && !removed.contains(page_id),
            None => false,
        }
    }

    pub fn read_summary(&self) -> std::io::Result<Vec<SummaryChapter>> {
        let text = std::fs::read_to_string(self.root.join("book/src/SUMMARY.md"))?;
        Ok(parse_summary(&text))
    }

    /// Builds the table of contents with Turkish titles where available.
    /// Files are read from disk on every call so newly translated pages show up immediately.
    pub fn toc(&self) -> std::io::Result<Vec<ChapterInfo>> {
        let summary = self.read_summary()?;
        let removed = self.removed();
        let mut out = Vec::with_capacity(summary.len());
        for ch in summary {
            let pages: Vec<PageInfo> = ch
                .pages
                .iter()
                .map(|p| {
                    let tr = read_if_exists(&self.tr_page_path(&p.id));
                    let title = tr
                        .as_deref()
                        .and_then(first_heading)
                        .unwrap_or_else(|| p.title.clone());
                    PageInfo {
                        id: p.id.clone(),
                        title,
                        translated: tr.is_some(),
                        has_exercise: self.page_has_exercise(&p.id, &removed),
                    }
                })
                .collect();
            let title = pages.first().map(|p| p.title.clone()).unwrap_or_default();
            out.push(ChapterInfo {
                id: ch.id,
                title,
                pages,
            });
        }
        Ok(out)
    }

    /// Returns (markdown, translated) for a page, Turkish first.
    pub fn page_markdown(&self, page_id: &str) -> std::io::Result<(String, bool)> {
        if let Some(md) = read_if_exists(&self.tr_page_path(page_id)) {
            return Ok((md, true));
        }
        Ok((std::fs::read_to_string(self.en_page_path(page_id))?, false))
    }
}

fn read_if_exists(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_summary_structure() {
        let s = "# Summary\n\n- [Welcome](01_intro/00_welcome.md)\n  - [Syntax](01_intro/01_syntax.md)\n\n\
                 - [A Basic Calculator](02_basic_calculator/00_intro.md)\n  - [Branching: `if`/`else`](02_basic_calculator/03_if_else.md)\n\n\
                 * [Going further](going_further.md)\n";
        let ch = parse_summary(s);
        assert_eq!(ch.len(), 3);
        assert_eq!(ch[0].id, "01_intro");
        assert_eq!(ch[0].pages.len(), 2);
        assert_eq!(ch[0].pages[1].id, "01_intro/01_syntax");
        assert_eq!(ch[1].pages[1].title, "Branching: `if`/`else`");
        assert_eq!(ch[2].id, "going_further");
        assert_eq!(ch[2].pages[0].id, "going_further");
    }

    #[test]
    fn parses_real_summary() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let c = Content::new(root);
        let toc = c.toc().unwrap();
        let total: usize = toc
            .iter()
            .flat_map(|c| &c.pages)
            .filter(|p| p.has_exercise)
            .count();
        let removed = c.removed();
        assert_eq!(total, 98 - removed.len());
        // Every listed id is a real exercise, and removed ones are reported as such.
        for id in removed.iter() {
            let (ch, ex) = id.split_once('/').expect("chapter/exercise");
            assert!(c.exercise_source_exists(ch, ex), "{id}");
            assert!(!c.exercise_exists(ch, ex), "{id}");
            let page = toc.iter().flat_map(|c| &c.pages).find(|p| &p.id == id);
            assert!(page.is_some_and(|p| !p.has_exercise), "{id}");
        }
        assert!(c.exercise_exists("02_basic_calculator", "04_panics"));
        assert_eq!(toc.last().unwrap().id, "going_further");
    }

    #[test]
    fn removed_file_parsing_and_reload() {
        assert_eq!(
            parse_removed(r#"{ "removed": ["a/b", " c/d/ "] }"#).unwrap(),
            HashSet::from(["a/b".to_string(), "c/d".to_string()])
        );
        assert!(parse_removed("{}").unwrap().is_empty());
        assert!(parse_removed("[").is_err());

        let tmp = std::env::temp_dir().join(format!("removed-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("tr")).unwrap();
        std::fs::create_dir_all(tmp.join("exercises/ch/ex")).unwrap();
        std::fs::write(tmp.join("exercises/ch/ex/Cargo.toml"), "").unwrap();
        let c = Content::new(tmp.clone());
        assert!(c.exercise_exists("ch", "ex")); // missing file = nothing removed
        let f = tmp.join("tr/removed-exercises.json");
        std::fs::write(&f, r#"{ "removed": ["ch/ex"] }"#).unwrap();
        assert!(!c.exercise_exists("ch", "ex"));
        // A newer mtime is picked up.
        std::fs::write(&f, r#"{ "removed": [] }"#).unwrap();
        let later = SystemTime::now() + std::time::Duration::from_secs(5);
        std::fs::File::options()
            .write(true)
            .open(&f)
            .unwrap()
            .set_modified(later)
            .unwrap();
        assert!(c.exercise_exists("ch", "ex"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn notes_loading() {
        let tmp = std::env::temp_dir().join(format!("notes-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("tr/notes/ch")).unwrap();
        let c = Content::new(tmp.clone());
        // Missing file.
        assert!(c.page_notes("ch/missing").is_empty());
        // Present file, `before` as string, null and omitted.
        std::fs::write(
            tmp.join("tr/notes/ch/page.json"),
            r#"{ "notes": [ { "before": "structure", "markdown": "**a**" },
                            { "before": null, "markdown": "b" },
                            { "markdown": "c" } ] }"#,
        )
        .unwrap();
        assert_eq!(
            c.page_notes("ch/page"),
            vec![
                PageNote {
                    before: Some("structure".into()),
                    markdown: "**a**".into()
                },
                PageNote {
                    before: None,
                    markdown: "b".into()
                },
                PageNote {
                    before: None,
                    markdown: "c".into()
                },
            ]
        );
        // Invalid JSON and wrong shape are ignored.
        std::fs::write(tmp.join("tr/notes/ch/bad.json"), "{ \"notes\": [").unwrap();
        assert!(c.page_notes("ch/bad").is_empty());
        std::fs::write(
            tmp.join("tr/notes/ch/shape.json"),
            r#"{ "notes": [ { "before": 1 } ] }"#,
        )
        .unwrap();
        assert!(c.page_notes("ch/shape").is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn real_welcome_notes_parse() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let c = Content::new(root.clone());
        if c.notes_path("01_intro/00_welcome").is_file() {
            assert!(!c.page_notes("01_intro/00_welcome").is_empty());
        }
    }

    #[test]
    fn heading_extraction() {
        assert_eq!(
            first_heading("# Panic'ler {#panics}\n\ntext").as_deref(),
            Some("Panic'ler")
        );
        assert_eq!(
            first_heading("intro\n```rust\n# hidden\n```\n# `Deref` trait\n").as_deref(),
            Some("`Deref` trait")
        );
        assert_eq!(first_heading("## Not h1\n"), None);
    }

    #[test]
    fn segment_validation() {
        assert!(is_valid_segment("02_basic_calculator"));
        assert!(!is_valid_segment(".."));
        assert!(!is_valid_segment("a/b"));
        assert!(!is_valid_segment(""));
    }
}
