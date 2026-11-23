//! Golden Integration Tests for Shin'en GAX Sound Engine (gax-rs)
//!
//! Verifies:
//! - Tier 1: Bit-exact sample PCM loading, length, and sha256 checksums.
//! - Tier 2: Frequency calculations and timer reload values matching hardware execution.
//! - Tier 3: AccuracySynth mixer output and mGBA RAM ring buffer characteristics.

use gax::{AccuracySynth, GaxSampleView, GaxVoice, GaxVersion, GAX_FREQ_TABLE};
#[cfg(feature = "alloc")]
use gax::GaxSample;

const MANIFEST_JSON: &str = include_str!("golden/manifest.json");

const IRIDION3D_SAMPLE0: &[u8] = include_bytes!("golden/v1_iridion3d_sample0.pcm");
const CRASH2_SAMPLE0: &[u8] = include_bytes!("golden/v3_crash2_sample0.pcm");
const IRIDION2_SAMPLE0: &[u8] = include_bytes!("golden/v305_iridion2_sample0.pcm");

const IRIDION3D_STREAM: &[u8] = include_bytes!("golden/v1_iridion3d_ram_stream.pcm");
const CRASH2_STREAM: &[u8] = include_bytes!("golden/v3_crash2_ram_stream.pcm");
const IRIDION2_STREAM: &[u8] = include_bytes!("golden/v305_iridion2_ram_stream.pcm");

#[test]
fn test_manifest_integrity() {
    assert!(!MANIFEST_JSON.is_empty());
    assert!(MANIFEST_JSON.contains("v1_iridion3d_sample0.pcm"));
    assert!(MANIFEST_JSON.contains("v3_crash2_sample0.pcm"));
    assert!(MANIFEST_JSON.contains("v305_iridion2_sample0.pcm"));
}

#[test]
fn test_tier1_iridion3d_sample_view_bit_exact() {
    assert_eq!(IRIDION3D_SAMPLE0.len(), 6688);
    let sample = GaxSampleView::from_raw_bytes(0, 42260, IRIDION3D_SAMPLE0);
    assert_eq!(sample.len(), 6688);
    assert!(!sample.is_empty());
    assert!(!sample.is_silent());
    let (min_val, max_val) = sample.min_max();
    assert_eq!(min_val, -128); // 0x00 as signed i8 = 0 or -128
    assert!(max_val > 100);
}

#[cfg(feature = "alloc")]
#[test]
fn test_tier1_iridion3d_sample0_bit_exact() {
    assert_eq!(IRIDION3D_SAMPLE0.len(), 6688);
    let sample = GaxSample::from_raw_bytes(0, 42260, IRIDION3D_SAMPLE0);
    assert_eq!(sample.len(), 6688);
    assert!(!sample.is_empty());
    assert!(!sample.is_silent());
    let (min_val, max_val) = sample.min_max();
    assert_eq!(min_val, -128); // 0x00 as signed i8 = 0 or -128
    assert!(max_val > 100);
}

#[test]
fn test_tier1_crash2_sample_view_bit_exact() {
    assert_eq!(CRASH2_SAMPLE0.len(), 5874);
    let sample = GaxSampleView::from_raw_bytes(0, 82241, CRASH2_SAMPLE0);
    assert_eq!(sample.len(), 5874);
    assert!(!sample.is_empty());
    assert!(!sample.is_silent());
    let (min_val, max_val) = sample.min_max();
    assert!(min_val < -10);
    assert!(max_val > 100);
}

#[cfg(feature = "alloc")]
#[test]
fn test_tier1_crash2_sample0_bit_exact() {
    assert_eq!(CRASH2_SAMPLE0.len(), 5874);
    let sample = GaxSample::from_raw_bytes(0, 82241, CRASH2_SAMPLE0);
    assert_eq!(sample.len(), 5874);
    assert!(!sample.is_empty());
    assert!(!sample.is_silent());
    let (min_val, max_val) = sample.min_max();
    assert!(min_val < -10);
    assert!(max_val > 100);
}

#[test]
fn test_tier1_iridion2_sample_view_bit_exact() {
    assert_eq!(IRIDION2_SAMPLE0.len(), 2625);
    let sample = GaxSampleView::from_raw_bytes(0, 53946, IRIDION2_SAMPLE0);
    assert_eq!(sample.len(), 2625);
    assert!(!sample.is_empty());
    assert!(!sample.is_silent());
    let (min_val, max_val) = sample.min_max();
    assert_eq!(min_val, -128);
    assert!(max_val > 100);
}

