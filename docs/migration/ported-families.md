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

## GigabyteAorusMouseController

- Rust package: `openrustygb-driver-gigabyte-aorus-m2`
- Device: Gigabyte Aorus M2 mouse
- Exact match: VID `1044`, PID `7A40`, interface `3`, usage page `FF01`, usage `0001`
- Preserved model: one Mouse LED; Direct, Static, Breathing, Spectrum Cycle, Flashing, Double Flash, and Off modes; brightness `0..100`; inverse speed range `22..0` with default `11`
- Preserved output: exact eight-byte `CD` direct-color and `CC` hardware-mode feature reports
- Preserved behavior: Direct mode first sends the native Static hardware packet to apply brightness and then sends the Direct RGB packet; Off emits Static with black and zero brightness
- Verification: exact matcher rejection tests, direct and mode packet goldens, Direct brightness and Off tests, bounds and model-shape tests, executable read-only probe and guarded mode command, workspace Clippy and tests
- Deleted native files: `GigabyteAorusMouseController.cpp`, `GigabyteAorusMouseController.h`, `GigabyteAorusMouseControllerDetect.cpp`, `RGBController_GigabyteAorusMouse.cpp`, `RGBController_GigabyteAorusMouse.h`

Physical Gigabyte Aorus M2 hardware was not present for this contraction. The
family remains release-blocked by the global hardware-evidence policy until a
matching device completes its live test.

## DarkProject

- Rust package: `openrustygb-driver-dark-project-kd3b-v2`
- Device: Dark Project KD3B V2 keyboard
- Exact match: VID `195D`, PID `2061`, interface `2`, usage page `FFC2`, usage `0004`
- Preserved model: 87-key ANSI keyboard, one Keyboard matrix zone with the native 6x18 map, and Direct mode value `1`
- Preserved output: exact two 256-byte reports with headers `08 07 00 00 00` and `08 07 00 01 00`; the native 87-entry packet map places red and green in the first report and blue in the second
- Safety correction: each transaction requires all 87 key colors, preserving a complete frame instead of reproducing the native single-LED method's incorrect remap to the first packet slot
- Verification: exact matcher rejection tests, dual-report packet and channel-split goldens, color-count, matrix-map and model-shape tests, executable read-only probe and guarded per-key command, workspace Clippy and tests
- Deleted native files: `DarkProjectControllerDetect.cpp`, `DarkProjectKeyboardController.cpp`, `DarkProjectKeyboardController.h`, `RGBController_DarkProjectKeyboard.cpp`, `RGBController_DarkProjectKeyboard.h`

Physical Dark Project KD3B V2 hardware was not present for this contraction.
The family remains release-blocked by the global hardware-evidence policy until
a matching device completes its live test.

## StreamDeckController

- Rust package: `openrustygb-driver-elgato-stream-deck-mk2`
- Device: Elgato Stream Deck MK.2
- Match: VID `0FD9`, PID `0080`, interface `0`; the native detector does not constrain HID usage
- Preserved model: 15 buttons in one 3x5 Button Matrix zone with Direct per-button color mode value `0`
- Preserved output: each button color is encoded as a 72x72 RGB JPEG at quality 95 and placed in a zero-padded 1024-byte `02 07` report with the button index and little-endian JPEG length
- Preserved feature reports: 32-byte `03 08` brightness and `03 02` reset reports
- Safety correction: oversized JPEGs are rejected instead of advertising the full length while silently truncating the payload; all 15 output reports accept either the full buffer or Windows' report-ID-excluded completion length and reject anything shorter
- Verification: product/interface matcher tests, JPEG framing and padding tests, complete frame-order test, exact feature-report tests, matrix and model-shape tests, executable read-only probe and guarded per-button command, workspace Clippy and tests
- Deleted native files: `ElgatoStreamDeckController.cpp`, `ElgatoStreamDeckController.h`, `ElgatoStreamDeckControllerDetect.cpp`, `RGBController_ElgatoStreamDeck.cpp`, `RGBController_ElgatoStreamDeck.h`

