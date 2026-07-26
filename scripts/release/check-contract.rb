#!/usr/bin/env ruby
# frozen_string_literal: true

require "yaml"

ROOT = File.expand_path("../..", __dir__)

def assert_contract(condition, message)
  return if condition

  warn "error: #{message}"
  exit 1
end

workflow = YAML.load_file(ENV.fetch("RELEASE_WORKFLOW", File.join(ROOT, ".github/workflows/release.yml")))
jobs = workflow.fetch("jobs")
assert_contract(workflow.fetch("permissions") == { "contents" => "read" }, "top-level permissions must be contents: read only")
assert_contract(workflow.fetch("concurrency") == {
                  "group" => "release-${{ github.ref }}",
                  "cancel-in-progress" => false
                }, "release workflow must serialize each ref without cancelling an in-progress release")
%w[verify build-binaries dry-run-images publish-images publish-release].each do |name|
  assert_contract(jobs.key?(name), "missing #{name} job")
end

expected_matrix = {
  "ubuntu-24.04" => "x86_64-unknown-linux-gnu",
  "ubuntu-24.04-arm" => "aarch64-unknown-linux-gnu",
  "macos-15-intel" => "x86_64-apple-darwin",
  "macos-15" => "aarch64-apple-darwin"
}
matrix = jobs.fetch("build-binaries").dig("strategy", "matrix", "include")
actual_matrix = matrix.to_h { |entry| [entry["runner"], entry["target"]] }
assert_contract(actual_matrix == expected_matrix, "binary matrix must contain exactly four native targets")

%w[publish-images publish-release].each do |name|
  job = jobs.fetch(name)
  assert_contract(job.fetch("needs").include?("verify"), "#{name} must need verify")
  assert_contract(job["environment"] == "release", "#{name} must use the release environment")
  guard = job.fetch("if")
  assert_contract(guard.include?("github.ref_type == 'tag'") && guard.include?("startsWith(github.ref_name, 'v')"),
                  "#{name} must have a version-tag guard")
end
jobs.each do |name, job|
  assert_contract(job["continue-on-error"] != true, "#{name} job must not continue on error")
  next if %w[publish-images publish-release].include?(name)

  assert_contract((job["permissions"] || {}).values.none? { |value| value == "write" },
                  "non-publish job #{name} must not have write permissions")
end
assert_contract(jobs.fetch("publish-release").fetch("needs").include?("publish-images"),
                "GitHub Release must wait for image publication")

{
  "publish-images" => %w[packages attestations artifact-metadata id-token],
  "publish-release" => %w[contents attestations artifact-metadata id-token]
}.each do |job_name, permissions|
  granted = jobs.fetch(job_name).fetch("permissions")
  permissions.each { |name| assert_contract(granted[name] == "write", "#{job_name} needs #{name}: write") }
end

all_steps = jobs.values.flat_map { |job| job.fetch("steps", []) }
all_steps.each do |step|
  assert_contract(step["continue-on-error"] != true, "release steps must not continue on error")
  next unless step["uses"]

  assert_contract(step["uses"].match?(%r{@[0-9a-f]{40}$}), "action is not SHA pinned: #{step["uses"]}")
end

verify_runs = jobs.fetch("verify").fetch("steps").map { |step| step["run"] }.compact
assert_contract(verify_runs.any? { |run| run.include?("cargo install cargo-audit --version 0.22.2 --locked") },
                "verify must install the pinned cargo-audit version")
makefile = File.read(File.join(ROOT, "Makefile"))
assert_contract(makefile.include?("cargo-audit audit --deny warnings"),
                "release verification must call cargo-audit directly to avoid Cargo alias shadowing")
triggers = workflow[true] || workflow["on"]
assert_contract(triggers.key?("workflow_dispatch"), "workflow must support non-public manual dry runs")

published = jobs.fetch("publish-images").fetch("steps").select { |step| step["uses"]&.start_with?("docker/build-push-action@") }
assert_contract(published.length == 2, "runtime and action images must both publish")
published.each do |step|
  config = step.fetch("with")
  assert_contract(config["platforms"] == "linux/amd64,linux/arm64", "published image must be multiarch")
  assert_contract(config["provenance"] == "mode=max" && config["sbom"] == true, "published image needs provenance and SBOM")
  assert_contract(config.fetch("build-args").include?("ROOMCI_VERSION=${{ steps.version.outputs.value }}") &&
                  config.fetch("build-args").include?("ROOMCI_REVISION=${{ github.sha }}"),
                  "published image must carry release version and revision labels")
  expected_image = config["file"] == "Dockerfile" ? "ghcr.io/albert-einshutoin/roomci:" : "ghcr.io/albert-einshutoin/roomci-action:"
  assert_contract(config["tags"].include?(expected_image), "#{config["file"]} must publish its matching GHCR coordinate")
end
image_attestations = jobs.fetch("publish-images").fetch("steps").select do |step|
  step["uses"]&.start_with?("actions/attest@")
end
assert_contract(image_attestations.map { |step| step.dig("with", "subject-name") }.sort == [
                  "ghcr.io/albert-einshutoin/roomci",
                  "ghcr.io/albert-einshutoin/roomci-action"
                ], "both GHCR images must receive GitHub attestations")
assert_contract(image_attestations.all? { |step| step.dig("with", "push-to-registry") == true },
                "GHCR attestations must be pushed to the registry")

%w[Dockerfile Dockerfile.action].each do |path|
  dockerfile = File.read(File.join(ROOT, path))
  assert_contract(dockerfile.scan(/FROM .+@sha256:[0-9a-f]{64}/).length == 2,
                  "#{path} must pin both base stages by digest")
  %w[org.opencontainers.image.source org.opencontainers.image.version
     org.opencontainers.image.revision org.opencontainers.image.licenses].each do |label|
    assert_contract(dockerfile.include?(label), "#{path} is missing OCI label #{label}")
  end
end

dry_run = jobs.fetch("dry-run-images").fetch("steps").select { |step| step["uses"]&.start_with?("docker/build-push-action@") }
assert_contract(dry_run.length == 2 && dry_run.all? { |step| step.fetch("with")["push"] == false },
                "non-public dry run must build both images with push:false")

%w[README.md README.ja.md docs/17_docker_ci_design.md docs/17_docker_ci_design.ja.md].each do |path|
  assert_contract(File.read(File.join(ROOT, path)).include?("ghcr.io/albert-einshutoin/roomci"),
                  "#{path} must use the real GHCR coordinate")
end
%w[docs/RELEASING.md docs/RELEASING.ja.md].each do |path|
  text = File.read(File.join(ROOT, path))
  assert_contract(text.include?("gh attestation verify") && text.include?("SHA256SUMS"),
                  "#{path} must document checksum and attestation verification")
  assert_contract(text.include?("Windows") && text.include?("crates.io"),
                  "#{path} must list deferred channels")
end

puts "Release distribution contract is valid."
