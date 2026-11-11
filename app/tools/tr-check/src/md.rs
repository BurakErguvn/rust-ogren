//! Structural comparison of a translated markdown page with its original.

use std::collections::{BTreeMap, HashMap, HashSet};

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

use crate::issue::Issue;
use crate::rs;
use crate::slug::IdGen;

pub fn options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_HEADING_ATTRIBUTES
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Pos {
    val: String,
    line: usize,
}

#[derive(Debug, Default)]
struct Heading {
    level: u8,
    text: String,
    id: Option<String>,
    /// `{.class key=value}` parts, normalised to a string.
    attrs: String,
    line: usize,
}

#[derive(Debug, Default)]
struct CodeBlock {
    fenced: bool,
    info: String,
    content: String,
    line: usize,
    content_line: Option<usize>,
}

impl CodeBlock {
    fn is_rust(&self) -> bool {
        self.fenced && self.info.starts_with("rust")
    }
    fn first_content_line(&self) -> usize {
        self.content_line.unwrap_or(self.line + 1)
    }
}

#[derive(Debug)]
struct Span {
    text: String,
    block: Option<usize>,
    line: usize,
}

#[derive(Debug, Default)]
struct Doc {
    headings: Vec<Heading>,
    code: Vec<CodeBlock>,
    links: Vec<Pos>,
    fn_refs: Vec<Pos>,
    fn_defs: Vec<Pos>,
    spans: Vec<Span>,
    html: Vec<Pos>,
    hard_breaks: Vec<usize>,
    items: Vec<usize>,
    blockquotes: Vec<usize>,
    /// One entry per table: number of cells (header included).
    tables: Vec<Pos>,
    paragraphs: Vec<usize>,
    /// Start line of every block-level element, in document order.
    blocks: Vec<usize>,
}

struct LineIndex(Vec<usize>);

impl LineIndex {
    fn new(src: &str) -> Self {
        let mut starts = vec![0];
        starts.extend(src.match_indices('\n').map(|(i, _)| i + 1));
        LineIndex(starts)
    }
    fn line(&self, offset: usize) -> usize {
        self.0.partition_point(|&s| s <= offset)
    }
}

fn is_block(tag: &Tag) -> bool {
    matches!(
        tag,
        Tag::Paragraph
            | Tag::Heading { .. }
            | Tag::BlockQuote(_)
            | Tag::CodeBlock(_)
            | Tag::HtmlBlock
            | Tag::Item
            | Tag::FootnoteDefinition(_)
            | Tag::TableCell
    )
}

fn is_block_end(tag: &TagEnd) -> bool {
    matches!(
        tag,
        TagEnd::Paragraph
            | TagEnd::Heading(_)
            | TagEnd::BlockQuote(_)
            | TagEnd::CodeBlock
            | TagEnd::HtmlBlock
            | TagEnd::Item
            | TagEnd::FootnoteDefinition
            | TagEnd::TableCell
    )
}

/// Extracts HTML tags (`<div class="warning">`, `</div>`, `<br/>`, comments) from raw HTML.
fn html_tags(s: &str, out: &mut Vec<String>) {
    let mut rest = s;
    while let Some(i) = rest.find('<') {
        let after = &rest[i + 1..];
        if after.starts_with("!--") {
            match after.find("-->") {
                Some(end) => {
                    out.push("<!-- -->".to_string());
                    rest = &after[end + 3..];
                }
                None => break,
            }
            continue;
        }
        let starts_tag = after
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '/' || c == '!' || c == '?');
        if !starts_tag {
            rest = after;
            continue;
        }
        match after.find('>') {
            Some(end) => {
                let tag = &rest[i..i + 1 + end + 1];
                out.push(tag.split_whitespace().collect::<Vec<_>>().join(" "));
                rest = &after[end + 1..];
            }
            None => break,
        }
    }
}

