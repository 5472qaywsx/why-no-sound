//! Check 6: Bluetooth Profile Trap
//! Detects if Bluetooth is using HSP/HFP instead of A2DP.

use crate::runner::run_command;
use crate::types::CheckResult;

const CHECK_NAME: &str = "bluetooth_profile";

/// Check if Bluetooth audio is in the wrong profile mode.
pub fn check_bluetooth_profile() -> CheckResult {
    let mut debug_info = String::new();

    // Get default sink to check if it's Bluetooth
    let default_sink_output = run_command("pactl", &["get-default-sink"]);
    let default_sink = default_sink_output.stdout.trim();

    // List cards to find Bluetooth devices
    let cards_output = run_command("pactl", &["list", "cards"]);
    debug_info.push_str(&format!(
        "pactl list cards (bluetooth info):\n{}\n",
        cards_output
            .stdout
            .lines()
            .filter(|l| {
                l.contains("Name:")
                    || l.contains("bluez")
                    || l.contains("bluetooth")
                    || l.contains("Active Profile:")
                    || l.contains("a2dp")
                    || l.contains("hsp")
                    || l.contains("hfp")
                    || l.contains("headset")
            })
            .collect::<Vec<_>>()
            .join("\n")
    ));

    if !cards_output.success {
        return CheckResult::ok(CHECK_NAME, "No Bluetooth audio issues (cannot list cards)")
            .with_debug(debug_info);
    }

    // Parse Bluetooth cards
    let bt_cards = parse_bluetooth_cards(&cards_output.stdout);

    if bt_cards.is_empty() {
        return CheckResult::ok(CHECK_NAME, "No Bluetooth audio devices connected")
            .with_debug(debug_info);
    }

    // Check each Bluetooth card for HSP/HFP profile
    let mut issues: Vec<String> = Vec::new();
    let mut has_active_bt = false;

    for card in &bt_cards {
        // Check if this card is the active sink
        let is_active = default_sink.contains(&card.name)
            || card.sinks.iter().any(|s| default_sink.contains(s.as_str()));

        if is_active {
            has_active_bt = true;
        }

        // Check for problematic profiles
        let profile_lower = card.active_profile.to_lowercase();
        let is_hsp_hfp = profile_lower.contains("hsp")
            || profile_lower.contains("hfp")
            || profile_lower.contains("headset-head-unit");

        let has_a2dp = card
            .available_profiles
            .iter()
            .any(|p| p.to_lowercase().contains("a2dp"));

        if is_hsp_hfp {
            if has_a2dp {
                issues.push(format!(
                    "'{}' is in call/headset mode ({}), A2DP available",
                    card.description, card.active_profile
                ));
            } else {
                issues.push(format!(
                    "'{}' is in call/headset mode ({}), A2DP not available",
                    card.description, card.active_profile
                ));
            }
        }
    }

    if !issues.is_empty() {
        let has_a2dp_available = issues.iter().any(|i| i.contains("A2DP available"));

        if has_a2dp_available && has_active_bt {
            return CheckResult::error(
                CHECK_NAME,
                format!("Bluetooth headset in call mode: {}", issues.join("; ")),
                "Switch Bluetooth profile to A2DP (high-quality audio) in sound settings",
            )
            .with_debug(debug_info);
        } else if has_active_bt {
            return CheckResult::warning(
                CHECK_NAME,
                format!("Bluetooth in low-quality mode: {}", issues.join("; ")),
                "A2DP profile may not be available. Check if device supports it.",
            )
            .with_debug(debug_info);
        } else {
            // Bluetooth is connected but not the active output
            return CheckResult::warning(
                CHECK_NAME,
                format!(
                    "Bluetooth device in call mode but not active output: {}",
                    issues.join("; ")
                ),
                "If using Bluetooth, switch profile to A2DP for better quality",
            )
            .with_debug(debug_info);
        }
    }

    CheckResult::ok(
        CHECK_NAME,
        if has_active_bt {
            "Bluetooth audio profile is optimal (A2DP)"
        } else if !bt_cards.is_empty() {
            "Bluetooth device connected with correct profile"
        } else {
            "No Bluetooth audio issues"
        },
    )
    .with_debug(debug_info)
}

struct BluetoothCard {
    name: String,
    description: String,
    active_profile: String,
    available_profiles: Vec<String>,
    sinks: Vec<String>,
}

fn parse_bluetooth_cards(output: &str) -> Vec<BluetoothCard> {
    let mut cards = Vec::new();
    let mut current_name = String::new();
    let mut current_desc = String::new();
    let mut current_profile = String::new();
    let mut current_profiles: Vec<String> = Vec::new();
    let mut current_sinks: Vec<String> = Vec::new();
    let mut in_profiles_section = false;
    let mut in_sinks_section = false;
    let mut is_bluetooth = false;

    for line in output.lines() {
        let trimmed = line.trim();

        // New card boundary
        if trimmed.starts_with("Name:") {
            // Save previous card if it's Bluetooth
            if is_bluetooth && !current_name.is_empty() {
                cards.push(BluetoothCard {
                    name: current_name.clone(),
                    description: current_desc.clone(),
                    active_profile: current_profile.clone(),
                    available_profiles: current_profiles.clone(),
                    sinks: current_sinks.clone(),
                });
            }

            // Reset
            current_name = trimmed
                .strip_prefix("Name:")
                .unwrap_or("")
                .trim()
                .to_string();
            is_bluetooth = current_name.contains("bluez") || current_name.contains("bluetooth");
            current_desc.clear();
            current_profile.clear();
            current_profiles.clear();
            current_sinks.clear();
            in_profiles_section = false;
            in_sinks_section = false;
        }

        if !is_bluetooth {
            continue;
        }

        if trimmed.starts_with("device.description = ") {
            current_desc = trimmed
                .strip_prefix("device.description = ")
                .unwrap_or("")
                .trim_matches('"')
                .to_string();
        }

        if trimmed.starts_with("Active Profile:") {
            current_profile = trimmed
                .strip_prefix("Active Profile:")
                .unwrap_or("")
                .trim()
                .to_string();
        }

        if trimmed.starts_with("Profiles:") {
            in_profiles_section = true;
            in_sinks_section = false;
            continue;
        }

        if trimmed.starts_with("Sinks:") {
            in_sinks_section = true;
            in_profiles_section = false;
            continue;
        }

        if in_profiles_section {
            // Profile lines look like: "a2dp-sink: A2DP Sink (sinks: 1, sources: 0, priority: 40, available: yes)"
            if let Some(colon_pos) = trimmed.find(':') {
                let profile_name = trimmed[..colon_pos].trim();
                if !profile_name.is_empty() && !profile_name.starts_with("Part of") {
                    current_profiles.push(profile_name.to_string());
                }
            }
        }

        if in_sinks_section {
            // Sink lines contain sink names
            if trimmed.contains("bluez") || trimmed.starts_with("#") {
                current_sinks.push(trimmed.to_string());
            }
        }
    }

    // Don't forget the last card
    if is_bluetooth && !current_name.is_empty() {
        cards.push(BluetoothCard {
            name: current_name,
            description: current_desc,
            active_profile: current_profile,
            available_profiles: current_profiles,
            sinks: current_sinks,
        });
    }

    cards
}
