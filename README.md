# gax-rs 🎵

**Zero-dependency, pure Rust parser, software synthesizer, and golden-tested emulator for the Shin'en GAX Sound Engine (Game Boy Advance).**

[![Crates.io](https://img.shields.io/crates/v/gax.svg)](https://crates.io/crates/gax)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)
[![no_std](https://img.shields.io/badge/no__std-compatible-success.svg)](#features)

The **GAX Sound Engine** was created by Manfred Linzner and Bernhard Wodok at **Shin'en Multimedia**. Known for its incredible CPU efficiency (~11% CPU load, <2 KB RAM) and rich multi-channel sound, it powered over 100 commercial GBA titles, including *Iridion 3D*, *Iridion II*, *Crash Bandicoot 2: N-Tranced*, *Payback*, and *Maya the Bee*.

`gax-rs` provides a clean-room, reverse-engineered implementation designed for audio preservation, DAWs, PC source ports, and emulators.

> **Looking for a ready-to-use CLI?** You can scan GBA ROMs, extract WAV samples, and compile SoundFont 2.04 (`.sf2`) banks directly from the command line using our **[gbakit](https://github.com/JidouGhost-TL/gbakit)** toolkit (`gbakit audio scan`, `gbakit audio export`, `gbakit audio soundfont`).

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
1. **Tier 1 (Version-Aware PCM & Continuity)**: 3 reference sample fixtures across GAX v1.99d (*Iridion 3D*, signed `i8`), v3.01 (*Crash Bandicoot 2: N-Tranced*, unsigned `0x80`-centered), and v3.05 (*Iridion II*, unsigned `0x80`-centered).
2. **Tier 2 (Timer Reload & Version Detection)**: Hardware timer reload math (`GAX_FREQ_TABLE`) and ROM version signature detection (`GaxVersion::detect_from_bytes`).
3. **Tier 3 (1,792 / 1,792 Byte-Exact mGBA ARM Mixer Goldens)**: 8 controlled ARM execution vectors (`1,792` output bytes, SHA-256 manifest in `tests/golden/manifest.json`) verified strictly byte-for-byte (`assert_eq!`) across GAX v1.99d, v3.01, and v3.05 (1:1 pitch, resampled pitch, and multi-voice saturation clamping), plus 3 hardware RAM stream captures.

---

## Benchmarks

```sh
cargo bench --bench bench_audio
```

| Benchmark | Throughput |
| --- | ---: |
| `AccuracySynth::mix_voice_pcm8` (1:1 pitch, 1,024f) | ~2.34 GSamples/s |
| `AccuracySynth::mix_voice_pcm8` (resampled `0x1500`, 1,024f) | ~522 MSamples/s |

---

## Community & Support

Come discuss the wonders of GBA RE, modding, etc. — also the best way to report issues or bugs, or ask for help on your own project: https://discord.gg/khwxFSVNrS

---

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