Physical Elgato Stream Deck MK.2 hardware was not present for this contraction.
The family remains release-blocked by the global hardware-evidence policy until
a matching device completes its live test.

## AresonController

- Rust package: `openrustygb-driver-areson-mice`
- Devices: ZET GAMING Edge Air Pro and Elit wired/wireless variants, plus Redragon M914 NIX wired/wireless
- Exact matches: VID `25A7`; PIDs `FA3F`, `FA40`, `FA48`, `FA49`, `FA7B`, and `FA7C`; interface `1`; usage `FF02:0002`
- Preserved model: one Mouse LED; Static, Rainbow Wave, Breathing, Spectrum Cycle, Single Color Wave, Colorful Breathing, and Off modes; brightness and speed ranges `1..10`
- Preserved output: exact 17-byte `08 07` feature report, mode-specific high/low speed tables, ten-level brightness table, wrapping `55` checksum across mode through brightness, and `4A` terminator
- Preserved mode behavior: modes without user colors encode black, Static ignores speed, and Off encodes black with zero speed and brightness
- Safety correction: mode-dependent settings are validated before indexing the native one-based tables
- Verification: all six exact matcher tests, Static/Rainbow/Off packet goldens, speed and brightness bounds, model-shape tests, executable read-only probe and guarded hardware-mode command, workspace Clippy and tests
- Deleted native files: `AresonController.cpp`, `AresonController.h`, `AresonControllerDetect.cpp`, `RGBController_Areson.cpp`, `RGBController_Areson.h`

Physical supported Areson mouse hardware was not present for this contraction.
The family remains release-blocked by the global hardware-evidence policy until
a matching device completes its live test.

## SayoDeviceController

- Rust package: `openrustygb-driver-sayodevice-e1`
- Device: SayoDevice E1 knob
- Match: VID `8089`, PID `0007`, usage `FF11:0002`; the native detector does not constrain interface
- Preserved model: one Underglow LED; Direct, Breathing, Wave, Switch, and Blink modes; animation speed `0..3`; optional loop-table random color; explicit manual save
- Preserved output: exact zero-padded 64-byte `21 12` reports, little-endian 16-bit word-sum checksum, 27-byte lighting payload, packed inverse speed/color/mode byte, and six-byte persistent-save payload
- Preserved lifecycle: every output accepts the Windows report-ID-excluded completion length, rejects shorter writes, and then non-blockingly drains all pending 64-byte input reports
- Safety correction: the Rust model exposes the intended single LED instead of reproducing the native setup loop's duplicate LED entry; persistent save requires a separate explicit confirmation flag
- Verification: product/usage matcher tests, Direct/Breathing/save packet goldens, checksum, speed inversion and bounds tests, pending-input drain test, model-shape test, executable read-only probe and guarded mode/save commands, workspace Clippy and tests
- Deleted native files: `SayoDeviceController.cpp`, `SayoDeviceController.h`, `SayoDeviceControllerDetect.cpp`, `RGBController_SayoDevice.cpp`, `RGBController_SayoDevice.h`

Physical SayoDevice E1 hardware was not present for this contraction. The
family remains release-blocked by the global hardware-evidence policy until a
matching device completes its live test.

## WushiController