fn parse(src: &str) -> Doc {
    let lines = LineIndex::new(src);
    let mut doc = Doc::default();
    let mut heading: Option<Heading> = None;
    let mut code: Option<CodeBlock> = None;
    let mut stack: Vec<usize> = Vec::new();

    for (ev, range) in Parser::new_ext(src, options()).into_offset_iter() {
        let line = lines.line(range.start);
        match ev {
            Event::Start(tag) => {
                if is_block(&tag) {
                    doc.blocks.push(line);
                    stack.push(doc.blocks.len() - 1);
                }
                match tag {
                    Tag::Heading {
                        level,
                        id,
                        classes,
                        attrs,
                    } => {
                        let mut a: Vec<String> = classes.iter().map(|c| format!(".{c}")).collect();
                        a.extend(attrs.iter().map(|(k, v)| match v {
                            Some(v) => format!("{k}={v}"),
                            None => k.to_string(),
                        }));
                        heading = Some(Heading {
                            level: level as u8,
                            id: id.map(|s| s.to_string()),
                            attrs: a.join(" "),
                            line,
                            ..Default::default()
                        });
                    }
                    Tag::CodeBlock(kind) => {
                        let (fenced, info) = match kind {
                            CodeBlockKind::Fenced(info) => (true, info.trim().to_string()),
                            CodeBlockKind::Indented => (false, String::new()),
                        };
                        code = Some(CodeBlock {
                            fenced,
                            info,
                            line,
                            ..Default::default()
                        });
                    }
                    Tag::Link { dest_url, .. } => doc.links.push(Pos {
                        val: format!("link {dest_url}"),
                        line,
                    }),
                    Tag::Image { dest_url, .. } => doc.links.push(Pos {
                        val: format!("image {dest_url}"),
                        line,
                    }),
                    Tag::FootnoteDefinition(label) => doc.fn_defs.push(Pos {
                        val: label.to_string(),
                        line,
                    }),
                    Tag::Item => doc.items.push(line),
                    Tag::BlockQuote(_) => doc.blockquotes.push(line),
                    Tag::Paragraph => doc.paragraphs.push(line),
                    Tag::Table(_) => doc.tables.push(Pos {
                        val: "0".into(),
                        line,
                    }),
                    Tag::TableCell => {
                        if let Some(t) = doc.tables.last_mut() {
                            let n: usize = t.val.parse().unwrap_or(0);
                            t.val = (n + 1).to_string();
                        }
                    }
                    _ => {}
                }
            }
            Event::End(tag) => {
                if is_block_end(&tag) {
                    stack.pop();
                }
                match tag {
                    TagEnd::Heading(_) => doc.headings.extend(heading.take()),
                    TagEnd::CodeBlock => doc.code.extend(code.take()),
                    _ => {}
                }
            }
            Event::Text(t) => {
                if let Some(c) = code.as_mut() {
                    if c.content_line.is_none() {
                        c.content_line = Some(line);
                    }
                    c.content.push_str(&t);
                } else if let Some(h) = heading.as_mut() {
                    h.text.push_str(&t);
                }
            }
            Event::Code(t) => {
                if let Some(h) = heading.as_mut() {
                    h.text.push_str(&t);
                }
                doc.spans.push(Span {
                    text: t.to_string(),
                    block: stack.last().copied(),
                    line,
                });
            }
            Event::Html(t) | Event::InlineHtml(t) => {
                let mut tags = Vec::new();
                html_tags(&t, &mut tags);
                doc.html
                    .extend(tags.into_iter().map(|val| Pos { val, line }));
            }
            Event::FootnoteReference(label) => doc.fn_refs.push(Pos {
                val: label.to_string(),
                line,
            }),
            Event::HardBreak => doc.hard_breaks.push(line),
            _ => {}
        }
    }
    doc
}

fn line_list(lines: &[usize]) -> String {
    const MAX: usize = 15;
    let mut s: Vec<String> = lines.iter().take(MAX).map(|l| l.to_string()).collect();
    if lines.len() > MAX {
        s.push("...".into());
    }
    if s.is_empty() {
        "none".into()
    } else {
        s.join(", ")
    }
}

