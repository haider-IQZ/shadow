First experimental Shadow package-manager preview.

- Rust CLI with embedded Lua 5.4 and trusted local build recipes.
- `.shadow` archives (zstd-compressed tar with a JSON manifest).
- Dedicated user-owned prefix with readable `Cellar/name/version-rN` paths.
- Build, install, list, run, and remove commands; serialized package mutations.
- User-local CLI installer/updater with SHA-256 verification and previous-binary backup.

This is a package-lifecycle prototype, not a complete distro package manager.
No dependency resolution, download recipes, package signatures, build sandbox,
upgrade command, or power-loss recovery yet. Packages have one entry-point
executable named after the package. Archive symlinks and special files are rejected.
Do not run as root or use a shared/writable-by-others prefix. Use only trusted
recipes and packages. Test in a disposable VM.

The Linux x86_64 CLI is statically linked with musl and embeds Lua. Recipe build
tools (such as `cc`) must be installed separately on the VM. The hello example
compiles a native C program using that VM's compiler and system libraries.
