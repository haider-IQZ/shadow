#!/usr/bin/env bash
# Build as your UID in a container; source checkout is read-only, output stays local.
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
output=${SHADOW_SOURCE_WORK:-"$repo/.shadow-source"}
mkdir -p "$output"
output=$(cd -- "$output" && pwd)
docker build -f "$repo/scripts/source-builder.Dockerfile" -t shadow-source-toolchain "$repo/scripts"
docker run --rm --init --user "$(id -u):$(id -g)" \
    --mount "type=bind,src=$repo,dst=/workspace,readonly" \
    --mount "type=bind,src=$output,dst=/work" \
    -e CARGO_TARGET_DIR=/work/target \
    -e SHADOW_HOST_PKGCONFIG=/usr/lib/pkgconfig:/usr/share/pkgconfig \
    shadow-source-toolchain bash -eu -c '
        cargo build --locked
        /work/target/debug/shadow build-closure recipes/source/kitty.lua \
          --output-dir /work/artifacts --build-root /work/build "$@"
    ' shadow-build "$@"
printf 'Artifacts: %s/artifacts\n' "$output"
