# gax-rs 🎵

**Zero-dependency, pure Rust parser, software synthesizer, and golden-tested emulator for the Shin'en GAX Sound Engine (Game Boy Advance).**

[![Crates.io](https://img.shields.io/crates/v/gax.svg)](https://crates.io/crates/gax)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![no_std](https://img.shields.io/badge/no__std-compatible-success.svg)](#features)

The **GAX Sound Engine** was created by Manfred Linzner and Bernhard Wodok at **Shin'en Multimedia**. Known for its incredible CPU efficiency (~11% CPU load, <2 KB RAM) and rich multi-channel sound, it powered over 100 commercial GBA titles, including *Iridion 3D*, *Iridion II*, *Crash Bandicoot 2: N-Tranced*, *Payback*, and *Maya the Bee*.

`gax-rs` provides a clean-room, reverse-engineered implementation designed for audio preservation, DAWs, PC source ports, and emulators.

---

## Features

- **Zero External Dependencies**: Pure Rust with no dependencies in `[dependencies]`.
- **`#![no_std]` Compatible**: Ready for embedded targets and WASM runtimes (`alloc` required for dynamic sequence parsing).
- **Dual Synthesis Engines**:
  - **`AccuracySynth`**: Emulates the original fixed-point math, 8-bit signed PCM mixing, and hardware sample rates (13,379 Hz / 18,157 Hz), matching the ARM driver's DMA ring buffer.
  - **`StudioSynth`**: Renders at 48,000 Hz, 32-bit floating-point precision with 64-tap band-limited sinc interpolation and true stereo panning.
- **Golden Tested**: Every event dispatcher, envelope slope, and mixing curve is verified against cycle-accurate ARM execution traces captured from mGBA.

---

## Supported Engine Versions

1. **GAX v1.x** (e.g. *Iridion 3D*) — Early format with direct PCM soundpools.
2. **GAX v2.x** — Transitional revision.
3. **GAX v3.01 / v3.05** (e.g. *Crash Bandicoot 2: N-Tranced*, *Top Gear Rally*) — Widespread standard revision with compressed sequence streams and multi-pass reverb.

---

## Quickstart

### Zero-Allocation Mixing (`#![no_std]`)

```rust
use gax::{AccuracySynth, GaxSampleView, GaxVoice};

// Wrap ROM bytes directly without heap allocation
let pcm_bytes: &[u8] = &[0x00, 0x10, 0x20, 0x10, 0x00, 0xF0, 0xE0, 0xF0];
let sample = GaxSampleView::from_raw_bytes(0, 18158, pcm_bytes);

// 1:1 pitch step (0x1000), full volume (128), centered pan (64)
let mut voice = GaxVoice::new(0, 0x1000, 128, 64);
let mut dma_left = [0i8; 64];
let mut dma_right = [0i8; 64];

AccuracySynth::mix_voice_view(&sample, &mut voice, &mut dma_left, &mut dma_right);
```

---

## Golden Test Methodology

To ensure 100% mathematical accuracy without guesswork:
1. **Tier 1 (Bit-Exact PCM)**: Reference samples extracted directly from ROM are compared against mGBA memory dumps.
2. **Tier 2 (Sequencer State Transitions)**: Active channels, note pitches, ADSR levels, and loop triggers are asserted against the ARM driver's RAM state at every 60 Hz frame.
3. **Tier 3 (End-to-End Stream Correlation)**: The software mixer output is correlated against raw PCM dumped from hardware DMA registers (`REG_FIFO_A` and `REG_FIFO_B`).

---

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
