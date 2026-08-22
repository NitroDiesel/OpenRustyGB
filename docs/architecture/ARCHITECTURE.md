# OpenRustyGB architecture

Status: accepted foundation for the Rust rewrite

Parity target: OpenRGB commit `8121ee29f46d58f90a56348eb5bf7a64f52f923b`.

## Product boundary

OpenRustyGB preserves the pinned product's observable controller model, SDK
v0-v6 behavior, legacy settings and profiles, local/remote/plugin ownership,
process modes, and release surfaces. It does not translate the C++ class
graph. Production binaries and shipped plugins are Rust-owned; the old tree is
an unshipped migration oracle until every checked ledger row is green.

Existing Qt plugin binaries cannot be loaded by the final egui product.
Capabilities migrate to a versioned WebAssembly component API with declarative
UI and menu trees, host-owned virtual-controller leases, namespaced settings
and profile data, bounded SDK extensions, and capability-checked host calls.

## Caller contract

Every product surface receives a capability-bound `Session`:

```rust
pub trait SessionView {
    fn latest(&self) -> Arc<SystemSnapshot>;
    fn subscribe(&self, after: EventCursor) -> EventStream;
    fn submit(&self, intent: Intent) -> Result<CommandTicket, SubmitError>;
    async fn execute(&self, intent: Intent) -> Result<CommandReceipt, CommandError>;
}
```

This is a deep boundary. The session authorizes the caller, validates domain
values, resolves generation-checked controller references, orders operations,
normalizes completion, and publishes events. It does not expose SDK packets,
JSON syntax trees, OS handles, plugin runtimes, egui types, or device I/O.

## Runtime ownership

```text
RuntimeSupervisor
  |-- CommandRouter       authority, target resolution, operation sequence
  |-- ControllerRegistry immutable aggregate snapshot only
  |-- EventHub            canonical event order and bounded subscriptions
  |-- PersistenceActor    sole writer of legacy-compatible files
  |-- LocalProvider ------ ControllerActor(s) ------ DeviceWriter(s)
  |-- NetworkProvider ---- ProxyControllerActor(s) - SDK connection owner
  `-- PluginProvider ----- VirtualControllerActor(s) component resource owner
