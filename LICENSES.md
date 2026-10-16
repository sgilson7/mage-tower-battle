# Licences

Every file in this repository that is not code written for it: its source, its author, its terms. The code is under the repository's licence.

## Assets

| Path | Source | Terms |
| --- | --- | --- |
| `data/models.json` | written by `tools/models.py` in this repository | this repository's licence |
| `data/sounds.json` | chip sound recipes written for this repository | this repository's licence |

## Vendored code

| Path | Project and version | Author | Licence | Checksum |
| --- | --- | --- | --- | --- |
| `web/net/vendor/trystero-nostr-0.25.4.js` | [Trystero](https://github.com/dmotz/trystero) 0.25.4, Nostr strategy, bundled by esm.sh | Dan Motzenbecker | MIT | `web/net/vendor/SHA256SUMS` |

## Code taken from other repositories

| From | What | Licence |
| --- | --- | --- |
| Builder (`sgilson7/builder-setting`) and Galactic Survivors | the online kit: `web/net/rtc.js`, `web/net/config.js`, `web/online.js`, `crates/core/tests/online_fence.rs`, `tests/browser/online.py` | MIT, Sam Gilson |
| Galactic Survivors (`crates/core/tests/rig_space.rs`) | the logbook's structure in `crates/core/tests/spell_space.rs`: row numbering, the shuffle, stale-line keys, the index and summary files, the date and commit stamps | MIT, Sam Gilson |
| sgilson7/long-time-comin (same author) | `web/gl.js` and `data/shaders/scene.vert` adapted from its renderer; `crates/core/src/mesh.rs` from its `mesh.rs` | MIT |
| sgilson7/builder-setting kits (same author) | `fx.rs`, `rng.rs`, `ticks.rs`, `draw.rs`, `render2d.js` | MIT |

A file with no licence, a likeness, a recording or any third-party picture is a worklist row in `SECOND-ORDER.md` for the person to decide, and stays out of git until they do.
