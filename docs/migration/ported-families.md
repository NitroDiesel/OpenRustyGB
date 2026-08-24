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

## NvidiaESAController

- Rust package: `openrustygb-driver-nvidia-esa-xps-730x`
- Device: NVIDIA ESA lighting in the Dell XPS 730x
- Match: VID `0955`, PID `000A`, usage page `FFDE`, usage `0002`; the native detector accepts any interface
- Preserved model: case with Static mode value `0`, five single-LED zones named Front Drive Bays, Front USB, Rear, Internal, and Front Audio
- Preserved output: commands `42..46` followed by inverted 4-bit RGB channels, one 4-byte output report per zone
- Verification: flexible-match tests, inverse-channel boundaries, zone packet golden, five-command ordering, short-write rejection, model-shape test, executable read-only probe, workspace Clippy and tests
- Deleted native files: `NvidiaESAController.cpp`, `NvidiaESAController.h`, `NvidiaESAControllerDetect.cpp`, `RGBController_NvidiaESA.cpp`, `RGBController_NvidiaESA.h`

Physical NVIDIA ESA hardware was not present for this contraction. The family
remains release-blocked by the global hardware-evidence policy until a matching
device completes its live test.

## MadCatzCyborgController

- Rust package: `openrustygb-driver-madcatz-cyborg-light`
- Device: MadCatz Cyborg Gaming Light
- Match: VID `06A3`, PID `0DC5`; the native detector accepts any interface and usage
- Preserved model: accessory with Direct mode value `0`, brightness `0..100`, one Cyborg zone, and one LED
- Preserved output: feature reports `A1 00` to enable, `A6 00 II` for clamped intensity, and `A2 00 RR GG BB 00 00 00 00` for color
- Verification: flexible-match test, mixed-size packet and ordering golden, brightness clamp boundaries, model-shape test, executable read-only probe, workspace Clippy and tests
- Deleted native files: `MadCatzCyborgController.cpp`, `MadCatzCyborgController.h`, `MadCatzCyborgControllerDetect.cpp`, `RGBController_MadCatzCyborg.cpp`, `RGBController_MadCatzCyborg.h`

Physical MadCatz hardware was not present for this contraction. The family
remains release-blocked by the global hardware-evidence policy until a matching
device completes its live test.

## ThingMController

- Rust package: `openrustygb-driver-thingm-blink1-mk2`
- Device: ThingM blink(1) mk2
- Match: VID `27B8`, PID `01ED`, usage page `FF00`, usage `0001`; the native detector accepts any interface
- Preserved model: LED strip with Off value `0`, Direct value `1`, Fade value `2`, one `blink(1) mk2` zone, and LEDs A and B
- Preserved output: one 9-byte feature report per LED with command `01 63`, RGB, the low 16 bits of fade speed in big-endian order, LED ID, and a trailing zero
- Preserved mode behavior: Off forces both LEDs black, Direct forces zero speed, Fade applies the requested speed, and every mode update writes LED A before LED B
- Verification: flexible-interface and exact-usage tests, packet and speed goldens, mode-semantics test, two-LED ordering test, model-shape test, executable read-only probe, workspace Clippy and tests
- Deleted native files: `BlinkController.cpp`, `BlinkController.h`, `RGBController_BlinkController.cpp`, `RGBController_BlinkController.h`, `ThingMControllerDetect.cpp`

Physical ThingM hardware was not present for this contraction. The family
remains release-blocked by the global hardware-evidence policy until a matching
device completes its live test.

## MSI3ZoneController

- Rust package: `openrustygb-driver-msi-3-zone-keyboard`
- Device: MSI/SteelSeries 3-Zone laptop keyboard
- Match: VID `1770`, PID `FF00`; the native detector accepts any interface and usage
- Preserved model: laptop with Direct mode value `0`, a three-LED Keyboard zone, a one-LED Aux zone, and the original four LED names
- Preserved output: seven 8-byte feature reports with prefix `01 02 40`, zone IDs `1..7`, RGB, and suffix `EC`
- Preserved ordering: keyboard Left, Middle, Right, then Aux; native report IDs `4..7` all reuse the Aux color
- Verification: flexible-match test, packet and Aux-reuse goldens, seven-report ordering test, model-shape test, executable read-only probe, workspace Clippy and tests
- Deleted native files: `MSI3ZoneController.cpp`, `MSI3ZoneController.h`, `MSI3ZoneControllerDetect.cpp`, `RGBController_MSI3Zone.cpp`, `RGBController_MSI3Zone.h`

