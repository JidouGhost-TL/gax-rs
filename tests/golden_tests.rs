//! Golden Integration Tests for Shin'en GAX Sound Engine (gax-rs)
//!
//! Verifies:
//! - Tier 1: Version-aware sample PCM loading (v1 signed vs v3 unsigned `0x80` bias)
//!   and waveform continuity checks verifying zero-crossing smoothness.
//! - Tier 2: Hardware timer reload calculations and engine version detection.
//! - Tier 3: Strict byte-for-byte `AccuracySynth` verification against real mGBA ARM
//!   mixer execution dumps across GAX v1.99d, v3.01, and v3.05 (1:1 pitch, non-unity
//!   resampled pitch, stereo/volume scaling, and multi-voice saturation clamping).

use gax::{AccuracySynth, GaxSampleView, GaxVersion, GaxVoice, GAX_FREQ_TABLE};
#[cfg(feature = "alloc")]
use gax::GaxSample;

const MANIFEST_JSON: &str = include_str!("golden/manifest.json");

const IRIDION3D_SAMPLE0: &[u8] = include_bytes!("golden/v1_iridion3d_sample0.pcm");
const CRASH2_SAMPLE0: &[u8] = include_bytes!("golden/v3_crash2_sample0.pcm");
const IRIDION2_SAMPLE0: &[u8] = include_bytes!("golden/v305_iridion2_sample0.pcm");

const V1_IRIDION3D_MIX_1TO1: &[u8] = include_bytes!("golden/v1_iridion3d_mix_1to1.bin");
const V1_IRIDION3D_MIX_RESAMPLED: &[u8] = include_bytes!("golden/v1_iridion3d_mix_resampled.bin");
const V1_IRIDION3D_MIX_CLAMPED: &[u8] = include_bytes!("golden/v1_iridion3d_mix_clamped.bin");

const V3_CRASH2_MIX_1TO1: &[u8] = include_bytes!("golden/v3_crash2_mix_1to1.bin");
const V3_CRASH2_MIX_RESAMPLED: &[u8] = include_bytes!("golden/v3_crash2_mix_resampled.bin");
const V3_CRASH2_MIX_CLAMPED: &[u8] = include_bytes!("golden/v3_crash2_mix_clamped.bin");

const V305_IRIDION2_MIX_1TO1: &[u8] = include_bytes!("golden/v305_iridion2_mix_1to1.bin");
const V305_IRIDION2_MIX_RESAMPLED: &[u8] = include_bytes!("golden/v305_iridion2_mix_resampled.bin");

/// Mean absolute adjacent sample difference (MAD) for continuity verification.
fn adjacent_mad_signed(data: &[u8]) -> f64 {
    let mut sum: u64 = 0;
    for w in data.windows(2) {
        sum += ((w[1] as i8 as i32) - (w[0] as i8 as i32)).unsigned_abs() as u64;
    }
    sum as f64 / (data.len() - 1) as f64
}

fn adjacent_mad_unsigned(data: &[u8]) -> f64 {
    let mut sum: u64 = 0;
    for w in data.windows(2) {
        let a = (w[0] ^ 0x80) as i8 as i32;
        let b = (w[1] ^ 0x80) as i8 as i32;
        sum += (b - a).unsigned_abs() as u64;
    }
    sum as f64 / (data.len() - 1) as f64
}

#[test]
fn test_manifest_integrity() {
    assert!(!MANIFEST_JSON.is_empty());
    for expected in [
        "v1_iridion3d_sample0.pcm",
        "v3_crash2_sample0.pcm",
        "v305_iridion2_sample0.pcm",
        "v1_iridion3d_mix_1to1.bin",
        "v1_iridion3d_mix_resampled.bin",
        "v1_iridion3d_mix_clamped.bin",
        "v3_crash2_mix_1to1.bin",
        "v3_crash2_mix_resampled.bin",
        "v3_crash2_mix_clamped.bin",
        "v305_iridion2_mix_1to1.bin",
        "v305_iridion2_mix_resampled.bin",
    ] {
        assert!(MANIFEST_JSON.contains(expected), "manifest missing {expected}");
    }
}