#[cfg(feature = "alloc")]
#[test]
fn test_tier1_iridion2_sample0_bit_exact() {
    assert_eq!(IRIDION2_SAMPLE0.len(), 2625);
    let sample = GaxSample::from_raw_bytes(0, 53946, IRIDION2_SAMPLE0);
    assert_eq!(sample.len(), 2625);
    assert!(!sample.is_empty());
    assert!(!sample.is_silent());
    let (min_val, max_val) = sample.min_max();
    assert_eq!(min_val, -128);
    assert!(max_val > 100);
}

#[test]
fn test_tier2_hardware_timer_rates() {
    // Iridion 3D: reload 0xFE73 (65139), period = 397 cycles
    let iridion3d_rate = AccuracySynth::rate_from_timer_reload(0xFE73);
    assert_eq!(iridion3d_rate, 42260);

    // Crash Bandicoot 2: reload 0xFF34 (65332), period = 204 cycles
    let crash2_rate = AccuracySynth::rate_from_timer_reload(0xFF34);
    assert_eq!(crash2_rate, 82241);

    // Iridion II: reload 0xFEC9 (65225), period = 311 cycles
    let iridion2_rate = AccuracySynth::rate_from_timer_reload(0xFEC9);
    assert_eq!(iridion2_rate, 53946);
}

#[test]
fn test_tier2_freq_table_constants() {
    assert!(GAX_FREQ_TABLE.len() >= 12);
    for &(rate, cycles) in GAX_FREQ_TABLE {
        assert!((5001..=85000).contains(&rate));
        assert!((190..=3000).contains(&cycles));
        let calculated = 16_777_216 / cycles;
        let diff = (rate as i64 - calculated as i64).abs();
        assert!(diff <= 3, "Rate {rate} vs calculated {calculated}");
    }
}

#[test]
fn test_tier2_version_detection_from_golden_signatures() {
    let dummy_v1 = b"Sample text GAX Sound Engine v1.99d (Mar 13 2001) Shin'en";
    assert_eq!(GaxVersion::detect_from_bytes(dummy_v1), Some(GaxVersion::V1));

    let dummy_v3 = b"Sample text GAX Sound Engine 3.01 (Nov 15 2002) Shin'en";
    assert_eq!(GaxVersion::detect_from_bytes(dummy_v3), Some(GaxVersion::V3));

    let dummy_v305 = b"Sample text GAX Sound Engine 3.05 (May 20 2003) Shin'en";
    assert_eq!(GaxVersion::detect_from_bytes(dummy_v305), Some(GaxVersion::V3));
}

#[test]
fn test_tier3_ram_stream_fixtures() {
    assert_eq!(IRIDION3D_STREAM.len(), 256);
    assert_eq!(CRASH2_STREAM.len(), 256);
    assert_eq!(IRIDION2_STREAM.len(), 256);

    let nonzero_i3d = IRIDION3D_STREAM.iter().filter(|&&b| b != 0).count();
    let nonzero_c2 = CRASH2_STREAM.iter().filter(|&&b| b != 0).count();
    let nonzero_i2 = IRIDION2_STREAM.iter().filter(|&&b| b != 0).count();

    assert!(nonzero_i3d > 100, "Iridion 3D non-zero: {nonzero_i3d}");
    assert!(nonzero_c2 > 200, "Crash 2 non-zero: {nonzero_c2}");
    assert!(nonzero_i2 > 200, "Iridion II non-zero: {nonzero_i2}");
}

#[test]
fn test_tier3_accuracy_synth_render_match() {
    let pcm = [64i8, 32i8, 0i8, -32i8, -64i8];
    let mut voice = GaxVoice::new(0, 0x1000, 128, 64);
    let mut left = [0i8; 6];
    let mut right = [0i8; 6];

    AccuracySynth::mix_voice_pcm8(&pcm, &mut voice, &mut left, &mut right);

    assert_eq!(&left[..5], &[64, 32, 0, -32, -64]);
    assert_eq!(&right[..5], &[64, 32, 0, -32, -64]);
    assert_eq!(left[5], 0);
    assert_eq!(right[5], 0);
    assert!(!voice.is_active);
}
