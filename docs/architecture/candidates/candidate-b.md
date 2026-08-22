# Candidate B — actor-owned controllers with capability-projected ports

**Scope.** This is a design sketch for a complete Rust replacement of upstream
commit 8121ee29f46d58f90a56348eb5bf7a64f52f923b. It deliberately specifies no
C++ or Qt runtime dependency. It preserves upstream-compatible semantics at
the product boundaries, not its class hierarchy.

## Usage (caller's view, written first)

The product has one runtime and gives each caller a small, capability-bound
port. A caller can inspect immutable state, subscribe to events, and submit an
intent; it never holds a mutable controller, a HID handle, or an SDK packet.

### 1. The eframe controller page

~~~rust
use openrustygb_core::{AppIntent, ColorTarget, ControllerOperation, Rgb8};
use openrustygb_ui::EframeBridge;

fn wheel_color_page(bridge: &EframeBridge, controller: ControllerId, rgb: Rgb8) {
    // This only queues a semantic intent. Eframe cannot open or write a device.
    bridge.submit(AppIntent::controller(
        controller,
        ControllerOperation::set_colors(ColorTarget::whole_device(), vec![rgb]),
    ));
}

fn update(&mut self, ctx: &egui::Context) {
    for event in self.bridge.drain_events() {
        self.view.apply(event); // immutable app projection; request_repaint is internal
    }
    render_controller(&mut self.view, ctx, &self.bridge);
}
~~~

The page presents a mode, zone, LED, matrix, or direct-color control only when
that capability exists in its ControllerSnapshot. For the exact Haste 2
adapter, it renders one scroll-wheel direct-color control and no DPI, save,
firmware, or polling controls.

### 2. A CLI profile command

~~~rust
use openrustygb_core::{AppIntent, ProfileRef, Runtime, RunMode};

async fn activate_night_profile(data_dir: DataDir) -> anyhow::Result<()> {
    let runtime = Runtime::connect_or_start(RunMode::Cli, data_dir).await?;
    let reply = runtime.cli_port()
        .execute(AppIntent::apply_profile(ProfileRef::named("Night")?))
        .await?;

    println!("{}", reply.human_summary());
    runtime.shutdown().await?;
    Ok(())
}
~~~

The port resolves matching devices, applies the profile with each controller's
documented write ordering, and reports skipped or unsafe targets. The CLI
neither edits JSON itself nor invokes a driver directly.

### 3. An explicit HyperX Haste 2 lighting action

~~~rust
use openrustygb_core::{AppIntent, ControllerSelector, ControllerOperation, Rgb8};

async fn set_haste_wheel_red(port: &ControlPort) -> Result<IntentResult, AppError> {
    let selector = ControllerSelector::usb(0x03F0, 0x0B97)
        .on_hid_interface(2)
        .vendor_usage(0xFF90, 0xFF00);
    let haste = port.view().await?.one(selector)?;

    // The only supported mutation is a direct color for the wheel LED.
    port.execute(AppIntent::controller(
        haste.id,
        ControllerOperation::set_colors(ColorTarget::whole_device(), vec![Rgb8::RED]),
    )).await
}
~~~

This call travels through the same validation, serialized writer, telemetry,
and teardown path as every other controller. There is no public API by which a
screen, CLI, SDK client, or plugin can send an arbitrary HID report.

## Shape

### Load-bearing domain types

All public types below are Rust domain values. Values carrying socket framing,
JSON layout details, HID report bytes, Qt values, OS handles, and plugin
component plumbing stay private to their adapter crates.

