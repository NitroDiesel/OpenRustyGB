# Upstream architecture history constraints

These constraints come from direct upstream commit and merge-request statements rather than inference from the current layout.

## Detection ownership

MR !3149 separated detection so local `DetectionManager` and a local `NetworkClient` could expose equivalent detection progress and controller-list behavior. It also sought to reduce `ResourceManager` exposure so internal changes would be less likely to break the plugin API.

- MR: `https://gitlab.com/CalcProgrammer1/OpenRGB/-/merge_requests/3149`
- Commit: `96a9efc48b2dd0ef5e35cd775b549cd41ae23c68`
- Detection progress follow-up: `https://gitlab.com/CalcProgrammer1/OpenRGB/-/merge_requests/3158`

The Rust facade must therefore translate local and remote detection sources into the same consumer events while keeping the local detector as a separate owner.

## Controller API

Commit `2bf805fecce70bd8f8db2029382de47f1864d0ae` protected previously public state, added controlled accessors, synchronization, update reasons, hidden controllers, per-zone modes, richer zones/segments, value-owned matrix maps, and JSON descriptions. MR !2935 ties this to API v5, SDK v6, profiles, and hotplug.

- Commit: `https://gitlab.com/CalcProgrammer1/OpenRGB/-/commit/2bf805fecce70bd8f8db2029382de47f1864d0ae`
- MR: `https://gitlab.com/CalcProgrammer1/OpenRGB/-/merge_requests/2935`

The evidence strongly supports forcing mutation through synchronized, callback-aware operations. Do not expose mutable controller internals to UI, network, profiles, or plugins.

## Shutdown and deadlock avoidance

Commit `6ddcb16787c50cd1cc30121eef5f36657bfd64f2` moved base teardown into explicit `Shutdown()` so the update thread stops before a concrete controller deletes its transport. The same change unlocks the controller-ID registry before notifying and joining removed network workers.

- Commit: `https://gitlab.com/CalcProgrammer1/OpenRGB/-/commit/6ddcb16787c50cd1cc30121eef5f36657bfd64f2`

The preserved invariant is ordered cancellation and ownership, not the literal C++ locking implementation:

1. stop accepting or scheduling new work;
2. cancel and join writers/workers;
3. withdraw callbacks and drain in-flight access;
4. remove controller handles from public snapshots;
5. drop concrete transports and state.

Never invoke callbacks, perform blocking sends, or join a worker while holding a lock that operation may need.

Later MRs !3386 and !3400 reinforce snapshot-based registries, path-based unplug lookup, callbacks copied under lock and invoked unlocked, and explicit accounting for in-flight callback walks.
