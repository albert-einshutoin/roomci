//! `roomci` command-line entry point.
//!
//! Two subcommands:
//!
//! - `roomci run <scenarios...>` — load, validate, and execute one or more
//!   scenarios. Optionally emits JSON/Markdown/JUnit reports for the *last*
//!   scenario and exits non-zero if any scenario fails. With `--dry-run` only
//!   validates without executing. With `--verbose` prints the timeline; with
//!   `--quiet` suppresses per-scenario detail.
//! - `roomci validate <scenarios...>` — load and validate one or more scenarios
//!   without executing them.

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{ArgGroup, Parser, Subcommand};
use roomci_core::{run_scenario, RunReport, RunResult};
use roomci_report::{to_json, to_junit, to_markdown};
use roomci_scenario::{load_scenario, validate_scenario};
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(
    name = "roomci",
    version,
    about = "Local-first smart-home QA and operations emulator for CI"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run one or more scenarios and emit reports for the last one.
    #[command(group(ArgGroup::new("verbosity").args(["verbose", "quiet"])))]
    Run {
        /// Scenario YAML files to execute, in order.
        #[arg(required = true)]
        scenarios: Vec<PathBuf>,
        /// Write a JUnit XML report for the last scenario.
        #[arg(long)]
        junit: Option<PathBuf>,
        /// Write a Markdown report for the last scenario.
        #[arg(long, alias = "report-md")]
        markdown: Option<PathBuf>,
        /// Write a JSON report for the last scenario.
        #[arg(long, alias = "report-json")]
        json: Option<PathBuf>,
        /// Print every timeline event for each scenario.
        #[arg(long)]
        verbose: bool,
        /// Suppress per-scenario detail; only print the aggregate summary.
        #[arg(long)]
        quiet: bool,
        /// Validate only; do not execute scenarios.
        #[arg(long)]
        dry_run: bool,
    },
    /// Validate scenario files.
    Validate {
        #[arg(required = true)]
        scenarios: Vec<PathBuf>,
    },
}

#[derive(Debug, Error)]
enum CliError {
    #[error(transparent)]
    Scenario(#[from] roomci_scenario::ScenarioError),
    #[error(transparent)]
    Core(#[from] roomci_core::CoreError),
    #[error("failed to render JSON report: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to write {path}: {source}")]
    Write {
        path: String,
        source: std::io::Error,
    },
}

fn main() -> ExitCode {
    match run_cli(Cli::parse()) {
        Ok(exit) => exit,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}

fn run_cli(cli: Cli) -> Result<ExitCode, CliError> {
    match cli.command {
        Command::Run {
            scenarios,
            junit,
            markdown,
            json,
            verbose,
            quiet,
            dry_run,
        } => run_scenarios(RunOptions {
            scenarios,
            junit,
            markdown,
            json,
            verbose,
            quiet,
            dry_run,
        }),
        Command::Validate { scenarios } => {
            for scenario in scenarios {
                let scenario_file = load_scenario(&scenario)?;
                validate_scenario(&scenario_file)?;
                println!("valid: {}", scenario.display());
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

struct RunOptions {
    scenarios: Vec<PathBuf>,
    junit: Option<PathBuf>,
    markdown: Option<PathBuf>,
    json: Option<PathBuf>,
    verbose: bool,
    quiet: bool,
    dry_run: bool,
}

fn run_scenarios(options: RunOptions) -> Result<ExitCode, CliError> {
    let total = options.scenarios.len();
    let mut passed = 0_usize;
    let mut failed = 0_usize;
    let mut last_report: Option<RunReport> = None;

    for (index, path) in options.scenarios.iter().enumerate() {
        let scenario_file = load_scenario(path)?;
        validate_scenario(&scenario_file)?;

        if options.dry_run {
            if !options.quiet {
                println!(
                    "[{n}/{total}] dry-run valid: {path}",
                    n = index + 1,
                    total = total,
                    path = path.display()
                );
            }
            passed += 1;
            continue;
        }

        let report = run_scenario(&scenario_file)?;
        match report.result {
            RunResult::Passed => passed += 1,
            RunResult::Failed => failed += 1,
        }

        if !options.quiet {
            print_scenario_summary(index + 1, total, path, &report, options.verbose);
        }

        last_report = Some(report);
    }

    if let Some(report) = last_report.as_ref() {
        if let Some(path) = &options.json {
            write_file(path, &to_json(report)?)?;
        }
        if let Some(path) = &options.markdown {
            write_file(path, &to_markdown(report))?;
        }
        if let Some(path) = &options.junit {
            write_file(path, &to_junit(report))?;
        }
    }

    println!(
        "summary: {passed} passed, {failed} failed (of {total})",
        passed = passed,
        failed = failed,
        total = total
    );

    Ok(if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn print_scenario_summary(
    index: usize,
    total: usize,
    path: &Path,
    report: &RunReport,
    verbose: bool,
) {
    println!(
        "[{index}/{total}] scenario: {name} ({path})",
        index = index,
        total = total,
        name = report.scenario_name,
        path = path.display()
    );
    println!("  result: {:?}", report.result);
    println!("  assertions: {}", report.assertions.len());

    if verbose {
        for event in &report.timeline {
            let target = event.target.as_deref().unwrap_or("-");
            println!(
                "    {at} [{event_type}] {target}: {message}",
                at = event.at,
                event_type = event.event_type,
                target = target,
                message = event.message
            );
        }
    }
}

fn write_file(path: &Path, contents: &str) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| CliError::Write {
            path: parent.display().to_string(),
            source,
        })?;
    }
    fs::write(path, contents).map_err(|source| CliError::Write {
        path: path.display().to_string(),
        source,
    })
}
