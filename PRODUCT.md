# Product

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Stack

Rust-native desktop application using `egui`/`eframe`, selected with the user's approval for Windows, Linux, and macOS.

## Users

People who want vendor-independent control of RGB-capable hardware across Windows, Linux, and macOS. The first directly validated user setup is a Windows laptop with a connected HyperX Pulsefire Haste 2 mouse.

## Product Purpose

OpenRustyGB is a complete Rust reimplementation of OpenRGB. It must preserve OpenRGB's device discovery, lighting control, profiles, SDK/server behavior, plugins, command-line workflows, and cross-platform packaging while replacing the C/C++ and Qt implementation with Rust.

Success means verified feature and device parity, safe hardware behavior, and downloadable installers for supported desktop platforms.

## Positioning

OpenRustyGB combines OpenRGB-compatible behavior with a memory-safe Rust implementation and a Rust-native interface. It is not a thin wrapper around the upstream C++ application.

## Operating Context

The application discovers local RGB devices, exposes zones, LEDs, modes, brightness and colors, stores profiles, supports automation through its CLI and SDK/server protocol, and must coexist safely with operating-system HID access and other hardware-control software.

## Capabilities and Constraints

- Preserve the functions and features of the upstream OpenRGB repository.
- Use Rust for the application, device implementations, protocols, UI, tests, and packaging logic owned by this project.
- Preserve Windows, Linux, and macOS support and publish platform installers only after the relevant platform is verified.
- Support the connected HyperX Pulsefire Haste 2 identified on Windows as USB VID `0x03F0`, PID `0x0B97`.
- Hardware validation may use reversible HID lighting commands. Firmware updates and persistent device-memory writes are excluded unless separately authorized.
- Retain upstream GPL-2.0 license obligations and attribution.
- Keep the upstream Git history and an `upstream` remote for traceability.
- The public GitHub destination is the user's account, `NitroDiesel`; the repository name is `OpenRustyGB` unless the user changes it.
- Public release is gated on verified feature parity rather than an incomplete preview being presented as complete.

## Brand Commitments

The product name is OpenRustyGB. Upstream OpenRGB terminology should remain recognizable where compatibility matters, while the interface may be redesigned for a clearer native desktop workflow.

## Evidence on Hand

- Full upstream OpenRGB repository and history in this workspace.
- Upstream Qt UI, controller implementations, protocol code, profiles, tests, packaging, and release configuration.
- Live Windows PnP evidence for the HyperX Pulsefire Haste 2 at VID `0x03F0`, PID `0x0B97`, including mouse, keyboard/media, and vendor-defined HID interfaces.
- No fabricated compatibility claims or installer claims may be used; releases require build and hardware evidence.

## Product Principles

- Preserve behavior before changing it.
- Make hardware writes explicit, bounded, and reversible.
- Encode controller and protocol invariants in Rust types.
- Keep detection, transport, device policy, and presentation independently testable.
- Publish only artifacts that were built and verified on their claimed platforms.

## Accessibility & Inclusion

The redesigned desktop interface must support keyboard navigation, readable contrast, scalable text, non-color-only status communication, and reduced-motion operation.
