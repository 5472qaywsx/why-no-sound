//! Audio diagnostic checks module.

mod audio_stack;
mod bluetooth;
mod device_presence;
mod mute_state;
mod sink_inputs;
mod sink_validity;

pub use audio_stack::check_audio_stack;
pub use bluetooth::check_bluetooth_profile;
pub use device_presence::check_audio_devices;
pub use mute_state::check_mute_state;
pub use sink_inputs::check_sink_inputs;
pub use sink_validity::check_default_sink;

use crate::types::CheckResult;

/// Run all diagnostic checks in the correct order.
/// Returns results in a deterministic order for consistent reporting.
pub fn run_all_checks() -> Vec<CheckResult> {
    vec![
        check_audio_stack(),
        check_audio_devices(),
        check_default_sink(),
        check_mute_state(),
        check_sink_inputs(),
        check_bluetooth_profile(),
    ]
}