/// Ordered sequences that must be identical.
fn cmp_seq(what: &str, orig: &[Pos], tr: &[Pos], issues: &mut Vec<Issue>) {
    if orig.len() != tr.len() {
        issues.push(Issue::error(
            None,
            format!(
                "{what}: original has {}, translation has {}",
                orig.len(),
                tr.len()
            ),
        ));
    }
    for i in 0..orig.len().max(tr.len()) {
        match (orig.get(i), tr.get(i)) {
            (Some(a), Some(b)) if a.val == b.val => continue,
            (Some(a), Some(b)) => issues.push(Issue::error(
                Some(b.line),
                format!(
                    "{what} #{}: expected `{}` (original line {}), found `{}`",
                    i + 1,
                    a.val,
                    a.line,
                    b.val
                ),
            )),
            (Some(a), None) => issues.push(Issue::error(
                None,
                format!(
                    "{what} #{}: `{}` (original line {}) is missing",
                    i + 1,
                    a.val,
                    a.line
                ),
            )),
            (None, Some(b)) => issues.push(Issue::error(
                Some(b.line),
                format!("{what} #{}: unexpected `{}`", i + 1, b.val),
            )),
            (None, None) => unreachable!(),
        }
        // Only the first mismatch: after it the sequences are usually misaligned.
        break;
    }
}

fn cmp_count(what: &str, orig: &[usize], tr: &[usize], issues: &mut Vec<Issue>) {
    if orig.len() != tr.len() {
        issues.push(Issue::error(
            None,
            format!(
                "{what}: original has {} (lines {}), translation has {} (lines {})",
                orig.len(),
                line_list(orig),
                tr.len(),
                line_list(tr)
            ),
        ));
    }
}

fn heading_src(h: &Heading) -> String {
    format!("{} {}", "#".repeat(h.level as usize), h.text)
}

fn check_headings(o: &Doc, t: &Doc, issues: &mut Vec<Issue>) {
    let mut ids = IdGen::default();
    let expected: Vec<String> = o
        .headings
        .iter()
        .map(|h| ids.next_id(&h.text, h.id.as_deref()))
        .collect();

    if o.headings.len() != t.headings.len() {
        issues.push(Issue::error(
            None,
            format!(
                "headings: original has {}, translation has {}",
                o.headings.len(),
                t.headings.len()
            ),
        ));
    }
    for (i, (a, b)) in o.headings.iter().zip(&t.headings).enumerate() {
        let want = &expected[i];
        if a.level != b.level {
            issues.push(Issue::error(
                Some(b.line),
                format!(
                    "heading #{} has level {}, original `{}` (line {}) has level {}",
                    i + 1,
                    b.level,
                    heading_src(a),
                    a.line,
                    a.level
                ),
            ));
            // Levels out of sync: the rest would be noise.
            break;
        }
        match &b.id {
            None => issues.push(Issue::error(
                Some(b.line),
                format!(
                    "heading must end with ` {{#{want}}}` (original: `{}`, line {})",
                    heading_src(a),
                    a.line
                ),
            )),
            Some(id) if id != want => issues.push(Issue::error(
                Some(b.line),
                format!(
                    "heading anchor is `{{#{id}}}`, expected `{{#{want}}}` (original: `{}`, line {})",
                    heading_src(a),
                    a.line
                ),
            )),
            Some(_) => {}
        }
        if a.attrs != b.attrs {
            issues.push(Issue::error(
                Some(b.line),
                format!(
                    "heading attributes `{}` differ from original `{}` (line {})",
                    b.attrs, a.attrs, a.line
                ),
            ));
        }
    }
}