// ============================================================================
// Tier 1: Version-aware PCM decoding & waveform continuity
// ============================================================================

#[test]
fn test_tier1_v1_iridion3d_signed_continuity() {
    assert_eq!(IRIDION3D_SAMPLE0.len(), 6688);
    let view = GaxSampleView::from_rom_bytes(0, 42260, GaxVersion::V1, IRIDION3D_SAMPLE0);
    assert!(!view.is_unsigned);
    assert_eq!(view.len(), 6688);
    assert!(!view.is_silent());

    // Signed interpretation must be dramatically smoother than unsigned on GAX v1
    let mad_signed = adjacent_mad_signed(IRIDION3D_SAMPLE0);
    let mad_unsigned = adjacent_mad_unsigned(IRIDION3D_SAMPLE0);
    assert!(
        mad_signed < mad_unsigned,
        "GAX v1 sample should be smoother as signed i8 (signed MAD={mad_signed:.2}, unsigned MAD={mad_unsigned:.2})"
    );
}

#[test]
fn test_tier1_v3_crash2_unsigned_continuity() {
    assert_eq!(CRASH2_SAMPLE0.len(), 5874);
    let view = GaxSampleView::from_rom_bytes(0, 82241, GaxVersion::V3, CRASH2_SAMPLE0);
    assert!(view.is_unsigned);
    assert_eq!(view.len(), 5874);
    assert!(!view.is_silent());

    // Unsigned 0x80-centered decoding eliminates zero-crossing wrap discontinuities
    let mad_signed = adjacent_mad_signed(CRASH2_SAMPLE0);
    let mad_unsigned = adjacent_mad_unsigned(CRASH2_SAMPLE0);
    assert!(
        mad_unsigned * 2.0 < mad_signed,
        "GAX v3.01 sample should be far smoother as unsigned u8 (unsigned MAD={mad_unsigned:.2}, signed MAD={mad_signed:.2})"
    );

    // First sample in Crash 2 attack is near silence (0x80 -> ~0), not near -128
    assert!(
        view.sample_i8(0).abs() <= 8,
        "Expected near-zero attack sample after unsigned decode, got {}",
        view.sample_i8(0)
    );
}

#[test]
fn test_tier1_v305_iridion2_unsigned_continuity() {
    assert_eq!(IRIDION2_SAMPLE0.len(), 2625);
    let view = GaxSampleView::from_rom_bytes(0, 53946, GaxVersion::V3, IRIDION2_SAMPLE0);
    assert!(view.is_unsigned);
    assert_eq!(view.len(), 2625);
    assert!(!view.is_silent());

    let mad_signed = adjacent_mad_signed(IRIDION2_SAMPLE0);
    let mad_unsigned = adjacent_mad_unsigned(IRIDION2_SAMPLE0);
    assert!(
        mad_unsigned * 2.0 < mad_signed,
        "GAX v3.05 sample should be far smoother as unsigned u8 (unsigned MAD={mad_unsigned:.2}, signed MAD={mad_signed:.2})"
    );
}

#[cfg(feature = "alloc")]
#[test]
fn test_tier1_owned_sample_matches_view_all_versions() {
    for (ver, rate, raw) in [
        (GaxVersion::V1, 42260, IRIDION3D_SAMPLE0),
        (GaxVersion::V3, 82241, CRASH2_SAMPLE0),
        (GaxVersion::V3, 53946, IRIDION2_SAMPLE0),
    ] {
        let view = GaxSampleView::from_rom_bytes(0, rate, ver, raw);
        let owned = GaxSample::from_rom_bytes(0, rate, ver, raw);
        assert_eq!(owned.len(), view.len());
        assert_eq!(owned.min_max(), view.min_max());
        for i in 0..view.len() {
            assert_eq!(owned.pcm_data[i], view.sample_i8(i), "mismatch at {i}");
        }
    }
}

