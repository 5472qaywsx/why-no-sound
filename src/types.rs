//! Core types for the why-no-sound diagnostic tool.

use serde::{Deserialize, Serialize};

/// Status of a diagnostic check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Ok,
    Warning,
    Error,
}

impl CheckStatus {
    /// Returns the emoji representation for human output.
    pub fn emoji(&self) -> &'static str {
        match self {
            CheckStatus::Ok => "✅",
            CheckStatus::Warning => "⚠️",
            CheckStatus::Error => "❌",
        }
    }
}

/// Result of a single diagnostic check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Name of the check (for identification).
    pub name: String,
    /// Status of the check.
    pub status: CheckStatus,
    /// Human-readable message explaining the result.
    pub message: String,
    /// Optional suggestion for fixing the issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    /// Debug information (raw command output).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_info: Option<String>,
}

impl CheckResult {
    /// Create a new OK result.
    pub fn ok(name: &str, message: impl Into<String>) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Ok,
            message: message.into(),
            suggestion: None,
            debug_info: None,
        }
    }

    /// Create a new Warning result.
    pub fn warning(name: &str, message: impl Into<String>, suggestion: impl Into<String>) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Warning,
            message: message.into(),
            suggestion: Some(suggestion.into()),
            debug_info: None,
        }
    }

    /// Create a new Error result.
    pub fn error(name: &str, message: impl Into<String>, suggestion: impl Into<String>) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Error,
            message: message.into(),
            suggestion: Some(suggestion.into()),
            debug_info: None,
        }
    }

    /// Add debug info to this result.
    pub fn with_debug(mut self, debug: impl Into<String>) -> Self {
        self.debug_info = Some(debug.into());
        self
    }
}

/// The final diagnostic report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    /// All check results in order.
    pub checks: Vec<CheckResult>,
    /// Summary of the diagnosis.
    pub summary: String,
    /// Probable root cause (if any issues found).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probable_cause: Option<String>,
    /// Ordered list of suggested fixes.
    pub suggested_fixes: Vec<String>,
}
