//! Hidden test code (app/API.md, "Gizli testler").
//!
//! Test code would give away hints about the solution, so learners only see the source
//! *without* its top-level `#[cfg(test)]` items. On disk every file is stored as
//!
//! ```text
//! <visible content> + "\n" + <hidden blocks>
//! ```
//!
//! so line numbers of the visible part equal line numbers on disk. The hidden blocks are
//! remembered per file in the workspace state and re-appended on every save.
//!
//! Extraction uses `rustc_lexer`, so braces inside strings, chars and comments never
//! confuse the item boundaries; the visible text is byte-identical to the input outside
//! the removed ranges.

use std::collections::HashMap;

use ra_ap_rustc_lexer::{tokenize, FrontmatterAllowed, TokenKind};

/// Placeholder shown instead of a source location inside hidden code.
pub const HIDDEN_LOCATION: &str = "(gizli test kodu)";

#[derive(Debug, Clone, Copy)]
struct Tok {
    kind: TokenKind,
    start: usize,
    end: usize,
}

fn lex(src: &str) -> Vec<Tok> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    for t in tokenize(src, FrontmatterAllowed::No) {
        let len = t.len as usize;
        out.push(Tok {
            kind: t.kind,
            start: pos,
            end: pos + len,
        });
        pos += len;
    }
    out
}

fn is_trivia(k: TokenKind) -> bool {
    matches!(
        k,
        TokenKind::Whitespace | TokenKind::LineComment { .. } | TokenKind::BlockComment { .. }
    )
}

fn is_comment(k: TokenKind) -> bool {
    matches!(
        k,
        TokenKind::LineComment { .. } | TokenKind::BlockComment { .. }
    )
}

fn is_open(k: TokenKind) -> bool {
    matches!(
        k,
        TokenKind::OpenBrace | TokenKind::OpenParen | TokenKind::OpenBracket
    )
}

fn is_close(k: TokenKind) -> bool {
    matches!(
        k,
        TokenKind::CloseBrace | TokenKind::CloseParen | TokenKind::CloseBracket
    )
}

/// Result of splitting one source file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Extraction {
    /// The source without its top-level `#[cfg(test)]` items.
    pub visible: String,
    /// The removed items (with their leading comments), each ending in `\n`, separated
    /// by a blank line. Empty if nothing was removed.
    pub hidden: String,
    /// Number of removed items.
    pub blocks: usize,
    /// Anything unusual (nested `#[cfg(test)]`, complex cfg, unterminated item, ...).
    pub notes: Vec<String>,
}

