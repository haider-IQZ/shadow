# Shadow engineering guidelines

## Modular design

- Do not create god files or god objects.
- Give each module and file one clear responsibility, not a collection of unrelated responsibilities.
- Separate CLI handling, recipe evaluation, building, package format, installation, dependency resolution, persistence, and repository transport behind explicit interfaces.
- Keep entry points thin. Split modules along real responsibility boundaries rather than arbitrary line counts.

## Production-quality implementation

- Never simplify away required correctness, security, error handling, or lifecycle behavior to make an implementation easier.
- Write production-quality code within the agreed scope. Do not substitute stubs, mock behavior, hardcoded success paths, or temporary hacks for working functionality.
- Prefer clear, maintainable implementations over unnecessary complexity. A smaller scope is acceptable; silently weakened guarantees are not.
- Validate untrusted inputs at boundaries. Treat recipes as executable code and package archives as untrusted input.
- Handle filesystem ownership, path traversal, symlinks, interrupted operations, and state consistency explicitly when implementing package operations.
- Propagate actionable errors with context. Do not swallow failures or use panics for expected runtime errors.
- Keep tests lean and risk-based. Prioritize core package lifecycle behavior, destructive filesystem operations, and security boundaries. Avoid exhaustive permutations, redundant tests, and tests of trivial implementation details; add regression tests for actual bugs.
- Run formatting, linting, and the focused tests relevant to a change before declaring work complete. Do not build a large test suite merely to increase coverage numbers.
- Document assumptions, limitations, and guarantees honestly. Do not claim isolation, atomicity, durability, or reproducibility without implementing and testing the required mechanisms.
- Keep development and tests confined to explicit test prefixes; never modify host-managed package files or require elevated permissions without explicit authorization.
