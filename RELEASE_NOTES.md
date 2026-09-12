Shadow v0.3.0 — exact source dependency closures (experimental)

- Dependency-first Lua source builds with pinned source checksums.
- Exact version/revision/archive-hash dependency records and shared installations.
- Coexisting library revisions; removal blocked while packages reference them.
- Library packages and multiple executable exports.
- Relative ELF runtime paths plus explicit host ABI/display-library auditing.
- Kitty source recipes, private Python, retained sources/licenses, and an Arch builder.
- X11 software-rendered Kitty smoke test; real GPU/Wayland validation remains pending.
- Independent v3 app catalog, without dropping legacy hello catalogs.

Update the CLI using the existing public curl installer. App package sets are
published separately after validation; availability is controlled by the v3 catalog.

This remains a development preview, not a security-maintained distro. Dependencies
are development pins, not a promise of newest versions. Compiler/bootstrap tools,
Linux/glibc/C++ ABI, graphics/display/session interfaces, fonts, and certificates
still come from the host. No full sandboxing, automatic upgrades/GC, independent
signatures, or crash/power-loss recovery. See docs/source-packages.md.
