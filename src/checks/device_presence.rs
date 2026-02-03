//! Check 2: Audio Device Presence
//! Detects whether at least one audio card exists.

use crate::runner::run_command;
use crate::types::CheckResult;

const CHECK_NAME: &str = "audio_devices";

/// Check if any audio devices are present on the system.
pub fn check_audio_devices() -> CheckResult {
    let output = run_command("aplay", &["-l"]);
    let debug_info = format!("aplay -l:\n{}{}", output.stdout, output.stderr);

    if !output.success {
        // aplay might not be installed
        if output.stderr.contains("not found") || output.stderr.contains("No such file") {
            return CheckResult::warning(
                CHECK_NAME,
                "Cannot check audio devices (aplay not installed)",
                "Install alsa-utils package for full diagnostics",
            )
            .with_debug(debug_info);
        }
    }

    // Check for "no soundcards found" message
    if output.stderr.contains("no soundcards found")
        || output.stdout.contains("no soundcards found")
    {
        return CheckResult::error(
            CHECK_NAME,
            "No audio devices detected",
            "Possible cause: missing driver or disabled device in BIOS",
        )
        .with_debug(debug_info);
    }

    // Check if there's at least one card listed
    let has_cards = output.stdout.lines().any(|line| line.starts_with("card "));

    if has_cards {
        // Count the number of cards
        let card_count = output
            .stdout
            .lines()
            .filter(|line| line.starts_with("card "))
            .count();

        CheckResult::ok(
            CHECK_NAME,
            format!("{} audio device(s) detected", card_count),
        )
        .with_debug(debug_info)
    } else {
        CheckResult::error(
            CHECK_NAME,
            "No audio devices detected",
            "Possible cause: missing driver or disabled device in BIOS",
        )
        .with_debug(debug_info)
    }
}
