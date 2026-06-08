# Changelog

## 0.3.1

### Fixed

- **`helyos deployments`, `helyos pods`, and `helyos routes` no longer hide resources.**
  These read-only listings silently filtered by the active context's default project, so
  they could show *"0 total"* while `helyos status` reported N running. They now list
  everything by default and filter only when `-p/--project` is passed explicitly.

### Changed

- Depends on helyos-core 0.1.8.
