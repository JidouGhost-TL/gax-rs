//! # gax-rs
//!
//! Pure Rust, zero-dependency parser, software synthesizer, and golden-tested
//! emulator for the **Shin'en GAX Sound Engine** on the Game Boy Advance.
//!
//! GAX was developed by Manfred Linzner and Bernhard Wodok at Shin'en Multimedia,
//! powering over 100 GBA soundtracks including *Iridion 3D*, *Iridion II*,
//! *Crash Bandicoot 2: N-Tranced*, *Payback*, and *Maya the Bee*.
//!
//! ## Features
//! - `#![no_std]` compatible (allocator required for parsing dynamic sequences).
//! - Dual synthesis pipelines:
//!   - `AccuracySynth`: Bit-exact fixed-point software mixer matching the ARM driver's DMA ring buffer.
//!   - `StudioSynth`: Audiophile 48 kHz 32-bit floating point rendering with band-limited sinc interpolation.
//! - Golden test suite verified against cycle-accurate mGBA ARM driver execution traces.
//!
//! ## Quickstart
//!
//! ```rust
//! use gax::{AccuracySynth, GaxSampleView, GaxVoice};
//!
//! // Wrap ROM bytes directly without heap allocation (`#![no_std]`)
//! let pcm_bytes: &[u8] = &[0x00, 0x10, 0x20, 0x10, 0x00, 0xF0, 0xE0, 0xF0];
//! let sample = GaxSampleView::from_raw_bytes(0, 18158, pcm_bytes);
//!
//! // 1:1 pitch step (0x1000), full volume (128), centered pan (64)
//! let mut voice = GaxVoice::new(0, 0x1000, 128, 64);
//! let mut dma_left = [0i8; 8];
//! let mut dma_right = [0i8; 8];
//!
//! AccuracySynth::mix_voice_view(&sample, &mut voice, &mut dma_left, &mut dma_right);
//! assert_eq!(dma_left[2], 0x20);
//! assert_eq!(dma_right[2], 0x20);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod error;
pub mod sample;
pub mod sequence;
pub mod synth;
pub mod version;

pub use error::{Error, Result};
pub use sample::{GaxSample, GaxSampleView};
pub use sequence::{GaxEvent, GaxPattern, GaxSong};
pub use synth::{sqrt_f64, AccuracySynth, GaxVoice, StudioSynth, GAX_FREQ_TABLE};
pub use version::GaxVersion;