```

A provider may add, update, or withdraw only controllers covered by its lease.
Every controller actor owns the controller's mutable desired/applied state and
the only writer for its transport. The registry merges value-owned source
deltas and publishes structurally shared `Arc<SystemSnapshot>` values. It does
no device, network, plugin, callback, or filesystem I/O.

`ControllerRef { id, incarnation }` rejects ABA/stale commands after unplug or
reconnect. Zone topology uses checked LED ranges and matrix indices rather than
self-referential pointers. Constructors validate color cardinality, range
bounds, mode capabilities, segment ranges, and matrix dimensions once.

## Command ordering

Policy is derived from the command variant, never chosen by UI or SDK code.

- Whole-device color and mode writes may coalesce while pending. A replaced
  ticket completes as `Superseded`, not `Applied`.
- Zone, single-LED, zone-mode, configuration, resize, segment, and save
  operations are barriers. The actor drains preceding coalescible work,
  executes the barrier synchronously, and starts no later command first.
- Accepted commands receive a monotonic operation sequence. Events retain the
  pinned numeric update reason.
- Outcomes distinguish requested, attempted, applied, failed, superseded, and
  uncertain. A process interruption cannot be reported as applied and is not
  automatically replayed without a driver-specific idempotence contract.
- Callback data is copied to bounded subscriber queues and delivered outside
  controller, registry, provider, and event locks. Lagging consumers resync
  from `latest()`.

## Detection and driver boundary

`DriverFamily` owns matching, declared read-only probing, blueprint creation,
activation, serialization, tests, and its stable detector settings key.
Enumeration produces an immutable hardware inventory. `ProbeIo` deliberately
has no write method. A claim arbiter rejects ambiguous highest-specificity
matches instead of depending on registration order.

The checked catalog contains every pinned family and detector with stable ID,
platform, transport, match/claim rules, safe-read declaration, source
provenance, fixtures, Rust owner, and evidence state. Build tooling fails on a
missing, duplicate, orphaned, ambiguous, or falsely complete record.

Concrete transport capabilities remain private. Consumers submit semantic
commands; only a validated family adapter can create exact wire transactions.

## Pulsefire Haste 2 safety contract

The Haste 2 is a dedicated family and never shares the legacy Haste protocol.
Its live Windows match is exact:

```text
VID 03F0, PID 0B97, HID interface 2, usage page FF90, usage FF00
```

Discovery records a dormant, one-LED `Mouse` controller and performs zero
writes. The first explicit user direct-color command re-enumerates and
revalidates the exact vendor interface before opening it. The adapter can only
construct one indivisible two-report transaction:

```text
44 01 01 00 ...                         65-byte output report
44 02 00 00 RR GG BB 00 ...             65-byte output report
```

The submitted API buffer is always exactly 65 bytes. A backend may report 64
accepted payload bytes after consuming the report-ID byte; anything shorter
fails the transaction. There is no feature-report,
DPI, firmware, reset, save, profile memory, polling, lift-off, keepalive,
automatic profile restore, arbitrary report, or recovery replay path. The
writer is cancelled and joined before its HID handle is dropped. Lighting code
never opens interface 0 or interface 1.

## SDK and persistence boundaries

The SDK codec privately owns explicit little-endian framing, the 16-byte ORGB
header, packet/version layouts, an 8 MiB pre-allocation limit, v0 negotiation,
v0-v5 connection-local indices, v6 stable IDs and acknowledgements, callbacks,
feature flags, and loopback-only local-client authority. Domain identities are
never SDK indices.

Persistence initially reads and writes `OpenRGB.json`, `Configuration.json`,
and `profiles/*.json`. DTOs remain private. Unknown compatible fields round
trip. One profile-match constructor intentionally omits HID location and I2C
bus number while retaining stable identity and I2C address. Writes use a temp
file, flush, atomic replace, and recoverable backup. A new schema version needs
a bidirectional, rollback-tested migration.

## Crate boundaries

```text
openrustygb-app             one executable and launch-plan composition
openrustygb-domain          identities, snapshots, commands, events, validation
openrustygb-compat          generated pinned enums, flags, reasons, field metadata
openrustygb-runtime         supervisor, router, actors, registry, shutdown
openrustygb-driver-api      internal detector/driver/lease capability traits
openrustygb-driver-catalog  generated inventory and evidence ledger
drivers/*                   family-owned matcher, serializer, and tests
openrustygb-transport       Rust HID/USB/I2C/serial/network implementations
openrustygb-sdk             private v0-v6 codec, client source, server gateway
openrustygb-persistence     legacy codecs, schemas, profiles, atomic transactions
openrustygb-plugin-protocol component values and declarative UI/menu contracts
openrustygb-plugin-host     WebAssembly component supervision and grants
openrustygb-ui-egui         eframe views, tray, accessibility, localization
openrustygb-platform-*      Windows, Linux, and macOS lifecycle adapters
openrustygb-testkit         fake transports, traces, fixtures, differential tools
xtask                       catalog, parity, packaging, signing, and release gates
```

Dependencies point inward to domain values and narrow capability traits.
Drivers do not depend on runtime, UI, SDK, persistence, or plugins. Adapters
translate once at their boundary.

## Shutdown contract

Unplug and shutdown share an idempotent retirement state machine:

1. close admission for the affected runtime/provider/controller;
2. cancel and join writers and provider workers without holding shared locks;
3. close subscriptions and drain in-flight callback walks/RPC replies;
4. publish one removal snapshot/event and invalidate the incarnation;
5. drop the dormant driver, transport, and provider resources.

Root shutdown first closes external ingress, retires providers/controllers,
flushes the persistence actor, stops platform integration, and then joins the
event/runtime tasks. Suspend uses the quiesce boundary and resume re-enumerates
rather than trusting stale device paths.

## Completion and publication gate

The port ledger is the source of truth. Release is forbidden until it proves:

- all 197 pinned controller-family directories and 224 detector source files
  units have implemented Rust owners and required evidence;
- controller semantics, SDK v0-v6, settings/profiles, plugin capabilities,
  GUI/tray/CLI/server/client/service modes, and platform lifecycle are covered;
- the physical Haste 2 gate verifies lighting plus unchanged pointer, buttons,
  wheel, DPI, polling, reconnect, cancellation, and teardown behavior;
- every Windows, Linux, and macOS artifact installs, launches a real product
  mode, exercises a virtual controller through SDK and persistence, and
  uninstalls cleanly;
- checksums, SBOMs, signatures/notarization status, and public release assets
  are verified;
- shipped source and dependencies contain no C, C++, Qt, qmake, C++ CMake
  product build, native Qt plugin ABI, or C++ FFI dependency.

The detailed arena record is in `arena-synthesis.md`; the full base sketch is
in `candidates/candidate-a.md`.
