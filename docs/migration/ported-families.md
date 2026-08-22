# Contracted controller families

This ledger records upstream family directories whose native implementation
has been replaced and deleted from the current tree. Git history retains the
pinned source and provenance.

## GameSirController

- Rust package: `openrustygb-driver-gamesir-nova-lite-2`
- Device: GameSir Nova 2 Lite
- Exact match: VID `3537`, PID `100F`, interface `2`, usage page `FF7A`, usage `0001`
- Preserved model: gamepad, Static mode value `FFFF`, one Controller zone, one Main LED, per-LED color
- Preserved output: one 64-byte HID report beginning `05 08 0A 01 03 RR GG BB 00 CC`
- Checksum: low byte of the sum of bytes 0 through 8
- Verification: exact-match rejection tests, packet golden, checksum wrap, short-write rejection, model-shape test, executable read-only probe, workspace Clippy and tests
- Deleted native files: `GameSirController.cpp`, `GameSirController.h`, `GameSirControllerDetect.cpp`, `RGBController_GameSir.cpp`, `RGBController_GameSir.h`

Physical GameSir hardware was not present for this contraction. The family
remains release-blocked by the global hardware-evidence policy until a matching
device completes its live test.
