# Eruption SDK Compatibility Policy

This policy defines expectations for SDK stability across Rust, Python, C/C++, and shell interfaces.

## Versioning

- Public SDK changes should follow semantic versioning principles.
- Backward-incompatible API changes require a major version bump.
- Backward-compatible additions should use minor version bumps.
- Patch versions are reserved for bug fixes and non-breaking internal improvements.

## Deprecation Policy

- Deprecated APIs should emit clear documentation markers and migration guidance.
- Deprecated APIs should remain available for at least one minor release cycle unless there is a security or safety reason.
- Removals must be listed in release notes and changelog with explicit replacement guidance.

## Behavioral Compatibility

- Existing API calls should preserve semantics unless a breaking change is declared.
- Error codes and error classes should remain stable or be mapped with compatibility aliases.
- Serialization and wire-level changes should be versioned and documented.

## Documentation Requirements

- Every public API change must include updated docs and an example when applicable.
- Breaking or potentially surprising behavior changes must include migration notes.

## Testing Expectations

- Add regression coverage for fixed SDK bugs.
- Validate core flows at minimum: connect, query, update profile/effects, error handling.