fn check_code(o: &Doc, t: &Doc, issues: &mut Vec<Issue>) {
    if o.code.len() != t.code.len() {
        issues.push(Issue::error(
            None,
            format!(
                "code blocks: original has {} (lines {}), translation has {} (lines {})",
                o.code.len(),
                line_list(&o.code.iter().map(|c| c.line).collect::<Vec<_>>()),
                t.code.len(),
                line_list(&t.code.iter().map(|c| c.line).collect::<Vec<_>>()),
            ),
        ));
    }
    for (i, (a, b)) in o.code.iter().zip(&t.code).enumerate() {
        if a.fenced != b.fenced || a.info != b.info {
            let show = |c: &CodeBlock| {
                if c.fenced {
                    format!("```{}", c.info)
                } else {
                    "indented block".to_string()
                }
            };
            issues.push(Issue::error(
                Some(b.line),
                format!(
                    "code block #{}: expected `{}` (original line {}), found `{}`",
                    i + 1,
                    show(a),
                    a.line,
                    show(b)
                ),
            ));
            if o.code.len() != t.code.len() {
                break;
            }
            continue;
        }
        if a.is_rust() {
            let r = rs::compare(&a.content, &b.content);
            if let Some(m) = r.mismatch {
                let mut m = m;
                m.orig_line += a.first_content_line() - 1;
                m.tr_line += b.first_content_line() - 1;
                issues.push(Issue::error(
                    Some(m.tr_line),
                    format!("rust code block #{}: {}", i + 1, m.describe()),
                ));
            }
        } else if a.content != b.content {
            let al: Vec<&str> = a.content.split('\n').collect();
            let bl: Vec<&str> = b.content.split('\n').collect();
            let k = al
                .iter()
                .zip(&bl)
                .position(|(x, y)| x != y)
                .unwrap_or(al.len().min(bl.len()));
            let lang = if a.info.is_empty() {
                "untagged"
            } else {
                &a.info
            };
            issues.push(Issue::error(
                Some(b.first_content_line() + k),
                format!(
                    "{lang} code block #{} must be byte-identical; first difference at original line {}: expected `{}`, found `{}`",
                    i + 1,
                    a.first_content_line() + k,
                    al.get(k).copied().unwrap_or("<end of block>"),
                    bl.get(k).copied().unwrap_or("<end of block>"),
                ),
            ));
        }
    }
}

fn check_spans(o: &Doc, t: &Doc, issues: &mut Vec<Issue>) {
    // Global multiset: positive = missing from the translation, negative = extra.
    let mut diff: BTreeMap<&str, isize> = BTreeMap::new();
    for s in &o.spans {
        *diff.entry(&s.text).or_default() += 1;
    }
    for s in &t.spans {
        *diff.entry(&s.text).or_default() -= 1;
    }
    diff.retain(|_, d| *d != 0);
    if diff.is_empty() {
        return;
    }

    // Localise using the block structure when it lines up 1:1.
    let aligned = o.blocks.len() == t.blocks.len();
    fn per_block(spans: &[Span]) -> HashMap<(usize, &str), isize> {
        let mut m: HashMap<(usize, &str), isize> = HashMap::new();
        for s in spans {
            if let Some(b) = s.block {
                *m.entry((b, s.text.as_str())).or_default() += 1;
            }
        }
        m
    }
    let (ob, tb) = (per_block(&o.spans), per_block(&t.spans));

    for (text, d) in diff {
        let missing = d > 0;
        let (src, other_counts, own_counts) = if missing {
            (&o.spans, &tb, &ob)
        } else {
            (&t.spans, &ob, &tb)
        };
        let mut left = d.unsigned_abs();
        let mut seen = HashSet::new();
        let mut unplaced: Vec<usize> = Vec::new();
        for s in src.iter().filter(|s| s.text == text) {
            if left == 0 {
                break;
            }
            let located = aligned
                && s.block.is_some_and(|b| {
                    let own = own_counts.get(&(b, text)).copied().unwrap_or(0);
                    let other = other_counts.get(&(b, text)).copied().unwrap_or(0);
                    own > other && seen.insert(b)
                });
            if located {
                let b = s.block.unwrap_or_default();
                left -= 1;
                if missing {
                    issues.push(Issue::error(
                        Some(t.blocks[b]),
                        format!(
                            "inline code `{text}` is missing in this block (original line {})",
                            s.line
                        ),
                    ));
                } else {
                    issues.push(Issue::error(
                        Some(s.line),
                        format!(
                            "inline code `{text}` is not in the corresponding original block (line {})",
                            o.blocks[b]
                        ),
                    ));
                }
            } else {
                unplaced.push(s.line);
            }
        }
        if left > 0 {
            let lines = line_list(&unplaced);
            issues.push(if missing {
                Issue::error(
                    None,
                    format!(
                        "inline code `{text}` occurs {left} time(s) fewer than in the original (original lines {lines})"
                    ),
                )
            } else {
                Issue::error(
                    None,
                    format!(
                        "inline code `{text}` occurs {left} time(s) more than in the original (translation lines {lines})"
                    ),
                )
            });
        }
    }
}

