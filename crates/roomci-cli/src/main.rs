//! `roomci` command-line entry point.
//!
//! Two subcommands:
//!
//! - `roomci run <scenario>` — load, validate, execute a scenario, optionally
//!   emit JSON/Markdown/JUnit reports, and exit non-zero on assertion failure.
//! - `roomci validate <scenarios...>` — load and validate one or more scenarios
//!   without executing them.

use std::{fs, path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};
use roomci_core::{run_scenario, RunResult};
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
    /// Run a scenario to completion and emit reports.
    Run {
        scenario: PathBuf,
        #[arg(long)]
        junit: Option<PathBuf>,
        #[arg(long, alias = "report-md")]
        markdown: Option<PathBuf>,
        #[arg(long, alias = "report-json")]
        json: Option<PathBuf>,
    },
    /// Validate scenario files.
    Validate { scenarios: Vec<PathBuf> },
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
    #[error("validate requires at least one scenario path")]
    MissingScenario,
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
            scenario,
            junit,
            markdown,
            json,
        } => {
            let scenario_file = load_scenario(&scenario)?;
            validate_scenario(&scenario_file)?;
            let report = run_scenario(&scenario_file)?;

            if let Some(path) = json {
                write_file(&path, &to_json(&report)?)?;
            }
            if let Some(path) = markdown {
                write_file(&path, &to_markdown(&report))?;
            }
            if let Some(path) = junit {
                write_file(&path, &to_junit(&report))?;
            }

            println!("scenario: {}", report.scenario_name);
            println!("result: {:?}", report.result);
            println!("assertions: {}", report.assertions.len());

            Ok(match report.result {
                RunResult::Passed => ExitCode::SUCCESS,
                RunResult::Failed => ExitCode::from(1),
            })
        }
        Command::Validate { scenarios } => {
            if scenarios.is_empty() {
                return Err(CliError::MissingScenario);
            }
            for scenario in scenarios {
                let scenario_file = load_scenario(&scenario)?;
                validate_scenario(&scenario_file)?;
                println!("valid: {}", scenario.display());
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn write_file(path: &PathBuf, contents: &str) -> Result<(), CliError> {
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
