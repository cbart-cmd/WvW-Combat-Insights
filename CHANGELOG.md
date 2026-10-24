# Changelog

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