/// Significant-token view over a lexed file.
struct Parser<'a> {
    src: &'a str,
    toks: Vec<Tok>,
    /// Indices into `toks` of non-trivia tokens.
    sig: Vec<usize>,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        let toks = lex(src);
        let sig = (0..toks.len())
            .filter(|&i| !is_trivia(toks[i].kind))
            .collect();
        Parser { src, toks, sig }
    }

    fn kind(&self, i: usize) -> Option<TokenKind> {
        self.sig.get(i).map(|&t| self.toks[t].kind)
    }

    fn text(&self, i: usize) -> &'a str {
        let t = self.toks[self.sig[i]];
        &self.src[t.start..t.end]
    }

    fn is_ident(&self, i: usize, s: &str) -> bool {
        self.kind(i) == Some(TokenKind::Ident) && self.text(i) == s
    }

    /// Index of the token closing the group opened at `open` (or the last token).
    fn matching(&self, open: usize) -> usize {
        let mut depth = 0i32;
        let mut j = open;
        while j < self.sig.len() {
            let k = self.kind(j).expect("in range");
            if is_open(k) {
                depth += 1;
            } else if is_close(k) {
                depth -= 1;
                if depth <= 0 {
                    return j;
                }
            }
            j += 1;
        }
        self.sig.len().saturating_sub(1)
    }

    /// `[` `cfg` `(` `test` `)` `]` with `open` at `[` and `close` at `]`.
    fn is_cfg_test(&self, open: usize, close: usize) -> bool {
        close == open + 5
            && self.is_ident(open + 1, "cfg")
            && self.kind(open + 2) == Some(TokenKind::OpenParen)
            && self.is_ident(open + 3, "test")
            && self.kind(open + 4) == Some(TokenKind::CloseParen)
    }

    fn mentions_cfg_test(&self, open: usize, close: usize) -> bool {
        (open..=close).any(|i| self.is_ident(i, "cfg"))
            && (open..=close).any(|i| self.is_ident(i, "test"))
    }

    /// Whether the item starting at `i` (after its attributes) ends with `;` even when it
    /// contains a top-level brace group (`const X: S = S { .. };`).
    fn ends_with_semicolon(&self, mut i: usize) -> bool {
        if self.is_ident(i, "pub") {
            i += 1;
            if self.kind(i) == Some(TokenKind::OpenParen) {
                i = self.matching(i) + 1;
            }
        }
        if self.kind(i) != Some(TokenKind::Ident) {
            return false;
        }
        match self.text(i) {
            "use" | "static" | "type" | "let" => true,
            "const" => {
                !(self.is_ident(i + 1, "fn")
                    || self.is_ident(i + 1, "unsafe")
                    || self.is_ident(i + 1, "async")
                    || self.is_ident(i + 1, "extern"))
            }
            "extern" => self.is_ident(i + 1, "crate"),
            _ => false,
        }
    }

    /// Index of the last token of the item starting at `i`. `None` if the file ends first.
    fn item_end(&self, i: usize) -> Option<usize> {
        let semi = self.ends_with_semicolon(i);
        let mut depth = 0i32;
        let mut j = i;
        while j < self.sig.len() {
            let k = self.kind(j).expect("in range");
            if is_open(k) {
                depth += 1;
            } else if is_close(k) {
                depth -= 1;
                if depth < 0 {
                    return Some(j); // stray closing delimiter at top level
                }
                if depth == 0 && k == TokenKind::CloseBrace && !semi {
                    return Some(j);
                }
            } else if depth == 0 && k == TokenKind::Semi {
                return Some(j);
            }
            j += 1;
        }
        None
    }

    /// Total number of `#[cfg(test)]` attributes anywhere in the file.
    fn count_cfg_test(&self) -> usize {
        (0..self.sig.len())
            .filter(|&i| {
                self.kind(i) == Some(TokenKind::Pound)
                    && self.kind(i + 1) == Some(TokenKind::OpenBracket)
                    && self.is_cfg_test(i + 1, self.matching(i + 1))
            })
            .count()
    }
}

fn line_start(src: &str, pos: usize) -> usize {
    src[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0)
}

fn only_ws(s: &str) -> bool {
    s.chars().all(|c| c == ' ' || c == '\t' || c == '\r')
}

