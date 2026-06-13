//! Golden snapshot of the `RunReport` JSON output contract for every
//! `examples/*.yaml`.
//!
//! `run_scenario` is fully deterministic (virtual time + ordered `BTreeMap`s
//! and `Vec`s, no wall-clock), so any change to the serialized report — schema
//! fields, timeline `event_type`/`message` strings, assertion
//! `name`/`assertion_type`/`impact_level` — is caught here. This is the safety
//! net that lets behavior-preserving refactors prove they changed nothing.
//!
//! Both passing and intentionally-failing scenarios (e.g.
//! `dali_scene_partial_failure`) are pinned: the test only asserts the output
//! does not change, not that scenarios pass.
//!
//! Regenerate after an *intentional* behavior change with:
//! `UPDATE_GOLDEN=1 cargo test -p roomci-core --test golden_reports`

use std::{fs, path::PathBuf};

use roomci_core::run_scenario;
use roomci_scenario::load_scenario;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

#[test]
fn run_reports_match_golden_files() {
    let examples_dir = repo_root().join("examples");
    let golden_dir = golden_dir();
    let update = std::env::var("UPDATE_GOLDEN").is_ok();

    // Top-level `.yaml` only: `examples/adapters/` and `examples/controllers/`
    // are not scenarios, and `read_dir` does not recurse.
    let mut yaml_paths: Vec<_> = fs::read_dir(&examples_dir)
        .expect("examples directory must exist")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "yaml"))
        .collect();
    yaml_paths.sort();
    assert!(!yaml_paths.is_empty(), "no example scenarios found");

    let mut failures = Vec::new();
    for path in yaml_paths {
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let scenario = load_scenario(&path).expect("example must parse");
        let report = run_scenario(&scenario).expect("example must run");
        let actual = serde_json::to_string_pretty(&report).unwrap();
        let golden_path = golden_dir.join(format!("{stem}.json"));

        if update {
            fs::create_dir_all(&golden_dir).unwrap();
            fs::write(&golden_path, format!("{actual}\n")).unwrap();
            continue;
        }

        let expected = fs::read_to_string(&golden_path).unwrap_or_else(|_| {
            panic!(
                "missing golden file {golden_path:?}; regenerate with \
                 `UPDATE_GOLDEN=1 cargo test -p roomci-core --test golden_reports`"
            )
        });
        if actual.trim_end_matches('\n') != expected.trim_end_matches('\n') {
            failures.push(stem);
        }
    }

    assert!(
        failures.is_empty(),
        "RunReport output diverged from golden files {failures:?}; if intentional, \
         regenerate with `UPDATE_GOLDEN=1 cargo test -p roomci-core --test golden_reports`"
    );
}
