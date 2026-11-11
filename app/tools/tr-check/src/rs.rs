//! Comment-insensitive comparison of Rust source code.
//!
//! Both sides are lexed with `rustc_lexer`; whitespace and every kind of comment
//! (`//`, `///`, `//!`, `/* */`, `/** */`, `/*! */`) are dropped and the remaining
//! tokens must be identical in kind and text.

use ra_ap_rustc_lexer::{tokenize, FrontmatterAllowed, TokenKind};

use crate::issue::Issue;

#[derive(Debug)]
pub struct Tok<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    /// 1-based line of the token's first character.
    pub line: usize,
}

#[derive(Debug)]
pub struct Lexed<'a> {
    pub tokens: Vec<Tok<'a>>,
    /// Number of comments whose text contains `TODO`.
    pub todo_comments: usize,
    /// Number of lines in the source (at least 1).
    pub lines: usize,
}

pub fn lex(src: &str) -> Lexed<'_> {
    let mut tokens = Vec::new();
    let mut todo_comments = 0;
    let mut pos = 0usize;
    let mut line = 1usize;
    for t in tokenize(src, FrontmatterAllowed::No) {
        let len = t.len as usize;
        let text = &src[pos..pos + len];
        match t.kind {
            TokenKind::Whitespace => {}
            TokenKind::LineComment { .. } | TokenKind::BlockComment { .. } => {
                if text.contains("TODO") {
                    todo_comments += 1;
                }
            }
            kind => tokens.push(Tok { kind, text, line }),
        }
        line += text.bytes().filter(|&b| b == b'\n').count();
        pos += len;
    }
    Lexed {
        tokens,
        todo_comments,
        lines: line,
    }
}

/// First difference between two token streams. Lines are 1-based and relative to
/// the compared sources; a `None` token means that side ended.
#[derive(Debug, PartialEq, Eq)]
pub struct Mismatch {
    pub index: usize,
    pub orig_line: usize,
    pub tr_line: usize,
    pub orig_tok: Option<String>,
    pub tr_tok: Option<String>,
}

impl Mismatch {
    pub fn describe(&self) -> String {
        let show = |t: &Option<String>| match t {
            Some(s) => format!("`{}`", abbreviate(s)),
            None => "end of code".to_string(),
        };
        format!(
            "code differs outside comments (token #{}): original line {} has {}, translation line {} has {}",
            self.index + 1,
            self.orig_line,
            show(&self.orig_tok),
            self.tr_line,
            show(&self.tr_tok),
        )
    }
}

fn abbreviate(s: &str) -> String {
    let flat: String = s
        .chars()
        .flat_map(|c| match c {
            '\n' => vec!['\\', 'n'],
            '\t' => vec!['\\', 't'],
            c => vec![c],
        })
        .collect();
    if flat.chars().count() > 60 {
        let head: String = flat.chars().take(57).collect();
        format!("{head}...")
    } else {
        flat
    }
}

pub struct RsReport {
    pub mismatch: Option<Mismatch>,
    pub orig_todo: usize,
    pub tr_todo: usize,
}

pub fn compare(orig: &str, tr: &str) -> RsReport {
    let a = lex(orig);
    let b = lex(tr);
    let mut mismatch = None;
    let n = a.tokens.len().max(b.tokens.len());
    for i in 0..n {
        let (x, y) = (a.tokens.get(i), b.tokens.get(i));
        let same = match (x, y) {
            (Some(x), Some(y)) => x.kind == y.kind && x.text == y.text,
            _ => false,
        };
        if !same {
            mismatch = Some(Mismatch {
                index: i,
                orig_line: x.map_or(a.lines, |t| t.line),
                tr_line: y.map_or(b.lines, |t| t.line),
                orig_tok: x.map(|t| t.text.to_string()),
                tr_tok: y.map(|t| t.text.to_string()),
            });
            break;
        }
    }
    RsReport {
        mismatch,
        orig_todo: a.todo_comments,
        tr_todo: b.todo_comments,
    }
}