// ============================================================================
// Tier 2: Hardware timer reload constants & version detection
// ============================================================================

#[test]
fn test_tier2_hardware_timer_rates() {
    assert_eq!(AccuracySynth::rate_from_timer_reload(0xFE73), 42260);
    assert_eq!(AccuracySynth::rate_from_timer_reload(0xFF34), 82241);
    assert_eq!(AccuracySynth::rate_from_timer_reload(0xFEC9), 53946);
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
fn test_tier2_version_detection_and_pcm_signedness() {
    let v1 = GaxVersion::detect_from_bytes(b"GAX Sound Engine v1.99d (Mar 13 2001) Shin'en").unwrap();
    let v2 = GaxVersion::detect_from_bytes(b"GAX Sound Engine 2.31 (Apr 19 2002) Shin'en").unwrap();
    let v301 = GaxVersion::detect_from_bytes(b"GAX Sound Engine 3.01 (Sep 13 2002) Shin'en").unwrap();
    let v305 = GaxVersion::detect_from_bytes(b"GAX Sound Engine 3.05 (May 20 2003) Shin'en").unwrap();

    assert_eq!(v1, GaxVersion::V1);
    assert_eq!(v2, GaxVersion::V2);
    assert_eq!(v301, GaxVersion::V3);
    assert_eq!(v305, GaxVersion::V3);

    assert!(!v1.is_unsigned_pcm());
    assert!(!v2.is_unsigned_pcm());
    assert!(v301.is_unsigned_pcm());
    assert!(v305.is_unsigned_pcm());
}

// ============================================================================
// Tier 3: True byte-for-byte mGBA ARM golden mixer verification
// ============================================================================

#[test]
fn test_tier3_v1_iridion3d_arm_golden_1to1() {
    assert_eq!(V1_IRIDION3D_MIX_1TO1.len(), 256);
    let view = GaxSampleView::from_rom_bytes(0, 42260, GaxVersion::V1, IRIDION3D_SAMPLE0);

    for (ch, &vol) in [256i32, 128i32].iter().enumerate() {
        let mut acc = [0i16; 128];
        let mut dma = [0i8; 128];
        let mut pos = 0i32;

        AccuracySynth::mix_v1_voice_i16(view.pcm_data, &mut pos, 0x0800, vol, &mut acc, false);
        AccuracySynth::clamp_v1_i16_to_dma8(&acc, &mut dma, 1024);

        let expected = &V1_IRIDION3D_MIX_1TO1[ch * 128..(ch + 1) * 128];
        for i in 0..128 {
            assert_eq!(
                dma[i], expected[i] as i8,
                "GAX v1 1:1 ARM mismatch ch={ch} frame={i}"
            );
        }
    }
}

#[test]
fn test_tier3_v1_iridion3d_arm_golden_resampled() {
    assert_eq!(V1_IRIDION3D_MIX_RESAMPLED.len(), 256);
    let view = GaxSampleView::from_rom_bytes(0, 42260, GaxVersion::V1, IRIDION3D_SAMPLE0);

    for (ch, &vol) in [200i32, 100i32].iter().enumerate() {
        let mut acc = [0i16; 128];
        let mut dma = [0i8; 128];
        let mut pos = 0i32;

        AccuracySynth::mix_v1_voice_i16(view.pcm_data, &mut pos, 0x0C80, vol, &mut acc, false);
        AccuracySynth::clamp_v1_i16_to_dma8(&acc, &mut dma, 1024);

        let expected = &V1_IRIDION3D_MIX_RESAMPLED[ch * 128..(ch + 1) * 128];
        for i in 0..128 {
            assert_eq!(
                dma[i], expected[i] as i8,
                "GAX v1 resampled ARM mismatch ch={ch} frame={i}"
            );
        }
    }
}

#[test]
fn test_tier3_v1_iridion3d_arm_golden_clamped() {
    assert_eq!(V1_IRIDION3D_MIX_CLAMPED.len(), 128);
    let view = GaxSampleView::from_rom_bytes(0, 42260, GaxVersion::V1, IRIDION3D_SAMPLE0);

    let mut acc = [0i16; 128];
    let mut dma = [0i8; 128];
    let mut pos1 = 0i32;
    let mut pos2 = 64i32 << 11;

    AccuracySynth::mix_v1_voice_i16(view.pcm_data, &mut pos1, 0x0800, 384, &mut acc, false);
    AccuracySynth::mix_v1_voice_i16(view.pcm_data, &mut pos2, 0x0800, 384, &mut acc, true);
    AccuracySynth::clamp_v1_i16_to_dma8(&acc, &mut dma, 1024);

    let mut sat_count = 0usize;
    for i in 0..128 {
        assert_eq!(
            dma[i], V1_IRIDION3D_MIX_CLAMPED[i] as i8,
            "GAX v1 clamped ARM mismatch frame={i}"
        );
        if dma[i] == 127 || dma[i] == -128 {
            sat_count += 1;
        }
    }
    assert!(sat_count > 50, "Expected heavy clamping saturation, got {sat_count}");
}

#[test]
fn test_tier3_v3_crash2_arm_golden_1to1() {
    assert_eq!(V3_CRASH2_MIX_1TO1.len(), 256);
    let view = GaxSampleView::from_rom_bytes(0, 82241, GaxVersion::V3, CRASH2_SAMPLE0);
    let loop_len_fixed = (view.len() as u32) << 11;

    for (ch, &vol) in [255u32, 128u32].iter().enumerate() {
        let mut acc = [0u16; 128];
        let mut dma = [0i8; 128];
        let mut pos = 3328u32 << 11;

        AccuracySynth::mix_v3_voice_view_u16(&view, &mut pos, 0x0800, vol, loop_len_fixed, &mut acc);
        AccuracySynth::clamp_v3_u16_to_dma8(&acc, &mut dma, vol);

        let expected = &V3_CRASH2_MIX_1TO1[ch * 128..(ch + 1) * 128];
        for i in 0..128 {
            assert_eq!(
                dma[i], expected[i] as i8,
                "GAX v3.01 1:1 ARM mismatch ch={ch} frame={i}"
            );
        }
    }
}

#[test]
fn test_tier3_v3_crash2_arm_golden_resampled() {
    assert_eq!(V3_CRASH2_MIX_RESAMPLED.len(), 256);
    let view = GaxSampleView::from_rom_bytes(0, 82241, GaxVersion::V3, CRASH2_SAMPLE0);
    let loop_len_fixed = (view.len() as u32) << 11;

    for (ch, &vol) in [200u32, 96u32].iter().enumerate() {
        let mut acc = [0u16; 128];
        let mut dma = [0i8; 128];
        let mut pos = 3328u32 << 11;

        AccuracySynth::mix_v3_voice_view_u16(&view, &mut pos, 0x0B40, vol, loop_len_fixed, &mut acc);
        AccuracySynth::clamp_v3_u16_to_dma8(&acc, &mut dma, vol);

        let expected = &V3_CRASH2_MIX_RESAMPLED[ch * 128..(ch + 1) * 128];
        for i in 0..128 {
            assert_eq!(
                dma[i], expected[i] as i8,
                "GAX v3.01 resampled ARM mismatch ch={ch} frame={i}"
            );
        }
    }
}

#[test]
fn test_tier3_v3_crash2_arm_golden_clamped() {
    assert_eq!(V3_CRASH2_MIX_CLAMPED.len(), 128);
    let view = GaxSampleView::from_rom_bytes(0, 82241, GaxVersion::V3, CRASH2_SAMPLE0);
    let loop_len_fixed = (view.len() as u32) << 11;

    let mut acc = [0u16; 128];
    let mut dma = [0i8; 128];
    let mut pos1 = 3328u32 << 11;
    let mut pos2 = 3328u32 << 11;

    AccuracySynth::mix_v3_voice_view_u16(&view, &mut pos1, 0x0800, 255, loop_len_fixed, &mut acc);
    AccuracySynth::mix_v3_voice_view_u16(&view, &mut pos2, 0x0800, 255, loop_len_fixed, &mut acc);
    AccuracySynth::clamp_v3_u16_to_dma8(&acc, &mut dma, 510);

    let mut sat_pos = 0usize;
    let mut sat_neg = 0usize;
    for i in 0..128 {
        assert_eq!(
            dma[i], V3_CRASH2_MIX_CLAMPED[i] as i8,
            "GAX v3.01 clamped ARM mismatch frame={i}"
        );
        if dma[i] == 127 {
            sat_pos += 1;
        } else if dma[i] == -128 {
            sat_neg += 1;
        }
    }
    assert!(sat_pos > 0 && sat_neg > 0, "Expected both +127 and -128 saturation");
}

#[test]
fn test_tier3_v305_iridion2_arm_golden_1to1() {
    assert_eq!(V305_IRIDION2_MIX_1TO1.len(), 256);
    let view = GaxSampleView::from_rom_bytes(0, 53946, GaxVersion::V3, IRIDION2_SAMPLE0);
    let loop_len_fixed = (view.len() as u32) << 11;

    for (ch, &vol) in [255u32, 160u32].iter().enumerate() {
        let mut acc = [0u16; 128];
        let mut dma = [0i8; 128];
        let mut pos = 1664u32 << 11;

        AccuracySynth::mix_v3_voice_view_u16(&view, &mut pos, 0x0800, vol, loop_len_fixed, &mut acc);
        AccuracySynth::clamp_v3_u16_to_dma8(&acc, &mut dma, vol);

        let expected = &V305_IRIDION2_MIX_1TO1[ch * 128..(ch + 1) * 128];
        for i in 0..128 {
            assert_eq!(
                dma[i], expected[i] as i8,
                "GAX v3.05 1:1 ARM mismatch ch={ch} frame={i}"
            );
        }
    }
}

#[test]
fn test_tier3_v305_iridion2_arm_golden_resampled() {
    assert_eq!(V305_IRIDION2_MIX_RESAMPLED.len(), 256);
    let view = GaxSampleView::from_rom_bytes(0, 53946, GaxVersion::V3, IRIDION2_SAMPLE0);
    let loop_len_fixed = (view.len() as u32) << 11;

    for (ch, &vol) in [220u32, 110u32].iter().enumerate() {
        let mut acc = [0u16; 128];
        let mut dma = [0i8; 128];
        let mut pos = 1664u32 << 11;

        AccuracySynth::mix_v3_voice_view_u16(&view, &mut pos, 0x0640, vol, loop_len_fixed, &mut acc);
        AccuracySynth::clamp_v3_u16_to_dma8(&acc, &mut dma, vol);

        let expected = &V305_IRIDION2_MIX_RESAMPLED[ch * 128..(ch + 1) * 128];
        for i in 0..128 {
            assert_eq!(
                dma[i], expected[i] as i8,
                "GAX v3.05 resampled ARM mismatch ch={ch} frame={i}"
            );
        }
    }
}

#[test]
fn test_tier3_accuracy_synth_view_unsigned_render() {
    let raw_unsigned: [u8; 5] = [0xC0, 0xA0, 0x80, 0x60, 0x40]; // +64, +32, 0, -32, -64
    let view = GaxSampleView::from_rom_bytes(0, 82241, GaxVersion::V3, &raw_unsigned);
    let mut voice = GaxVoice::new(0, 0x1000, 128, 64);
    let mut left = [0i8; 6];
    let mut right = [0i8; 6];

    AccuracySynth::mix_voice_view(&view, &mut voice, &mut left, &mut right);

    assert_eq!(&left[..5], &[64, 32, 0, -32, -64]);
    assert_eq!(&right[..5], &[64, 32, 0, -32, -64]);
    assert_eq!(left[5], 0);
    assert_eq!(right[5], 0);
    assert!(!voice.is_active);
}
