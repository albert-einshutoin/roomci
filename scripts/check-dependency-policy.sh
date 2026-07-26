#!/bin/sh
set -eu

require() {
  file=$1
  pattern=$2
  if ! grep -Fiq -- "$pattern" "$file"; then
    echo "error: $file is missing required dependency-policy text: $pattern" >&2
    exit 1
  fi
}

ruby <<'RUBY'
require "yaml"

dependabot = YAML.safe_load(File.read(".github/dependabot.yml"), aliases: false)
abort "error: Dependabot config version must be 2" unless dependabot["version"] == 2
updates = dependabot.fetch("updates")
%w[cargo github-actions].each do |ecosystem|
  entry = updates.find { |candidate| candidate["package-ecosystem"] == ecosystem }
  abort "error: missing Dependabot entry for #{ecosystem}" unless entry
  abort "error: #{ecosystem} Dependabot must scan the repository root" unless entry["directory"] == "/"
  abort "error: #{ecosystem} Dependabot must target main" unless entry["target-branch"] == "main"
  abort "error: #{ecosystem} Dependabot must run weekly" unless entry.dig("schedule", "interval") == "weekly"
  valid_group = entry.fetch("groups").values.any? do |group|
    group["patterns"] == ["*"] && group["update-types"]&.sort == %w[minor patch]
  end
  abort "error: #{ecosystem} Dependabot must group all minor and patch updates" unless valid_group
end

workflow = YAML.safe_load(File.read(".github/workflows/smart-home-ci.yml"), aliases: true)
abort "error: workflow permissions must be contents: read only" unless workflow["permissions"] == { "contents" => "read" }
quality_job = workflow.fetch("jobs").fetch("quality-gates")
abort "error: quality-gates job cannot be conditionally disabled" if quality_job.key?("if")
abort "error: quality-gates job cannot continue on error" if quality_job.key?("continue-on-error")
steps = quality_job.fetch("steps")
required_runs = [
  "cargo install cargo-audit --locked --version 0.22.2",
  "cargo-audit audit --deny warnings",
]
required_runs.each do |command|
  step = steps.find { |candidate| candidate["run"] == command }
  abort "error: missing exact enabled workflow command: #{command}" unless step
  abort "error: dependency gate cannot continue on error" if step["continue-on-error"]
  abort "error: dependency gate cannot be conditionally disabled" if step.key?("if")
end
RUBY

test ! -e .cargo/audit.toml
require docs/DEPENDENCY_POLICY.md 'RUSTSEC-2026-0190'
require docs/DEPENDENCY_POLICY.md 'anyhow 1.0.104'
require docs/DEPENDENCY_POLICY.md 'There is no active RustSec exception'
require docs/DEPENDENCY_POLICY.md 'serde_yaml'
require docs/DEPENDENCY_POLICY.md 'golden'
require docs/DEPENDENCY_POLICY.md 'property'
require docs/DEPENDENCY_POLICY.md 'accept/reject parity'
require docs/DEPENDENCY_POLICY.md 'tools/vscode-roomci/package.json'
require docs/DEPENDENCY_POLICY.md 'Cargo aliases cannot shadow'
require docs/RELEASE_CHECKLIST.md 'cargo-audit audit --deny warnings'
require docs/RELEASE_CHECKLIST.ja.md 'cargo-audit audit --deny warnings'
require README.md 'docs/DEPENDENCY_POLICY.md'
require README.ja.md 'docs/DEPENDENCY_POLICY.md'
require CHANGELOG.md 'dependency security policy'

echo 'Dependency policy contract is valid.'
