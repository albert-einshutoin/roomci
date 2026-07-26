# Releasing roomci

The first distribution surface is GitHub Releases and GHCR. A pull request or
manual dispatch performs a non-public dry run: it verifies the workspace and
uploads private workflow artifacts, but cannot create a release or push an
image. Only a `vMAJOR.MINOR.PATCH` tag may publish.

## Release contract

1. Set the workspace version in `Cargo.toml`, then create the matching tag
   `v<version>`. The release workflow rejects a mismatch before packaging.
2. Run `make release-verify` locally. It validates the workflow contract and
   runs `cargo-audit audit --deny warnings` without Cargo alias resolution.
3. Push the exact version tag. The verified matrix creates tarballs for
   `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
   `x86_64-apple-darwin`, and `aarch64-apple-darwin`.
4. The tag-only jobs generate `SHA256SUMS`, attest the release artifacts, make
   the GitHub Release, and publish multi-architecture images.

Published runtime image:

```text
ghcr.io/albert-einshutoin/roomci:<version>
```

The companion action image is published as
`ghcr.io/albert-einshutoin/roomci-action:<version>`, but `action.yml` continues
to reference the repository-local Dockerfile until the first public release is
verified. This prevents a new action consumer from receiving an unverified
registry image through a mutable tag.

The initial release deliberately publishes no mutable `latest`, major, or
minor tags. If a tag run fails after one image is pushed, remove only the
partial GHCR version after reviewing the workflow evidence, then rerun the
unchanged tag. Never move the Git tag or overwrite an existing version image.

## Install and verify a release

Choose the archive target for the current machine, then verify the checksum and
GitHub attestation before placing the binary on `PATH`:

```bash
VERSION=0.1.1
TARGET=x86_64-unknown-linux-gnu
ARCHIVE="roomci-v${VERSION}-${TARGET}.tar.gz"
BASE="https://github.com/albert-einshutoin/roomci/releases/download/v${VERSION}"
curl -fLO "$BASE/$ARCHIVE"
curl -fLO "$BASE/SHA256SUMS"
grep -F "  $ARCHIVE" SHA256SUMS > "$ARCHIVE.sha256"
test "$(wc -l < "$ARCHIVE.sha256")" -eq 1
shasum -a 256 -c "$ARCHIVE.sha256"
gh attestation verify "$ARCHIVE" --repo albert-einshutoin/roomci
tar -xzf "$ARCHIVE"
mkdir -p "$HOME/.local/bin"
install -m 0755 "roomci-v${VERSION}-${TARGET}/roomci" "$HOME/.local/bin/roomci"
roomci --version
roomci validate "roomci-v${VERSION}-${TARGET}/examples/local_first_cloud_outage.yaml"
```

For the container distribution, pin a version, inspect the published manifest
for `linux/amd64` and `linux/arm64`, and pull the resulting digest:

```bash
VERSION=0.1.1
IMAGE="ghcr.io/albert-einshutoin/roomci:${VERSION}"
docker buildx imagetools inspect "$IMAGE"
docker pull "$IMAGE"
docker run --rm "$IMAGE" --version
```

Upgrade by changing only `VERSION`, then repeat checksum, attestation,
`--version`, and scenario validation before replacing the installed binary or
deployment image. Do not upgrade production automation from `:latest`.

## Deferred channels

musl Linux, Windows, crates.io, and Homebrew are intentionally deferred. They
need their own compatibility, signing, and support policy rather than being
implied by the four initial native artifacts.
