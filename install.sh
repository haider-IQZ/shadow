#!/usr/bin/env bash
# curl -fsSL https://raw.githubusercontent.com/haider-IQZ/shadow/main/install.sh | bash
set -euo pipefail
umask 077

repo=haider-IQZ/shadow
version="${SHADOW_VERSION:-}"
bin_dir="${SHADOW_BIN_DIR:-$HOME/.local/bin}"

[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || { echo 'Shadow currently supports Linux x86_64.' >&2; exit 1; }
[[ $EUID != 0 ]] || { echo 'Run without sudo: Shadow installs for your user.' >&2; exit 1; }
for tool in curl sha256sum mktemp flock; do
    command -v "$tool" >/dev/null || { echo "Please install $tool first." >&2; exit 1; }
done
fetch() {
    curl --fail --silent --show-error --location --proto '=https' --proto-redir '=https' \
        --connect-timeout 15 --max-time 300 --retry 2 "$@"
}
if [[ -z $version ]]; then
    latest=$(fetch --output /dev/null --write-out '%{url_effective}' "https://github.com/$repo/releases/latest")
    version=${latest##*/}
fi
[[ $version =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "Invalid release: $version" >&2; exit 1; }

mkdir -p "$bin_dir"
exec 9>"$bin_dir/.shadow-install.lock"
flock -x 9
work=$(mktemp -d "$bin_dir/.shadow-install.XXXXXXXX")
trap 'rm -rf -- "$work"' EXIT
asset=shadow-linux-x86_64
base="https://github.com/$repo/releases/download/$version"
fetch --max-filesize 104857600 --output "$work/$asset" "$base/$asset"
fetch --max-filesize 1048576 --output "$work/SHA256SUMS" "$base/SHA256SUMS"
(
    cd "$work"
    checksum=$(awk -v name="$asset" '$2 == name { print $1 }' SHA256SUMS)
    [[ $checksum =~ ^[0-9a-f]{64}$ ]] || { echo 'Invalid release checksum.' >&2; exit 1; }
    printf '%s  %s\n' "$checksum" "$asset" | sha256sum --check --strict --status -
)
chmod 755 "$work/$asset"
"$work/$asset" --version
if [[ -e $bin_dir/shadow || -L $bin_dir/shadow ]]; then
    [[ -f $bin_dir/shadow && ! -L $bin_dir/shadow ]] || { echo 'Refusing to replace a non-regular executable.' >&2; exit 1; }
    cp -- "$bin_dir/shadow" "$work/shadow.previous"
    mv -fT -- "$work/shadow.previous" "$bin_dir/shadow.previous"
fi
mv -fT -- "$work/$asset" "$bin_dir/shadow"
printf '\nInstalled! Next:\n'
case ":$PATH:" in
    *":$bin_dir:"*) ;;
    *) printf '  export PATH="%s:$PATH"\n' "$bin_dir" ;;
esac
printf '  shadow install hello\n  shadow run hello\n\nRerun this installer to update Shadow.\n'
