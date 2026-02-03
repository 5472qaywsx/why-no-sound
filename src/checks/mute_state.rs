//! Check 4: Mute State (Critical)
//! Detects if audio is muted at the sink level.

use crate::runner::run_command;
use crate::types::CheckResult;

const CHECK_NAME: &str = "mute_state";

/// Check if audio is muted at the sink level.
pub fn check_mute_state() -> CheckResult {
    let mut debug_info = String::new();

    // Get default sink
    let default_sink_output = run_command("pactl", &["get-default-sink"]);
    let default_sink = default_sink_output.stdout.trim();

    if default_sink.is_empty() {
        return CheckResult::warning(
            CHECK_NAME,
            "Cannot check mute state (no default sink)",
            "Set a default output device first",
        );
    }

    // Get sink mute status
    let sinks_output = run_command("pactl", &["list", "sinks"]);
    debug_info.push_str(&format!(
        "pactl list sinks (mute info):\n{}\n",
        sinks_output
            .stdout
            .lines()
            .filter(|l| { l.contains("Name:") || l.contains("Mute:") || l.contains("Volume:") })
            .collect::<Vec<_>>()
            .join("\n")
    ));

    if !sinks_output.success {
        return CheckResult::warning(
            CHECK_NAME,
            "Cannot check mute state",
            "Ensure audio server is running",
        )
        .with_debug(debug_info);
    }

    // Parse sink info to find mute status and volume
    let (is_muted, volume_percent) = parse_mute_and_volume(&sinks_output.stdout, default_sink);

    match is_muted {
        Some(true) => CheckResult::error(
            CHECK_NAME,
            "Output is muted",
            "Unmute in sound settings or press the mute key",
        )
        .with_debug(debug_info),
        Some(false) => {
            // Check for very low volume
            if let Some(vol) = volume_percent {
                if vol < 5 {
                    return CheckResult::warning(
                        CHECK_NAME,
                        format!("Volume is very low ({}%)", vol),
                        "Increase volume in sound settings",
                    )
                    .with_debug(debug_info);
                }
                CheckResult::ok(
                    CHECK_NAME,
                    format!("Output is not muted (volume: {}%)", vol),
                )
                .with_debug(debug_info)
            } else {
                CheckResult::ok(CHECK_NAME, "Output is not muted").with_debug(debug_info)
            }
        }
        None => CheckResult::warning(
            CHECK_NAME,
            "Could not determine mute state",
            "Check sound settings manually",
        )
        .with_debug(debug_info),
    }
}

fn parse_mute_and_volume(output: &str, target_sink: &str) -> (Option<bool>, Option<u32>) {
    let mut in_target_sink = false;
    let mut muted: Option<bool> = None;
    let mut volume: Option<u32> = None;

    for line in output.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Name:") {
            let name = trimmed.strip_prefix("Name:").unwrap_or("").trim();
            in_target_sink = name == target_sink;
            if !in_target_sink {
                // Reset if we're in a different sink
                continue;
            }
        }

        if !in_target_sink {
            continue;
        }

        if trimmed.starts_with("Mute:") {
            let mute_value = trimmed.strip_prefix("Mute:").unwrap_or("").trim();
            muted = Some(mute_value.eq_ignore_ascii_case("yes"));
        }

        if trimmed.starts_with("Volume:") && volume.is_none() {
            // Parse volume percentage from a line like:
            // Volume: front-left: 65536 / 100% / 0.00 dB,   front-right: 65536 / 100% / 0.00 dB
            if let Some(percent_pos) = trimmed.find('%') {
                // Find the number before the %
                let before_percent = &trimmed[..percent_pos];
                let num_start = before_percent
                    .rfind(|c: char| !c.is_ascii_digit())
                    .map(|i| i + 1)
                    .unwrap_or(0);
                if let Ok(vol) = before_percent[num_start..].parse::<u32>() {
                    volume = Some(vol);
                }
            }
        }

        // Stop after we have both values
        if muted.is_some() && volume.is_some() {
            break;
        }
    }

    (muted, volume)
}
