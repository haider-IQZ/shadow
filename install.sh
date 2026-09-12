#!/usr/bin/env bash
# Installs/updates the Shadow CLI only. Packages use a separate --root.
set -euo pipefail
umask 077

repo="${SHADOW_REPO:-haider-IQZ/shadow}"
version="${SHADOW_VERSION:-}"
bin_dir="${SHADOW_BIN_DIR:-$HOME/.local/bin}"

if [[ $(uname -s) != Linux || $(uname -m) != x86_64 ]]; then
    echo 'This preview release supports Linux x86_64 only.' >&2
    exit 1
fi
if [[ $EUID == 0 ]]; then
    echo 'Run this installer as your normal user, not with sudo.' >&2
    exit 1
fi
for tool in gh sha256sum mktemp flock; do
    command -v "$tool" >/dev/null || { echo "Missing dependency: $tool" >&2; exit 1; }
done
if [[ -z $version ]]; then
    version=$(gh api "repos/$repo/releases/latest" --jq .tag_name)
fi
[[ $version =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "Invalid release tag: $version" >&2; exit 1; }

mkdir -p "$bin_dir"
exec 9>"$bin_dir/.shadow-install.lock"
flock -x 9
work=$(mktemp -d "$bin_dir/.shadow-install.XXXXXXXX")
trap 'rm -rf -- "$work"' EXIT
asset=shadow-linux-x86_64
gh release download "$version" --repo "$repo" --pattern "$asset" --pattern SHA256SUMS --dir "$work"
(
    cd "$work"
    # Check only this asset, with a strict checksum-line format.
    checksum=$(awk -v name="$asset" '$2 == name { print $1 }' SHA256SUMS)
    [[ $checksum =~ ^[0-9a-f]{64}$ ]] || { echo 'Missing or invalid checksum' >&2; exit 1; }
    printf '%s  %s\n' "$checksum" "$asset" | sha256sum --check --strict -
)
chmod 755 "$work/$asset"
"$work/$asset" --version
if [[ -e $bin_dir/shadow || -L $bin_dir/shadow ]]; then
    [[ -f $bin_dir/shadow && ! -L $bin_dir/shadow ]] || { echo 'Refusing to replace a non-regular executable' >&2; exit 1; }
    cp -- "$bin_dir/shadow" "$work/shadow.previous"
    mv -fT -- "$work/shadow.previous" "$bin_dir/shadow.previous"
fi
# Same-filesystem rename: readers see either the old CLI or the new CLI.
mv -fT -- "$work/$asset" "$bin_dir/shadow"
printf 'Installed Shadow %s to %s/shadow\n' "$version" "$bin_dir"
printf 'Ensure %s is in PATH. Rerun this script to update.\n' "$bin_dir"
