use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        })
    }
}

/// A problem in a translated file. `line` is 1-based and refers to the translation.
#[derive(Debug, Clone)]
pub struct Issue {
    pub severity: Severity,
    pub line: Option<usize>,
    pub msg: String,
}

impl Issue {
    pub fn error(line: Option<usize>, msg: impl Into<String>) -> Self {
        Issue {
            severity: Severity::Error,
            line,
            msg: msg.into(),
        }
    }

    pub fn warning(line: Option<usize>, msg: impl Into<String>) -> Self {
        Issue {
            severity: Severity::Warning,
            line,
            msg: msg.into(),
        }
    }
}
