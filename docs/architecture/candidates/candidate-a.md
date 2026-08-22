# Candidate A: source-owned controller actors

Parity target: OpenRGB commit `8121ee29f46d58f90a56348eb5bf7a64f52f923b`.

This candidate treats OpenRGB's observable domain as the compatibility contract, not its C++ class graph. Local detection, every remote SDK connection, and every plugin instance remain distinct owners. They publish the same immutable controller view through one runtime facade; a controller actor serializes every mutation and owns the only writer for its transport.

## Usage sketch (written first)

The product surfaces share three operations: take a current snapshot, subscribe to semantic events, and submit an intent. They never receive a mutable controller, transport, SDK packet, plugin ABI object, or persistence DTO.

### Call site 1: eframe GUI and tray

```rust
fn run_gui(plan: LaunchPlan) -> anyhow::Result<()> {
    let running = Runtime::start(plan)?.wait_ready()?;
    let session = running.gui_session();

    eframe::run_native(
        "OpenRustyGB",
        eframe::NativeOptions::default(),
        Box::new(move |cc| Ok(Box::new(OpenRustyApp::new(cc, session)))),
    )?;

    // `run_native` is blocking. The process owner, not the egui App, owns shutdown.
    running.shutdown(ShutdownReason::GuiClosed)?.wait()?;
    Ok(())
}

impl eframe::App for OpenRustyApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        while let Some(event) = self.session.try_next_event() {
            self.model.reduce(event);             // UI-local state only
        }

        let system = self.session.latest();       // Arc<SystemSnapshot>
        render_controller_pages(ctx, &system, |controller, color| {
            let ticket = self.session.submit(Intent::Controller {
                target: controller.reference,
                command: ControllerCommand::SetColors {
                    selection: LedSelection::All,
                    colors: ColorRequest::Uniform(color),
                },
            })?;
            self.pending.insert(ticket.id(), ticket);
            Ok(())
        });
    }
}
```

The skin, schema renderer, localization, dynamic device pages, information view, and plugin panels consume `SystemSnapshot`; they do not coordinate detection, profile application, hardware access, or SDK addressing.

### Call site 2: CLI profile activation

```rust
async fn activate_profile(plan: LaunchPlan, name: &str) -> anyhow::Result<()> {
    let running = Runtime::start(plan).await_ready().await?;
    let receipt = running
        .cli_session()
        .execute(Intent::Profiles(ProfileIntent::Activate(
            ProfileName::parse(name)?,
        )))
        .await?;

    receipt.require_success()?; // completed only after all synchronous barriers
    running.shutdown(ShutdownReason::CliComplete).await?.wait().await?;
    Ok(())
}
```

Profile loading is one operation. The facade loads and validates the legacy JSON, calculates profile matches, notifies plugins, orders controller commands, and commits the active-profile event. The CLI does not call those stages itself.

### Call site 3: SDK server adapter

```rust
async fn serve_peer(stream: TcpStream, host: RuntimeHandle) -> Result<(), SdkError> {
    let peer = PeerIdentity::from_socket(&stream)?;
    let session = host.sdk_session(peer); // authority derived from the socket, never client data
    let mut connection = SdkConnection::accept(stream, SDK_LIMITS).await?;

    while let Some(frame) = connection.read_frame().await? {
        // Header, packet IDs, version layouts, and legacy indices remain codec-private.
        let request = connection.codec_mut().decode(frame, session.latest())?;
        let outcome = session.execute(request.intent).await;
        for reply in connection.codec_mut().encode(outcome, session.latest())? {
            connection.write_frame(reply).await?;
        }
    }
    Ok(())
}
```

`SdkConnection` adds real policy: the 16-byte `ORGB` header, the 8 MiB pre-allocation limit, v0 timeout negotiation, v0-v5 connection-local index resolution, v6 controller IDs and acknowledgements, feature flags, callback queue behavior, and loopback-only local-client authority. It is not a pass-through to the runtime.

## Data model and signatures

The sketches below are contracts. Bodies are intentionally absent.

