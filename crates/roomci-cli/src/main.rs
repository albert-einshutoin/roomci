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
//! - `roomci adapter validate <contracts...>` — load and validate one or more
//!   company adapter contract files without executing scenarios.
//! - `roomci debug <scenario>` — execute one scenario and emit deterministic
//!   debugging artifacts for timeline, state diff, and assertion review.
//! - `roomci serve --config <scenario>` — start a localhost-bound HTTP control
//!   and report API. With `--check`, only validates the service-mode config
//!   without starting a long-running process.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{ArgGroup, Parser, Subcommand};
use roomci_core::{run_scenario, RunReport, RunResult, StateMap};
use roomci_report::{
    to_json, to_junit, to_markdown, to_observability_json, to_timeline_json, to_timeline_ndjson,
};
use roomci_scenario::{
    load_adapter_contract, load_scenario, validate_adapter_contract, validate_scenario,
    yaml_map_to_json, ScenarioFile,
};
use roomci_serve::{run_serve, ServeOptions};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(
    name = "roomci",
    version,
    about = "MQTT / edge / device QA contract emulator for CI"
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
        /// Write a stable timeline JSON export for the last scenario.
        #[arg(long)]
        timeline_json: Option<PathBuf>,
        /// Write a stable newline-delimited timeline JSON export for the last scenario.
        #[arg(long)]
        timeline_ndjson: Option<PathBuf>,
        /// Write an observability summary JSON artifact for the last scenario.
        #[arg(long)]
        observability_json: Option<PathBuf>,
        /// Override the run identifier used in JSON, timeline, and observability artifacts.
        #[arg(long)]
        run_id: Option<String>,
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
    /// Explain scenario execution for authoring and failure debugging.
    Debug {
        /// Scenario YAML file to execute and explain.
        scenario: PathBuf,
        /// Write deterministic debug JSON.
        #[arg(long)]
        debug_json: Option<PathBuf>,
        /// Write deterministic debug Markdown.
        #[arg(long)]
        debug_md: Option<PathBuf>,
    },
    /// Validate company adapter contract files.
    Adapter {
        #[command(subcommand)]
        command: AdapterCommand,
    },
    /// Validate service-mode configuration.
    Serve {
        /// Scenario YAML file used as service-mode configuration.
        #[arg(long)]
        config: PathBuf,
        /// Validate service-mode config and exit without blocking.
        #[arg(long)]
        check: bool,
        /// HTTP bind host. Defaults to loopback for local and CI safety.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// HTTP bind port. Use 0 to let the OS choose a free port.
        #[arg(long, default_value_t = 8080)]
        port: u16,
        /// Optional embedded MQTT PoC port. Supports CONNECT + QoS0 PUBLISH.
        #[arg(long)]
        mqtt_port: Option<u16>,
        /// Optional embedded Modbus TCP PoC port.
        #[arg(long)]
        modbus_port: Option<u16>,
        /// Allow binding to a non-loopback host.
        #[arg(long)]
        allow_non_loopback: bool,
    },
}

