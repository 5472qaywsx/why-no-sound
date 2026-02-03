//! Check 3: Default Sink Validity
//! Detects if the default sink exists, is not suspended, and is not a disconnected HDMI.

use crate::runner::run_command;
use crate::types::CheckResult;

const CHECK_NAME: &str = "default_sink";

/// Check if the default sink is valid and usable.
pub fn check_default_sink() -> CheckResult {
    let mut debug_info = String::new();

    // Get default sink name
    let default_sink_output = run_command("pactl", &["get-default-sink"]);
    debug_info.push_str(&format!(
        "pactl get-default-sink:\n{}\n",
        default_sink_output.stdout.trim()
    ));

    if !default_sink_output.success {
        return CheckResult::error(
            CHECK_NAME,
            "Cannot determine default sink (audio server not responding)",
            "Ensure PipeWire or PulseAudio is running",
        )
        .with_debug(debug_info);
    }

    let default_sink = default_sink_output.stdout.trim();
    if default_sink.is_empty() {
        return CheckResult::error(
            CHECK_NAME,
            "No default sink configured",
            "Set a default output device in your sound settings",
        )
        .with_debug(debug_info);
    }

    // Get sink details
    let sinks_output = run_command("pactl", &["list", "sinks"]);
    debug_info.push_str(&format!(
        "pactl list sinks (truncated):\n{}\n",
        sinks_output.stdout.chars().take(2000).collect::<String>()
    ));

    if !sinks_output.success {
        return CheckResult::warning(CHECK_NAME, "Cannot list sinks", "Check audio server status")
            .with_debug(debug_info);
    }

    // Parse the sinks to find the default one
    let sink_info = parse_sink_info(&sinks_output.stdout, default_sink);

    match sink_info {
        None => CheckResult::error(
            CHECK_NAME,
            format!("Default sink '{}' not found in sink list", default_sink),
            "Your default audio device may have been removed. Select a new output device.",
        )
        .with_debug(debug_info),
        Some(info) => {
            // Check for SUSPENDED state
            if info.state.to_uppercase() == "SUSPENDED" {
                return CheckResult::warning(
                    CHECK_NAME,
                    "Default sink is SUSPENDED (no active audio streams)",
                    "This is normal when nothing is playing. Try playing audio.",
                )
                .with_debug(debug_info);
            }

            // Check for HDMI that might be disconnected
            let is_hdmi = info.name.to_lowercase().contains("hdmi")
                || info.description.to_lowercase().contains("hdmi");

            if is_hdmi {
                // Check if there's an active port or if it's unplugged
                if info.active_port.to_lowercase().contains("unavailable")
                    || info.port_availability == "not available"
                {
                    return CheckResult::error(
                        CHECK_NAME,
                        format!(
                            "Default output is HDMI ({}) but appears disconnected",
                            info.description
                        ),
                        "Switch output to Built-in Audio or connect your HDMI display",
                    )
                    .with_debug(debug_info);
                }
            }

            // Sink is valid
            CheckResult::ok(CHECK_NAME, format!("Default sink: {}", info.description))
                .with_debug(debug_info)
        }
    }
}

struct SinkInfo {
    name: String,
    description: String,
    state: String,
    active_port: String,
    port_availability: String,
}

fn parse_sink_info(output: &str, target_sink: &str) -> Option<SinkInfo> {
    let mut current_name = String::new();
    let mut current_desc = String::new();
    let mut current_state = String::new();
    let mut current_active_port = String::new();
    let mut current_port_availability = String::new();
    let mut in_target_sink = false;
    let mut in_ports_section = false;

    for line in output.lines() {
        let trimmed = line.trim();

        // Detect sink boundary
        if trimmed.starts_with("Name:") {
            // Save previous sink if it was the target
            if in_target_sink {
                return Some(SinkInfo {
                    name: current_name,
                    description: current_desc,
                    state: current_state,
                    active_port: current_active_port,
                    port_availability: current_port_availability,
                });
            }

            current_name = trimmed
                .strip_prefix("Name:")
                .unwrap_or("")
                .trim()
                .to_string();
            in_target_sink = current_name == target_sink;
            current_desc.clear();
            current_state.clear();
            current_active_port.clear();
            current_port_availability.clear();
            in_ports_section = false;
        }

        if !in_target_sink {
            continue;
        }

        if trimmed.starts_with("Description:") {
            current_desc = trimmed
                .strip_prefix("Description:")
                .unwrap_or("")
                .trim()
                .to_string();
        } else if trimmed.starts_with("State:") {
            current_state = trimmed
                .strip_prefix("State:")
                .unwrap_or("")
                .trim()
                .to_string();
        } else if trimmed.starts_with("Active Port:") {
            current_active_port = trimmed
                .strip_prefix("Active Port:")
                .unwrap_or("")
                .trim()
                .to_string();
        } else if trimmed.starts_with("Ports:") {
            in_ports_section = true;
        } else if in_ports_section && trimmed.contains(&current_active_port) {
            // Look for availability in the port line
            if trimmed.contains("not available") {
                current_port_availability = "not available".to_string();
            } else if trimmed.contains("available") {
                current_port_availability = "available".to_string();
            }
        }
    }

    // Check the last sink
    if in_target_sink {
        return Some(SinkInfo {
            name: current_name,
            description: current_desc,
            state: current_state,
            active_port: current_active_port,
            port_availability: current_port_availability,
        });
    }

    None
}