/// Checks a translated `.rs` file against its original.
pub fn check(orig: &str, tr: &str) -> Vec<Issue> {
    let r = compare(orig, tr);
    let mut issues = Vec::new();
    if let Some(m) = &r.mismatch {
        issues.push(Issue::error(Some(m.tr_line), m.describe()));
    }
    if r.orig_todo != r.tr_todo {
        issues.push(Issue::warning(
            None,
            format!(
                "original has {} comment(s) containing `TODO`, translation has {}",
                r.orig_todo, r.tr_todo
            ),
        ));
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORIG: &str = r#"// TODO: Implement the `speed` function.
/// Returns the speed.
fn speed(start: u32, end: u32, time_elapsed: u32) -> u32 {
    /* distance */ let d = end - start;
    //! weird but lexes
    if time_elapsed == 0 {
        panic!("The journey took no time at all, that's impossible!");
    }
    d / time_elapsed // 👇 here
}

#[test]
#[should_panic(expected = "The journey took no time at all, that's impossible!")]
fn by_zero() {
    speed(0, 10, 0);
}
"#;

    #[test]
    fn identical_passes() {
        let r = compare(ORIG, ORIG);
        assert!(r.mismatch.is_none());
        assert_eq!(r.orig_todo, 1);
        assert_eq!(r.tr_todo, 1);
    }

    #[test]
    fn translated_comments_pass() {
        let tr = ORIG
            .replace(
                "// TODO: Implement the `speed` function.",
                "// TODO: `speed` fonksiyonunu implement et.",
            )
            .replace("/// Returns the speed.", "/// Hızı döndürür.")
            .replace("/* distance */", "/** mesafe, çok satırlı\n   bir yorum */")
            .replace("// 👇 here", "// 👇 burada");
        let r = compare(ORIG, &tr);
        assert!(r.mismatch.is_none(), "{:?}", r.mismatch);
        assert_eq!(r.tr_todo, 1);
    }

    #[test]
    fn moved_comment_line_breaks_pass() {
        let orig = "// A long comment that\n// spans two lines.\nfn f() {}\n";
        let tr =
            "// Tek satırda uzun bir yorum.\n\n\n// Ya da üç\n// satır\n// olabilir.\nfn f() {}\n";
        assert!(compare(orig, tr).mismatch.is_none());
        let tr = "fn f() {} // Satır sonunda yorum.\n";
        assert!(compare(orig, tr).mismatch.is_none());
    }

    #[test]
    fn changed_string_literal_fails() {
        let tr = ORIG.replacen(
            "\"The journey took no time at all, that's impossible!\"",
            "\"Yolculuk hiç zaman almadı!\"",
            1,
        );
        let m = compare(ORIG, &tr).mismatch.expect("must fail");
        assert_eq!(m.orig_line, 7);
        assert_eq!(m.tr_line, 7);
        assert_eq!(
            m.orig_tok.as_deref(),
            Some("\"The journey took no time at all, that's impossible!\"")
        );
        assert_eq!(m.tr_tok.as_deref(), Some("\"Yolculuk hiç zaman almadı!\""));
    }

    #[test]
    fn changed_identifier_fails() {
        let tr = ORIG
            .replace("// TODO: Implement", "// TODO: Implement\n// ek satır")
            .replace("let d = end", "let mesafe = end");
        let m = compare(ORIG, &tr).mismatch.expect("must fail");
        assert_eq!((m.orig_line, m.tr_line), (4, 5));
        assert_eq!(m.orig_tok.as_deref(), Some("d"));
        assert_eq!(m.tr_tok.as_deref(), Some("mesafe"));
    }

    #[test]
    fn code_swallowed_by_comment_fails() {
        let orig = "// note\nlet x = 1;\n";
        let tr = "// not let x = 1;\n";
        let m = compare(orig, tr).mismatch.expect("must fail");
        assert_eq!(m.tr_tok, None);
        assert_eq!(m.orig_tok.as_deref(), Some("let"));
    }

    #[test]
    fn extra_code_fails() {
        let m = compare("fn f() {}\n", "fn f() {}\nfn g() {}\n")
            .mismatch
            .expect("must fail");
        assert_eq!(m.orig_tok, None);
        assert_eq!(m.tr_tok.as_deref(), Some("fn"));
        assert_eq!(m.tr_line, 2);
    }

    #[test]
    fn todo_count() {
        let r = compare("// TODO: x\nfn f() {}\n", "// yapılacak: x\nfn f() {}\n");
        assert!(r.mismatch.is_none());
        assert_eq!((r.orig_todo, r.tr_todo), (1, 0));
    }
}