/// Compares a translated page with its original.
pub fn check(orig: &str, tr: &str) -> Vec<Issue> {
    let o = parse(orig);
    let t = parse(tr);
    let mut issues = Vec::new();

    check_headings(&o, &t, &mut issues);
    check_code(&o, &t, &mut issues);
    cmp_seq("link/image destinations", &o.links, &t.links, &mut issues);
    cmp_seq("footnote references", &o.fn_refs, &t.fn_refs, &mut issues);
    cmp_seq("footnote definitions", &o.fn_defs, &t.fn_defs, &mut issues);
    check_spans(&o, &t, &mut issues);
    cmp_seq("HTML tags", &o.html, &t.html, &mut issues);
    cmp_count(
        "hard line breaks",
        &o.hard_breaks,
        &t.hard_breaks,
        &mut issues,
    );
    cmp_count("list items", &o.items, &t.items, &mut issues);
    cmp_count("block quotes", &o.blockquotes, &t.blockquotes, &mut issues);
    cmp_seq("table cell counts", &o.tables, &t.tables, &mut issues);
    if o.paragraphs.len() != t.paragraphs.len() {
        issues.push(Issue::warning(
            None,
            format!(
                "paragraphs: original has {}, translation has {} (translation should be paragraph by paragraph)",
                o.paragraphs.len(),
                t.paragraphs.len()
            ),
        ));
    }

    issues.sort_by_key(|i| i.line.unwrap_or(0));
    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::issue::Severity;

    const ORIG: &str = r#"# Branching: `if`/`else`

Rust's standard library provides **range** type[^weird-ranges].\
Check the [`u32` docs](https://doc.rust-lang.org/std/primitive.u32.html) and
the [book](../01_intro/00_welcome.md#workshop-runner-wr).

## Function arguments are variables

- `a` is one
- `b` is two
    - nested

```rust
// The function
fn speed(a: u32) -> u32 {
    a * 2 // double it
}
```

```text
error[E0308]: mismatched types
```

<div class="warning">

Be careful with `panic!`.

</div>

![diagram](img/diagram.png)

| Type | Size |
|------|------|
| `u8` | 1    |

[^weird-ranges]: You can use ranges with other types too.
"#;

    const GOOD: &str = r#"# Dallanma: `if`/`else` {#branching-ifelse}

Rust'ın standart kütüphanesi **range** tipini sağlar[^weird-ranges].\
[book](../01_intro/00_welcome.md#workshop-runner-wr) ve [`u32` belgeleri](https://doc.rust-lang.org/std/primitive.u32.html)
demeyelim, sırayı koruyalım: [`u32` belgeleri](https://doc.rust-lang.org/std/primitive.u32.html) ve
[kitap](../01_intro/00_welcome.md#workshop-runner-wr).

## Fonksiyon argümanları birer değişkendir {#function-arguments-are-variables}

- bir: `a`
- iki: `b`
    - iç içe

```rust
// Fonksiyon
// iki satıra bölündü
fn speed(a: u32) -> u32 {
    a * 2 // ikiyle çarp
}
```

```text
error[E0308]: mismatched types
```

<div class="warning">

`panic!` ile dikkatli ol.

</div>

![diyagram](img/diagram.png)

| Tip | Boyut |
|-----|-------|
| `u8` | 1   |

[^weird-ranges]: Range'leri başka tiplerle de kullanabilirsin.
"#;

    fn errors(orig: &str, tr: &str) -> Vec<Issue> {
        check(orig, tr)
            .into_iter()
            .filter(|i| i.severity == Severity::Error)
            .collect()
    }

    fn good() -> String {
        // Drop the deliberately reordered line from GOOD so links stay in order.
        GOOD.lines()
            .filter(|l| !l.starts_with("[book]"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn correct_translation_passes() {
        let issues = check(ORIG, &good());
        assert!(issues.is_empty(), "{issues:#?}");
    }

    #[test]
    fn reordered_links_fail() {
        let e = errors(ORIG, GOOD);
        assert!(
            e.iter().any(|i| i.msg.contains("link/image destinations")),
            "{e:#?}"
        );
    }

    #[test]
    fn changed_link_target_fails() {
        let tr = good().replace("primitive.u32.html", "primitive.u64.html");
        let e = errors(ORIG, &tr);
        assert_eq!(e.len(), 1, "{e:#?}");
        assert!(e[0].msg.contains("primitive.u32.html"));
        assert_eq!(e[0].line, Some(4));
    }

    #[test]
    fn changed_image_target_fails() {
        let tr = good().replace("img/diagram.png", "img/diyagram.png");
        assert_eq!(errors(ORIG, &tr).len(), 1);
    }

    #[test]
    fn missing_heading_suffix_fails() {
        let tr = good().replace(" {#function-arguments-are-variables}", "");
        let e = errors(ORIG, &tr);
        assert_eq!(e.len(), 1, "{e:#?}");
        assert!(e[0].msg.contains("{#function-arguments-are-variables}"));
        assert_eq!(e[0].line, Some(7));
    }

    #[test]
    fn wrong_heading_slug_fails() {
        let tr = good().replace("{#branching-ifelse}", "{#dallanma-ifelse}");
        let e = errors(ORIG, &tr);
        assert_eq!(e.len(), 1, "{e:#?}");
        assert!(e[0].msg.contains("expected `{#branching-ifelse}`"));
    }

    #[test]
    fn heading_level_and_count_fail() {
        let tr = good().replace("## Fonksiyon", "### Fonksiyon");
        assert!(errors(ORIG, &tr)[0].msg.contains("level"));
        let tr = format!("{}\n\n## Ek başlık {{#extra}}\n", good());
        assert!(errors(ORIG, &tr).iter().any(|i| i
            .msg
            .contains("headings: original has 2, translation has 3")));
    }

    #[test]
    fn changed_text_code_block_fails() {
        let tr = good().replace("mismatched types", "uyumsuz tipler");
        let e = errors(ORIG, &tr);
        assert_eq!(e.len(), 1, "{e:#?}");
        assert!(e[0].msg.contains("byte-identical"));
        assert_eq!(e[0].line, Some(22));
    }

    #[test]
    fn changed_rust_code_fails() {
        let tr = good().replace("a * 2", "a * 3");
        let e = errors(ORIG, &tr);
        assert_eq!(e.len(), 1, "{e:#?}");
        assert!(e[0].msg.contains("`2`") && e[0].msg.contains("`3`"));
        assert_eq!(e[0].line, Some(17));
    }

    #[test]
    fn code_block_info_must_match() {
        let tr = good().replace("```text", "```console");
        assert!(errors(ORIG, &tr)[0].msg.contains("```text"));
    }

    #[test]
    fn inline_code_must_be_kept() {
        let tr = good().replace("`panic!` ile", "panic! ile");
        let e = errors(ORIG, &tr);
        assert_eq!(e.len(), 1, "{e:#?}");
        assert!(e[0].msg.contains("`panic!` is missing"));
        assert_eq!(e[0].line, Some(27));

        let tr = good().replace("`panic!` ile", "`panik!` ile");
        assert_eq!(errors(ORIG, &tr).len(), 2);
    }

    #[test]
    fn footnotes_html_breaks_lists() {
        let tr = good().replace("[^weird-ranges].", "[^ranges].");
        assert!(!errors(ORIG, &tr).is_empty());
        let tr = good().replace("<div class=\"warning\">", "<div>");
        assert!(errors(ORIG, &tr)[0].msg.contains("HTML tags"));
        let tr = good().replace("sağlar[^weird-ranges].\\", "sağlar[^weird-ranges].");
        assert!(errors(ORIG, &tr)[0].msg.contains("hard line breaks"));
        let tr = good().replace("    - iç içe\n", "");
        assert!(errors(ORIG, &tr)[0].msg.contains("list items"));
    }

    #[test]
    fn paragraph_count_is_a_warning() {
        let tr = good().replace(
            "`panic!` ile dikkatli ol.",
            "`panic!` ile dikkatli ol.\n\nEk paragraf.",
        );
        let all = check(ORIG, &tr);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].severity, Severity::Warning);
    }

    #[test]
    fn html_tag_extraction() {
        let mut v = Vec::new();
        html_tags(
            "<div class=\"warning\">\ntext a < b\n<!-- note -->\n</div>",
            &mut v,
        );
        assert_eq!(v, ["<div class=\"warning\">", "<!-- -->", "</div>"]);
    }
}
