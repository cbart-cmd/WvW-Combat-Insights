# Development instructions

- Read [nexusbase/README.md](nexusbase/README.md) before changing infrastructure.
- Keep reusable infrastructure in `nexusbase`, addon behavior in `src`.
- Identity, signature, updater URL and storage paths are in `addon.toml`.
  Do not change them as part of a refactor or reset users' saved settings.
- Prefer existing helpers, standard libraries and installed dependencies.
  Make the smallest complete change; avoid unrelated fixes and speculative features.
- Read callers before changing shared behavior. Preserve error handling,
  validation, public APIs, FFI contracts and unload safety.
- Add a focused regression test for non-trivial logic. Run the smallest useful
  checks first, then the Windows checks documented in CONTRIBUTING.md.
- Update README.md, readme.html, CHANGELOG.md and PROGRESO.md for functional changes.
- Project-owned contributions use GPL-3.0-or-later with ADDITIONAL-PERMISSION.md.
  Retain third-party notices;
  game artwork is not GPL. Regenerate dependency notices when Cargo.lock changes.
- Never commit credentials, private combat logs, personal settings or build outputs.
- Do not overwrite unrelated working-tree changes.
- Do not add a Copilot co-author trailer to commits.
- Do not publish releases, push changes, rewrite history or change repository
  visibility without explicit authorization.
- The public checkout must build on its own, without sibling repositories.
- Keep the Nexus DLL name and update endpoint stable. Distribution must use real
  PE binaries, not Git LFS pointers. Verify MZ bytes, size and hashes after upload.
- CI is read-only and does not deploy. Include corresponding source and license
  notices with distributed binaries; preserve MD5 compatibility and add SHA-256.
- Report what was tested and any remaining limitations at the end of the task.