/// Splits `src` into visible and hidden parts. See the module docs.
pub fn extract(src: &str) -> Extraction {
    let p = Parser::new(src);
    let mut out = Extraction::default();
    // (removal_start, block_start, block_end) byte ranges.
    let mut ranges: Vec<(usize, usize, usize)> = Vec::new();
    let mut found_cfg_test = 0usize;

    let mut i = 0usize;
    while i < p.sig.len() {
        let item_start = i;
        // Inner attribute `#![...]`.
        if p.kind(i) == Some(TokenKind::Pound)
            && p.kind(i + 1) == Some(TokenKind::Bang)
            && p.kind(i + 2) == Some(TokenKind::OpenBracket)
        {
            i = p.matching(i + 2) + 1;
            continue;
        }
        let mut cfg_test = false;
        while p.kind(i) == Some(TokenKind::Pound) && p.kind(i + 1) == Some(TokenKind::OpenBracket) {
            let close = p.matching(i + 1);
            if p.is_cfg_test(i + 1, close) {
                cfg_test = true;
                found_cfg_test += 1;
            } else if p.mentions_cfg_test(i + 1, close) {
                out.notes.push(format!(
                    "complex cfg attribute left visible: {}",
                    &src[p.toks[p.sig[i]].start..p.toks[p.sig[close]].end]
                ));
            }
            i = close + 1;
        }
        if i >= p.sig.len() {
            if cfg_test {
                out.notes
                    .push("`#[cfg(test)]` without an item at end of file".into());
            }
            break;
        }
        let end = match p.item_end(i) {
            Some(e) => e,
            None => {
                if cfg_test {
                    out.notes
                        .push("unterminated `#[cfg(test)]` item left visible".into());
                }
                break;
            }
        };
        if cfg_test {
            ranges.push(block_range(&p, item_start, end, &mut out.notes));
        }
        i = end + 1;
    }

    let total = p.count_cfg_test();
    if total > found_cfg_test {
        out.notes.push(format!(
            "{} nested (non top-level) `#[cfg(test)]` attribute(s) left visible",
            total - found_cfg_test
        ));
    }

    let mut visible = String::with_capacity(src.len());
    let mut hidden_blocks: Vec<String> = Vec::new();
    let mut pos = 0usize;
    for &(rm_start, b_start, b_end) in &ranges {
        visible.push_str(&src[pos..rm_start]);
        let mut block = src[b_start..b_end].to_string();
        if !block.ends_with('\n') {
            block.push('\n');
        }
        hidden_blocks.push(block);
        pos = b_end;
    }
    if !ranges.is_empty() && visible.is_empty() {
        // Block(s) at the very start of the file: drop the blank lines that followed.
        let rest = &src[pos..];
        let mut skip = 0usize;
        for line in rest.split_inclusive('\n') {
            if line.ends_with('\n') && only_ws(&line[..line.len() - 1]) {
                skip += line.len();
            } else {
                break;
            }
        }
        pos += skip;
    }
    visible.push_str(&src[pos..]);
    out.blocks = hidden_blocks.len();
    out.hidden = hidden_blocks.join("\n");
    out.visible = visible;
    out
}

/// Byte ranges for the item `sig[item_start..=end]`: (removal start incl. preceding blank
/// lines, hidden block start incl. directly preceding comments, block end incl. the rest
/// of its last line).
fn block_range(
    p: &Parser<'_>,
    item_start: usize,
    end: usize,
    notes: &mut Vec<String>,
) -> (usize, usize, usize) {
    let src = p.src;
    let first_tok = p.sig[item_start];
    let mut start = p.toks[first_tok].start;

    // Directly preceding comments (no blank line in between, each starting its own line).
    let mut k = first_tok;
    while k > 0 {
        let t = p.toks[k - 1];
        if t.kind == TokenKind::Whitespace {
            if src[t.start..t.end].matches('\n').count() >= 2 {
                break;
            }
        } else if is_comment(t.kind) {
            if !only_ws(&src[line_start(src, t.start)..t.start]) {
                break;
            }
            start = t.start;
        } else {
            break;
        }
        k -= 1;
    }
    let ls = line_start(src, start);
    if only_ws(&src[ls..start]) {
        start = ls;
    } else {
        notes.push("`#[cfg(test)]` item does not start on its own line".into());
    }

    // Rest of the last line (trailing whitespace / comments and the newline).
    let last_tok = p.sig[end];
    let mut block_end = p.toks[last_tok].end;
    let mut k = last_tok + 1;
    while k < p.toks.len() {
        let t = p.toks[k];
        let text = &src[t.start..t.end];
        match t.kind {
            TokenKind::Whitespace => {
                if let Some(nl) = text.find('\n') {
                    block_end = t.start + nl + 1;
                    break;
                }
                block_end = t.end;
            }
            TokenKind::LineComment { .. } => block_end = t.end,
            TokenKind::BlockComment { .. } if !text.contains('\n') => block_end = t.end,
            _ => {
                notes.push("code follows a `#[cfg(test)]` item on the same line".into());
                block_end = p.toks[last_tok].end;
                break;
            }
        }
        k += 1;
    }

    // Blank lines before the block are removed with it.
    let mut rm_start = start;
    while rm_start > 0 {
        let prev_ls = line_start(src, rm_start - 1);
        if only_ws(&src[prev_ls..rm_start - 1]) {
            rm_start = prev_ls;
        } else {
            break;
        }
    }
    (rm_start, start, block_end)
}

