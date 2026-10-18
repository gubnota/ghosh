#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
release_tag="${1:?Usage: build-release.sh vX.Y.Z}"
node scripts/verify-release.mjs "$release_tag"
export CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1
case "$(uname -s)/$(uname -m)" in
  Darwin/arm64)
    release_target=aarch64-apple-darwin
    release_platform=macos-arm64
    export MACOSX_DEPLOYMENT_TARGET=14.0
    release_bundle=app
    ;;
  Linux/x86_64)
    release_target=x86_64-unknown-linux-gnu
    release_platform=linux-x86_64
    release_bundle=deb
    ;;
  *) echo "Build natively on macOS arm64 or Linux x86_64." >&2; exit 1 ;;
esac
if command -v rustup >/dev/null 2>&1; then
  rustup target add "$release_target"
fi
npm run tauri build -- --target "$release_target" --bundles "$release_bundle"
release_output="artifacts/${release_tag}-${release_platform}"
mkdir -p "$release_output"
release_stage="$(mktemp -d)"
trap 'rm -rf "$release_stage"' EXIT
cp LICENSE-MIT LICENSE-APACHE THIRD_PARTY_NOTICES.md README.md "$release_stage/"
if [[ "$release_platform" == macos-arm64 ]]; then
  cp -R "target/$release_target/release/bundle/macos/Ghosh.app" "$release_stage/"
else
  cp "target/$release_target/release/ghosh-desktop" "$release_stage/ghosh"
  cp target/"$release_target"/release/bundle/deb/*.deb "$release_output/"
fi
tar -czf "$release_output/ghosh-${release_tag}-${release_platform}.tar.gz" -C "$release_stage" .
(cd "$release_output"
  if [[ "$release_platform" == linux-x86_64 ]]; then
    sha256sum *.tar.gz *.deb > SHA256SUMS
  else
    shasum -a 256 *.tar.gz > SHA256SUMS
  fi
)
echo "Artifacts: $release_output"