~~~rust
// crates/domain/src/controller.rs
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ControllerId(pub uuid::Uuid);

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceType {
    // Generated from the pinned upstream numeric table. Every existing
    // discriminant is literal and tested; Unknown is last.
    Motherboard = 0,
    // ...
    Unknown = UPSTREAM_UNKNOWN_DEVICE_TYPE,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControllerSnapshot {
    pub id: ControllerId,
    pub origin: ControllerOrigin,
    pub identity: PublicIdentity,
    pub metadata: ControllerMetadata,
    pub device_type: DeviceType,
    pub flags: ControllerFlags,
    pub topology: LightingTopology,
    pub state: LightingState,
    pub capabilities: ControllerCapabilities,
    pub revision: StateRevision,
    pub availability: Availability,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LightingTopology {
    /// The upstream flat LED order, never pointers into zones.
    pub leds: Vec<Led>,
    pub zones: Vec<Zone>,
    pub segments: Vec<Segment>,
    pub matrix: Option<MatrixMap>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LedRange {
    pub first: LedIndex,
    pub len: core::num::NonZeroU16,
}

impl LedRange {
    /// Fails if the range is outside the flat LED sequence or overlaps where
    /// the upstream contract forbids overlap.
    pub fn checked(first: LedIndex, len: u16, led_count: usize) -> Result<Self, TopologyError> {
        todo!("validate a value-owned range; do not retain LED references")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LightingState {
    pub active_mode: ModeId,
    pub modes: Vec<ModeDefinition>,
    pub colors: Vec<Rgb8>,
    pub brightness: Option<Brightness>,
    pub speed: Option<EffectSpeed>,
    pub direction: Option<EffectDirection>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ControllerOperation {
    SetMode { mode: ModeSelection },
    SetColors { target: ColorTarget, colors: Vec<Rgb8> },
    SetZoneMode { zone: ZoneId, mode: ModeSelection },
    SetLedColor { led: LedIndex, color: Rgb8 },
    Save,
    // Generated compatibility variants retain every supported controller
    // operation and update reason from the pinned snapshot.
}

impl ControllerOperation {
    pub fn set_colors(target: ColorTarget, colors: Vec<Rgb8>) -> Self {
        Self::SetColors { target, colors }
    }

    fn delivery(&self) -> DeliveryClass {
        // Whole-device colors and mode writes are coalescible. Zone, LED,
        // zone-mode, and Save are barriers and complete synchronously.
        todo!("derive policy from the operation, not from a UI caller")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppIntent {
    Controller { id: ControllerId, operation: ControllerOperation },
    ApplyProfile { profile: ProfileRef },
    SaveProfile { draft: ProfileDraft },
    Settings { change: SettingChange },
    Detection { command: DetectionCommand },
    Lifecycle { command: LifecycleCommand },
}

impl AppIntent {
    pub fn controller(id: ControllerId, operation: ControllerOperation) -> Self {
        Self::Controller { id, operation }
    }
    pub fn apply_profile(profile: ProfileRef) -> Self {
        Self::ApplyProfile { profile }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IntentResult {
    Queued { ticket: UpdateTicket, visible_revision: StateRevision },
    Applied { receipt: ApplyReceipt },
    ProfileApplied { report: ProfileApplyReport },
    ViewChanged { revision: AppRevision },
}

pub struct ControlPort {
    // An opaque, host-issued grant fixes the caller identity and capabilities.
    // A caller cannot forge loopback-local or service privileges.
    grant: CallerGrant,
    kernel: std::sync::Arc<Kernel>,
}

impl ControlPort {
    pub async fn view(&self) -> Result<AppView, AppError> {
        todo!("copy one immutable registry projection")
    }

    pub fn subscribe(&self) -> EventSubscription {
        todo!("register a bounded receiver; no callback executes under a lock")
    }

    pub async fn execute(&self, intent: AppIntent) -> Result<IntentResult, AppError> {
        todo!("validate, authorize, route, order, persist, and publish")
    }
}
~~~

The generated compatibility table owns the numeric DeviceType values, mode
flags, controller flags, update-reason values, and JSON description field
names. It has a single source of truth: a pinned, reviewed extraction from the
upstream snapshot. Unknown values round-trip as raw compatible bits or an
explicit Unknown value; they are not silently renumbered.

Color, topology, and mode constructors validate at the boundary. Internally,
a ValidatedOperation can assume a target exists, an exact color cardinality is
legal, and a direct-only controller has no unsupported mode or save operation.
This turns common hardware misuse into an invalid value instead of a late
driver branch.

### Crate and module map

| Crate/module | Owns | May depend on |
| --- | --- | --- |
| openrustygb-domain | Controller values, topology, capabilities, intents, snapshots, errors | Rust core types only |
| openrustygb-compat | Generated pinned numeric tables, JSON/protocol fixture metadata, port ledger readers | domain |
| openrustygb-core | Runtime, Kernel, owner leases, actor/registry/event semantics, grants | domain, storage and provider traits |
| openrustygb-hardware | Rust OS transport adapters, no-write inventory, catalog selection, detector contracts | domain, platform-native Rust bindings |
| drivers/family-* | One Rust adapter family per inventory record, private serializers and physical fixtures | domain, hardware |
| openrustygb-storage | Settings/profile/configuration records, migration and atomic repository | domain, compat |
| openrustygb-sdk | Private ORGB codecs, client/server sessions, network provider | domain, core |
| openrustygb-plugin-api and plugin-host | Versioned component ABI, permission grants, declarative plugin values | domain, core, storage |
| openrustygb-platform | Service, power, autostart, udev/systemd/tmpfiles/D-Bus, IOKit lifecycle events | domain, core |
| openrustygb-ui | eframe shell, localization/accessibility, tray view bridge | domain, core, plugin-host |
| openrustygb-app | One executable, RunMode composition and command parsing | all public product crates |
| xtask/release | Inventory audit, golden fixture checks, packaging/release evidence | manifests and test crates |

The dependency graph points inward to domain and never upward from a driver,
codec, storage adapter, or UI. There is no C++ bridge crate, Qt binding crate,
or production build script that compiles non-Rust application code.

### Kernel and controller ownership

Runtime is the composition root for desktop, tray, CLI, standalone, daemon,
SDK server, and service launches. It offers different ControlPorts, not
different implementations of controllers.

~~~rust
// crates/core/src/lib.rs
pub struct Runtime {
    supervisor: Supervisor,
    ports: Ports,
}

impl Runtime {
    pub async fn start(spec: LaunchSpec) -> Result<Self, StartError> {
        // 1. Open/migrate persistence, 2. start platform lifecycle adapter,
        // 3. start providers and the SDK gateway requested by RunMode,
        // 4. return capability-bound ports and immutable projection feeds.
        todo!("composition root only")
    }

    pub async fn connect_or_start(mode: RunMode, data_dir: DataDir) -> Result<Self, StartError> {
        todo!("use an existing compatible local server when requested")
    }

    pub fn ui_port(&self) -> ControlPort { todo!("desktop/tray grant") }
    pub fn cli_port(&self) -> ControlPort { todo!("CLI grant") }
    pub async fn shutdown(self) -> Result<(), ShutdownError> {
        todo!("ordered, idempotent shutdown")
    }
}

struct Kernel {
    registry: ControllerRegistry,
    persistence: StateRepository,
    providers: ProviderSet,
    broker: EventBroker,
    policy: PolicyEngine,
}

impl Kernel {
    async fn execute(&self, grant: &CallerGrant, intent: AppIntent)
        -> Result<IntentResult, AppError>
    {
        // Validate domain values -> authorize the caller -> resolve the current
        // owner -> choose delivery class -> enqueue/await it -> atomically
        // update visible projection and persistence -> emit canonical events.
        // This is policy, not a pass-through wrapper.
        todo!("the single semantic command route")
    }
}

struct ControllerActor {
    id: ControllerId,
    driver: Box<dyn ActivatedController>,
    desired: ControllerState,
    applied: ControllerState,
    inbox: tokio::sync::mpsc::Receiver<ControllerMessage>,
    events: ControllerEventSink,
    cancellation: tokio_util::sync::CancellationToken,
}

enum ControllerMessage {
    Apply { operation: ValidatedOperation, reply: Reply<IntentResult> },
    Describe { reply: Reply<ControllerSnapshot> },
    Shutdown { reply: Reply<()> },
}

impl ControllerActor {
    async fn run(mut self) {
        // Keep at most one pending whole-device color and one pending
        // whole-device mode update after the last barrier. If a newer update
        // replaces one, complete the older ticket as Superseded, never Applied.
        // Before a barrier, flush coalescible writes in sequence order; execute
        // zone, LED, zone-mode, and Save synchronously; emit update reasons
        // after the transition fixed by the upstream trace contract.
        // On cancellation reject new work, finish the shutdown handshake, and
        // let Supervisor join this task before dropping driver.
        todo!("sole writer and sole mutable controller state owner")
    }
}

trait ActivatedController: Send {
    fn initial_snapshot(&self) -> ControllerSnapshot;
    async fn apply(&mut self, operation: DriverOperation) -> Result<DriverReceipt, DriverError>;
    async fn close(self: Box<Self>) -> Result<(), DriverError>;
}
~~~

There is one actor and one serialized writer per active controller, including
the concrete local controller, a network proxy, and a plugin-created virtual
controller. The corresponding provider retains lifecycle ownership: the local
detector owns local actors, NetworkProvider owns remote proxy actors, and
PluginProvider owns virtual-controller actors. ControllerRegistry only
publishes immutable snapshots and routes requests through an owner lease; it
does not claim to own every controller.

ControllerRegistry uses an ArcSwap-style immutable AppView projection.
Listeners receive events through bounded channels and re-read a projection,
which gives UI, CLI, profiles, SDK, and plugins the same controller list and
detection-progress semantics without mutable aliases. A full channel has an
explicit resync event rather than unbounded memory or silent loss.

### Detection and the 197-family driver inventory

Detection is split by ownership, not by a generic load-validate-save pipeline.
The DriverCatalog owns matching policy and the detector/driver pair owns
hardware-specific knowledge.

~~~rust
// crates/hardware/src/catalog.rs; all items are internal to the hardware boundary.
trait Detector: Send + Sync {
    fn enumerate(&self, inventory: &HardwareInventory) -> Vec<DetectionClaim>;
    async fn attach_read_only(
        &self,
        claim: DetectionClaim,
        access: ReadOnlyAccess,
    ) -> Result<Vec<AttachedController>, DetectError>;
}

trait DriverFactory: Send + Sync {
    fn family(&self) -> FamilyId;
    fn matches(&self, claim: &DetectionClaim) -> MatchScore;
    async fn activate(
        &self,
        attached: AttachedController,
        permissions: DriverPermissions,
    ) -> Result<Box<dyn ActivatedController>, DetectError>;
}

struct DriverCatalog {
    entries: &'static [InventoryEntry],
    factories: &'static [Box<dyn DriverFactory>],
}

impl DriverCatalog {
    fn select(&self, claims: Vec<DetectionClaim>) -> Selection {
        // Pure, deterministic matching; conflicts become a diagnosable
        // AmbiguousMatch, not a probe that writes to a device.
        todo!("match VID/PID/path/bus facts to exactly one approved adapter")
    }
}
~~~

At build time, a checked inventory manifest contains all 197 upstream
controller-family directories and all 224 detector source files, their
Rust replacements, transport class, fixtures, parity state, and upstream
provenance. The exact Haste 2 adapter is tracked as an additive adapter record
instead of being silently folded into the unsafe legacy-Haste path. A cargo
xtask inventory audit fails if a source entry is missing, duplicated, orphaned,
or marked complete without its required tests. This lets implementation grow
family by family without a registry of handwritten, drifting maps.

Enumeration and attach-read-only have no write capability. They may enumerate
or read identity information needed to safely identify a device, but the
handle exposed at that stage has no output or feature-report API. A configured
startup profile is a separate, auditable post-attach command: for ordinary
compatible controllers it can finish before the ready snapshot is published,
matching upstream startup behavior; it is never part of detection. Per-driver
safety policy can prohibit it, as it does for Haste 2.

### Exact HyperX Pulsefire Haste 2 adapter

The driver is a tiny, dedicated Rust module with a deliberately impoverished
transport type. It accepts only VID 03F0, PID 0B97, HID interface 2, usage
page FF90, and usage FF00. It never opens interface 0 (mouse) or interface 1
(keyboard, consumer, and system controls) for lighting.

~~~rust
// crates/drivers/hyperx-haste2/src/lib.rs
const HASTE_2: ExactHidMatch = ExactHidMatch {
    vid: 0x03F0,
    pid: 0x0B97,
    interface_number: 2,
    usage_page: 0xFF90,
    usage: 0xFF00,
};

struct Haste2Rgb {
    /// Private output-only endpoint; this type has no DPI, feature, firmware,
    /// reset, save, profile, polling-rate, or read/write-any-report method.
    output: OutputReport65,
}

impl Haste2Rgb {
    fn description(identity: PublicIdentity) -> ControllerSnapshot {
        // One wheel LED, direct color capability, no save and no other mutable
        // mouse setting. No color is emitted here.
        todo!("build a direct-only controller description")
    }

    async fn set_direct_color(&mut self, color: Rgb8) -> Result<DriverReceipt, DriverError> {
        let primer = OutputReport65::zero_padded([0x44, 0x01, 0x01])?;
        let rgb = OutputReport65::zero_padded([
            0x44, 0x02, 0x00, 0x00, color.r, color.g, color.b,
        ])?;
        self.output.write_exact(primer).await?;
        self.output.write_exact(rgb).await?;
        Ok(DriverReceipt::applied())
    }
}

impl TryFrom<AttachedHid> for Haste2Rgb {
    type Error = DetectError;
    fn try_from(device: AttachedHid) -> Result<Self, Self::Error> {
        // Recheck the exact match and 65-byte output-report availability before
        // retaining an output handle. Opening/constructing does not write.
        todo!("reject every non-exact HID interface")
    }
}
~~~

OutputReport65 accepts exactly 65 bytes and OutputReport65::write_exact treats
a short write as an error. The controller actor owns the output handle, so
those two zero-padded output reports are always serialized and transport
errors reach the caller. It has no automatic write, keepalive, profile
autoload, save command, or generic report escape hatch. Its physical gate
requires an explicit direct-color test followed by pointer movement, buttons,
wheel, DPI, polling, disconnect/reconnect, and teardown observations; the
test must show that those non-lighting functions remain intact.

### Storage, profiles, and schema-driven settings

StateRepository is the only writer of OpenRGB.json, Configuration.json, and
profiles. Its compatibility adapter reads legacy JSON into typed records and
retains recognized extension data verbatim under a namespaced opaque value
until a migration owns it. UI, CLI, SDK, and plugins work with Settings,
Profile, and ProfileApplyReport domain types rather than JSON nodes.

~~~rust
// crates/storage/src/lib.rs
pub trait StateRepository: Send + Sync {
    async fn load(&self) -> Result<LoadedState, StorageError>;
    async fn transact(&self, change: StateChange) -> Result<StateRevision, StorageError>;
    async fn preview_migration(&self) -> Result<MigrationPreview, StorageError>;
}

pub struct ProfileMatchKey {
    stable_identity: StableIdentity,
    i2c_address: Option<u16>,
    // There is intentionally no HID location or I2C bus number.
}

impl ProfileMatchKey {
    pub fn from_identity(identity: &DeviceIdentity) -> Self {
        // Preserve stable identity fields and I2C address; deliberately omit
        // HID location and bus number to match upstream profile semantics.
        todo!("centralize profile matching in one constructor")
    }
}
~~~

All compatibility migrations are versioned, previewable, backed up, and
atomic-at-commit. Tests cover old documents, unknown settings preservation,
manual names/geometry/device configuration, optional profile base color,
plugin profile namespaces, malformed inputs, and the intentional
location/bus-insensitive matching rule. Runtime-registered settings schemas
feed both validation and the egui settings page; they do not become ad hoc UI
state.

### SDK v0–v6 and remote providers

SdkGateway owns all protocol framing and has a contract table per version.
The 16-byte ORGB header, packet IDs/layouts, 8 MiB maximum, v0 timeout
negotiation, v5-and-earlier index addressing, v6 stable IDs and
acknowledgements, callbacks, and feature flags are encoded in a private wire
module. Golden packet fixtures generated from the pinned upstream snapshot
test every accepted and rejected frame before the C++ reference is removed.

~~~rust
// crates/sdk/src/gateway.rs
pub struct SdkGateway { /* private codec, session table, and ControlPort */ }

enum ProtocolVersion { V0, V1, V2, V3, V4, V5, V6 }

struct SdkSession {
    version: ProtocolVersion,
    addressing: AddressingProjection,
    callback_stream: EventSubscription,
    grant: CallerGrant,
}

impl SdkGateway {
    pub async fn serve(self, listener: BoundListener) -> Result<(), SdkError> {
        // Check the 16-byte header and declared size before allocation; reject
        // > 8 MiB; negotiate v0 timeout; decode privately; map to AppIntent.
        todo!("one codec-to-domain boundary")
    }

    fn new_session(&self, peer: PeerIdentity, version: ProtocolVersion)
        -> Result<SdkSession, SdkError>
    {
        // Only a verified loopback peer receives the local-client grant.
        // A header or proxy claim is never treated as loopback proof.
        todo!("bind session compatibility behavior")
    }
}
~~~

For v0–v5, AddressingProjection maintains the upstream-compatible current
index view and updates it from registry events; it never leaks an index into
the core. Version 6 resolves stable ControllerId values and has the exact
acknowledgement behavior defined by its fixture table. NetworkProvider uses
the same session semantics to own remote proxy controllers, translating local
and remote detection progress into the canonical provider events consumed by
ControllerRegistry.

### Rust-native plugin migration

The current Qt/C++ ABI cannot be honestly loaded in a pure Rust egui process:
it returns QWidget and QMenu pointers and exposes a Qt object model. Candidate
B treats that as a compatibility migration, not an unsafe FFI promise.

The native replacement is a versioned Wasm Component interface with a
declarative host-rendered UI. A component can provide virtual controllers,
settings schemas, namespaced profile data, tray actions, localized strings,
and SDK command extensions. It returns data-only PluginUiTree and TrayAction
descriptions; eframe renders them, so no QWidget/QMenu or Qt lifetime crosses
the boundary. A plugin receives an opaque HostCapability grant and can request
only approved intents. SDK extensions receive bounded, versioned extension
payloads, not an ORGB packet or a socket.

~~~rust
// conceptual WIT/Rust host-facing shape; component ABI is versioned separately.
pub trait PluginComponentV1 {
    fn manifest(&self) -> PluginManifest;
    fn settings_schema(&self) -> Vec<SettingSchema>;
    fn virtual_controllers(&mut self) -> Result<Vec<VirtualController>, PluginError>;
    fn profile_load(&mut self, data: NamespacedProfileData) -> Result<(), PluginError>;
    fn profile_save(&mut self) -> Result<NamespacedProfileData, PluginError>;
    fn ui(&self, view: PluginViewContext) -> PluginUiTree;
    fn tray_actions(&self) -> Vec<TrayAction>;
    fn handle_sdk_extension(&mut self, request: SdkExtensionRequest)
        -> Result<SdkExtensionReply, PluginError>;
}
~~~

Migration is explicit and versioned:

1. PluginMigrate inventories old plugin metadata and imports compatible
   settings/profile namespaces without executing a binary plugin.
2. Plugin authors port capability-by-capability to Component V1 and use the
   host conformance suite for virtual controllers, schemas, profile data, tray
   actions, UI, and SDK extensions.
3. A Rust-only optional LegacyBridge can talk over a documented socket to a
   user-started, separately installed upstream process. It never loads a Qt
   binary, exposes Qt types, or ships Qt/C++ code. It is an external
   interoperability escape hatch, not native-plugin support and not a parity
   claim for arbitrary binary plugins.
4. The bridge can be removed after affected plugins have native replacements;
   the main executable remains entirely Rust throughout.

The boundary is major-versioned and can host V1 and V2 components side by
side during a declared window. Plugins cannot acquire raw HID, firmware, or
arbitrary process capability. That preserves the upstream plugin capabilities
listed above while keeping hardware policy, ownership, and UI lifetime in the
Rust host.

### Egui, platform modes, and release matrix

openrustygb-ui is an eframe shell over AppView and ControlPort. It has one
controller-page renderer driven by topology/capabilities, a schema-driven
settings renderer, dynamic language catalogs, tray bridge, keyboard-first
focus order, contrast tokens, scalable text, non-color status labels, and a
reduced-motion switch. It has no device, profile-file, or socket dependency.

PlatformLifecycle is a Rust trait with Windows service/power/autostart,
Linux udev/systemd/tmpfiles/D-Bus, and macOS IOKit/autostart implementations.
RunMode selects which adapters are composed, not a forked product:
Desktop/Tray, CLI, Standalone, HeadlessServer, LocalClient, and WindowsService
all use the same Kernel and controller actors. Suspend/resume produces
canonical lifecycle events and schedules a safe rescan; it does not create a
hidden hardware-write path.

ReleaseManifest is checked data, not prose. It has these required artifacts:

| Platform | Required artifacts |
| --- | --- |
| Windows | i686 and x86_64 portable ZIP; x86_64 MSI |
| Linux | AppImage and DEB for i386, amd64, armhf, arm64; x86_64 RPM; x86_64 and arm64 Flatpak |
| macOS | Intel and Apple Silicon app bundles published as ZIPs |

Native CI runners build and install each artifact. A release gate requires
platform package installation and launch, domain/codec/migration/plugin/UI
tests, platform lifecycle evidence, inventory coverage, and claimed-hardware
evidence. Build success or --version alone cannot mark an installer released.
The final deletion gate also fails if the current product tree contains a C,
C++, header, Qt project, CMake/qmake C++ build, or C++ FFI dependency; upstream
history, GPL-2.0 notices, the upstream remote, provenance ledger, and
pre-generated test fixtures remain for traceability.

## Concurrency and shutdown contract

* A provider owns its controller actors and can withdraw only its own lease.
  The registry stores immutable projections, not mutable controller objects.
* One controller actor owns one active driver and its writer. A controller
  operation has a monotonic sequence; coalescing can supersede only an earlier
  unflushed whole-device operation after the same barrier.
* Zone, single-LED, zone-mode, and save operations first flush preceding
  coalescible operations, then await their hardware result. Whole-device
  color/mode operations return Queued with a ticket and later yield Applied or
  Superseded. SDK response/ack timing is pinned per protocol fixture.
* Store writes run through one transaction owner. Event consumers are
  independent receivers; no consumer can block the registry or controller
  writer.
* Shutdown and unplug are idempotent and use this order: (1) close admission
  for the affected runtime/provider, (2) cancel and join controller writers
  and provider workers, (3) close subscriptions and drain in-flight callback
  walks, (4) publish a snapshot without the withdrawn handles, then (5) drop
  concrete transports and controller state. No mutex/registry guard is held
  while a callback, blocking send, or join runs.
* Driver close is called only after its actor has joined, which guarantees the
  Haste 2 HID handle outlives its writer and never receives a late report.

## Compatibility and parity gates

The port ledger is a checked, reviewable table with one row for every upstream
feature, 197 family directories, 224 detector source variants, SDK packet/version behavior,
profile/settings rule, platform mode, plugin capability, and release artifact.
Every row names a Rust owner, fixture or integration test, evidence state, and
the upstream commit/path it replaces. No row may be "assumed equivalent."

| Gate | Required proof before its row is complete |
| --- | --- |
| Semantic controller model | Golden operation traces preserve numeric types/flags, modes, zones, flat LED ranges, matrices, callbacks, update reasons, configuration, and save semantics. |
| Drivers and detection | Inventory audit, no-write enumeration tests, match-conflict tests, mock transport byte traces, and per-family hardware/fixture evidence. |
| HyperX Haste 2 | Exact matcher rejection cases; exactly two 65-byte reports for a color; short-write/error/cancel tests; observed Windows pointer/button/wheel/DPI/polling/reconnect/teardown check. |
| Persistence | OpenRGB.json, Configuration.json, and profile corpus migration round trips, unknown-data preservation, atomic-failure recovery, and exact profile match-key tests. |
| SDK v0–v6 | Header, packet/layout, 8 MiB, timeout, index/stable-ID, ack, callback, flag, and loopback privilege golden tests plus client/server interop. |
| Plugins | Component ABI conformance for all listed capabilities, migration fixtures, declarative UI rendering, and explicit LegacyBridge limitation test. |
| UI and modes | Eframe interaction/accessibility/localization snapshots plus desktop, tray, CLI, standalone, server, local-client, service, suspend/resume integration tests. |
| Release | Native artifact build, install, launch, checksum/SBOM/license validation, and platform-specific evidence for every published matrix cell. |
| C++/Qt removal | Port-ledger has no incomplete row; source-tree prohibition passes; no production process links or loads C++/Qt. |

The upstream snapshot may be used only as an offline fixture generator while
porting. Before its current-tree sources are deleted, generated fixture hashes
and operation/packet traces are checked into the Rust test corpus. Thereafter
the test suite is self-contained Rust while Git history and the upstream remote
remain intact.

## Red-flag screen

| Red flag | Result |
| --- | --- |
| Shallow module | Clear: ControlPort has three operations and hides authorization, routing, ownership resolution, coalescing, persistence, and events. DriverCatalog and StateRepository each own a substantive policy boundary rather than exposing its steps. |
| Information leakage | Clear: wire packets, JSON ASTs, HID reports, OS handles, Qt objects, and component plumbing are private. Compatibility tables are generated once and consumed as domain values. |
| Temporal decomposition | Clear: modules own controller state, drivers, storage, protocol, plugins, platform lifecycle, or UI knowledge; they are not named load/validate/transform/save pipelines. |
| Pass-through method | Clear: the Kernel route adds validation, grants, lifecycle/ordering policy, persistence, and canonical events. Provider/SDK/UI adapters adapt distinct boundaries; any future method that only forwards an unchanged call is deleted or moved. |

## Rationale (one page)

### Problem

OpenRustyGB must replace a 469k-line C++/Qt product while preserving observable
controller, protocol, persistence, plugin, and platform behavior. The
non-obvious constraint is ownership: local detectors, remote clients, and
plugins own different controllers, while upstream-compatible consumers require
one safe view. The design also needs a strict no-write discovery boundary and
an exact Haste 2 adapter whose legacy relatives are unsafe to reuse.

### Usage (caller's view)

The three calls at the top are the complete normal model: obtain a
capability-bound port, read an immutable view or subscribe, and execute an
intent. A desktop page paints from snapshots, CLI asks to apply a profile, and
the Haste call requests only a direct color. None needs to learn transport,
locking, JSON, protocol versions, or who owns a controller.

### Shape

Per-controller actors own mutation and hardware, while a snapshot registry
unifies local, remote, and virtual controller views. The narrow ControlPort
is deep: it hides grants, validation, routing, coalescing, persistence, and
event production behind view/subscribe/execute. Validated domain values encode
topology/range and capability invariants; storage, SDK, drivers, and plugins
each translate at a single boundary. This follows boundary-discipline,
separate-before-serializing-shared-state, encode-lessons-in-structure, and
make-operations-idempotent.

### Synthesis decision

Pending parent arena synthesis. Candidate B contributes the owner-leased
actor/snapshot registry, a compile-checked source inventory, and a
capability-projected Wasm plugin migration path. It rejects a fictitious
in-process Qt ABI bridge as incompatible with the all-Rust end state.

### Tradeoffs accepted

* We accept actor/channel machinery in exchange for a single writer and a
  shutdown order that cannot race a transport drop.
* We accept generated compatibility tables and fixture maintenance in exchange
  for stable numeric and wire behavior without retaining C++ runtime code.
* We accept explicit migration of Qt plugins in exchange for memory-safe,
  versioned UI and ownership boundaries.
* We accept native-runner release work in exchange for truthful installers
  rather than cross-compiled artifacts with unverified platform claims.

### Alternatives considered

* A shared mutable ResourceManager with locks lost because callers would need
  to understand lock/callback timing and controller ownership; it hides little
  of the real complexity.
* A global event-sourced controller store lost because it exposes replay and
  consistency machinery to ordinary hardware callers while still needing a
  per-device writer.
* Native Rust dynamic libraries with a stable C-like ABI lost because it
  exposes ABI/lifetime policy to plugin authors and cannot represent existing
  QWidget/QMenu plugins without reintroducing Qt.

### Open questions and risks

* Which upstream plugin packages and non-virtual capabilities need a named
  migration owner before the LegacyBridge retirement date?
* Which 197-family hardware fixtures can be legally captured and redistributed
  before the upstream source tree disappears from the current checkout?
* What code-signing identities and native CI runners are available for the
  Windows MSI, Linux architecture matrix, and both macOS bundles?
* Is profile autoload expected to remain enabled for every existing controller,
  or should new no-auto-write policies extend beyond Haste 2?

### Next implementation step

Create the Rust workspace with domain compatibility tables, the immutable
registry/actor contract, and the inventory ledger before porting a first
detector or UI page.