```rust
pub struct Runtime;

impl Runtime {
    pub fn start(plan: LaunchPlan) -> Result<StartingRuntime, StartError> {
        todo!("validate the launch topology, construct source owners, and start the root supervisor")
    }
}

pub struct StartingRuntime { /* root task plus readiness receiver */ }
pub struct RunningRuntime { /* sole shutdown owner plus cloneable RuntimeHandle */ }
pub struct RuntimeHandle { /* bounded ingress and immutable snapshot receiver */ }

impl StartingRuntime {
    pub async fn await_ready(self) -> Result<RunningRuntime, StartError> {
        todo!("wait for the selected local/remote source to publish equivalent initial state")
    }
    pub fn wait_ready(self) -> Result<RunningRuntime, StartError> {
        todo!("blocking desktop wrapper around await_ready")
    }
}

impl RunningRuntime {
    pub fn gui_session(&self) -> Session { todo!() }
    pub fn cli_session(&self) -> Session { todo!() }
    pub fn handle(&self) -> RuntimeHandle { todo!() }
    pub fn shutdown(self, reason: ShutdownReason) -> Result<ShutdownTicket, ShutdownError> {
        todo!("begin the ordered, idempotent shutdown protocol")
    }
}

impl RuntimeHandle {
    pub(crate) fn sdk_session(&self, peer: PeerIdentity) -> Session {
        todo!("derive SessionAuthority; grant local-client privileges only to a loopback peer")
    }
    pub(crate) fn plugin_session(&self, instance: PluginInstanceId) -> Session {
        todo!("bind manifest capabilities and the plugin's resource scope")
    }
}

#[derive(Clone)]
pub struct Session { /* authority, bounded command sender, watch receiver, event cursor */ }

impl Session {
    pub fn latest(&self) -> Arc<SystemSnapshot> { todo!() }
    pub fn subscribe(&self, after: EventCursor) -> EventStream { todo!() }
    pub fn try_next_event(&mut self) -> Option<AppEvent> { todo!() }
    pub fn submit(&self, intent: Intent) -> Result<CommandTicket, SubmitError> {
        todo!("authorize, validate at ingress, and enqueue without blocking an egui frame")
    }
    pub async fn execute(&self, intent: Intent) -> Result<CommandReceipt, CommandError> {
        todo!("submit and await the operation's defined completion point")
    }
}
```

`LaunchPlan` makes the one-executable topology explicit without exposing task ordering:

```rust
pub struct LaunchPlan {
    pub frontend: Frontend,
    pub controller_source: ControllerSourcePlan,
    pub sdk_server: Option<SdkServerPlan>,
    pub platform: PlatformMode,
    pub paths: AppPaths,
}

pub enum Frontend {
    GuiAndTray,
    Cli(CliAction),
    Headless,
}

pub enum ControllerSourcePlan {
    StandaloneLocal,
    LocalSdkClient { endpoint: SdkEndpoint },
    RemoteSdkClient { endpoint: SdkEndpoint },
}

pub enum PlatformMode {
    Interactive,
    WindowsService,
}
```

Standalone local detection and local-client SDK mode both implement `ControllerSource`; each must emit the same `DetectionEvent` and controller-list semantics. `ControllerSourcePlan` chooses ownership, not a different consumer API.

### Stable, value-owned controller state

```rust
#[repr(i32)]
pub enum DeviceType {
    Motherboard = 0, Dram = 1, Gpu = 2, Cooler = 3, LedStrip = 4,
    Keyboard = 5, Mouse = 6, MouseMat = 7, Headset = 8, HeadsetStand = 9,
    Gamepad = 10, Light = 11, Speaker = 12, Virtual = 13, Storage = 14,
    Case = 15, Microphone = 16, Accessory = 17, Keypad = 18, Laptop = 19,
    Monitor = 20, Unknown = 21,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct ControllerId(NonZeroU64);       // runtime-stable, never reused
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Incarnation(NonZeroU32);        // prevents stale-handle/ABA use after reconnect
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct ControllerRef { pub id: ControllerId, pub incarnation: Incarnation }

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Rgb8 { pub r: u8, pub g: u8, pub b: u8 }
#[derive(Clone, Copy, Eq, PartialEq, Hash)] pub struct LedIndex(u32);
#[derive(Clone, Copy, Eq, PartialEq, Hash)] pub struct ZoneIndex(u32);
#[derive(Clone, Copy, Eq, PartialEq, Hash)] pub struct ModeIndex(u32);

pub struct SystemSnapshot {
    pub revision: StateRevision,
    pub detection: DetectionSnapshot,
    pub controllers: Arc<[Arc<ControllerSnapshot>]>, // canonical presentation order
    pub profiles: Arc<ProfilesSnapshot>,
    pub settings: Arc<SettingsSnapshot>,
    pub plugins: Arc<PluginsSnapshot>,
    pub server: Arc<ServerSnapshot>,
}

pub struct ControllerSnapshot {
    pub reference: ControllerRef,
    pub identity: DeviceIdentity,
    pub metadata: DisplayMetadata,
    pub device_type: DeviceType,
    pub capabilities: ControllerCapabilities,
    pub modes: Arc<[ModeView]>,
    pub active_mode: Option<ModeIndex>,
    pub zones: Arc<[ZoneView]>,
    pub leds: Arc<[LedView]>,               // exact upstream flat LED order
    pub colors: Arc<[Rgb8]>,                // same length and order as `leds`
    pub device_configuration: ConfigTree,
    pub health: ControllerHealth,
    pub revision: ControllerRevision,
}

pub struct ZoneView {
    pub name: Arc<str>,
    pub kind: ZoneType,
    pub leds: LedRange,                     // start + length; no self-referential pointer
    pub segments: Arc<[SegmentView]>,
    pub matrix: Option<MatrixMap>,           // value-owned indices into the flat LED array
    pub mode: Option<ModeIndex>,
    pub flags: ZoneFlags,                    // retains unknown compatibility bits
}

pub struct LedRange { pub start: LedIndex, pub len: u32 }
pub struct MatrixMap { pub width: u32, pub height: u32, pub cells: Arc<[Option<LedIndex>]> }
```