/// On-disk content for a file: visible part, a newline, then the hidden blocks.
pub fn compose(visible: &str, hidden: &str) -> String {
    if hidden.is_empty() {
        return visible.to_string();
    }
    let mut s = String::with_capacity(visible.len() + hidden.len() + 1);
    s.push_str(visible);
    s.push('\n');
    s.push_str(hidden);
    s
}

/// Inverse of [`compose`]: the visible part if `disk` ends with the given hidden blocks.
pub fn split_disk<'a>(disk: &'a str, hidden: &str) -> Option<&'a str> {
    if hidden.is_empty() {
        return Some(disk);
    }
    disk.strip_suffix(hidden)?.strip_suffix('\n')
}

/// First 1-based line of the hidden part of a composed file.
pub fn first_hidden_line(visible: &str) -> u32 {
    visible.bytes().filter(|&b| b == b'\n').count() as u32 + 2
}

/// Why hidden test code must stay visible: it contains something the learner has to edit
/// (`todo!()`, `unimplemented!()`, a `TODO` comment or a `__` placeholder).
pub fn learner_edit_marker(code: &str) -> Option<&'static str> {
    let p = Parser::new(code);
    for t in &p.toks {
        if is_comment(t.kind) && code[t.start..t.end].contains("TODO") {
            return Some("TODO comment");
        }
    }
    for i in 0..p.sig.len() {
        if p.kind(i) != Some(TokenKind::Ident) {
            continue;
        }
        let text = p.text(i);
        if (text == "todo" || text == "unimplemented") && p.kind(i + 1) == Some(TokenKind::Bang) {
            return Some(if text == "todo" {
                "todo!()"
            } else {
                "unimplemented!()"
            });
        }
        if text.len() >= 2 && text.bytes().all(|b| b == b'_') {
            return Some("`__` placeholder");
        }
    }
    None
}

/// All comment text of a Rust source, one comment per line.
pub fn comment_text(code: &str) -> String {
    lex(code)
        .iter()
        .filter(|t| is_comment(t.kind))
        .map(|t| &code[t.start..t.end])
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------------------
// Scrubbing compiler / test output
// ---------------------------------------------------------------------------------------

/// Exercise-relative file -> first hidden line (1 = whole file hidden).
pub type HiddenLines = HashMap<String, u32>;

pub fn is_hidden(map: &HiddenLines, file: &str, line: u32) -> bool {
    map.get(file).is_some_and(|&first| line >= first)
}

/// Parses `<digits>[:<digits>]` at the start of `s`; returns (line, consumed bytes).
fn parse_line_col(s: &str) -> Option<(u32, usize)> {
    let digits = s.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    let line: u32 = s[..digits].parse().ok()?;
    let mut used = digits;
    let rest = &s[digits..];
    if let Some(r) = rest.strip_prefix(':') {
        let col = r.bytes().take_while(u8::is_ascii_digit).count();
        if col > 0 {
            used += 1 + col;
        }
    }
    Some((line, used))
}

/// Replaces `file:LINE[:COL]` references into hidden code with [`HIDDEN_LOCATION`]
/// (e.g. `panicked at src/lib.rs:40:9:` -> `panicked at (gizli test kodu):`).
pub fn scrub_locations(text: &str, map: &HiddenLines) -> String {
    if map.is_empty() {
        return text.to_string();
    }
    let mut out = text.to_string();
    for file in map.keys() {
        let needle = format!("{file}:");
        let mut result = String::with_capacity(out.len());
        let mut rest = out.as_str();
        while let Some(pos) = rest.find(&needle) {
            let boundary_ok = pos == 0 || {
                let prev = rest[..pos].chars().next_back().unwrap_or(' ');
                !(prev.is_alphanumeric() || prev == '_' || prev == '/' || prev == '.')
            };
            let after = &rest[pos + needle.len()..];
            match parse_line_col(after) {
                Some((line, used)) if boundary_ok && is_hidden(map, file, line) => {
                    result.push_str(&rest[..pos]);
                    result.push_str(HIDDEN_LOCATION);
                    rest = &after[used..];
                }
                _ => {
                    result.push_str(&rest[..pos + needle.len()]);
                    rest = after;
                }
            }
        }
        result.push_str(rest);
        out = result;
    }
    out
}

/// `--> file:line:col` / `::: file:line:col` header of a rustc snippet.
fn snippet_header(line: &str) -> Option<(&str, u32)> {
    let t = line.trim_start();
    let rest = t.strip_prefix("-->").or_else(|| t.strip_prefix(":::"))?;
    let loc = rest.trim();
    let (file, tail) = loc.split_once(':')?;
    let (l, _) = parse_line_col(tail)?;
    Some((file, l))
}

/// Line number of a numbered snippet line (` 12 |     code`, ` 12 +     added`).
fn snippet_line_number(line: &str) -> Option<u32> {
    let t = line.trim_start();
    let digits = t.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    let after = t[digits..].trim_start_matches(' ');
    match after.chars().next() {
        Some('|' | '+' | '-' | '~') => t[..digits].parse().ok(),
        _ => None,
    }
}

/// Gutter-only line of a snippet (`   |`, `   |     ^^^ label`, `...`).
fn is_gutter_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with('|') || t == "..."
}

