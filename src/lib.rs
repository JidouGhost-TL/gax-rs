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
