Shadow v0.2.0 — direct installation and named packages

Install with curl, then `shadow install hello`. No GitHub authentication, Git
checkout, Rust compiler, or Lua installation required.

- Public HTTPS CLI installer with checksum verification and previous-CLI backup.
- Named package downloads from a release-pinned catalog, verified with SHA-256.
- Prebuilt static native hello package; local Lua recipe builds still supported.
- Persistent default package root at ~/.local/share/shadow.
- Cached CI builds and release package smoke test.

Linux x86_64 only. The catalog currently contains only hello. This remains an
experimental package-lifecycle preview: no dependency resolver, app upgrades,
independent package signatures, build sandbox, or power-loss recovery yet.
Local recipes execute with user privileges. Test in a disposable VM, without sudo.