/// Removes source excerpts of hidden lines from rustc's rendered output and replaces
/// locations pointing into hidden code.
pub fn scrub_rendered(text: &str, map: &HiddenLines) -> String {
    if map.is_empty() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut file: Option<String> = None;
    let mut skipping = false;
    for line in text.split_inclusive('\n') {
        let body = line.trim_end_matches('\n');
        if let Some((f, _)) = snippet_header(body) {
            file = Some(f.to_string());
            skipping = false;
            out.push_str(&scrub_locations(line, map));
            continue;
        }
        if let Some(n) = snippet_line_number(body) {
            skipping = file.as_deref().is_some_and(|f| is_hidden(map, f, n));
            if !skipping {
                out.push_str(line);
            }
            continue;
        }
        if is_gutter_line(body) {
            if !skipping {
                out.push_str(line);
            }
            continue;
        }
        // Sub-diagnostic snippets without a header (`help:` suggestions) refer to the file
        // of the last header, so `file` is kept.
        skipping = false;
        out.push_str(&scrub_locations(line, map));
    }
    collapse_gutters(&out)
}

/// Collapses runs of empty gutter lines (`   |`) left behind by removed excerpts.
fn collapse_gutters(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut prev_empty_gutter = false;
    for line in text.split_inclusive('\n') {
        let empty_gutter = line.trim() == "|";
        if empty_gutter && prev_empty_gutter {
            continue;
        }
        prev_empty_gutter = empty_gutter;
        out.push_str(line);
    }
    out
}