- Rust package: `openrustygb-driver-wushi-l50`
- Device: JSAUX RGB Docking Station / Wushi L50
- Match: VID `306F`, PID `1234`; the native detector does not constrain interface or usage
- Preserved model: four Dock zones; Direct, Breathing, Rainbow Wave, Spectrum Cycle, Race Cycle, and Stacking modes; brightness `1..2`, speed `1..4`, and left/right direction where supported
- Preserved output: zero-padded 65-byte feature report with command `16`, effect, speed, brightness, four RGB triplets, and direction flags
- Preserved platform framing: Windows prepends report ID `CC` and shifts the command by one byte; non-Windows platforms place command `16` at byte zero
- Preserved mode defaults: Breathing uses its selected color in zone 1 and native white defaults elsewhere; firmware-generated effects use white defaults; unsupported speed, brightness, and direction controls are ignored exactly by mode
- Safety correction: mode-dependent speed and brightness values are validated before report construction
- Verification: product-only matcher tests, platform-specific Direct packet tests, effect/default/direction packet tests, settings bounds and model-shape tests, executable read-only probe and guarded hardware-mode command, workspace Clippy and tests
- Deleted native files: `WushiL50USBController.cpp`, `WushiL50USBController.h`, `WushiL50USBDetect.cpp`, `RGBController_WushiL50USB.cpp`, `RGBController_WushiL50USB.h`

Physical JSAUX RGB Docking Station hardware was not present for this
contraction. The family remains release-blocked by the global hardware-evidence
policy until a matching device completes its live test.

## RedragonController

- Rust package: `openrustygb-driver-redragon-mice`
- Devices: Redragon M711 Cobra, M715 Dagger, M716 Inquisitor, M908 Impact, M602 Griffin, M808 Storm, M801 Sniper, M810 Taipan, M987 Reaping, M921 Azzinoth, and M711-FPS-1 Cobra FPS
- Exact matches: VID `04D9`; PIDs `FC30`, `FC39`, `FC3A`, `FC4D`, `FC38`, `FC5F`, `FC58`, `FA7E`, `FC69`, `FC40`, and `FC62`; interface `2`; usage page `FFA0`; the native detector does not constrain usage
- Preserved model: one Mouse LED; Static, Wave, Breathing, Rainbow, and Flashing modes, plus the native random-color Breathing variant
- Preserved lifecycle: opening a device selects profile zero at address `002C` and applies it before accepting lighting commands
- Preserved output: exact zero-padded 16-byte feature reports with report ID `02`; little-endian addresses; color/mode data at byte eight; and the `F1 02 04` apply report after each change
- Preserved mode behavior: hardware effects encode the native always-on flag and speed zero; Rainbow exposes no user color while the remaining displayed effects retain per-LED color
- Verification: all eleven product/interface/usage-page matcher tests, initialization and apply packet goldens, color and hardware-mode packet goldens, model-shape tests, executable read-only probe and guarded hardware-mode command, workspace Clippy and tests
- Deleted native files: `RedragonControllerDetect.cpp`, `RedragonMouseController.cpp`, `RedragonMouseController.h`, `RGBController_RedragonMouse.cpp`, `RGBController_RedragonMouse.h`

Physical supported Redragon mouse hardware was not present for this contraction.
The family remains release-blocked by the global hardware-evidence policy until
a matching device completes its live test.

## GigabyteAorusPCCaseController

- Rust package: `openrustygb-driver-gigabyte-aorus-c300-glass`
- Device: Gigabyte AORUS C300 GLASS
- Match: VID `1044`, PID `7A30`, interface `0`, usage `FF01:0001`
- Preserved model: one Case LED; Custom, Off, Breathing, Spectrum Cycle, Flashing, and Double Flashing modes; brightness `0..9`; inverse speed range `10..6`
- Preserved output: three exact 9-byte feature reports in native order—`C8` color, `C9` hardware mode, and `B6` commit
- Preserved mode normalization: Custom forces normal speed `9`; Off sends black using Custom mode with brightness and speed `10`; Breathing forces maximum brightness `9`; Spectrum Cycle forces red and maximum brightness; flashing modes scale brightness by ten
- Safety correction: mode-dependent brightness and speed values are validated before packet construction
- Verification: exact endpoint matcher tests, Custom/Off/Spectrum/Double Flashing packet goldens, three-report ordering test, settings bounds and model-shape tests, executable read-only probe and guarded hardware-mode command, workspace Clippy and tests
- Deleted native files: `GigabyteAorusPCCaseController.cpp`, `GigabyteAorusPCCaseController.h`, `GigabyteAorusPCCaseControllerDetect.cpp`, `RGBController_GigabyteAorusPCCase.cpp`, `RGBController_GigabyteAorusPCCase.h`