Physical MSI 3-Zone hardware was not present for this contraction. The family
remains release-blocked by the global hardware-evidence policy until a matching
device completes its live test.

## NZXTMouseController

- Rust package: `openrustygb-driver-nzxt-lift-mouse`
- Device: NZXT Lift mouse
- Exact match: VID `1E71`, PID `2100`, interface `0`, usage page `FFCA`, usage `0001`
- Preserved model: mouse with Direct mode value `FFFF`, Left and Right three-LED zones, and six original LED names
- Preserved firmware handshake: exact 64-byte `43 81 00 01` request, input filtering until response `43 86`, and version bytes at offsets `3..5`
- Preserved output: one 64-byte `43 AE` direct report with the native LED order `2, 1, 0, 3, 4, 5` and color offsets `25, 29, 33, 37, 41, 45`
- Verification: exact-match rejection tests, firmware request and response-filtering golden, direct packet mapping golden, model-shape test, executable read-only probe, workspace Clippy and tests
- Deleted native files: `NZXTMouseController.cpp`, `NZXTMouseController.h`, `NZXTMouseControllerDetect.cpp`, `RGBController_NZXTMouse.cpp`, `RGBController_NZXTMouse.h`

Physical NZXT Lift hardware was not present for this contraction. The family
remains release-blocked by the global hardware-evidence policy until a matching
device completes its live test.

## AsusMonitorController

- Rust package: `openrustygb-driver-asus-monitor`
- Devices: ASUS ROG STRIX XG27AQDMG and XG27UCG, ASUS ROG SWIFT PG32UCDM and PG32UCDMR
- Exact matches: VID `0B05`, product IDs `1BA3`, `1BB4`, `1B2B`, and `1C9B`, interface `1`, usage page `FF72`, usage `00A1`
- Preserved model: monitor with Direct mode value `0`, one dynamic Monitor zone, and LED names generated from the queried device count
- Preserved discovery: exact 65-byte `EC B0` output request and LED count from input byte 32, including the native zero count on an empty read
- Preserved initialization: exact 65-byte `EC 35` report with bytes 5 and 8 set to `FF` and `01`
- Preserved output: dynamic `EC 40 84` per-LED RGB report with the LED count at byte 4 and RGB data starting at byte 5
- Verification: four-model exact-match tests, query and initialization goldens, dynamic direct-packet golden, overflow rejection, model-shape test, executable read-only probe, workspace Clippy and tests
- Deleted native files: `AsusMonitorController.cpp`, `AsusMonitorController.h`, `AsusMonitorControllerDetect.cpp`, `RGBController_AsusMonitor.cpp`, `RGBController_AsusMonitor.h`

Physical supported ASUS monitor hardware was not present for this contraction.
The family remains release-blocked by the global hardware-evidence policy until
a matching device completes its live test.

## TecknetController

- Rust package: `openrustygb-driver-tecknet-m008`
- Device: Tecknet M008 mouse
- Match: VID `04D9`, PID `FC05`, usage page `FFA0`, usage `0001`; the native detector accepts any interface
- Preserved model: mouse with Direct value `0`, Off value `FF`, Breathing value `1`, one Logo zone, and one Logo LED
- Preserved output: 16-byte feature report beginning `02 04`, inverted RGB channels, brightness at byte 5, and the native breathing speed table `00 06 03 01` at byte 6
- Safety correction: Off emits the intended zero-brightness, zero-speed report without indexing a two-row native table with mode value `FF`
- Verification: flexible-interface matcher test, inverted RGB packet golden, complete breathing-speed table test, safe Off test, model-shape test, executable read-only probe, workspace Clippy and tests
- Deleted native files: `TecknetController.cpp`, `TecknetController.h`, `TecknetControllerDetect.cpp`, `RGBController_Tecknet.cpp`, `RGBController_Tecknet.h`

Physical Tecknet hardware was not present for this contraction. The family
remains release-blocked by the global hardware-evidence policy until a matching
device completes its live test.

## HyperXMousematController

