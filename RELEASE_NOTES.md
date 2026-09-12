Shadow v0.2.1 — terminal progress

- Animated catalog/download/verification/install stages on interactive terminals.
- Download bar with real bytes, total size, and transfer speed.
- Colored success messages, with NO_COLOR support.
- Plain stderr output when redirected; list output remains on stdout.
- Download subprocess cleanup on failures and catalog size verification.
- Legacy catalog retained for v0.2.0 clients.

Update using the same public curl installer, then try `shadow install hello`.
The catalog still contains only hello. No changes to the preview's dependency,
sandboxing, or recovery limitations; see README.md.
