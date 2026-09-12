# Source-built packages and exact dependency closures

## Model

A source recipe pins a name/version/revision, HTTPS source digest, optional
checksummed data resources, executable exports, and dependency recipe paths.
`shadow plan recipes/source/kitty.lua` prints a deterministic dependency-first
order. Cycles and two recipes providing the same name inside one application
closure are errors. Different applications may use different library versions.

`shadow build-closure` builds every dependency from source and emits separate
`name-version-rN.shadow` artifacts. Format 2 manifests record exact identities
**and archive SHA-256 hashes**, not floating version ranges. All transitive
runtime dependencies are recorded; headers such as SIMDe are currently retained
in that closure too. Build-only dependencies are not yet modeled separately.

The builder supplies each recipe's exact dependency prefixes to compiler flags,
pkg-config, CMake, and build-command PATH. It rewrites dynamic ELF RPATH entries
to `$ORIGIN` paths referencing exact sibling revisions, and audits DT_NEEDED
against the closure and the explicit host ABI/display bridge. It rejects Nix
runtime interpreters in these target artifacts. This does not detect every
possible library loaded through computed `dlopen` paths or embedded data paths;
application-specific runtime validation is still necessary.

## Build on NixOS using the Arch container

```sh
bash scripts/build-source.sh
# After a failure, reuse completed immutable revisions:
bash scripts/build-source.sh --resume
```

Requires Docker. The checkout is mounted read-only. Compilation runs as your UID,
with writable state only in `.shadow-source` (override with `SHADOW_SOURCE_WORK`).
The initial toolchain image uses Arch's compiler, Rust, Go, Python/Meson, pkg-config,
CMake, Ninja, gperf, tar, and patchelf. These are bootstrap build tools, not packages
that Shadow claims to have built. The image digest is pinned; pacman's build-tool
updates are not snapshot-pinned, so this is **not** a reproducible toolchain yet.

The app's Python, compression, crypto, text shaping, font/rendering, and color
management libraries are built by Shadow. The host provides the Linux/glibc/C++
ABI, GPU drivers and GL/EGL, display/input libraries, session services, font
configuration/fonts, certificates, and basic shell utilities. Kitty's launcher
sets its private Python home inside the interpreter configuration, relative to its
own prefix, without exporting it into terminal children. Host font configuration
remains intentional.

The source recipes are development pins, not an assurance that every version is
the newest or security-reviewed. Do not distribute the artifacts as a stable or
security-maintained distro. A source build is not evidence that all runtime paths
or desktop behaviors work; graphical runtime testing is required before release.

## Build commands

```sh
shadow plan recipes/source/kitty.lua
shadow build-closure recipes/source/kitty.lua \
  --output-dir /work/artifacts --build-root /work/build
```

A source callback receives:

- `ctx.prefix`: final prefix in the build environment.
- `ctx.destdir`: DESTDIR staging root for configure/make-style installation.
- `ctx.output`: staging path corresponding to `ctx.prefix`.
- `ctx.deps.NAME`: exact absolute dependency prefix during building.
- `ctx.relative_deps.NAME`: dependency prefix relative to this package's prefix,
  for generating relocatable launchers.
- `ctx.source_archives`: retained source inputs for redistribution; Kitty also
  archives its vendored Go dependencies here.
- `ctx.jobs`: bounded parallelism.
- `ctx.run({...})`: execute arguments in the extracted source directory.

`--resume` explicitly trusts completed local immutable revisions whose archive
hashes and dependency records match. When provenance exists it also checks the
recipe digest. Bump a revision after changing a recipe; use a fresh build root
when changing the toolchain or build engine. This is not an automatic remote
build cache. Failed compile/audit work is retained with diagnostic paths.

Recipes are executable trusted code. Shell commands are not sandboxed by the Rust
engine. The container limits host filesystem exposure, but is not a hardened
sandbox for hostile recipes; source builds retain network access (Go modules are
verified by upstream go.sum). Pinned source downloads, upstream license files,
and extra resource digests are recorded in package provenance. A public binary
release must include corresponding sources and any required license notices.

## Local installation and removal

Place an application archive beside its exact dependency archives:

```sh
shadow install ./.shadow-source/artifacts/kitty-0.48.2-r3.shadow
shadow run kitty
shadow run kitten -- --version
shadow list
shadow remove kitty
```

Shadow recursively installs missing dependencies and shares matching installed
revisions. An occupied revision with another archive digest is rejected. Libraries
can coexist at multiple revisions; executable names cannot have two active
owners. Removal checks installed references first. Use `name@version-rN` to select
an unreferenced library when several versions exist. App removal intentionally
retains dependencies; garbage collection and upgrades are separate future work.

Each package is a separate installation transaction. A failing app install may
leave successfully installed dependencies. Crash/power-loss recovery and global
closure rollback are not implemented. Do not remove packages from running apps.

## Repository publication

The v3 catalog at `repository/catalog-v3.json` is independent of CLI releases.
It maps app aliases to immutable artifacts and lists exact dependency hashes.
Publish all referenced release assets **before** updating that file. A CLI update
must not reset the app catalog. Catalog validation rejects missing or mismatched
dependency artifacts. Legacy v1/v2 release catalogs remain for older clients.

The manual GitHub `Source package build` workflow builds source artifacts for
review; it does not automatically publish untested Kitty packages. It caches
completed revisions and Go inputs separately from normal CLI CI.

For a graphical X11 software-rendering check in a disposable Arch runtime with
Xvfb, xdotool, Mesa, and fonts installed, run `bash scripts/smoke-kitty.sh`. It
opens a real window, runs a terminal child, checks environment isolation, and
checks process mappings for Shadow-owned core libraries. This is not a Wayland,
NVIDIA, or real-GPU validation.