- Rust package: `openrustygb-driver-hyperx-mousemat`
- Devices: HyperX Fury Ultra, HyperX Pulsefire Mat, and HyperX Pulsefire Mat RGB Mouse Pad XL
- Exact matches: Fury Ultra VID `0951`, PID `1705`, interface `0`; Pulsefire Mat VID `03F0`, PID `0F8D`, interface `1`, usage `FF90:FF00`; XL VID `0951`, PID `1741`, with the native Windows interface `1`/usage `FF90:FF00` and non-Windows interface `0`/usage `000C:0001` split
- Preserved model: Direct mode value `FFFF`; 15-LED Underglow plus five-LED Strip zones for standard devices, and a two-LED Underglow zone for the XL
- Preserved output: one 65-byte `00 04 F2` profile-selection feature report followed by two 65-byte reports containing 16 `81 RR GG BB` slots each
- Preserved lifecycle contract: the native 50 ms direct-mode refresh interval is exported for the long-running controller host
- Safety correction: the two or 20 model colors are zero-padded to the protocol's 32 slots instead of reproducing the native out-of-bounds reads
- Verification: all platform-specific matcher tests, three-report packet goldens, unused-slot padding tests, exact color-count and model-shape tests, executable read-only probe, workspace Clippy and tests
- Deleted native files: `HyperXMousematController.cpp`, `HyperXMousematController.h`, `HyperXMousematControllerDetect.cpp`, `RGBController_HyperXMousemat.cpp`, `RGBController_HyperXMousemat.h`

Physical supported HyperX mousemat hardware was not present for this
contraction. The family remains release-blocked by the global hardware-evidence
policy until a matching device completes its live test.

## LegoDimensionsToypadBaseController

- Rust package: `openrustygb-driver-lego-dimensions-toypad`
- Device: Lego Dimensions Toypad Base
- Match: VID `0E6F`, PID `0241`; the native detector accepts any interface and usage
- Preserved model: LED strip with Center, Left, and Right single-LED zones; Direct, Flash value `C3`, and Fade value `C2` modes; effect speed range `0..255` with default `127`
- Preserved initialization: exact 32-byte activation report containing the native `(c) LEGO 2014` payload and checksum `F7`
- Preserved direct output: Center, Left, and Right 32-byte reports in order, with zone values `1..3`, `55 06 C0 02` command bytes, RGB, and wrapping checksum
- Preserved effects: all-zone Flash and Fade reports with native timing, pulse count, RGB positions, and wrapping checksums
- Verification: product-only matcher test, activation/direct/Flash/Fade packet goldens, zone-order and checksum tests, model-shape test, executable read-only probe, workspace Clippy and tests
- Deleted native files: `LegoDimensionsToypadBaseController.cpp`, `LegoDimensionsToypadBaseController.h`, `LegoDimensionsToypadBaseControllerDetect.cpp`, `RGBController_LegoDimensionsToypadBase.cpp`, `RGBController_LegoDimensionsToypadBase.h`

Physical Lego Dimensions Toypad Base hardware was not present for this
contraction. The family remains release-blocked by the global hardware-evidence
policy until a matching device completes its live test.

## AOCMousematController

- Rust package: `openrustygb-driver-aoc-amm700-mousemat`
- Device: AOC AGON AMM700
- Exact match: VID `3938`, PID `1162`, interface `1`, usage page and usage `FF19`
- Preserved model: one Mousemat LED; Direct, Spectrum Cycle, Breathing, Flashing, Wave, and Rainbow Wave modes with brightness `0..3`, inverse speed range `3..1`, and direction support
- Preserved output: exact 32-byte feature report with mode, brightness, speed, direction, RGB, and all native constant bytes; random Breathing and Flashing protocol variants remain selectable
- Verification: exact matcher rejection tests, direct/effect packet goldens, settings bounds, model-shape test, executable read-only probe and guarded mode command, workspace Clippy and tests
- Deleted native files: `AOCMousematController.cpp`, `AOCMousematController.h`, `AOCMousematControllerDetect.cpp`, `RGBController_AOCMousemat.cpp`, `RGBController_AOCMousemat.h`

Physical AOC AMM700 hardware was not present for this contraction. The family
remains release-blocked by the global hardware-evidence policy until a matching
device completes its live test.
