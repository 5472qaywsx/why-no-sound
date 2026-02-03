//! Check 5: Active Streams Misrouted
//! Detects if apps are bound to a non-default sink.

use crate::runner::run_command;
use crate::types::CheckResult;

const CHECK_NAME: &str = "sink_inputs";

/// Check if there are active stream inputs that might be misrouted.
pub fn check_sink_inputs() -> CheckResult {
    let mut debug_info = String::new();

    // Get default sink
    let default_sink_output = run_command("pactl", &["get-default-sink"]);
    let default_sink = default_sink_output.stdout.trim().to_string();

    if default_sink.is_empty() {
        return CheckResult::warning(
            CHECK_NAME,
            "Cannot check stream routing (no default sink)",
            "Set a default output device first",
        );
    }

    // Get sink inputs
    let inputs_output = run_command("pactl", &["list", "sink-inputs"]);
    debug_info.push_str(&format!(
        "pactl list sink-inputs:\n{}\n",
        inputs_output.stdout
    ));

    if !inputs_output.success {
        return CheckResult::warning(
            CHECK_NAME,
            "Cannot list active audio streams",
            "Ensure audio server is running",
        )
        .with_debug(debug_info);
    }

    // Parse sink inputs
    let inputs = parse_sink_inputs(&inputs_output.stdout);

    if inputs.is_empty() {
        return CheckResult::ok(CHECK_NAME, "No active audio streams (nothing playing)")
            .with_debug(debug_info);
    }

    // Get sink index to name mapping
    let sinks_output = run_command("pactl", &["list", "sinks"]);
    let sink_map = parse_sink_index_map(&sinks_output.stdout);

    // Check for misrouted streams
    let mut misrouted: Vec<String> = Vec::new();

    for input in &inputs {
        // Resolve sink index to name if possible
        let sink_name = sink_map
            .iter()
            .find(|(idx, _)| *idx == input.sink_index)
            .map(|(_, name)| name.as_str())
            .unwrap_or("");

        if !sink_name.is_empty() && sink_name != default_sink {
            misrouted.push(format!(
                "'{}' is playing to '{}'",
                input.app_name, sink_name
            ));
        }
    }

    if !misrouted.is_empty() {
        CheckResult::warning(
            CHECK_NAME,
            format!(
                "{} stream(s) playing to non-default output: {}",
                misrouted.len(),
                misrouted.join(", ")
            ),
            "Move streams to default output in sound settings or pavucontrol",
        )
        .with_debug(debug_info)
    } else {
        CheckResult::ok(
            CHECK_NAME,
            format!("{} active stream(s) correctly routed", inputs.len()),
        )
        .with_debug(debug_info)
    }
}

struct SinkInput {
    app_name: String,
    sink_index: u32,
}

fn parse_sink_inputs(output: &str) -> Vec<SinkInput> {
    let mut inputs = Vec::new();
    let mut current_sink_index: Option<u32> = None;
    let mut current_app_name = String::new();

    for line in output.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Sink:") {
            // Save previous input
            if let Some(idx) = current_sink_index {
                inputs.push(SinkInput {
                    app_name: if current_app_name.is_empty() {
                        "Unknown".to_string()
                    } else {
                        current_app_name.clone()
                    },
                    sink_index: idx,
                });
            }

            // Parse new input
            let sink_str = trimmed.strip_prefix("Sink:").unwrap_or("").trim();
            current_sink_index = sink_str.parse().ok();
            current_app_name.clear();
        }

        if trimmed.starts_with("application.name = ") {
            current_app_name = trimmed
                .strip_prefix("application.name = ")
                .unwrap_or("")
                .trim_matches('"')
                .to_string();
        }

        // Fallback to media.name if no application.name
        if current_app_name.is_empty() && trimmed.starts_with("media.name = ") {
            current_app_name = trimmed
                .strip_prefix("media.name = ")
                .unwrap_or("")
                .trim_matches('"')
                .to_string();
        }
    }

    // Don't forget the last one
    if let Some(idx) = current_sink_index {
        inputs.push(SinkInput {
            app_name: if current_app_name.is_empty() {
                "Unknown".to_string()
            } else {
                current_app_name
            },
            sink_index: idx,
        });
    }

    inputs
}

fn parse_sink_index_map(output: &str) -> Vec<(u32, String)> {
    let mut map = Vec::new();
    let mut current_index: Option<u32> = None;

    for line in output.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Sink #") {
            current_index = trimmed.strip_prefix("Sink #").and_then(|s| s.parse().ok());
        }

        if let Some(idx) = current_index {
            if trimmed.starts_with("Name:") {
                let name = trimmed.strip_prefix("Name:").unwrap_or("").trim();
                map.push((idx, name.to_string()));
                current_index = None;
            }
        }
    }

    map
}
