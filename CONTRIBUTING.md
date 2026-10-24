# Contributing

Use [Issues](https://github.com/cbart-cmd/WvW-Combat-Insights/issues) for bugs
and focused feature requests. Search existing reports first.

Include the addon, Nexus and arcdps versions, steps to reproduce, expected and
actual behavior, and whether the problem affects live reports or saved logs.
Do not post account names, character names, chat, private paths, credentials or
unredacted combat logs. Crop/redact screenshots before uploading. A ZIP is not
anonymization; use synthetic examples whenever possible.

Report security issues privately as described in [SECURITY.md](SECURITY.md).

## Build and checks

Use Windows x64, stable Rust with rustfmt, and Visual Studio Build Tools with
the Desktop development with C++ workload and a Windows SDK.

```powershell
cargo fmt --all -- --check
cargo test --locked --all-targets
cargo build --locked --release
python tools\collect_licenses.py --test
python tools\collect_licenses.py --check
```

The checkout builds independently; sibling addon repositories are not required.
The Python check uses only the standard library and Cargo's local dependency
metadata. After dependency changes, run `python tools\collect_licenses.py` and
review the generated notices. Game-data generators need Internet access, but
are not required for normal builds.
If Cargo reports missing cached packages for other targets, run
`cargo fetch --locked` before collecting notices.

Keep changes focused, preserve saved settings and the public API, and add a
small regression test for non-trivial behavior changes. Update both README.md
and readme.html when user-visible behavior changes.

Contributions to project-owned code are licensed under GPL-3.0-or-later with
the integration permission in [ADDITIONAL-PERMISSION.md](ADDITIONAL-PERMISSION.md).
Only submit code and assets you have permission to distribute; retain upstream
copyright and license notices.

## Distribution

CI has read-only repository permissions and never publishes an update.
Manual releases must include the corresponding source, LICENSE, ADDITIONAL-PERMISSION.md,
THIRD-PARTY-NOTICES.md and THIRD-PARTY-LICENSES.txt with the DLL and checksums.
Use SHA-256 for manual verification and retain MD5 for Nexus compatibility.
Do not commit personal logs or build outputs.