/// Short form of a diagnostic located in hidden code: the headline plus `= note:` /
/// `= help:` lines (with their continuation lines), no source excerpts.
pub fn headline_only(rendered: &str, map: &HiddenLines) -> String {
    let mut lines = rendered.lines();
    let mut out = String::new();
    if let Some(first) = lines.next() {
        out.push_str(&scrub_locations(first, map));
        out.push('\n');
    }
    let mut in_note = false;
    for line in lines {
        let t = line.trim_start();
        if t.starts_with("= note:") || t.starts_with("= help:") {
            in_note = true;
        } else if in_note
            && line.starts_with(' ')
            && !t.is_empty()
            && !is_gutter_line(line)
            && snippet_line_number(line).is_none()
            && snippet_header(line).is_none()
        {
            // continuation of a multi-line note
        } else {
            in_note = false;
            continue;
        }
        out.push_str(&scrub_locations(line, map));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ex(src: &str) -> Extraction {
        extract(src)
    }

    #[test]
    fn tests_module_at_end() {
        let src = "pub fn f() -> u32 {\n    1\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn t() {\n        assert_eq!(f(), 1);\n    }\n}\n";
        let e = ex(src);
        assert_eq!(e.visible, "pub fn f() -> u32 {\n    1\n}\n");
        assert!(e.hidden.starts_with("#[cfg(test)]\nmod tests {"));
        assert!(e.hidden.ends_with("}\n"));
        assert_eq!(e.blocks, 1);
        assert!(e.notes.is_empty());
        // The usual layout round-trips byte for byte.
        assert_eq!(compose(&e.visible, &e.hidden), src);
        assert_eq!(split_disk(src, &e.hidden), Some(e.visible.as_str()));
        assert_eq!(first_hidden_line(&e.visible), 5);
    }

    #[test]
    fn tests_module_in_the_middle() {
        let src =
            "fn a() {}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n}\n\nfn b() {}\n";
        let e = ex(src);
        assert_eq!(e.visible, "fn a() {}\n\nfn b() {}\n");
        assert_eq!(
            e.hidden,
            "#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n}\n"
        );
        let disk = compose(&e.visible, &e.hidden);
        assert_eq!(
            disk,
            "fn a() {}\n\nfn b() {}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n}\n"
        );
        // Visible line numbers are unchanged on disk.
        assert!(disk.starts_with(&e.visible));
    }

    #[test]
    fn preceding_comments_belong_to_the_block() {
        let src = "fn a() {}\n// unrelated, separated by a blank line\n\n// Tests below.\n/// doc\n#[cfg(test)]\nmod tests {}\n";
        let e = ex(src);
        assert_eq!(
            e.visible,
            "fn a() {}\n// unrelated, separated by a blank line\n"
        );
        assert_eq!(
            e.hidden,
            "// Tests below.\n/// doc\n#[cfg(test)]\nmod tests {}\n"
        );
        // A trailing comment on the previous code line is not part of the block.
        let e = ex("fn a() {} // keep\n#[cfg(test)]\nmod tests {}\n");
        assert_eq!(e.visible, "fn a() {} // keep\n");
        assert_eq!(e.hidden, "#[cfg(test)]\nmod tests {}\n");
    }

    #[test]
    fn cfg_test_on_other_items() {
        let src = "fn a() {}\n\n#[cfg(test)]\nfn helper() -> u8 {\n    7\n}\n\n#[cfg(test)]\nconst X: S = S { a: 1 };\n\n#[cfg(test)]\nuse std::fmt;\n\nfn b() {}\n";
        let e = ex(src);
        assert_eq!(e.visible, "fn a() {}\n\nfn b() {}\n");
        assert_eq!(e.blocks, 3);
        assert_eq!(
            e.hidden,
            "#[cfg(test)]\nfn helper() -> u8 {\n    7\n}\n\n#[cfg(test)]\nconst X: S = S { a: 1 };\n\n#[cfg(test)]\nuse std::fmt;\n"
        );
    }

    #[test]
    fn braces_in_strings_chars_and_comments() {
        let src = "fn a() {}\n\n#[cfg(test)]\n#[allow(unused)]\nmod tests {\n    const S: &str = \"}}}\";\n    const C: char = '}';\n    const R: &str = r#\"{ \"}\" \"#;\n    // }\n    /* } { */\n    fn x<'a>(s: &'a str) -> &'a str { s }\n    #[test]\n    fn t() { assert_eq!(format!(\"{{}}\"), \"{}\"); }\n} // end\nfn b() {}\n";
        let e = ex(src);
        assert_eq!(e.visible, "fn a() {}\nfn b() {}\n");
        assert!(e.hidden.ends_with("} // end\n"), "{}", e.hidden);
        assert!(e.notes.is_empty(), "{:?}", e.notes);
    }

    #[test]
    fn no_tests_and_unusual_cases() {
        let src = "// mentions #[cfg(test)] in a comment\nfn a() { let s = \"#[cfg(test)]\"; }\n";
        let e = ex(src);
        assert_eq!(e.visible, src);
        assert_eq!(e.hidden, "");
        assert_eq!(e.blocks, 0);
        assert!(e.notes.is_empty());
        assert_eq!(compose(src, ""), src);
        assert_eq!(split_disk(src, ""), Some(src));

        let nested =
            "mod inner {\n    #[cfg(test)]\n    mod tests {}\n}\n#[cfg(not(test))]\nfn x() {}\n";
        let e = ex(nested);
        assert_eq!(e.visible, nested);
        assert_eq!(e.notes.len(), 2, "{:?}", e.notes);

        // Inner attributes are skipped; the file may consist of tests only.
        let e = ex("#![allow(dead_code)]\n#[cfg(test)]\nmod tests {}\n");
        assert_eq!(e.visible, "#![allow(dead_code)]\n");
        let e = ex("#[cfg(test)]\nmod tests {}\n\nfn a() {}\n");
        assert_eq!(e.visible, "fn a() {}\n");
        // No trailing newline at EOF.
        let e = ex("fn a() {}\n#[cfg(test)]\nmod tests {}");
        assert_eq!(e.visible, "fn a() {}\n");
        assert_eq!(e.hidden, "#[cfg(test)]\nmod tests {}\n");
    }

    #[test]
    fn split_rejects_foreign_suffix() {
        assert_eq!(split_disk("fn a() {}\n", "#[cfg(test)]\nmod t {}\n"), None);
        let disk = compose("fn a() {}", "#[cfg(test)]\nmod t {}\n");
        assert_eq!(disk, "fn a() {}\n#[cfg(test)]\nmod t {}\n");
        assert_eq!(
            split_disk(&disk, "#[cfg(test)]\nmod t {}\n"),
            Some("fn a() {}")
        );
        assert_eq!(first_hidden_line("fn a() {}"), 2);
    }

    #[test]
    fn learner_markers() {
        assert_eq!(
            learner_edit_marker("#[test]\nfn t() { todo!() }"),
            Some("todo!()")
        );
        assert_eq!(
            learner_edit_marker("fn t() { assert_eq!(x, __); }"),
            Some("`__` placeholder")
        );
        assert_eq!(
            learner_edit_marker("// TODO: write a test\nfn t() {}"),
            Some("TODO comment")
        );
        assert_eq!(
            learner_edit_marker("fn t() { let _ = 1; let _x = \"__ TODO todo!()\"; }"),
            None
        );
    }

    fn map() -> HiddenLines {
        HiddenLines::from([
            ("src/lib.rs".to_string(), 10),
            ("tests/it.rs".to_string(), 1),
        ])
    }

    #[test]
    fn scrubs_panic_locations() {
        let m = map();
        assert_eq!(
            scrub_locations(
                "thread 't' panicked at src/lib.rs:12:9:\nboom at src/lib.rs:3:1 and tests/it.rs:2:5, xsrc/lib.rs:12:1",
                &m
            ),
            "thread 't' panicked at (gizli test kodu):\nboom at src/lib.rs:3:1 and (gizli test kodu), xsrc/lib.rs:12:1"
        );
        assert_eq!(
            scrub_locations("src/lib.rs:12:9", &HiddenLines::new()),
            "src/lib.rs:12:9"
        );
    }

    const RENDERED: &str = "error[E0308]: mismatched types\n  --> src/lib.rs:12:20\n   |\n12 |         assert_eq!(f(), \"x\");\n   |                    ^^^ expected `u32`, found `&str`\n   |\nnote: function defined here\n  --> src/lib.rs:2:8\n   |\n2  | pub fn f() -> u32 {\n   |        ^\n   = note: expected type `u32`\n              found reference `&str`\nhelp: try this\n   |\n12 -         assert_eq!(f(), \"x\");\n12 +         assert_eq!(f(), 1);\n   |\n   = help: consider something\n\n";

    #[test]
    fn hidden_diagnostic_headline() {
        let h = headline_only(RENDERED, &map());
        assert_eq!(
            h,
            "error[E0308]: mismatched types\n   = note: expected type `u32`\n              found reference `&str`\n   = help: consider something\n"
        );
    }

    #[test]
    fn strips_hidden_excerpts() {
        let s = scrub_rendered(RENDERED, &map());
        assert!(!s.contains("assert_eq"), "{s}");
        assert!(s.contains("--> (gizli test kodu)"), "{s}");
        assert!(s.contains("2  | pub fn f() -> u32 {"), "{s}");
        assert!(s.contains("--> src/lib.rs:2:8"), "{s}");
        assert!(!s.contains("|\n   |\n"), "{s}");
        // Nothing hidden -> unchanged.
        assert_eq!(scrub_rendered(RENDERED, &HiddenLines::new()), RENDERED);
    }
}
