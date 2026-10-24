# Changelog

## 0.2.20

- Version-only release to test Nexus auto-update from 0.2.19 through the existing
  direct DLL/MD5 endpoint. No behavior, identity or settings changes.
- Do not manually install this version when testing the Nexus update flow.

## 0.2.19

- Use the direct raw.githubusercontent.com DLL/MD5 endpoint with Nexus's Direct
  updater; retain addon identity, signature, data paths and user settings.
- Pin the endpoint and Direct provider in the project configuration regression.
- Publish the MD5 as exactly 32 lowercase hexadecimal ASCII bytes without BOM or newline.
- Older DLLs retain the previous GitHub URL. If Nexus cannot connect, replace the
  DLL manually once with GW2 closed; never overwrite an active game installation.

## 0.2.18

- Start live reports on observed allied PvP while the local player is in combat,
  including players who only heal. NPC-only fights and fights observed out of combat
  do not start a live report.
- Open live windows at the first refresh, without the saved-log duration threshold.
- Keep completed provisional fights as selectable session-history entries, including
  short fights and map transitions between refreshes. Matching EVTC logs replace them.
  Provisional entries are not saved to disk.
- Select the player's team when switching reports; preserve manual color changes
  while refreshing the same report. Migrate the previous enemy-first default once.
- Preserve existing updater URLs, addon identity, log calculations and healing coverage.

## 0.2.17

Initial public source snapshot, without the previous private Git history.

- Independent WvW team overview, player highlights and squad healing windows.
- Live provisional reports and EVTC/ZIP analysis, sharing the same calculations.
- English, Spanish, French and German UI; bundled official names and icons.
- Local reusable Nexus infrastructure and build-time addon identity.
- Source code released under GPL-3.0-or-later with a limited integration
  permission for GW2/Nexus/arcdps, and separate third-party notices.
- Windows CI checks formatting, tests and release builds.

This publication does not change addon behavior, version, updater endpoint,
DLL name or existing user settings. See [calculos.md](calculos.md) for calculation
definitions and known differences from other analyzers.
