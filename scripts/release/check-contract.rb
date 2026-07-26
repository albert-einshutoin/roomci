#!/usr/bin/env ruby
# frozen_string_literal: true

require "yaml"
require "json"
require "open3"

ROOT = File.expand_path("../..", __dir__)
ACTION_PINS = {
  "actions/checkout" => "fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09",
  "actions/upload-artifact" => "b7c566a772e6b6bfb58ed0dc250532a479d7789f",
  "actions/download-artifact" => "37930b1c2abaa49bbe596cd826c3c89aef350131",
  "actions/attest" => "f7c74d28b9d84cb8768d0b8ca14a4bac6ef463e6",
  "dtolnay/rust-toolchain" => "4cda84d5c5c54efe2404f9d843567869ab1699d4",
  "docker/setup-qemu-action" => "c7c53464625b32c7a7e944ae62b3e17d2b600130",
  "docker/setup-buildx-action" => "8d2750c68a42422c14e847fe6c8ac0403b4cbd6f",
  "docker/login-action" => "c94ce9fb468520275223c153574b00df6fe4bcc9",
  "docker/build-push-action" => "10e90e3645eae34f1e60eeb005ba3a3d33f178e8"
}.freeze

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
metadata_output, metadata_error, metadata_status = Open3.capture3(
  "cargo", "metadata", "--locked", "--format-version", "1", "--no-deps", chdir: ROOT
)
assert_contract(metadata_status.success?, "cargo metadata failed: #{metadata_error}")
packages = JSON.parse(metadata_output).fetch("packages")
workspace_versions = packages.map { |package| package.fetch("version") }.uniq
assert_contract(workspace_versions.length == 1,
                "all workspace packages must share one release version, found: #{workspace_versions.join(', ')}")
workspace_version = workspace_versions.fetch(0)
assert_contract(packages.all? { |package| package["publish"] == [] },
                "all workspace crates must remain publish=false while crates.io is deferred")
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
  expected_guard = "github.event_name == 'push' && github.ref_type == 'tag' && startsWith(github.ref_name, 'v')"
  assert_contract(job.fetch("if") == expected_guard,
                  "#{name} must publish only for a pushed version tag")
end
jobs.each do |name, job|
  assert_contract(job["continue-on-error"] != true, "#{name} job must not continue on error")
  next if %w[publish-images publish-release].include?(name)

  assert_contract((job["permissions"] || {}).values.none? { |value| value == "write" },
                  "non-publish job #{name} must not have write permissions")
end
assert_contract(jobs.fetch("publish-release").fetch("needs").include?("publish-images"),
                "GitHub Release must wait for image publication")
release_step = jobs.fetch("publish-release").fetch("steps").find do |step|
  step["run"]&.include?("gh release create")
end
assert_contract(release_step&.dig("env", "GH_REPO") == "${{ github.repository }}",
                "GitHub Release publication must identify the repository without relying on a checkout")
download_step = jobs.fetch("publish-release").fetch("steps").find do |step|
  step["uses"]&.start_with?("actions/download-artifact@")
end
assert_contract(download_step&.dig("with", "pattern") == "roomci-*",
                "GitHub Release must download only native binary artifacts, excluding BuildKit records")

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

  action, sha = step["uses"].split("@", 2)
  assert_contract(ACTION_PINS[action] == sha, "action is not pinned to the reviewed commit: #{step["uses"]}")
end

verify_runs = jobs.fetch("verify").fetch("steps").map { |step| step["run"] }.compact
assert_contract(verify_runs.any? { |run| run.include?("cargo install cargo-audit --version 0.22.2 --locked") },
                "verify must install the pinned cargo-audit version")
assert_contract(verify_runs.any? do |run|
                  run.include?("gh api --include") &&
                    run.include?("release_check_code") &&
                    run.include?("HTTP/[0-9.]+ 404") &&
                    run.include?("could not prove")
                end, "verify must distinguish an absent Release from API or network failure")
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
  assert_contract(!config["tags"].include?(":latest"), "initial releases must not mutate a latest tag")
end
image_publish_runs = jobs.fetch("publish-images").fetch("steps").map { |step| step["run"] }.compact
assert_contract(image_publish_runs.any? do |run|
                  run.include?("docker buildx imagetools inspect") &&
                    run.include?("image_check_code") &&
                    run.include?("manifest unknown") &&
                    run.include?("could not prove")
                end, "image publication must distinguish absent tags from registry or network failure")
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
  assert_contract(dockerfile.include?("cargo build --locked --release -p roomci-cli"),
                  "#{path} must enforce Cargo.lock during the release build")
  assert_contract(dockerfile.include?("ARG ROOMCI_VERSION=#{workspace_version}"),
                  "#{path} must default OCI labels to the workspace release version")
end

dry_run = jobs.fetch("dry-run-images").fetch("steps").select { |step| step["uses"]&.start_with?("docker/build-push-action@") }
assert_contract(dry_run.length == 2 && dry_run.all? { |step| step.fetch("with")["push"] == false },
                "non-public dry run must build both images with push:false")

%w[README.md README.ja.md docs/17_docker_ci_design.md docs/17_docker_ci_design.ja.md].each do |path|
  text = File.read(File.join(ROOT, path))
  assert_contract(text.include?("ghcr.io/albert-einshutoin/roomci"),
                  "#{path} must use the real GHCR coordinate")
  assert_contract(!text.include?("ghcr.io/albert-einshutoin/roomci:latest"),
                  "#{path} must not recommend a mutable GHCR tag")
end
%w[README.md README.ja.md examples/github-actions/roomci-poc.yml].each do |path|
  text = File.read(File.join(ROOT, path))
  assert_contract(text.include?("albert-einshutoin/roomci@v#{workspace_version}") &&
                  !text.include?("albert-einshutoin/roomci@main"),
                  "#{path} must pin the supported immutable Action release")
end
init_workflow = File.read(File.join(ROOT, "crates/roomci-cli/templates/github-actions-roomci.yml"))
assert_contract(init_workflow.include?("albert-einshutoin/roomci@v#{workspace_version}"),
                "init workflow must pin the workspace Action release")
assert_contract(init_workflow.include?("persist-credentials: false"),
                "init workflow checkout must not persist repository credentials")
init_settings = File.read(File.join(ROOT, "crates/roomci-cli/templates/settings.json"))
assert_contract(init_settings.include?("/roomci/v#{workspace_version}/schemas/scenario.schema.json"),
                "init schema URL must pin the workspace release")
%w[docs/RELEASING.md docs/RELEASING.ja.md].each do |path|
  text = File.read(File.join(ROOT, path))
  assert_contract(text.include?("gh attestation verify") && text.include?("SHA256SUMS"),
                  "#{path} must document checksum and attestation verification")
  assert_contract(text.include?("Windows") && text.include?("crates.io"),
                  "#{path} must list deferred channels")
  assert_contract(text.include?("VERSION=#{workspace_version}"),
                  "#{path} must document the current workspace release")
end

puts "Release distribution contract is valid."
