# License and third-party notices

## Project-owned code

Copyright (C) 2026 cbart-cmd and contributors.

WvW Combat Insights is free software: you can redistribute it and/or modify it
under the terms of the GNU General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
FOR A PARTICULAR PURPOSE. See [LICENSE](LICENSE) for the full license text.

The project also grants the limited integration permission in
[ADDITIONAL-PERMISSION.md](ADDITIONAL-PERMISSION.md), under GPL version 3,
section 7, for use with GW2, Nexus and arcdps. It does not waive the GPL
requirements for project-owned code or third-party terms.

Base SPDX identifier: `GPL-3.0-or-later`. This grant applies to project-owned code,
not to third-party artwork, trademarks or dependencies under their own licenses.

## Guild Wars 2 content

Guild Wars 2 artwork, names and game data are owned by ArenaNet and/or their
respective rights holders. All rights reserved. They are not relicensed under
the GPL, MIT or CC0 by this project.

- Profession icons: `nexusbase/assets/professions`, official GW2 API
  [professions endpoint](https://api.guildwars2.com/v2/professions).
- Elite specialization icons: `nexusbase/assets/specializations`, official GW2 API
  [specializations endpoint](https://api.guildwars2.com/v2/specializations).
- Weapon artwork: `nexusbase/assets/weapons`; individual URLs, hashes and sources
  are recorded in [sources.json](nexusbase/assets/weapons/sources.json).
- Official localized names and skill metadata: `nexusbase/gw2/game_names.rs` and
  `nexusbase/gw2/weapon_skills.rs`, generated from the
  [Guild Wars 2 API](https://wiki.guildwars2.com/wiki/API:Main).

Use of these resources is subject to ArenaNet's
[Content Terms of Use](https://www.arena.net/en/legal/content-terms-of-use).
Their presence here does not grant unrestricted reuse. If redistributing this
project or its assets, review the applicable terms independently.

Guild Wars 2 is a trademark of ArenaNet. This is an unofficial community project,
not affiliated with, endorsed by or sponsored by ArenaNet or NCSOFT.

## Dependencies

Exact dependency versions and Git revisions are recorded in [Cargo.lock](Cargo.lock).
[THIRD-PARTY-LICENSES.txt](THIRD-PARTY-LICENSES.txt) contains the dependency
inventory and bundled license/notice texts, including native Dear ImGui code.
Dependencies keep their original licenses and copyright notices.

Some Git dependencies do not declare a license in Cargo metadata. Their checked-in
license files were reviewed instead:

| Dependency | Source | License |
| --- | --- | --- |
| nexus / nexus_codegen | [nexus-rs](https://github.com/Zerthox/nexus-rs) | MIT, copyright 2024 Zerthox |
| evtc | [arcdps-rs](https://github.com/Zerthox/arcdps-rs) | MIT, copyright 2019-2022 greaka, 2022 Zerthox |
| gw2_mumble | [gw2-mumble-rs](https://github.com/Zerthox/gw2-mumble-rs) | MIT, copyright 2023 Zerthox |

The remaining dependency declarations are permissive licenses, including MIT,
Apache-2.0, BSD-3-Clause and Unicode-3.0. The bundled texts, not this summary,
define their terms. Regenerate and review the inventory when Cargo.lock changes.

Four crates omit license files from their published package. The collector uses
version-specific copies in `tools/licenses`, never a generic guessed license:

- bitfields / bitfields-impl 3.1.0: MIT notice from
  [the packaged source revision](https://github.com/gregorygaines/bitfields-rs/blob/261d58bb4d354ba6b3a3a4927c02b064e2076a28/LICENSE).
- winapi-i686-pc-windows-gnu / winapi-x86_64-pc-windows-gnu 0.4.0:
  the winapi-rs MIT notice, also bundled by winapi 0.3.9. These GNU-target import
  libraries are included in the inventory even though the Windows MSVC build
  does not link them.

## Protocol and methodology references

The code documents references to the
[arcdps EVTC format](https://www.deltaconnected.com/arcdps/evtc/README.txt),
[Healing Stats](https://github.com/Krappa322/arcdps_healing_stats),
[GW2 Elite Insights](https://github.com/baaron4/GW2-Elite-Insights-Parser) and
[AxiLog](https://github.com/darkharasho/axilog).
These are protocol/calculation references, not a claim of ownership or endorsement.
See [calculos.md](calculos.md) for pinned methodology references and limitations.

The independently implemented DEFLATE decoder references RFC 1951 and the
canonical Huffman decoding approach described by Mark Adler's
[`puff.c`](https://github.com/madler/zlib/blob/master/contrib/puff/puff.c).
No third-party implementation is relicensed by this reference.