Physical Gigabyte AORUS C300 GLASS hardware was not present for this contraction.
The family remains release-blocked by the global hardware-evidence policy until
a matching device completes its live test.

## ClevoLightbarController

- Rust package: `openrustygb-driver-clevo-lightbar`
- Device: CLEVO laptop lightbar using ITE 8291 revision 0.03
- Match: VID `048D`, PID `7001`, usage `FF03:0002`; the native detector does not constrain interface
- Preserved model: one Lightbar LED; Direct, Breathing, Wave, Bounce, Marquee, Scan, and Off modes; brightness `0..100`; UI speed `1..10`
- Preserved output: exact 8-byte feature reports for color, mono brightness, hardware mode, and the four-report device-specific off sequence
- Preserved behavior: color reports precede color-capable effects; Wave and Marquee send no color; UI speed is inverted to the device's `1=fastest, 10=slowest` encoding; Direct preserves the native default speed byte `11`
- Preserved metadata: shared Rust HID inventory now carries the USB release number and formats firmware as native `major.minor` text such as `3.07`
- Safety correction: mode-dependent brightness and speed values are validated before packet construction
- Verification: product/usage matcher tests across arbitrary interfaces, Direct/Wave/off/brightness packet goldens, inverse-speed and bounds tests, firmware and model-shape tests, executable read-only probe and guarded hardware-mode command, workspace Clippy and tests
- Deleted native files: `ClevoLightbarController.cpp`, `ClevoLightbarController.h`, `ClevoLightbarControllerDetect.cpp`, `RGBController_ClevoLightbar.cpp`, `RGBController_ClevoLightbar.h`

Physical CLEVO Lightbar hardware was not present for this contraction. The
family remains release-blocked by the global hardware-evidence policy until a
matching device completes its live test.

## SkydimoHIDController

- Rust package: `openrustygb-driver-skydimo-sk0902`
- Device: Skydimo SK0902 headset stand
- Match: VID `1A86`, PID `E316`, interface `0`, usage `FF00:0001`
- Preserved model: 49 individually addressable LEDs in the native serpentine 7x7 Matrix map with one Direct mode
- Preserved metadata: manufacturer and product strings form the displayed device name, with the same Skydimo/HID Device fallbacks
- Preserved output: RGB input is converted to GRB, split into 60/60/27-byte chunks starting at LEDs `0`, `20`, and `40`, and sent as exact 64-byte output reports
- Preserved integrity and commit: each data report carries polynomial-`07` CRC-8 over bytes `0..62`; the final `01 FF FF 31` report carries marker `1E` at byte 60 and intentionally no CRC
- Safety correction: frame construction requires the controller's exact 49-color topology instead of silently truncating or padding an invalid caller frame; every write is checked for its exact native length
- Verification: exact endpoint matcher tests, GRB/chunk/start-index/CRC/frame-end packet goldens, serpentine matrix and model-shape tests, executable read-only probe and guarded 49-color command, workspace Clippy and tests
- Deleted native files: `SkydimoHIDController.cpp`, `SkydimoHIDController.h`, `SkydimoHIDControllerDetect.cpp`, `RGBController_SkydimoHID.cpp`, `RGBController_SkydimoHID.h`

Physical Skydimo SK0902 hardware was not present for this contraction. The
family remains release-blocked by the global hardware-evidence policy until a
matching device completes its live test.

## EKController

