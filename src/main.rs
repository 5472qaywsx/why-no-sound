//! why-no-sound: A diagnostic CLI tool for Linux audio.

mod checks;
mod output;
mod report;
mod runner;
mod types;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "why-no-sound")]
#[command(version, about = "Diagnose why Linux audio isn't working")]
struct Args {
    /// Output results as JSON
    #[arg(long)]
    json: bool,

    /// Include debug info
    #[arg(long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();
    let check_results = checks::run_all_checks();
    let mut report = report::build_report(check_results);

    if !args.debug {
        for check in &mut report.checks {
            check.debug_info = None;
        }
    }

    if args.json {
        output::print_json(&report);
    } else {
        output::print_human(&report, args.debug);
    }
}
