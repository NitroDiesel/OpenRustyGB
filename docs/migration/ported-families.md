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

## FaustusController

- Rust package: `openrustygb-driver-faustus-keyboard`
- Device: ASUS TUF Laptop Keyboard using the Linux Faustus sysfs driver
- Exact match: all six attributes beneath `/sys/devices/platform/faustus/kbbl`: `kbbl_red`, `kbbl_green`, `kbbl_blue`, `kbbl_mode`, `kbbl_flags`, and `kbbl_set`
- Preserved model: laptop, Static, Breathing, Color Cycle, and Strobe modes, one Keyboard Backlight zone, and one Keyboard Backlight LED
- Preserved mode values: Static `0`, Breathing `1`, Color Cycle `2`, Strobe `3`; Breathing and Color Cycle expose speed `0..2` with default `1`
- Preserved output: lowercase hexadecimal RGB and mode values, flags `2a`, followed by commit value `2`
- Verification: exact six-file detection test, output-value golden, model-shape test, executable read-only probe, workspace Clippy and tests
- Deleted native files: `RGBController_Faustus_Linux.cpp`, `RGBController_Faustus_Linux.h`

Physical Faustus hardware and its Linux sysfs interface were not present for
this contraction. The family remains release-blocked by the global
hardware-evidence policy until a matching device completes its live test.

## LexipMouseController

- Rust package: `openrustygb-driver-lexip-np93-alpha`
- Device: Lexip NP93 Alpha gaming mouse
- Exact match: VID `04D8`, PID `FD0A`, interface `0`, usage page `0001`, usage `0002`
- Preserved model: mouse, Direct mode value `0`, one Mouse zone, one LED named LED 1, per-LED direct color
- Preserved output: one 64-byte HID report beginning `00 24 01 RR GG BB 00 64 80`
- Verification: exact-match rejection tests, packet golden, short-write rejection, model-shape test, executable read-only probe, workspace Clippy and tests
- Deleted native files: `LexipMouseController.cpp`, `LexipMouseController.h`, `LexipMouseControllerDetect.cpp`, `RGBController_LexipMouse.cpp`, `RGBController_LexipMouse.h`

Physical Lexip hardware was not present for this contraction. The family
remains release-blocked by the global hardware-evidence policy until a matching
device completes its live test.

## N5312AController

- Rust package: `openrustygb-driver-n5312a-mouse`
- Device: N5312A USB optical mouse controller, including identified ANT Esports KM540 and Marvo M115 devices
- Exact match: VID `4E53`, PID `5406`, interface `1`, usage page `FF01`, usage `0001`
- Preserved model: mouse with Direct, Breathing, Single Breath, and Off modes, one Mouse zone, and one LED
- Preserved controls: brightness `10..100`; Breathing and Single Breath speed `1..10`
- Preserved output: initialization feature report `07 A0`, color report `07 0B 01 RR GG BB`, then mode report `07 0A MM 01 01 01 SS BB`, all padded to 8 bytes
- Verification: exact-match rejection tests, initialization and mode packet goldens, range rejection, forced-black Off behavior, model-shape test, executable read-only probe, workspace Clippy and tests
- Deleted native files: `N5312AController.cpp`, `N5312AController.h`, `N5312AControllerDetect.cpp`, `RGBController_N5312A.cpp`, `RGBController_N5312A.h`

Physical N5312A hardware was not present for this contraction. The family
remains release-blocked by the global hardware-evidence policy until a matching
device completes its live test.

## PatriotViperMouseController

- Rust package: `openrustygb-driver-patriot-viper-v550`
- Device: Patriot Viper V550 mouse
- Exact match: VID `0C45`, PID `7E18`, interface `2`, usage page `FF18`, usage `0001`
- Preserved model: mouse with Direct mode value `1`, Left and Right three-LED zones, and one Mousewheel LED
- Preserved initialization: the original 64-byte feature report sent when the controller opens
- Preserved output: seven 64-byte feature reports in LED order with `01 13 II FF RR GG BB` and the original XOR parity adjustment in byte 63
- Verification: exact-match rejection tests, initialization golden, per-LED packet and checksum tests, seven-report ordering test, model-shape test, executable read-only probe, workspace Clippy and tests
- Deleted native files: `PatriotViperMouseController.cpp`, `PatriotViperMouseController.h`, `PatriotViperMouseControllerDetect.cpp`, `RGBController_PatriotViperMouse.cpp`, `RGBController_PatriotViperMouse.h`

Physical Viper V550 hardware was not present for this contraction. The family
remains release-blocked by the global hardware-evidence policy until a matching
device completes its live test.

## DreamCheekyController

- Rust package: `openrustygb-driver-dream-cheeky-webmail-notifier`
- Device: Dream Cheeky Webmail Notifier
- Match: VID `1D34`, PID `0004`; the native detector accepts any interface and usage
- Preserved model: accessory with Direct mode value `0`, one LED zone, and one LED
- Preserved initialization: four byte-exact 9-byte output reports in their original order
- Preserved output: one 9-byte color report with each 8-bit channel shifted to `0..63` and input `255` mapped to device maximum `64`
- Verification: flexible-match tests, initialization goldens, scaling boundary and packet tests, short-write rejection, model-shape test, executable read-only probe, workspace Clippy and tests
- Deleted native files: `DreamCheekyController.cpp`, `DreamCheekyController.h`, `DreamCheekyControllerDetect.cpp`, `RGBController_DreamCheeky.cpp`, `RGBController_DreamCheeky.h`

Physical Dream Cheeky hardware was not present for this contraction. The family
remains release-blocked by the global hardware-evidence policy until a matching
device completes its live test.
