# Rust-only migration contract

The final OpenRustyGB codebase must contain no C, C++, Objective-C, native
headers, Qt project/UI resources, CMake files, qmake files, or production
dependency that compiles or links native application code.

## Sequence

1. Keep the pinned upstream implementation only as an unshipped reference.
2. Port one owned feature or controller family to Rust.
3. Prove compatibility with protocol fixtures, state tests, and hardware
   evidence where the feature touches a physical device.
4. Delete the replaced native implementation and its obsolete build entries.
5. Repeat until the feature ledger, driver inventory, and platform matrix pass.
6. Delete the remaining reference tree and run the final source and dependency
   audits before building release artifacts.

Deleting native code before its Rust replacement passes would remove product
behavior rather than convert it, so contraction happens only after verification.

## Required gates

`cargo xtask inventory --require-parity` rejects an incomplete controller-family
port. `cargo xtask source-audit --require-rust-only` rejects native source,
headers, Qt resources, and native build descriptions. The release workflow must
also audit `cargo tree` and packaged binaries for native C/C++ build or linkage
before the first parity tag.

The current development checkout is expected to fail the Rust-only gate while
the pinned reference tree remains. No installer or parity release may be
published in that state.
