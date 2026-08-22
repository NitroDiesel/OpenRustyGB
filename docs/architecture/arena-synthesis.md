# Architecture arena synthesis

Date: 2026-08-22

Parity target: OpenRGB commit `8121ee29f46d58f90a56348eb5bf7a64f52f923b`.

## Decision

Use Candidate A's source-owned, per-controller actor architecture with an
immutable aggregate snapshot. Graft Candidate B's explicit provider leases,
compile-checked compatibility tables, and inventory evidence model. Preserve
Candidate C's distinction between requested, attempted, applied, failed,
superseded, and crash-uncertain effects, but keep those outcomes in the
operation/event model rather than introducing a durable global event journal.

The result has one semantic writer and one transport writer per controller.
Local detection, an SDK connection, or a plugin remains the lifecycle owner of
the controllers it supplies. A registry publishes structurally shared,
immutable views. UI, CLI, SDK, persistence, and plugins receive capability-
bound sessions that can read, subscribe, and submit domain intents; none can
obtain a transport or mutable controller.

## Candidate scoring

Scores are 1 (weak) through 5 (strong). The first two arena candidates
converged on the same actor/snapshot shape, so Candidate C was commissioned as
the structurally distinct event-sourced counter-design. The independent judge
could not complete because its account hit a usage limit; the parent reviewed
all three documents end to end against the predeclared criteria.

| Criterion | A | B | C |
| --- | ---: | ---: | ---: |
| SDK v0-v6 and legacy persistence behind a small surface | 5 | 4 | 5 |
| Ownership, callbacks, locks, unplug, and ordered shutdown | 5 | 5 | 4 |
| Measurable migration and eventual C++/Qt deletion | 5 | 5 | 4 |
| Typed hardware safety and exact Haste 2 isolation | 5 | 5 | 5 |
| GUI/CLI/service/platform/release and plugin migration | 5 | 5 | 5 |
| Interface depth with low accidental complexity | 5 | 5 | 4 |
| **Total** | **30** | **29** | **27** |

## Why A is the base

Candidate A most directly encodes the upstream ownership corrections: a
source owns controller lifetime, a controller actor owns mutable semantic
state and its serialized writer, and the aggregate registry owns only a
read-only projection. Its generation-checked `ControllerRef` prevents stale
commands after reconnect. Its delivery policy reproduces coalesced whole-
device writes and synchronous zone, LED, configuration, and save barriers
without exposing queues to consumers. Its shutdown protocol closes admission,
joins writers without holding shared locks, publishes removal once, and only
then drops transports.

Candidate B is compatible with that decision and strengthens it with named
provider leases and generated compatibility tables. It is not a competing
architecture after full review, so its useful mechanisms are grafts rather
than a separate base.

Candidate C is genuinely distinct and internally coherent. A durable total
order would improve replay, cross-surface causal diagnostics, and honest crash
uncertainty. It loses for this product because it creates a new persistent
format and privacy/retention policy, makes legacy JSON a materialized view,
adds checkpoint/compaction/replay failure modes, and globally sequences every
semantic admission. None is required to preserve the pinned OpenRGB behavior.
Per-controller actors already isolate slow devices and reproduce the required
ordering with less reader and operator load.

## Accepted grafts

- Provider-owned leases make local, remote, and plugin ownership explicit and
  prevent one provider from withdrawing another provider's controller.
- Generated compatibility tables own stable numeric enums, bit flags, update
  reasons, SDK packet metadata, and persistence field names.
- The port ledger is a build-checked artifact. A family or detector cannot be
  marked complete without its owner, provenance, fixtures, and evidence.
- Operation outcomes include `Requested`, `Attempted`, `Applied`, `Failed`,
  `Superseded`, and `Uncertain`. An interrupted attempt is never automatically
  replayed unless the driver contract proves idempotence and policy permits it.
- Haste 2 explicitly forbids automatic activation, recovery replay, profile
  autoload, keepalive, and every non-lighting report.

## Rejected elements

- No global durable event journal or JSON materializer.
- No global mutable controller registry.
- No process-per-controller topology.
- No in-process Qt/C++ plugin compatibility shim.
- No generic HID report API above a transport-private driver module.
- No automatic hardware writes during discovery.

## Verification consequence

The implementation must prove the selected contracts before breadth work:

1. domain constructors and compatibility discriminants;
2. controller incarnation, actor sequencing, coalescing, and barriers;
3. callback/event delivery outside locks and explicit resynchronization;
4. ordered, idempotent unplug and root shutdown;
5. inventory/catalog audits against 197 families and 224 detector source files;
6. exact 65-byte Haste 2 buffer goldens, native report-ID completion semantics,
   and a no-write discovery audit;
7. an explicit, reversible live lighting test only after fake-transport proof.
