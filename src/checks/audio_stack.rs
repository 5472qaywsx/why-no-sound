//! Check 1: Audio Stack Status
//! Detects whether PipeWire, WirePlumber, or PulseAudio is running.

use crate::runner::run_command;
use crate::types::CheckResult;

const CHECK_NAME: &str = "audio_stack";

/// Check the status of the audio stack (PipeWire, WirePlumber, PulseAudio).
pub fn check_audio_stack() -> CheckResult {
    let mut debug_info = String::new();

    // Check PipeWire
    let pipewire_output = run_command("systemctl", &["--user", "is-active", "pipewire"]);
    debug_info.push_str(&format!(
        "systemctl --user is-active pipewire:\n{}\n",
        pipewire_output.stdout.trim()
    ));
    let pipewire_running = pipewire_output.stdout.trim() == "active";

    // Check WirePlumber
    let wireplumber_output = run_command("systemctl", &["--user", "is-active", "wireplumber"]);
    debug_info.push_str(&format!(
        "systemctl --user is-active wireplumber:\n{}\n",
        wireplumber_output.stdout.trim()
    ));
    let wireplumber_running = wireplumber_output.stdout.trim() == "active";

    // Check PulseAudio via pactl
    let pactl_output = run_command("pactl", &["info"]);
    debug_info.push_str(&format!(
        "pactl info (first 500 chars):\n{}\n",
        pactl_output.stdout.chars().take(500).collect::<String>()
    ));
    let pactl_works = pactl_output.success;

    // Determine the server name from pactl info
    let server_name = pactl_output
        .stdout
        .lines()
        .find(|line| line.starts_with("Server Name:"))
        .map(|line| line.trim_start_matches("Server Name:").trim())
        .unwrap_or("");

    let is_pipewire_pulse = server_name.to_lowercase().contains("pipewire");

    // Analyze the results
    if pipewire_running && wireplumber_running {
        CheckResult::ok(CHECK_NAME, "PipeWire and WirePlumber are running").with_debug(debug_info)
    } else if pipewire_running && !wireplumber_running {
        CheckResult::warning(
            CHECK_NAME,
            "PipeWire is running but WirePlumber is not",
            "Start WirePlumber: systemctl --user start wireplumber",
        )
        .with_debug(debug_info)
    } else if !pipewire_running && pactl_works && !is_pipewire_pulse {
        // PulseAudio fallback mode
        CheckResult::ok(CHECK_NAME, "PulseAudio is running (legacy mode)").with_debug(debug_info)
    } else if !pipewire_running && pactl_works && is_pipewire_pulse {
        // PipeWire-pulse is responding but systemd says pipewire isn't active
        // This can happen with socket activation
        CheckResult::ok(CHECK_NAME, "PipeWire is running (socket-activated)").with_debug(debug_info)
    } else if !pactl_works {
        CheckResult::error(
            CHECK_NAME,
            "No audio server detected",
            "Start PipeWire: systemctl --user start pipewire pipewire-pulse wireplumber",
        )
        .with_debug(debug_info)
    } else {
        CheckResult::warning(
            CHECK_NAME,
            "Audio stack status is unclear",
            "Check your audio server manually: systemctl --user status pipewire",
        )
        .with_debug(debug_info)
    }
}