Constructors, not public fields, validate range bounds, `width * height`, mode limits, color counts, segment ranges, and matrix indices. Empty matrices are represented explicitly. Bitflags use `from_bits_retain` so a newer peer's unknown bits survive an SDK or persistence round trip. A compile-time discriminant test fixes every `DeviceType` number and asserts `Unknown` is last.

`DeviceIdentity` contains the observable vendor/name/description/version/serial/location fields plus a typed endpoint identity. The one profile matcher derives a different key:

```rust
impl DeviceIdentity {
    pub fn profile_match_key(&self) -> ProfileMatchKey {
        todo!("omit HID location and I2C bus number; retain the other stable fields and I2C address")
    }
}
```

No second module reimplements this omission rule.

### Semantic commands and completion

```rust
pub enum Intent {
    Controller { target: ControllerRef, command: ControllerCommand },
    Profiles(ProfileIntent),
    Settings(SettingsIntent),
    Plugins(PluginIntent),
    Rescan,
    Server(ServerIntent),
}

pub enum ControllerCommand {
    SetColors { selection: LedSelection, colors: ColorRequest },
    SetCustomMode,
    SetMode { scope: ModeScope, mode: ModeIndex, values: ModeValues },
    SaveMode { scope: ModeScope, mode: ModeIndex },
    ResizeZone { zone: ZoneIndex, size: u32 },
    ReplaceZoneConfiguration { zone: ZoneIndex, configuration: ZoneConfiguration },
    ClearSegments { zone: ZoneIndex },
    AddSegment { zone: ZoneIndex, segment: SegmentConfiguration },
    ReplaceDeviceConfiguration(DeviceConfiguration),
    SetDeviceSpecificConfiguration(ConfigTree),
    SetZoneSpecificConfiguration { zone: ZoneIndex, value: ConfigTree },
    SetHidden(bool),
}

pub enum LedSelection { All, Zone(ZoneIndex), One(LedIndex) }
pub enum ModeScope { Device, Zone(ZoneIndex) }
pub enum ColorRequest { Uniform(Rgb8), Exact(Arc<[Rgb8]>) }

pub struct CommandReceipt {
    pub operation: OperationId,
    pub accepted_at: OperationSequence,
    pub completion: Completion,
    pub resulting_revision: Option<StateRevision>,
}

pub enum Completion {
    AcceptedForCoalescedWrite,
    AppliedSynchronously,
    NoChange,
}
```

Command policy is derived privately from the command; callers cannot accidentally mark a save as coalescable. Whole-device LED and mode writes update semantic state in acceptance order and coalesce only while still pending for the same controller. Zone LED, single LED, zone mode, configuration, and save operations are barriers: the actor first drains the newest pending whole-device operation with a lower sequence, executes the barrier, and does not start a later operation before it completes. This gives old synchronous operations the same externally visible ordering without forcing egui or SDK code to manage a queue.

Every accepted mutation emits an `AppEvent::ControllerChanged` with the preserved numeric update reason. Callbacks are event subscriptions; events are copied into subscriber-owned queues and invoked/encoded outside registry or controller locks. A lagging in-process consumer receives `Lagged { current_revision }` and must refresh `latest()`. The SDK adapter separately reproduces the upstream bounded and coalescing callback behavior required by each protocol version.

## Ownership, concurrency, and shutdown

```text
RuntimeSupervisor
  |-- CommandRouter (authority, target resolution, operation sequencing; no I/O)
  |-- ControllerRegistry (sole aggregate snapshot writer)
  |-- EventHub (sole event-order owner)
  |-- PersistenceActor (sole filesystem writer)
  |-- LocalDetectionOwner ---- local ControllerActor(s) ---- one DeviceWriter each
  |-- RemoteSourceOwner(s) --- proxy ControllerActor(s) ---- one SDK connection owner
  `-- PluginSupervisor ------- virtual ControllerActor(s) -- one component store per plugin
