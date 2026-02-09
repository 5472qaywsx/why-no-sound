# why-no-sound

A single-binary Linux CLI tool that diagnoses why your audio isn't working and explains it in plain English.

## What This Tool Does

- **Detects audio server status** — PipeWire, WirePlumber, PulseAudio
- **Checks for audio devices** — Are your sound cards detected?
- **Validates your default output** — Is it valid? Connected? HDMI to nowhere?
- **Finds muted outputs** — The #1 cause of "no sound"
- **Detects misrouted streams** — Apps playing to the wrong device
- **Catches Bluetooth traps** — HSP/HFP mode instead of A2DP

## What This Tool Does NOT Do

- ❌ Modify your system (read-only diagnostics)
- ❌ Dump raw logs at you
- ❌ Require root privileges
- ❌ Depend on your distro
- ❌ Need network access
- ❌ Run as a daemon

## Installation

`git clone` this repo to your machine.

```bash
cargo build --release
sudo cp target/release/why-no-sound /usr/local/bin/
```

## Usage

```bash
# Human-readable diagnosis
why-no-sound

# JSON output for scripts
why-no-sound --json

# Include raw command output for debugging
why-no-sound --debug
```

## Example Output

```
🔊 why-no-sound — Linux Audio Diagnostic
─────────────────────────────────────────

✅ PipeWire and WirePlumber are running
✅ 2 audio device(s) detected
❌ Default output is HDMI (Intel Display Audio) but appears disconnected
   👉 Fix: Switch output to Built-in Audio or connect your HDMI display
✅ Output is not muted (volume: 75%)
✅ No active audio streams (nothing playing)
✅ No Bluetooth audio issues

─────────────────────────────────────────

❌ DIAGNOSIS: Issues detected

Found 1 error(s) and 0 warning(s). Most likely cause: Default output is HDMI...

🎯 Probable root cause:
   Default output is HDMI (Intel Display Audio) but appears disconnected

📋 Suggested fixes (in order):
   1. Switch output to Built-in Audio or connect your HDMI display
```

## Supported Systems

- **Audio Servers**: PipeWire, WirePlumber, PulseAudio
- **Hardware**: Any ALSA-compatible audio device
- **Distros**: Any Linux distribution (distro-agnostic)

## Philosophy

> "Linux audio isn't broken. It's just silent without explanation."

This tool exists because audio problems on Linux are rarely mysterious — but the answers are buried in arcane commands. `why-no-sound` runs those commands for you and translates the results into actionable information.

## Architecture

```
src/
├── main.rs          # CLI entry point (clap)
├── types.rs         # CheckResult, CheckStatus, DiagnosticReport
├── runner.rs        # Safe command execution
├── report.rs        # Report aggregation & root cause analysis
├── output.rs        # Human/JSON rendering
└── checks/
    ├── mod.rs            # Check orchestration
    ├── audio_stack.rs    # PipeWire/WirePlumber/PulseAudio
    ├── device_presence.rs # aplay -l
    ├── sink_validity.rs  # Default sink validation
    ├── mute_state.rs     # Mute detection
    ├── sink_inputs.rs    # Stream routing
    └── bluetooth.rs      # A2DP vs HSP/HFP
```

Each check is a pure function returning a `CheckResult`. Checks never panic and never print directly.

## Dependencies

Only four crates (by design):

- `clap` — CLI argument parsing
- `serde` — Serialization
- `serde_json` — JSON output
- `anyhow` — Error handling

## License

MIT
