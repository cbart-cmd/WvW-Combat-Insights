# Changelog

## 0.2.32

- Attribute deaths without a recorded killing blow to the enemy team that dealt the
  most damage to the victim team; fall back to the largest enemy roster when damage
  cannot distinguish a team.

## 0.2.31

- Window opacity now also fades the class-color row backgrounds instead of only the
  window background: the bar alpha is the row alpha times the window opacity.

## 0.2.30

- Healing rows use the elite specialization icon, like Highlights, instead of the
  core profession icon; it falls back to the profession icon when the elite is unknown.

## 0.2.29

- Class-color row background in Highlights and Healing (on by default): each row is
  tinted with the profession color and the name and numbers switch to white or
  near-black for contrast. Turn it off in right-click -> Style to return to
  transparent rows. Lives in the shared class_colors setting.
- Add 	ools/publish.ps1 and 	ools/verify-release.ps1: build, DLL/MD5 to the
  distribution repo, gitattributes and staged-blob checks, push, release with
  --latest=false, keep the two newest releases and verify every download URL.
- Add litei/ reference copies of the Elite Insights sources used to fix rules.

## 0.2.28

- Healing and barrier follow Elite Insights: every event whose IFF byte marks a
  friendly target counts, including the player, pets and allies outside the squad.
  Knowing the POV squad is no longer required.
- Barrier-absorbed damage no longer counts as damage, strike damage, condition
  damage or down contribution, matching Elite Insights health damage.

## 0.2.27

- A real log now replaces its provisional live fight when their event ranges
  overlap, instead of requiring full containment, so the same fight is no longer
  listed twice.

## 0.2.26

- The live report closes after arcdps' [session] wvwinactive timeout instead of
  immediately on exit combat, and is then kept in the session history.

## 0.2.25

- Kills, deaths and downs come from EVTC state changes, matching Elite Insights.

## 0.2.24

- Build with one codegen unit: smaller DLL, same analysis.

## 0.2.23

- Faster decompression and log analysis.

## 0.2.22

- Parser and live performance, fewer UI allocations, no row cap in Healing.

## 0.2.21

- Squad healing and barrier from Healing Stats, cleanses, estimated weapons and
  per-team danger ranking.
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