```

Each actor owns its mutable state. `ControllerRegistry` merges source deltas only at the read boundary and publishes `Arc<SystemSnapshot>` through a watch channel. Sources cannot mutate the aggregate, and two sources cannot write the same controller. A source-scoped key plus `ControllerId` distinguishes identical devices from different servers. The router and registry never perform device, network, plugin, or filesystem I/O.

Every mailbox is bounded. Hardware operations have a command-class-aware queue: one replaceable pending whole-color command, one replaceable pending whole-mode command, and a bounded FIFO of non-coalescable barriers. Overflow rejects the new operation explicitly; it never silently drops save/configuration work. Network callback queues preserve the pinned upstream behavior. CPU/blocking platform APIs run in owned workers, never on egui or async executor threads.

### Exact retirement protocol

Hot-unplug and process shutdown use the same idempotent state machine:

1. The source changes the controller gate from `Accepting` to `Quiescing`; new commands get `ControllerUnavailable`.
2. The controller actor stops scheduling, cancels its writer, and joins it without holding a registry/event/source lock. The writer returns a dormant transport rather than dropping it.
3. Event subscriptions for that controller are closed; in-flight callback walks/RPC replies are counted and drained. No callback is invoked under a lock.
4. The registry publishes one removal snapshot/event and invalidates the incarnation.
5. The dormant driver and concrete transport are dropped.

Root shutdown first closes all external ingress (GUI submitter, SDK listeners/connections, plugin calls, hotplug/rescan), then retires controller owners concurrently, flushes the persistence actor through an atomic commit, stops platform integration, and finally joins the event hub/runtime. A second shutdown request returns the first `ShutdownTicket`; it cannot start a competing teardown. Suspend uses the same quiesce boundary but retains restartable source specifications; resume re-enumerates rather than trusting stale paths.

## Driver registry and the 197-family inventory

The port uses one family package per upstream controller-family directory, grouped into workspace shards only to control build time. Family ownership stays intact: matcher declarations, read-only probe, controller blueprint, packet serializer, protocol tests, and detector setting keys live together. A generated catalog is data, not dynamic initialization order.

```rust
pub trait DriverFamily: Send + Sync + 'static {
    fn manifest(&self) -> &'static DriverManifest;
    fn candidates(&self, inventory: &HardwareSnapshot) -> Vec<CandidateToken> {
        todo!("pure matching over an immutable, side-effect-free hardware inventory")
    }
    fn probe(&self, candidate: CandidateToken, io: &dyn ProbeIo)
        -> Result<Option<ControllerBlueprint>, ProbeError>
    {
        todo!("ProbeIo has no write method; reads must be declared in the manifest")
    }
    fn activate(&self, blueprint: ControllerBlueprint, lease: ValidatedDeviceLease)
        -> Result<Box<dyn ControllerDriver>, ActivateError>
    {
        todo!("called only for a semantic command or an explicitly allowed post-publication action")
    }
}

pub trait ControllerDriver: Send + 'static {
    fn descriptor(&self) -> &ControllerDescriptor;
    fn execute(
        &mut self,
        command: &ValidatedControllerCommand,
        io: &mut dyn DeviceIo,
    ) -> Result<DriverOutcome, DriverError> {
        todo!("the adapter validates capabilities and is the only layer that invokes its wire serializer")
    }
    fn shutdown(&mut self, io: &mut dyn DeviceIo) -> Result<(), DriverError> {
        todo!("no implicit persistent save/reset unless the family contract explicitly requires it")
    }
}

