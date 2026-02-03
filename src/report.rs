//! Report aggregation and analysis.

use crate::types::{CheckResult, CheckStatus, DiagnosticReport};

/// Build a diagnostic report from check results.
pub fn build_report(checks: Vec<CheckResult>) -> DiagnosticReport {
    // Collect issues
    let errors: Vec<&CheckResult> = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Error)
        .collect();

    let warnings: Vec<&CheckResult> = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Warning)
        .collect();

    // Determine probable root cause (first error is most likely)
    let probable_cause = errors.first().map(|e| e.message.clone());

    // Build summary
    let summary = if errors.is_empty() && warnings.is_empty() {
        "Audio system appears healthy. If you still have no sound, the issue may be application-specific.".to_string()
    } else if errors.is_empty() {
        format!(
            "No critical issues found, but {} warning(s) detected that may affect audio.",
            warnings.len()
        )
    } else {
        format!(
            "Found {} error(s) and {} warning(s). Most likely cause: {}",
            errors.len(),
            warnings.len(),
            probable_cause.as_deref().unwrap_or("unknown")
        )
    };

    // Build ordered fix list (errors first, then warnings)
    let mut suggested_fixes: Vec<String> = Vec::new();

    for check in &errors {
        if let Some(ref suggestion) = check.suggestion {
            suggested_fixes.push(format!("{}: {}", check.message, suggestion));
        }
    }

    for check in &warnings {
        if let Some(ref suggestion) = check.suggestion {
            // Don't duplicate fixes
            let fix = format!("{}: {}", check.message, suggestion);
            if !suggested_fixes.contains(&fix) {
                suggested_fixes.push(fix);
            }
        }
    }

    DiagnosticReport {
        checks,
        summary,
        probable_cause,
        suggested_fixes,
    }
}
