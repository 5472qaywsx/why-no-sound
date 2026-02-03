//! Output rendering for human and JSON formats.

use crate::types::{CheckStatus, DiagnosticReport};

/// Print the report in human-readable format.
pub fn print_human(report: &DiagnosticReport, debug: bool) {
    println!();
    println!("🔊 why-no-sound — Linux Audio Diagnostic");
    println!("─────────────────────────────────────────");
    println!();

    // Print each check result
    for check in &report.checks {
        let emoji = check.status.emoji();
        println!("{} {}", emoji, check.message);

        if let Some(ref suggestion) = check.suggestion {
            println!("   👉 Fix: {}", suggestion);
        }

        if debug {
            if let Some(ref debug_info) = check.debug_info {
                println!();
                println!("   [DEBUG: {}]", check.name);
                for line in debug_info.lines() {
                    println!("   | {}", line);
                }
                println!();
            }
        }
    }

    // Print summary
    println!();
    println!("─────────────────────────────────────────");
    println!();

    // Determine overall status
    let has_errors = report.checks.iter().any(|c| c.status == CheckStatus::Error);
    let has_warnings = report
        .checks
        .iter()
        .any(|c| c.status == CheckStatus::Warning);

    if has_errors {
        println!("❌ DIAGNOSIS: Issues detected");
    } else if has_warnings {
        println!("⚠️  DIAGNOSIS: Potential issues");
    } else {
        println!("✅ DIAGNOSIS: System looks healthy");
    }

    println!();
    println!("{}", report.summary);

    if let Some(ref cause) = report.probable_cause {
        println!();
        println!("🎯 Probable root cause:");
        println!("   {}", cause);
    }

    if !report.suggested_fixes.is_empty() {
        println!();
        println!("📋 Suggested fixes (in order):");
        for (i, fix) in report.suggested_fixes.iter().enumerate() {
            println!("   {}. {}", i + 1, fix);
        }
    }

    println!();
}

/// Print the report as JSON.
pub fn print_json(report: &DiagnosticReport) {
    // Create a JSON-friendly version without debug info unless needed
    match serde_json::to_string_pretty(report) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing report to JSON: {}", e),
    }
}