- Rust package: `openrustygb-driver-ek-loop-connect`
- Device: EK Loop Connect
- Match: VID `0483`, PID `5750`, interface `0`, usage `FFA0:0001`
- Preserved model: one Loop Connect LED; Static, Breathing, Fading, Marquee, Covering Marquee, Pulse, Spectrum_Wave, Alternating, and Candle modes; speed `0..8`
- Preserved metadata: HID manufacturer and product strings form the displayed name, with `EK Loop Connect` as the fallback when either string is unavailable
- Preserved output: exact 63-byte HID output report built from the native per-mode 16-byte templates, RGB at bytes `16..18`, speed at byte `14`, command override `10` at byte `10`, and trailer `FF 00` at bytes `47..48`
- Preserved speed encoding: animated modes use the native lookup table `00,0C,19,25,32,3E,4B,57,64`; Static ignores speed and encodes zero
- Safety correction: animated speeds are validated before indexing the native table, RGB state is deterministic, and short HID writes fail instead of being silently ignored
- Verification: exact endpoint matcher tests, Static and Candle packet goldens, speed bounds and model-shape tests, executable read-only probe and guarded hardware-mode command, workspace Clippy and tests
- Deleted native files: `EKController.cpp`, `EKController.h`, `EKControllerDetect.cpp`, `RGBController_EKController.cpp`, `RGBController_EKController.h`

Physical EK Loop Connect hardware was not present for this contraction. The
family remains release-blocked by the global hardware-evidence policy until a
matching device completes its live test.

## LuxaforController

- Rust package: `openrustygb-driver-luxafor-flag`
- Device: Luxafor Flag
- Match: VID `04D8`, PID `F372`; the native detector does not constrain interface or usage
- Preserved model: Flag and Rear zones with three LEDs each; Direct plus Traffic Lights, Pattern 2, Pattern 3, Pattern 4, Police, Pattern 6, Pattern 7, and Pattern 8
- Preserved output: exact 9-byte HID reports with physical LED indices `1..6`; Direct uses mode `1`, fixed changing time `100`, and one packet per LED; patterns use mode `6`, pattern ID `1..8`, and repeat `255`
- Preserved dormant protocol: Rust packet constructors retain the native Fade, Strobe, and Wave formats even though OpenRGB currently leaves those modes hidden
- Preserved pseudo-modes: pattern description values remain `6 + (pattern << 8)`, matching the native controller's combined mode/type representation
- Safety correction: Direct commands require all six colors and every HID write must accept the complete native report
- Verification: product-only matcher tests, all five packet-format goldens, Direct physical-order and color-count tests, pseudo-mode and model-shape tests, executable read-only probe and guarded Direct/pattern commands, workspace Clippy and tests
- Deleted native files: `LuxaforController.cpp`, `LuxaforController.h`, `LuxaforControllerDetect.cpp`, `RGBController_Luxafor.cpp`, `RGBController_Luxafor.h`

Physical Luxafor Flag hardware was not present for this contraction. The family
remains release-blocked by the global hardware-evidence policy until a matching
device completes its live test.

## AOCMouseController

- Rust package: `openrustygb-driver-aoc-gm500-mouse`
- Device: AOC GM500 mouse
- Match: VID `3938`, PID `1179`, interface `1`, usage `FF19:FF19`
- Preserved model: Logo and Scroll Wheel single-LED zones; Direct, Spectrum Cycle, Breathing, Flashing, Wave, Rainbow Wave, and DPI modes
- Preserved mode variants: random-color Breathing and Flashing retain their native `80`-bit protocol variants even though upstream exposes them as a color behavior of the same displayed mode
- Preserved output: exact 60-byte feature report, including both RGB triplets at bytes `51..56`, the native fixed fields and trailers, brightness `0..3`, reverse-ordered speed `3..1`, and clockwise/counter-clockwise direction
- Preserved Direct behavior: Direct always uses native Static mode with high brightness, medium speed, and clockwise direction, matching `SendDirect`
- Safety correction: mode settings are validated before report construction and writes are isolated to the exact lighting endpoint
- Verification: exact endpoint matcher tests, Direct and random-mode packet goldens, brightness/speed bounds, DPI no-speed behavior, model-shape tests, executable read-only probe and guarded hardware-mode command, workspace Clippy and tests
- Deleted native files: `AOCMouseController.cpp`, `AOCMouseController.h`, `AOCMouseControllerDetect.cpp`, `RGBController_AOCMouse.cpp`, `RGBController_AOCMouse.h`

