# Shadow

A native package manager written in Rust, with Lua build recipes and readable
versioned package directories. Early experimental preview, Linux x86_64.

## Install Shadow

```sh
curl -fsSL https://raw.githubusercontent.com/haider-IQZ/shadow/main/install.sh | bash
```

No GitHub login, Git checkout, or compiler needed. Run as your normal user, not
with sudo. Requires curl and the standard Linux coreutils/util-linux tools.
The installer uses `~/.local/bin`; if that isn't already in your PATH, run:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

Add that export to your shell configuration to keep it across sessions.

## Install an app

```sh
shadow install hello
shadow run hello
shadow list
shadow remove hello
```

**Only `hello` is in the initial catalog.** It is a prebuilt, statically linked
native executable, not a recipe compiled on your machine. Other package names
report the available catalog entries. This is the first working slice, not yet
a general-purpose app repository or dependency resolver.

Apps default to `~/.local/share/shadow`. Their executables are exposed in
`~/.local/share/shadow/bin`, which you may add to PATH, or use `shadow run NAME`.
Use `shadow --root /your/dedicated/prefix ...` to override the package root.
Pacman continues to manage the host and Shadow manages only its own prefix.

## Terminal visuals

Interactive terminals show a catalog spinner, download bar with actual byte
counts and speed, verification/install stages, and a colored success line.
Short operations may finish before a progress frame is visible. Redirected
output and `TERM=dumb` use plain stderr messages, keeping `shadow list` stdout
machine-readable. `NO_COLOR` disables success colors.

## Update Shadow

Rerun the same curl command. The installer verifies the release binary's SHA-256
and atomically replaces the CLI, retaining `shadow.previous` beside it. It does
not upgrade installed apps. Override the release with `SHADOW_VERSION=v0.2.0`
or CLI location with `SHADOW_BIN_DIR` (set these on the `bash` process).

Downloads use public GitHub HTTPS. The package catalog pins each download to a
release tag and SHA-256; checksums are not independent publisher signatures.
You can download and inspect `install.sh` before running it instead of piping it.

## Local recipes and packages

```sh
shadow build ./recipes/hello.lua --output hello.shadow
shadow install ./hello.shadow
```

Local builds require their build tools (`cc` for hello). Lua recipes return
`name`, `version`, `revision`, and `build`; unknown fields are rejected.
`build(ctx)` receives `ctx.destdir` and `ctx.run({program, arg, ...})`. Commands
run in a temporary build directory and receive `SHADOW_RECIPE_DIR` in their
environment. Nonzero exits abort the build.

**Local recipes are executable code and are NOT sandboxed.** Only build trusted
recipes and install trusted packages. Builds inherit the host environment;
local artifacts are not necessarily portable or reproducible.

## Source dependencies (development)

The development engine builds pinned Lua dependency graphs, records exact archive
hashes, installs dependencies first, shares matching library revisions, and blocks
removal while referenced. Libraries may coexist at different revisions. ELF
runtime paths reference exact sibling prefixes rather than host app libraries.

Kitty's source recipes and Arch-container build instructions are in
[Source packages](docs/source-packages.md). They are not yet a published,
security-reviewed package set. The manual source-build workflow does not publish
packages automatically.

## Layout and current limitations

```text
~/.local/share/shadow/
  .shadow-root
  .lock
  Cellar/hello/1.0.0-r1/
    manifest.json
    payload/bin/hello
  bin/hello -> ../Cellar/.../payload/bin/hello
  work/
```

The prefix must be dedicated, user-controlled, and not shared with untrusted
users. Mutations are serialized. Packages are validated in staging, moved into
the Cellar, then activated with an executable link. An existing active package
or package revision is never overwritten; remove before reinstalling. Removal
deactivates first and then deletes the version. Do not remove running apps.

Interrupted operations can leave inactive Cellar or work directories. Automatic
recovery and power-loss durability are not implemented. Format v1 supports
regular files/directories only, at most 20,000 entries and 512 MiB of declared
extracted data. Links, traversal, duplicate archive paths, and special permission
bits are rejected. Format v1 exports `payload/bin/<package-name>`; format v2
supports libraries with no exports, multiple executable exports, and exact
checksummed dependency references. Installations are transactional per package,
not across an entire dependency closure. Complete host isolation, app upgrades,
package signatures, build sandboxing, and garbage collection remain future work.

## Development

```sh
nix develop
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo run -- --help
```

`flake.lock` pins the development environment including Rust nightly. To update,
run `nix flake update` and align the nightly date in CI. Lua is embedded through
`mlua` with vendored Lua 5.4; users do not need Lua installed.

CI caches downloads and separate test/musl build artifacts. Main builds warm the
release cache; tags publish the static CLI, prebuilt hello package, and catalog.
Nix development binaries may reference `/nix/store`; release binaries use musl
instead. VM images, test roots, and package build outputs are excluded from Git.
