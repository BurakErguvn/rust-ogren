//! Heading anchor slugs, matching mdBook / GitHub.

use std::collections::HashMap;

/// Slug of a heading's text content: lowercase; letters, digits, `-` and `_` are kept;
/// whitespace becomes `-`; everything else is dropped.
pub fn slug(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.trim().chars() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            out.extend(c.to_lowercase());
        } else if c.is_whitespace() {
            out.push('-');
        }
    }
    out
}

/// Generates unique ids for the headings of one page, the way mdBook does:
/// the second heading with the same slug gets `-1`, the third `-2`, ...
/// Headings with an explicit `{#id}` use it verbatim and don't touch the counter.
#[derive(Default)]
pub struct IdGen {
    counter: HashMap<String, usize>,
}

impl IdGen {
    pub fn next_id(&mut self, text: &str, explicit: Option<&str>) -> String {
        if let Some(id) = explicit {
            return id.to_string();
        }
        let base = slug(text);
        let n = self.counter.entry(base.clone()).or_insert(0);
        let id = if *n == 0 { base } else { format!("{base}-{n}") };
        *n += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_headings() {
        // Text content as produced by the markdown parser (backticks removed).
        let cases = [
            ("Branching: if/else", "branching-ifelse"),
            ("Workshop runner, wr", "workshop-runner-wr"),
            (
                "Come up with a \"sensible\" result",
                "come-up-with-a-sensible-result",
            ),
            ("overflow-checks", "overflow-checks"),
            ("Types, part 1", "types-part-1"),
            ("Signed vs. unsigned", "signed-vs-unsigned"),
            ("Domain-specific material", "domain-specific-material"),
            ("Copying values, pt. 1", "copying-values-pt-1"),
            ("String's memory layout", "strings-memory-layout"),
            (
                "No std::mem::size_of for the heap",
                "no-stdmemsize_of-for-the-heap",
            ),
            ("if/else is an expression", "ifelse-is-an-expression"),
            ("wrapping_ methods", "wrapping_-methods"),
            ("What are traits?", "what-are-traits"),
            ("Option's definition", "options-definition"),
            (
                "Function arguments are variables",
                "function-arguments-are-variables",
            ),
            ("if clauses", "if-clauses"),
        ];
        for (text, want) in cases {
            assert_eq!(slug(text), want, "slug of {text:?}");
        }
    }

    #[test]
    fn whitespace_and_unicode() {
        assert_eq!(slug("  Hello   World  "), "hello---world");
        assert_eq!(slug("Çalışma Örneği"), "çalışma-örneği");
        assert_eq!(slug("a\tb"), "a-b");
    }

    #[test]
    fn duplicates_get_counters() {
        let mut g = IdGen::default();
        assert_eq!(g.next_id("Example", None), "example");
        assert_eq!(g.next_id("Example", None), "example-1");
        assert_eq!(g.next_id("Other", Some("custom")), "custom");
        assert_eq!(g.next_id("Example", None), "example-2");
    }
}
