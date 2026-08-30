# OpenRustyGB agent entry point

Before changing migration code, device drivers, UI, packaging, or releases,
read [`docs/PROJECT_CONTEXT.md`](docs/PROJECT_CONTEXT.md). It records the goal,
current verified state, sources of truth, hardware rules, and completion gates.

Refresh migration counts with `cargo xtask inventory` and
`cargo xtask source-audit`. The checked output is authoritative when a number in
documentation is older than the working tree.

For every completed controller-family port, update
[`docs/migration/ported-families.md`](docs/migration/ported-families.md) and the
checked inventory in `xtask/src/main.rs` in the same commit.