pub trait ProbeIo { /* enumeration and explicitly safe reads; deliberately no write */ }
pub trait DeviceIo { /* transport-specific exact read/write operations, not exported to consumers */ }
```

`drivers/catalog/*.toml` is the source of truth for all 197 families and all 224 pinned detector source files, including platform variants: stable `DetectorId` (the exact persisted settings key), supported platforms, transport, exact identifiers or bus claim, probe read permissions, upstream source files, test fixtures, and port status. Rust `xtask catalog` generates the static match index and fails on a missing upstream family/detector, duplicate stable ID, or ambiguous exact match.

Platform inventory collectors enumerate HID, USB, I2C, serial, PCI, and discoverable network endpoints without writing. Exact keys are indexed before predicate-based/bus-wide detectors. A claim arbiter prevents two detectors from opening the same endpoint or bus concurrently; an ambiguous highest-specificity match produces a diagnostic rather than choosing by link order. Manual devices enter the local owner as typed manual candidates and use the same registry path. Detector enablement is keyed by stable `DetectorId`, preserving `OpenRGB.json` settings.

## Exact HyperX Pulsefire Haste 2 design

The Haste 2 is a separate family, not an entry in the legacy Haste matcher. Its matcher fails closed unless every lighting-interface field is present and exact:

```rust
const HASTE2_MATCH: HidMatch = HidMatch {
    vendor_id: 0x03F0,
    product_id: 0x0B97,
    interface_number: 2,
    usage_page: 0xFF90,
    usage: 0xFF00,
};

pub struct PulsefireHaste2Protocol;

impl PulsefireHaste2Protocol {
    fn wheel_color(color: Rgb8) -> ExactOutputTransaction<2, 65> {
        // Frame 0: [44, 01, 01, 00, ...] exactly 65 bytes.
        // Frame 1: [44, 02, 00, 00, RR, GG, BB, 00, ...] exactly 65 bytes.
        // No feature report and no report borrowed from legacy Haste devices.
        todo!("construct the two zero-padded output reports")
    }
}
```

The family advertises `Mouse`, one zone named `Scroll Wheel`, one LED, and direct/uniform color only. Its command vocabulary can produce only `wheel_color`; there is no DPI, firmware, reset, save, profile-memory, polling-rate, lift-off-distance, feature-report, or arbitrary-report method to call. `SaveMode` and unsupported mode/configuration requests fail before activation.

Discovery only records the inventory token and publishes a dormant controller. It neither opens nor writes any interface. The first explicit direct-color command re-enumerates and revalidates VID, PID, interface 2, usage page, usage, and path before opening only that vendor endpoint. A profile replay or discovery hook is not an explicit first command for this device and therefore cannot activate it. Once activated, its one writer executes both 65-byte outputs as an indivisible transaction; each write must return exactly 65 or the transaction fails, and no other command can interleave. There is no keepalive. Cancellation joins the writer before the HID handle is dropped.

The live-device gate records:

- zero writes through discovery and idle time;
- the exact two output reports for several colors and propagated short-write/transport errors;
- unchanged pointer movement, primary/secondary/auxiliary buttons, wheel input, DPI state, and polling behavior before and after lighting writes;
- correct vendor-interface-only open behavior, unplug/reconnect, repeated commands, cancellation, and process teardown;
- no occurrence of `32 01 01` or any unapproved report in the full HID trace.

Input verification observes normal OS input events; the lighting driver never opens `MI_00` or `MI_01`.

## SDK v0-6 compatibility boundary

`orgb-sdk-codec` owns explicit little-endian readers/writers and private per-version DTOs; it never uses Rust struct layout as wire layout. It rejects a bad magic value, arithmetic overflow, and `pkt_size > 8 MiB` before allocating. A connection-scoped `VersionCodec` preserves v0's timeout negotiation and layouts for v0 through v6.

For v0-v5, `LegacyAddressBook` resolves device indices against the connection's defined ordered view. For v6, `V6AddressBook` assigns non-reused `u32` server IDs that survive list reordering until controller removal; it maps them to `ControllerRef` and emits acknowledgements/status codes with the exact packet ID and layout. Neither addressing scheme enters the domain model. Packet IDs, profile/settings/plugin commands, callbacks, feature flags, detection progress, server/client names, and update-reason values have byte-golden tests. Only a socket peer proven loopback can receive `LOCAL_CLIENT` and device-information/detection privileges; a claimed hostname or flag cannot grant them.

The Rust SDK client implements `ControllerSource`. Remote proxy actors translate domain commands into versioned packets and translate replies/callbacks back into snapshots/events. Therefore local-client and standalone modes remain behaviorally substitutable to GUI, CLI, profiles, and plugins.

## Persistence and profiles

`orgb-persistence` initially writes the pinned legacy formats rather than inventing a new canonical schema:

- `OpenRGB.json` for process settings and runtime-registered schemas;
- `Configuration.json` for names, geometry, and device-specific configuration;
- `profiles/*.json` for lighting, optional base color, and plugin data.

Wire DTOs and `serde_json::Value` stay private. Decoding produces validated domain values plus an `UnknownFields` shadow so unknown compatible fields round-trip. Settings schemas register under one owner and feed both validation and the egui schema renderer. Writes use temp-file, flush, atomic replace, and recoverable previous-file backup; transaction recovery is idempotent after a crash. A format version is added only with a bidirectional migration, golden legacy fixtures, unknown-field tests, profile-match fixtures, and rollback tests. Profile matching calls only `DeviceIdentity::profile_match_key()`.

## Rust-native plugin capability boundary

The host never loads a Qt/C++ `.dll`, `.so`, or `.dylib`, and there is no C ABI shim. Rust plugins compile to signed/package-hashed WebAssembly components and implement the versioned `openrustygb:plugin@1` component interface. Wasmtime, capability manifests, fuel/memory limits, scoped resources, and bounded calls keep plugin lifetime out of the core's address/lifetime model. Official components and the host are Rust-owned; the ABI remains language-neutral by construction.

The boundary passes domain values, not `egui`, Qt, SDK, JSON, or transport objects:

| Qt API v5 capability | Rust component v1 replacement |
| --- | --- |
| `QWidget*` tab | declarative `PanelTree` plus typed `UiAction`; egui owns rendering |
| `QMenu*` tray menu | declarative `MenuTree` plus action IDs |
| controller access/callbacks | scoped snapshots, semantic intents, and event streams |
| virtual controller pointers | host-owned `VirtualControllerLease`; resources vanish on plugin exit |
| profile callbacks/data | ordered about-to-load/load/save hooks and JSON-compatible `PluginValue` |
| settings JSON | namespaced schema/value registration; host validates and persists |
| SDK plugin command | manifest-declared command namespace with bounded request/response bytes |
| logging/rescan | capability-checked host operations |

API version and capabilities negotiate independently. Minor additions are feature-gated; an incompatible major version does not instantiate. Plugin UI state is declarative, so an egui redesign does not break plugin binaries.

Legacy migration is explicit:

1. Inventory each shipped/known Qt plugin against the capability table and preserve its settings/profile keys and SDK command IDs in a migration manifest.
2. `cargo xtask plugin migrate <manifest>` generates a Rust component scaffold and fixture tests; it does not execute or translate the binary.
3. Port source and verify settings/profile round trips, virtual-controller lifecycle, UI/tray actions, callbacks, and SDK command fixtures.
4. Package the component alongside (never inside) the host executable and mark the legacy binary superseded.
5. The Rust host may report discovered legacy filenames as incompatible, but never loads or starts them. There is no promise of binary compatibility; a third-party plugin requires a source port or remains on legacy OpenRGB.

No Qt bridge is shipped, and the final deletion gate forbids Qt/C++ linkage. This preserves plugin *capabilities* through a stable new boundary without falsely claiming that existing ABI v5 binaries work.

## Crate and module map

```text
openrustygb                  one binary; parses CLI and hands a LaunchPlan to orgb-runtime
orgb-domain                 identities, snapshots, commands, events, flags, pure validation
orgb-runtime                supervisor, command router, source merge, controller actors, shutdown
orgb-driver-api             internal family/driver/probe traits and typed transport leases
orgb-driver-catalog         generated static index and the 197-family inventory ledger
orgb-drivers/*              family-owned matchers, adapters, serializers, and golden tests
orgb-transport              HID/USB/I2C/serial/network implementations and side-effect audit wrapper
orgb-sdk-codec              private ORGB v0-6 wire DTOs/codecs/address books
orgb-sdk                     client source, server sessions, callback queues, feature/authority policy
orgb-persistence             exact legacy codecs, schemas, profile matching, atomic transactions
orgb-plugin-protocol        component interface types, manifest, declarative UI/menu model
orgb-plugin-host / sdk      Wasmtime supervision and Rust plugin author SDK
orgb-ui-egui                eframe app, schema views, controller pages, tray and localization adapters
orgb-platform-{win,linux,mac} power/hotplug/autostart/service/DBus/IOKit integration
orgb-testkit                 reference fixtures, fake transports, trace and differential harnesses
xtask                       Rust build, catalog, parity, package, sign, verify, and release orchestration
```

The runtime depends inward on domain and driver traits. Drivers depend on domain plus their transport capability, never runtime/UI/SDK. SDK, UI, persistence, and plugins are adapters around domain/runtime. Platform crates implement ports selected by `LaunchPlan`. C/C++ remains only as an unshipped frozen oracle while fixtures are captured; no Rust binary links it.

## Platform and release model

The root supervisor presents the same lifecycle to all platform ports. Windows owns service control and power broadcasts; Linux owns udev/hotplug, systemd/tmpfiles integration, and D-Bus policy; macOS owns IOKit notifications and autostart. Platform events translate to `Suspend`, `Resume`, `DeviceArrival`, `DeviceRemoval`, and `Stop` rather than leaking native handles into the domain.

`release-matrix.toml` is the single source for `cargo xtask dist` and CI:

| OS | Architectures | Artifacts |
| --- | --- | --- |
| Windows | x86, x86_64 | portable ZIP for both; x86_64 MSI |
| Linux AppImage | i386, amd64, armhf, arm64 | AppImage |
| Linux Debian | i386, amd64, armhf, arm64 | DEB |
| Linux RPM | x86_64 | RPM |
| Linux Flatpak | amd64, arm64 | Flatpak bundle/repository artifact |
| macOS | x86_64, arm64 | signed/notarized `.app` in ZIP |

Packaging metadata, udev rules, service files, D-Bus policy, tmpfiles config, desktop files, icons, and licenses are artifact inputs. Each artifact is installed in a clean target environment, runs domain/codec fixture tests appropriate to the target, launches the real GUI or headless mode, exercises loopback SDK control with a virtual controller, migrates a fixture configuration/profile, verifies service/autostart integration where applicable, and uninstalls cleanly. `--version` alone is not a release test. Checksums, SBOMs, signatures/notarization status, and the parity report accompany the release. Publication is blocked unless every required target passes; missing signing credentials produce an unreleased candidate, not a falsely complete release.

## Migration sequence and parity gates

The rewrite is vertical and independently runnable; it is never a mixed-language product.

1. Freeze a machine-readable surface ledger from commit `8121ee29`: controller fields/flags/reasons, all SDK packets by version, persistence fixtures, CLI modes, UI journeys, platform behavior, 197 family and 224 detector-source entries, and plugin capabilities.
2. Build domain/runtime/persistence foundations and the Haste 2 vertical slice with fake transports, GUI, CLI, and SDK paths.
3. Port driver families in transport/vendor batches. Each port replaces one red ledger entry and includes trace/golden tests.
4. Complete SDK v0-6, profiles/settings, every frontend/lifecycle mode, platform integration, and Rust component plugins.
5. Run the full parity/release matrix. Only then remove all C/C++/Qt/dependency source and frozen oracle binaries, rerun license/dependency/source scans, and publish installers.

Required gates are:

- **Inventory:** exactly 197 upstream family entries and 224 detector source files are mapped to implemented Rust entries; no `todo`, disabled default feature, or unowned surface may count green.
- **Domain:** fixture-equivalent device identity/metadata, enum numbers, flags, modes and their values, speed/brightness/direction/colors, zones/segments/matrices/LED ordering, callbacks/reasons, configuration, hidden state, and save semantics.
- **SDK:** byte-for-byte packets and error/ack timing for v0-6, malformed/fuzz corpus, 8 MiB enforcement, v0 negotiation timeout, v0-v5 indices, v6 IDs, callbacks/flags, loopback authority, and cross-tests against the pinned server and representative third-party clients.
- **Persistence:** read/write/differential fixtures for all three legacy stores, unknown fields, runtime schemas, active/base colors, plugin data, exact matching omissions, crash recovery, and reversible migration.
- **Drivers:** side-effect-free detection trace, match/claim conflicts, wire goldens, semantic command coverage, short/failed I/O, hotplug, suspend/resume, and ordered shutdown for every family.
- **Haste 2:** the exact live-device gate above is mandatory on VID `03F0`, PID `0B97`, interface 2; fixture-only success is insufficient.
- **Product:** GUI/tray, CLI, standalone/local/remote clients, server/headless, Windows service, settings/localization runtime changes, profiles/manual devices/info/plugins/rescan/hotplug/autostart/power journeys.
- **Plugins:** every capability has a component contract test and every plugin required for release has a verified Rust replacement; legacy binary compatibility remains explicitly unsupported.
- **Release:** every artifact in the matrix installs, performs a real virtual-controller + SDK + persistence exercise, and uninstalls; public hashes/assets/signatures are verified after upload.
- **Deletion:** repository-owned product source and linked dependency scans find no C, C++, Qt, moc, qmake, or native plugin ABI. The old tree can be deleted without changing a Rust crate's build or tests.

## Red-flag screen

| Red flag | Result |
| --- | --- |
| Shallow module | Pass. Consumers get `latest`, `subscribe`, and `submit/execute`; the facade hides authority, target resolution, profile orchestration, sequencing, source ownership, and I/O. The larger driver and codec traits are internal knowledge boundaries, not consumer APIs. |
| Information leakage | Pass. SDK layouts, JSON DTOs, HID bytes, Qt migration details, platform handles, and egui types terminate in their owning adapters. `DeviceIdentity::profile_match_key` and the catalog each provide a single source of truth for cross-cutting invariants. |
| Temporal decomposition | Pass. Crates are grouped by owned knowledge (domain, source, controller family, SDK protocol, persistence format, platform), not generic load/validate/transform/save phases. Persistence owns all phases of its format transaction; a family owns match through serialization. |
| Pass-through method | Pass with one watch item. `Session::execute` performs authority, validation, resolution, sequencing, completion, and error normalization. SDK/UI/plugin/platform boundaries translate and enforce policy. During implementation, any wrapper that merely forwards identical arguments must be removed; `RunningRuntime::handle` is a capability/lifetime downgrade, not another command hop. |

## One-page rationale

### Problem

The target is a behavior-preserving, all-Rust replacement for a pinned OpenRGB snapshot whose controllers are owned by local detection, remote clients, or plugins; whose controller mutation and teardown rules were tightened specifically to avoid races and deadlocks; and whose public surface spans mutable hardware, SDK v0-6, legacy JSON, Qt plugin capabilities, multiple process modes, and three OS families. A class-for-class translation would preserve accidental pointer/lock structure while still failing Qt ABI compatibility. The design must instead preserve identity, ordering, wire/storage formats, ownership, callbacks, and lifecycle, add side-effect-free discovery, scale the proof across 197 families, and make the exact Haste 2 lighting endpoint incapable of issuing unrelated mouse commands.

### Usage (caller's view)

GUI, CLI, SDK, and plugins all read an immutable `SystemSnapshot`, subscribe to semantic `AppEvent`s, and submit one domain `Intent`; the three call sites at the top are the contract. Local and remote source choice does not alter that contract. A caller identifies a controller with a generation-checked `ControllerRef`, never a pointer or SDK index, and receives an operation receipt whose completion semantics are defined by the command.

### Shape

Candidate A uses source-owned actors feeding a single snapshot registry, with one actor and one serialized writer per controller. This encodes exclusive mutation and stale-reference rejection in ownership/types, per `separate-before-serializing-shared-state` and `encode-lessons-in-structure`. Commands are validated at ingress and against the current snapshot, while SDK/JSON/HID/platform representations remain private to adapters, per `boundary-discipline`. Whole writes coalesce behind barriers by policy derived from command variants, and shutdown is one idempotent state machine, per `make-operations-idempotent`. The consumer interface is deliberately small but deep: it hides source equivalence, authority, matching, orchestration, coalescing, callback safety, and I/O while exposing only real domain choices. Family packages own matching through serialization, so 197 ports add rows to a generated inventory rather than branches to a central manager. The host deliberately does not load Qt binaries; versioned components preserve capabilities through host-owned virtual resources and declarative egui surfaces.

### Synthesis decision

Candidate A selects distributed controller ownership plus a merged immutable read model as its base because it directly preserves the upstream ownership history and isolates slow/failing hardware without exposing concurrency. It incorporates a central command router only for authority, stable resolution, and operation sequencing; the router performs no I/O and is not a global controller owner. It rejects ABI emulation and shared mutable controller objects. Arena synthesis may graft a stronger inventory or migration mechanism from another candidate without weakening these ownership and boundary invariants.

### Tradeoffs accepted

- We accept actor/mailbox machinery in exchange for explicit ownership, bounded backpressure, per-controller isolation, and joinable teardown.
- We accept immutable snapshot cloning with structural sharing in exchange for lock-free consumer reads and safe callback lifetimes.
- We accept a source port requirement for Qt plugins in exchange for eliminating Qt/C++ and stabilizing UI/resource ownership.
- We accept generated catalog metadata and parity ledgers in exchange for proving that 197 families and 224 detector source variants are complete rather than relying on link-time registration.
- We accept delayed activation for the Haste 2's first user color command in exchange for zero writes on discovery and an auditable absence of DPI/firmware reports.
- We accept maintaining exact legacy SDK/JSON codecs at the boundary in exchange for preserving existing clients and user data while keeping the Rust domain clean.

### Alternatives considered

- A global `RwLock<Vec<Box<dyn Controller>>>` plus callbacks most closely mirrors C++, but it is shallow: callers inherit locks, pointer lifetime, index invalidation, and teardown ordering while the registry hides little.
- One global reducer with a generic effect queue offers a similarly small consumer API, but a single I/O owner couples unrelated devices and makes synchronous barriers, unplug, and transport-specific cancellation head-of-line concerns; the per-controller actor hides more failure and ordering complexity locally.
- OS processes for every controller and plugin maximize fault isolation, but expose serialization, deployment, supervision, and distributed transactions across ordinary controller calls. Components only at the untrusted plugin boundary retain most isolation with a much deeper in-process domain API.
- An out-of-process Qt compatibility bridge could run existing plugins, but keeps Qt/C++ in the delivered trust/lifecycle surface and cannot safely translate `QWidget*`, `QMenu*`, and controller pointers. Source migration is the only viable all-Rust shape.

### Open questions and risks

- Which third-party Qt plugins are release-blocking, and who owns their source ports when a repository is unavailable?
- Which legacy devices currently require a write during detection, and should they become explicitly activated/manual rather than violate the new global safety rule?
- Which SDK v0 timeout and callback-timing behaviors are depended on by real third-party clients beyond the pinned tests?
- Can all release targets support the selected pure-Rust transport crates without hidden C/C++ build dependencies, especially Linux i386/armhf and macOS IOKit?
- What signing identities and notarization secrets will be available to turn verified candidates into public installers?
- Should a Haste 2 remain dormant after every reconnect, or may an in-session explicit activation authorize subsequent reconnect restoration without becoming an automatic discovery write?

### Next implementation step

Build the domain fixture corpus and `orgb-domain` constructors, then prove the end-to-end Haste 2 vertical slice against an audited fake HID transport before any live write.