#[derive(Debug, Subcommand)]
enum AdapterCommand {
    /// Validate one or more adapter contract YAML files.
    Validate {
        #[arg(required = true)]
        contracts: Vec<PathBuf>,
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
    #[error("serve error: {0}")]
    Serve(String),
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
            timeline_json,
            timeline_ndjson,
            observability_json,
            run_id,
            verbose,
            quiet,
            dry_run,
        } => run_scenarios(RunOptions {
            scenarios,
            junit,
            markdown,
            json,
            timeline_json,
            timeline_ndjson,
            observability_json,
            run_id,
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
        Command::Debug {
            scenario,
            debug_json,
            debug_md,
        } => debug_scenario(&scenario, debug_json.as_deref(), debug_md.as_deref()),
        Command::Adapter {
            command: AdapterCommand::Validate { contracts },
        } => {
            for contract in contracts {
                let adapter_contract = load_adapter_contract(&contract)?;
                validate_adapter_contract(&adapter_contract)?;
                println!("adapter contract valid: {}", contract.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Serve {
            config,
            check,
            host,
            port,
            mqtt_port,
            modbus_port,
            allow_non_loopback,
        } => serve_config(
            &config,
            check,
            &host,
            port,
            mqtt_port,
            modbus_port,
            allow_non_loopback,
        ),
    }
}

fn serve_config(
    config: &Path,
    check: bool,
    host: &str,
    port: u16,
    mqtt_port: Option<u16>,
    modbus_port: Option<u16>,
    allow_non_loopback: bool,
) -> Result<ExitCode, CliError> {
    let scenario = load_scenario(config)?;
    validate_scenario(&scenario)?;
    if check {
        println!("serve config valid: {}", config.display());
        return Ok(ExitCode::SUCCESS);
    }
    run_serve(ServeOptions {
        scenario,
        host: host.to_string(),
        port,
        mqtt_port,
        modbus_port,
        allow_non_loopback,
    })
    .map_err(|error| CliError::Serve(error.to_string()))?;
    Ok(ExitCode::SUCCESS)
}

#[derive(Debug, Serialize)]
struct DebugReport {
    schema_version: &'static str,
    scenario_name: String,
    result: RunResult,
    execution_order: Vec<DebugTimelineEvent>,
    state_diffs: Vec<DebugStateDiff>,
    assertions: Vec<DebugAssertion>,
    failure_causes: Vec<String>,
    suggested_checks: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DebugTimelineEvent {
    sequence: usize,
    at: String,
    event_type: String,
    target: Option<String>,
    message: String,
}

#[derive(Debug, Serialize)]
struct DebugStateDiff {
    target: String,
    before: Option<StateMap>,
    after: Option<StateMap>,
}

#[derive(Debug, Serialize)]
struct DebugAssertion {
    name: String,
    assertion_type: String,
    passed: bool,
    message: String,
    impact_level: Option<String>,
    impact_message: Option<String>,
}

fn debug_scenario(
    scenario_path: &Path,
    debug_json: Option<&Path>,
    debug_md: Option<&Path>,
) -> Result<ExitCode, CliError> {
    let scenario = load_scenario(scenario_path)?;
    validate_scenario(&scenario)?;
    let report = run_scenario(&scenario)?;
    let debug = build_debug_report(&scenario, &report);

    if let Some(path) = debug_json {
        write_file(path, &serde_json::to_string_pretty(&debug)?)?;
    }
    if let Some(path) = debug_md {
        write_file(path, &debug_markdown(&debug))?;
    }

    println!(
        "debug: {} result={:?} events={} assertions={} failures={}",
        debug.scenario_name,
        debug.result,
        debug.execution_order.len(),
        debug.assertions.len(),
        debug.failure_causes.len()
    );

    Ok(if debug.failure_causes.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn build_debug_report(scenario: &ScenarioFile, report: &RunReport) -> DebugReport {
    let initial_state = scenario
        .devices
        .iter()
        .map(|device| (device.id.clone(), yaml_map_to_json(&device.state)))
        .collect::<BTreeMap<_, _>>();
    let mut targets = initial_state
        .keys()
        .chain(report.final_state.keys())
        .cloned()
        .collect::<Vec<_>>();
    targets.sort();
    targets.dedup();

    let state_diffs = targets
        .into_iter()
        .filter_map(|target| {
            let before = initial_state.get(&target).cloned();
            let after = report.final_state.get(&target).cloned();
            if before == after {
                None
            } else {
                Some(DebugStateDiff {
                    target,
                    before,
                    after,
                })
            }
        })
        .collect::<Vec<_>>();

    let assertions = report
        .assertions
        .iter()
        .map(|assertion| DebugAssertion {
            name: assertion.name.clone(),
            assertion_type: assertion.assertion_type.clone(),
            passed: assertion.passed,
            message: assertion.message.clone(),
            impact_level: assertion.impact_level.clone(),
            impact_message: assertion.impact_message.clone(),
        })
        .collect::<Vec<_>>();

    let failure_causes = assertions
        .iter()
        .filter(|assertion| !assertion.passed)
        .map(|assertion| {
            format!(
                "{} ({}) failed: {}",
                assertion.name, assertion.assertion_type, assertion.message
            )
        })
        .collect::<Vec<_>>();
    let suggested_checks = if failure_causes.is_empty() {
        vec!["No failed assertions. Review state_diffs for expected device changes.".to_string()]
    } else {
        vec![
            "Check the failed assertion target and condition against final_state.".to_string(),
            "Inspect execution_order around the failed target for missing commands, faults, or external observations.".to_string(),
            "Use --report-json with roomci run if you need the complete runtime report.".to_string(),
        ]
    };

    DebugReport {
        schema_version: "roomci.debug.v1",
        scenario_name: report.scenario_name.clone(),
        result: report.result,
        execution_order: report
            .timeline
            .iter()
            .enumerate()
            .map(|(sequence, event)| DebugTimelineEvent {
                sequence,
                at: event.at.clone(),
                event_type: event.event_type.clone(),
                target: event.target.clone(),
                message: event.message.clone(),
            })
            .collect(),
        state_diffs,
        assertions,
        failure_causes,
        suggested_checks,
    }
}

fn debug_markdown(debug: &DebugReport) -> String {
    let mut output = String::new();
    output.push_str(&format!("# roomci Debug: {}\n\n", debug.scenario_name));
    output.push_str(&format!("- Result: `{:?}`\n", debug.result));
    output.push_str(&format!("- Events: `{}`\n", debug.execution_order.len()));
    output.push_str(&format!("- Assertions: `{}`\n\n", debug.assertions.len()));

    output.push_str("## Execution Order\n\n");
    for event in &debug.execution_order {
        let target = event.target.as_deref().unwrap_or("-");
        output.push_str(&format!(
            "{}. `{}` `{}` `{}` - {}\n",
            event.sequence, event.at, event.event_type, target, event.message
        ));
    }

    output.push_str("\n## State Diffs\n\n");
    if debug.state_diffs.is_empty() {
        output.push_str("No state changes from initial device state.\n");
    } else {
        for diff in &debug.state_diffs {
            output.push_str(&format!("- `{}` changed\n", diff.target));
        }
    }

    output.push_str("\n## Assertions\n\n");
    for assertion in &debug.assertions {
        let status = if assertion.passed { "passed" } else { "failed" };
        output.push_str(&format!(
            "- `{}` `{}`: {}\n",
            status, assertion.assertion_type, assertion.message
        ));
    }

    output.push_str("\n## Failure Causes\n\n");
    if debug.failure_causes.is_empty() {
        output.push_str("No failed assertions.\n");
    } else {
        for cause in &debug.failure_causes {
            output.push_str(&format!("- {cause}\n"));
        }
    }

    output.push_str("\n## Suggested Checks\n\n");
    for check in &debug.suggested_checks {
        output.push_str(&format!("- {check}\n"));
    }
    output
}

struct RunOptions {
    scenarios: Vec<PathBuf>,
    junit: Option<PathBuf>,
    markdown: Option<PathBuf>,
    json: Option<PathBuf>,
    timeline_json: Option<PathBuf>,
    timeline_ndjson: Option<PathBuf>,
    observability_json: Option<PathBuf>,
    run_id: Option<String>,
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

        let mut report = run_scenario(&scenario_file)?;
        if let Some(run_id) = &options.run_id {
            report.run_id = run_id.clone();
        }
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
        if let Some(path) = &options.timeline_json {
            write_file(path, &to_timeline_json(report)?)?;
        }
        if let Some(path) = &options.timeline_ndjson {
            write_file(path, &to_timeline_ndjson(report)?)?;
        }
        if let Some(path) = &options.observability_json {
            write_file(path, &to_observability_json(report)?)?;
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