Physical AOC GM500 hardware was not present for this contraction. The family
remains release-blocked by the global hardware-evidence policy until a matching
device completes its live test.

## InstantMouseController

- Rust package: `openrustygb-driver-instant-mice`
- Devices: Advanced GTA 250, Anko KM43243952, Anko KM43277483, and AntEsports GM600 USB gaming mice
- Match: VID `30FA`; PIDs `1030`, `1440`, `1540`, and `1040`; interface `1`; usage `FF01:0001`
- Preserved model: one Mouse LED; Direct, Rainbow wave, Spectrum cycle, Breathing, Fill, Loop, and Off on all models; Enrpatured, Flicker, Ripple, and Star treck on the AntEsports GM600
- Preserved model quirk: the AntEsports GM600 uses breathing ID `09`; the other models use `08`
- Preserved output: exact 8-byte mode feature report plus six color feature reports for DPI slots `0,2,4,6,8,10`; 8-bit RGB is quantized and inverted into the native 4-bit channel encoding
- Preserved Off behavior: first select Direct mode with zero brightness, then send the native encoded black color to all six slots
- Safety correction: unsupported Ant-only modes and settings outside speed `0..5` or brightness `0..7` are rejected; inputs for fields not exposed by a mode are normalized to the upstream defaults
- Verification: all four endpoint matchers, color quantization and DPI-slot packet goldens, Direct/Breathing/Off sequence goldens, Ant mode isolation and model-shape tests, executable read-only probe and guarded mode command, workspace Clippy and tests
- Deleted native files: `InstantMouseController.cpp`, `InstantMouseController.h`, `InstantMouseControllerDetect.cpp`, `InstantMouseDevices.h`, `RGBController_InstantMouse.cpp`, `RGBController_InstantMouse.h`

Physical supported Instant mouse hardware was not present for this contraction.
The family remains release-blocked by the global hardware-evidence policy until
a matching device completes its live test.

## LaviewTechnologyController

- Rust package: `openrustygb-driver-glorious-model-i`
- Device: wired Glorious Model I mouse
- Match: VID `22D4`, PID `1503`, interface `1`, usage `FF01:0002`
- Preserved model: one Mouse LED; Custom, Flashing, Chase, Wave, Spectrum Cycle, Breathing, Spectrum Breathing, Rainbow Wave, and Off modes
- Preserved output: exact 64-byte `A1 0C` profile feature report; mode at byte `16`, brightness at `56`, speed at `58`, and the native 21-byte factory palette for hardware effects
- Preserved color behavior: Custom and Breathing place the selected RGB value at bytes `17..19`; other modes retain the native palette needed for effects omitted by the official control application
- Preserved metadata: HID manufacturer, serial number, and decimal USB release number remain the vendor, serial, and firmware surfaces
- Safety correction: public brightness `0..100` and animated speed `1..100` are validated, then safely saturated at the protocol maximum `64`; this replaces the native reversed `std::clamp` arguments and their undefined precondition
- Verification: exact endpoint matcher tests, Custom/Breathing color packet goldens, palette/effect packet golden, setting saturation and bounds tests, Off packet, firmware and model-shape tests, executable read-only probe and guarded mode command, workspace Clippy and tests
- Deleted native files: `LaviewTechnologyDetector.cpp`, `LaviewTechnologyController.cpp`, `LaviewTechnologyController.h`, `RGBController_LaviewTechnology.cpp`, `RGBController_LaviewTechnology.h`

Physical Glorious Model I hardware was not present for this contraction. The
family remains release-blocked by the global hardware-evidence policy until a
matching device completes its live test.
