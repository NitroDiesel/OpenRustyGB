# OpenRustyGB project context for agents

## Mission

OpenRustyGB is an all-Rust reimplementation of OpenRGB pinned to upstream commit
`8121ee29f46d58f90a56348eb5bf7a64f52f923b`. Preserve the observable behavior,
features, protocols, layouts, compatibility surfaces, and platform support of
that version. Rebuild those ideas with Rust-owned architecture instead of
translating the C++ class graph line by line.

The final source tree and shipped artifacts must contain no C, C++,
Objective-C, native headers, Qt, qmake, native application CMake, or C/C++ FFI
dependency. Native code currently remains only as an unshipped migration
reference. A successful conversion replaces verified behavior before deleting
its native owner.

The public repository is
[`NitroDiesel/OpenRustyGB`](https://github.com/NitroDiesel/OpenRustyGB). `main`
is the published integration branch. `upstream` points to the original OpenRGB
repository; the parity target stays pinned unless the owner explicitly changes
it.

## Sources of truth

Read only the branch relevant to the task:

| Task | Authoritative context |
| --- | --- |
| Product behavior and parity boundary | [`PRODUCT.md`](../PRODUCT.md) |
| Runtime, SDK, persistence, plugin, UI, and crate design | [`architecture/ARCHITECTURE.md`](architecture/ARCHITECTURE.md) |
| Accepted architecture decisions and tradeoffs | [`architecture/arena-synthesis.md`](architecture/arena-synthesis.md) |
| Completed controller families and their evidence | [`migration/ported-families.md`](migration/ported-families.md) |
| Native-code removal and release rule | [`migration/rust-only-contract.md`](migration/rust-only-contract.md) |
| Live family, detector, and package counts | `cargo xtask inventory` |
| Live native-source count and Rust-only gate | `cargo xtask source-audit` |
| Cross-platform validation | [`.github/workflows/rust-foundation.yml`](../.github/workflows/rust-foundation.yml) |

When documentation and executable checks disagree, investigate the drift and
repair both in the same change. Do not make a stale count look current by
editing prose alone.

## Verified state

Snapshot: 2026-09-01, after the HP Omen 30L migration.

- 43 of 197 pinned controller families are contracted to Rust.
- 182 of 224 detector sources remain native; 42 detector sources have Rust
  owners.
- 44 Rust driver packages exist. One package is the requested HyperX Pulsefire
  Haste 2 support that did not replace an upstream pinned family.
- 1,890 C/C++/Objective-C source or header files and 48 native or Qt build
  descriptions remain.
- The Rust-only source gate is intentionally blocked. There is no supported
  parity installer or release yet.
- Windows, Linux, and macOS CI validate formatting, strict Clippy, workspace
  tests, inventory consistency, and the expected release block.
- Packet fixtures and fake transports are evidence of serialization behavior.
  They are not physical-hardware validation. The port ledger states whether a
  matching device was present for each contracted family.

Run the two `xtask` commands before relying on these snapshot counts. The port
ledger names every completed family; keep this file compact instead of copying
that list here.

## What has been built

The repository has a working Rust workspace with domain values, driver
capability traits, HID transport with feature-report reads and writes,
controller actors, a CLI integration path,
family-owned driver packages, packet-level tests, and checked migration audits.
Completed drivers preserve exact HID matching, layouts, mode metadata,
validation, packet serialization, pacing, lifecycle messages, and guarded write
paths appropriate to each upstream family.

Recent contracted keyboard families include Skyloong GK104 Pro, Anne Pro 2,
Ionico, XPG Summoner, Ducky, Thermaltake Poseidon Z RGB, Red Square Keyrox, and
Valkyrie VK99. The MSI Raider A18 laptop keyboard and lightbar family is also
contracted, along with the HP Omen 30L motherboard lighting controller.
The ledger contains the complete set and verification evidence.

HyperX Pulsefire Haste 2 support remains part of the product. Keep it presented
as ordinary device support, not as the repository description or GitHub About
tagline. Its exact transport and safety limits are defined in the architecture
document.

The README must continue to state that the project is experimental, may be
buggy, and lacks physical-device evidence where that is true. Carry the same
warning into any future prerelease notes.

## Controller-family migration loop

Use one owned family as the unit of contraction:

1. Map every native detector, reader, writer, lifecycle path, mode, range,
   layout, persistence action, and timing rule. Completion means every native
   behavior in that family has a named Rust destination or a documented reason
   it cannot yet move.
2. Implement the matcher, typed settings, serializer, controller description,
   and transport transaction in `crates/drivers/<family>`. Preserve wire
   behavior while replacing undefined memory, unchecked ranges, and ambiguous
   endpoint selection with deterministic validation.
3. Add packet goldens, matcher negatives, layout checks, invalid-input tests,
   and transport failure tests. Wire a read-only CLI probe and explicit guarded
   commands through `openrustygb-app`.
4. Run focused crate and application checks. Use a matching physical device
   when available, starting with a read-only probe. Record hardware absence or
   the exact live result without inference.
5. Run the full workspace gates. Delete only that family’s native files after
   the Rust path passes. Remove empty directories with exact validated paths.
6. Add the family and detector to `xtask/src/main.rs`, append its evidence to
   the port ledger, rerun the audits, commit with Codex attribution when Codex
   authored material changes, push to `main`, and wait for cross-platform CI.

The Valkyrie migration added the narrow HID feature-report reader needed by
controllers that pair feature writes with acknowledgements. Keep that API
report-ID driven and use it only where the native protocol performs a feature
read.

## Hardware safety

- Enumeration and probe commands are read-only. State that explicitly in their
  output.
- Match the full endpoint identity required by upstream. Refuse ambiguous
  matches instead of selecting the first device.
- Require `--confirm-reversible-write` for transient lighting changes.
- Require `--confirm-persistent-write` for save, profile-memory, firmware-mode,
  or potentially retained hardware-effect changes.
- Keep firmware updates, resets, arbitrary reports, and persistent device-memory
  operations outside a lighting port unless the owner explicitly scopes and
  authorizes them.
- Claim live success only after an actual matching device completes the test.
  Preserve pointer, button, wheel, DPI, polling, reconnect, and teardown
  behavior when validating a mouse lighting path.

## Required local gates

Run from the repository root:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --quiet -- -D warnings
cargo test --workspace --quiet
cargo xtask inventory
cargo xtask source-audit
cargo xtask source-audit --require-rust-only
git diff --check
```

During migration, the final command with `--require-rust-only` must fail because
native source still exists. Treat an unexpected pass as an audit defect until
the entire parity ledger is complete. At release time, both
`cargo xtask inventory --require-parity` and
`cargo xtask source-audit --require-rust-only` must pass.

## Publication boundary

Do not publish an installer, tag, or parity release while either release gate
is blocked. The eventual release must provide verified Windows, Linux, and
macOS installers comparable to OpenRGB’s release surface. Test installation,
launch, a real product mode, SDK and persistence behavior, uninstall, checksums,
SBOMs, and signing or notarization status before calling those assets ready.

Use `Co-authored-by: Codex <codex@openai.com>` on material Codex-authored
commits. Preserve published history; attribution is added to new commits rather
than rewriting old ones.

## Handoff completion

Before ending an agent run, leave the repository in one of two observable
states:

- a verified, committed, pushed increment with its GitHub Actions result; or
- an uncommitted diagnostic state that identifies the exact blocker, preserves
  the native behavior oracle, and lists the next safe check.

Report the current inventory and release-gate state. Never describe the whole
rewrite as complete while either checked gate remains blocked.
