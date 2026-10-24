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
- Direct updater URL: `https://raw.githubusercontent.com/cbart-cmd/gw2addonsreleases/main/fight_analysis_danger.dll`.
  Nexus 2026.2.17.1210 can fail with `Failed to read connection` on the older
  `https://github.com/cbart-cmd/gw2addonsreleases/raw/refs/heads/main/fight_analysis_danger.dll`.
  Preserve Direct, identity, signature and data/settings paths. Older DLLs retain
  their URL and may need a one-time authorized manual replacement with GW2 closed.
  Never copy into the game without explicit authorization or while GW2 is running.
- Follow [Nexus publication checks](nexusbase/README.md#publicacion): inspect real
  GetAddonDef, unset all four LFS attributes, compare staged blob with release DLL,
  and verify exact new/old URLs plus release assets. MD5 must be exactly 32 lowercase
  hexadecimal ASCII bytes without BOM/newline. Reuse the original Nexus HTTP client
  and MD5 parser when possible; browser/curl/HttpClient alone do not prove compatibility.
  Wait for genuine CDN propagation; no version churn or query strings to hide stale data.
- CI is read-only and does not deploy. Include corresponding source and license
  notices with distributed binaries; preserve MD5 compatibility and add SHA-256.
- Report what was tested and any remaining limitations at the end of the task.
