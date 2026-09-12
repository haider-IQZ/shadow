# Shadow

Experimental native package manager written in Rust, with Lua build recipes.
The long-term goal is readable versioned packages with explicit dependency
versions, alongside the host package manager. This first release implements the
local package lifecycle, not dependency isolation or a complete distro.

## Install/update the CLI in an Arch VM

The repository is initially private. Authenticate GitHub CLI with an account
that has access. No Git checkout or Rust compiler is needed to install releases.

```sh
sudo pacman -S --needed github-cli base-devel
 gh auth login
mkdir -p "$HOME/shadow-preview"
cd "$HOME/shadow-preview"
gh release download --repo haider-IQZ/shadow --pattern install.sh --clobber
bash install.sh
export PATH="$HOME/.local/bin:$PATH"
shadow --version
```

Repeat the download and `bash install.sh` to update. The installer resolves one
release tag, downloads through authenticated GitHub HTTPS, verifies SHA-256, and
atomically replaces `~/.local/bin/shadow`. The previous CLI is retained as
`~/.local/bin/shadow.previous`. Checksums are integrity checks, not independent
publisher signatures. Do not run the installer with sudo.

Overrides: `SHADOW_VERSION=v0.1.0`, `SHADOW_REPO=owner/repo`, and
`SHADOW_BIN_DIR=/your/user-owned/bin`.

## First package

Download the example from the same release (initially `v0.1.0`):

```sh
gh release download v0.1.0 --repo haider-IQZ/shadow \
  --pattern hello.lua --pattern hello.c --clobber
shadow build hello.lua --output hello.shadow
shadow --root "$HOME/.local/share/shadow" install hello.shadow
shadow --root "$HOME/.local/share/shadow" list
shadow --root "$HOME/.local/share/shadow" run hello
"$HOME/.local/share/shadow/bin/hello"
shadow --root "$HOME/.local/share/shadow" remove hello
```

By default, the root is `.shadow-dev` relative to the current directory. Use an
explicit absolute `--root` for persistent VM testing. Build outputs are never
overwritten; delete the old archive or choose another output filename.

## Current layout and guarantees

```text
<root>/
  .shadow-root                       root format marker
  .lock                              exclusive operation lock
  Cellar/hello/1.0.0-r1/
    manifest.json                    installed package metadata
    payload/bin/hello                executable
  bin/hello                          activation symlink
  work/                              temporary extraction directories
```

A root must be dedicated to Shadow and owned/controlled by the current user.
Never share it with untrusted users. Pacman and Nix paths are not managed by
Shadow. The manifest in each installed version is the installed-package record;
active executable links determine the active package set.

Installation validates an archive in staging, moves it into the Cellar, then
creates one activation link. Existing versions are never overwritten. Removal
first deactivates the executable, then deletes that package version. Interrupted
operations can leave inactive Cellar directories or work directories; automatic
recovery and power-loss durability are **not** implemented. A failed activation
reports the retained files. Do not delete packages while their apps are running.

Format v1 allows regular files/directories only, at most 20,000 entries and
512 MiB of declared extracted data. It rejects archive links, traversal,
duplicate paths, and special permission bits. Each package must supply an
executable `payload/bin/<package-name>`. Additional executable exports, upgrades,
dependency resolution, signatures, and garbage collection are future work.

## Lua recipes

Recipes return a table with `name`, `version`, `revision`, and `build` fields;
unknown fields are rejected rather than silently ignored. `build(ctx)` receives:

- `ctx.destdir`: payload staging path.
- `ctx.run({program, arg, ...})`: execute an argument vector in a temporary build
  directory; nonzero exits abort the build.
- Child processes receive `SHADOW_RECIPE_DIR`, the directory containing the recipe.

**Recipes and build commands execute with your user privileges and are not
sandboxed.** Only run trusted recipes. The package artifact is validated before
publication. Builds inherit the host environment and are not reproducible or
portable by default. The hello recipe compiles against the build host's libc.

## Develop on NixOS

```sh
nix develop
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo run -- --help
```

The flake provides Rust nightly, rustfmt, Clippy, rust-analyzer, Rust sources,
Lua 5.4, and native compilation/archive tools. Rust embeds Lua using `mlua` with
vendored Lua, so the release CLI does not require a Lua installation.
`flake.lock` pins nixpkgs and rust-overlay. Deliberately refresh with
`nix flake update`; update the CI nightly date alongside toolchain changes.

NixOS-built development binaries can depend on `/nix/store`. GitHub releases
instead build a static musl x86_64 Linux CLI, and reject an unexpected dynamic
interpreter. CI runs formatting, Clippy, and two focused integration tests before
publishing tagged releases. VM images and local build outputs are excluded from Git.
