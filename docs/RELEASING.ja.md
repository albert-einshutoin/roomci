# roomci のリリース

最初の配布面は GitHub Releases と GHCR です。PR と手動実行は非公開の
dry run として検証・workflow artifact のみを行い、Release 作成や image
push はできません。公開できるのは `vMAJOR.MINOR.PATCH` タグだけです。

## リリース契約

1. `Cargo.toml` の workspace version と一致する `v<version>` タグを作成します。
   一致しないタグはパッケージング前に拒否されます。
2. ローカルで `make release-verify` を実行し、Cargo alias を経由しない
   `cargo-audit audit --deny warnings` を含む gate を通します。
3. 検証済み matrix は Linux x86_64/ARM64 と macOS Intel/Apple Silicon の
   `tar.gz`、`SHA256SUMS`、GitHub artifact attestation を作成します。
4. タグ専用 job が GitHub Release と multi-architecture GHCR image を公開します。

Runtime image は `ghcr.io/albert-einshutoin/roomci:<version>`、action image は
`ghcr.io/albert-einshutoin/roomci-action:<version>` です。ただし初回公開の
検証が完了するまで、`action.yml` は repository-local Dockerfile を参照します。
初回は可変な `latest`、major、minor tag を公開しません。片方の image 公開後に
失敗した場合は workflow evidence を確認し、不完全なGHCR versionだけを削除して
同じGit tagを再実行します。Git tagの移動や既存version imageの上書きは禁止です。

## install と検証

実行環境に合う target を選び、PATH へ置く前に checksum と GitHub
attestation を確認します。

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

container は version を固定して manifest（`linux/amd64` と `linux/arm64`）を
確認してから pull します。

```bash
VERSION=0.1.1
IMAGE="ghcr.io/albert-einshutoin/roomci:${VERSION}"
docker buildx imagetools inspect "$IMAGE"
docker pull "$IMAGE"
docker run --rm "$IMAGE" --version
```

upgrade は `VERSION` だけを変更し、checksum、attestation、`--version`、scenario
validate を再実行します。production automation で `:latest` は使用しません。

musl Linux、Windows、crates.io、Homebrew は互換性・署名・support policy を
個別に定義するまで明示的に延期します。
