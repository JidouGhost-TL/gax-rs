//! Software synthesis engines for GAX.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use crate::sample::GaxSample;
use crate::sample::GaxSampleView;

/// Compute square root of an `f64` value.
///
/// Under `feature = "std"`, delegates to standard library `f64::sqrt()`.
/// Under `#![no_std]`, computes the square root using 6 Newton-Raphson iterations
/// seeded by IEEE-754 exponent halving, achieving full 53-bit double precision.
#[inline]
pub fn sqrt_f64(val: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        val.sqrt()
    }
    #[cfg(not(feature = "std"))]
    {
        if val < 0.0 {
            return f64::NAN;
        }
        if val == 0.0 || val.is_infinite() || val.is_nan() {
            return val;
        }
        let bits = val.to_bits();
        let mut x = f64::from_bits((bits >> 1) + (1023u64 << 51));
        if x <= 0.0 || x.is_nan() {
            x = val;
        }
        for _ in 0..6 {
            x = 0.5 * (x + val / x);
        }
        x
    }
}

/// Exact Shin'en GAX timer reload table reverse-engineered from GAX binaries.
///
/// Each entry is `(sample_rate_hz, timer_period_cycles)`.
/// Timer reload register = `(65536 - timer_period_cycles) as u16`.
pub const GAX_FREQ_TABLE: &[(u32, u32)] = &[
    (5735, 2926),
    (10513, 1596),
    (11469, 1463),
    (13380, 1254),
    (15769, 1064),
    (18158, 924),
    (21025, 798),
    (26760, 627),
    (31537, 532),
    (36316, 462),
    (40138, 418),
    (42049, 399),
];

/// A single active playback voice in the GAX software synthesizer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GaxVoice {
    pub sample_id: u32,
    /// Fixed-point position in sample: integer part in high bits, 12-bit fractional phase in low bits.
    pub position_fixed: u32,
    /// Phase increment per output sample (12-bit fixed point fractional step).
    pub pitch_step: u32,
    /// Volume (0..128).
    pub volume: u8,
    /// Panning (0..128, 64 = center).
    pub pan: u8,
    pub is_active: bool,
}

impl GaxVoice {
    pub fn new(sample_id: u32, pitch_step: u32, volume: u8, pan: u8) -> Self {
        Self {
            sample_id,
            position_fixed: 0,
            pitch_step,
            volume: volume.min(128),
            pan: pan.min(128),
            is_active: true,
        }
    }
}

/// Bit-exact software mixer matching the ARM driver's DMA ring buffer loop.
#[derive(Debug, Clone)]
pub struct AccuracySynth {
    pub sample_rate: u32,
    pub channels: u8,
    #[cfg(feature = "alloc")]
    pub voices: Vec<GaxVoice>,
}

impl Default for AccuracySynth {
    fn default() -> Self {
        Self::new(13380)
    }
}

impl AccuracySynth {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            channels: 32,
            #[cfg(feature = "alloc")]
            voices: Vec::new(),
        }
    }

    /// Convert a sample rate in Hz to the GBA 16.78 MHz timer reload constant.
    ///
    /// Returns `0` if `rate == 0`.
    pub fn timer_reload_for_rate(rate: u32) -> u16 {
        if rate == 0 {
            return 0;
        }
        for &(r, cycles) in GAX_FREQ_TABLE {
            if r == rate {
                return (65536u32 - cycles) as u16;
            }
        }
        let cycles = 16_777_216u32.saturating_add(rate / 2) / rate;
        (65536u32.saturating_sub(cycles)) as u16
    }

    /// Convert a GBA timer reload constant back to sample rate in Hz.
    pub fn rate_from_timer_reload(reload: u16) -> u32 {
        let cycles = 65536u32.saturating_sub(reload as u32);
        if cycles == 0 {
            return 0;
        }
        for &(r, c) in GAX_FREQ_TABLE {
            if c == cycles {
                return r;
            }
        }
        (16_777_216u32 + cycles / 2) / cycles
    }

    /// Add an active voice to the synthesizer.
    #[cfg(feature = "alloc")]
    pub fn add_voice(&mut self, voice: GaxVoice) {
        self.voices.push(voice);
    }

    /// Clear all active voices.
    #[cfg(feature = "alloc")]
    pub fn clear_voices(&mut self) {
        self.voices.clear();
    }

    /// Mix a single 8-bit PCM voice into stereo 8-bit DMA ring buffers.
    pub fn mix_voice_pcm8(
        pcm_data: &[i8],
        voice: &mut GaxVoice,
        out_left: &mut [i8],
        out_right: &mut [i8],
    ) {
        if !voice.is_active || pcm_data.is_empty() {
            return;
        }

        let len = out_left.len().min(out_right.len());
        if len == 0 {
            return;
        }
        let pcm_len = pcm_data.len();

        let l_pan = (128 - voice.pan as i32) * (voice.volume as i32);
        let r_pan = (voice.pan as i32) * (voice.volume as i32);

        let out_left = &mut out_left[..len];
        let out_right = &mut out_right[..len];

        // 1:1 pitch step fast path
        if voice.pitch_step == 0x1000 {
            let start_pos = (voice.position_fixed >> 12) as usize;
            if start_pos >= pcm_len {
                voice.is_active = false;
                return;
            }
            let count = len.min(pcm_len - start_pos);
            let pcm_sub = &pcm_data[start_pos..start_pos + count];
            let out_l_sub = &mut out_left[..count];
            let out_r_sub = &mut out_right[..count];

            for ((out_l, out_r), &s_byte) in out_l_sub.iter_mut().zip(out_r_sub.iter_mut()).zip(pcm_sub) {
                let s = s_byte as i32;
                let l_sample = ((s * l_pan) >> 13) as i8;
                let r_sample = ((s * r_pan) >> 13) as i8;

                *out_l = out_l.saturating_add(l_sample);
                *out_r = out_r.saturating_add(r_sample);
            }

            voice.position_fixed = voice.position_fixed.wrapping_add((count as u32) << 12);
            if (voice.position_fixed >> 12) as usize >= pcm_len {
                voice.is_active = false;
            }
            return;
        }

        // Resampling path: subslice chunks eliminate bounds checks on out_left and out_right
        for i in 0..len {
            let int_pos = (voice.position_fixed >> 12) as usize;

            if int_pos >= pcm_len {
                voice.is_active = false;
                break;
            }

            let s = pcm_data[int_pos] as i32;

            let l_sample = ((s * l_pan) >> 13) as i8;
            let r_sample = ((s * r_pan) >> 13) as i8;

            out_left[i] = out_left[i].saturating_add(l_sample);
            out_right[i] = out_right[i].saturating_add(r_sample);

            voice.position_fixed = voice.position_fixed.wrapping_add(voice.pitch_step);
        }

        if (voice.position_fixed >> 12) as usize >= pcm_len {
            voice.is_active = false;
        }
    }

    /// Mix a voice playing a borrowed sample view into stereo 8-bit DMA ring buffers.
    ///
    /// Transparently decodes unsigned 8-bit PCM (`b ^ 0x80`) when `sample.is_unsigned` is set.
    pub fn mix_voice_view(
        sample: &GaxSampleView<'_>,
        voice: &mut GaxVoice,
        out_left: &mut [i8],
        out_right: &mut [i8],
    ) {
        if !sample.is_unsigned {
            Self::mix_voice_pcm8(sample.pcm_data, voice, out_left, out_right);
            return;
        }
        if !voice.is_active || sample.is_empty() {
            return;
        }
        let len = out_left.len().min(out_right.len());
        if len == 0 {
            return;
        }
        let pcm_len = sample.len();
        let l_pan = (128 - voice.pan as i32) * (voice.volume as i32);
        let r_pan = (voice.pan as i32) * (voice.volume as i32);

        for i in 0..len {
            let int_pos = (voice.position_fixed >> 12) as usize;
            if int_pos >= pcm_len {
                voice.is_active = false;
                break;
            }
            let s = sample.sample_i8(int_pos) as i32;
            let l_sample = ((s * l_pan) >> 13) as i8;
            let r_sample = ((s * r_pan) >> 13) as i8;
            out_left[i] = out_left[i].saturating_add(l_sample);
            out_right[i] = out_right[i].saturating_add(r_sample);
            voice.position_fixed = voice.position_fixed.wrapping_add(voice.pitch_step);
        }
        if (voice.position_fixed >> 12) as usize >= pcm_len {
            voice.is_active = false;
        }
    }

    /// Bit-exact GAX v1.x ARM voice mixer (`0x08021dc8` overwrite / `0x08021eb4` accumulate).
    ///
    /// - Fetches signed 8-bit PCM (`ldrsb`) using Q21.11 fixed-point position (`pos_fixed >> 11`, 1.0x = `0x800`).
    /// - Scales each sample via `(s * volume) >> 8` (arithmetic shift right).
    /// - Overwrites or accumulates (wrapping 16-bit) into `out_acc`.
    pub fn mix_v1_voice_i16(
        pcm_data: &[i8],
        pos_fixed: &mut i32,
        pitch_step: i32,
        volume: i32,
        out_acc: &mut [i16],
        accumulate: bool,
    ) {
        let pcm_len = pcm_data.len();
        let mut pos = *pos_fixed;
        for slot in out_acc.iter_mut() {
            if pos < 0 {
                break;
            }
            let idx = (pos >> 11) as usize;
            if idx >= pcm_len {
                break;
            }
            let s = pcm_data[idx] as i32;
            let scaled = s.wrapping_mul(volume) >> 8;
            if accumulate {
                *slot = (*slot as i32).wrapping_add(scaled) as i16;
            } else {
                *slot = scaled as i16;
            }
            pos = pos.wrapping_add(pitch_step);
        }
        *pos_fixed = pos;
    }

    /// Bit-exact GAX v1.x ARM master-volume & DirectSound 8-bit clamp routine (`0x08021c90`).
    ///
    /// Computes `((acc[i] as i32) * master_vol) >> 10` and clamps to `[-128, 127]`.
    pub fn clamp_v1_i16_to_dma8(acc: &[i16], out_dma: &mut [i8], master_vol: i32) {
        let len = acc.len().min(out_dma.len());
        for i in 0..len {
            let v = (acc[i] as i32).wrapping_mul(master_vol) >> 10;
            out_dma[i] = v.clamp(-128, 127) as i8;
        }
    }

    /// Bit-exact GAX v3.x ARM voice mixer (`0x080477bc` in v3.01 / `0x0801c23c` in v3.05).
    ///
    /// - Fetches unsigned 8-bit PCM (`ldrb`, `0x80` center) using Q21.11 fixed-point position
    ///   (`pos_fixed >> 11`, 1.0x = `0x800`), wrapping by `loop_len_fixed` when reaching `pcm_u8.len() << 11`.
    /// - Scales each unsigned sample via `(u * volume) >> 3` (matching packed 2x16-bit SIMD `mul` + `bic #0x70000` + `lsr #3`)
    ///   and accumulates into the 16-bit unsigned intermediate buffer `out_acc`.
    pub fn mix_v3_voice_u16(
        pcm_u8: &[u8],
        pos_fixed: &mut u32,
        pitch_step: u32,
        volume: u32,
        loop_len_fixed: u32,
        out_acc: &mut [u16],
    ) {
        if pcm_u8.is_empty() {
            return;
        }
        let pcm_len = pcm_u8.len();
        let end_fixed = (pcm_len.min((u32::MAX >> 11) as usize) as u32) << 11;
        let mut pos = *pos_fixed;
        for slot in out_acc.iter_mut() {
            let idx = (pos >> 11) as usize;
            if idx >= pcm_len {
                break;
            }
            let u = pcm_u8[idx] as u32;
            let scaled = (u.wrapping_mul(volume) >> 3) as u16;
            *slot = slot.wrapping_add(scaled);
            pos = pos.wrapping_add(pitch_step);
            if pos >= end_fixed && loop_len_fixed > 0 {
                pos = pos.wrapping_sub(loop_len_fixed);
                if pos >= end_fixed && loop_len_fixed <= end_fixed {
                    let loop_start = end_fixed - loop_len_fixed;
                    pos = loop_start + ((pos - loop_start) % loop_len_fixed);
                }
            }
        }
        *pos_fixed = pos;
    }

    /// Bit-exact GAX v3.x ARM voice mixer operating directly on a `GaxSampleView`.
    pub fn mix_v3_voice_view_u16(
        sample: &GaxSampleView<'_>,
        pos_fixed: &mut u32,
        pitch_step: u32,
        volume: u32,
        loop_len_fixed: u32,
        out_acc: &mut [u16],
    ) {
        if sample.is_empty() {
            return;
        }
        if sample.is_unsigned {
            Self::mix_v3_voice_u16(
                sample.as_bytes(),
                pos_fixed,
                pitch_step,
                volume,
                loop_len_fixed,
                out_acc,
            );
            return;
        }
        let pcm_len = sample.len();
        let end_fixed = (pcm_len.min((u32::MAX >> 11) as usize) as u32) << 11;
        let mut pos = *pos_fixed;
        for slot in out_acc.iter_mut() {
            let idx = (pos >> 11) as usize;
            if idx >= pcm_len {
                break;
            }
            let u = sample.sample_u8(idx) as u32;
            let scaled = (u.wrapping_mul(volume) >> 3) as u16;
            *slot = slot.wrapping_add(scaled);
            pos = pos.wrapping_add(pitch_step);
            if pos >= end_fixed && loop_len_fixed > 0 {
                pos = pos.wrapping_sub(loop_len_fixed);
                if pos >= end_fixed && loop_len_fixed <= end_fixed {
                    let loop_start = end_fixed - loop_len_fixed;
                    pos = loop_start + ((pos - loop_start) % loop_len_fixed);
                }
            }
        }
        *pos_fixed = pos;
    }

    /// Bit-exact GAX v3.x ARM DC-bias subtraction & DirectSound 8-bit clamp routine
    /// (`0x080474c0` in v3.01 / `0x0801bf40` in v3.05).
    ///
    /// Subtracts `total_volume << 4` (the accumulated `0x80` unsigned center bias across all mixed voices),
    /// clamps to `[-0xFE1, +0xFE0]` (`cmn #0xfe0; mvnlt #0xfe0; cmp #0xfe0; movgt #0xfe0`),
    /// and arithmetically shifts right by 5 bits (`asr #5`) into signed DirectSound 8-bit PCM.
    pub fn clamp_v3_u16_to_dma8(acc: &[u16], out_dma: &mut [i8], total_volume: u32) {
        let len = acc.len().min(out_dma.len());
        let bias = total_volume.wrapping_shl(4) as i32;
        for i in 0..len {
            let mut x = (acc[i] as i32).wrapping_sub(bias);
            if x < -0xFE0 {
                x = !0xFE0; // -0xFE1 -> (-4065 >> 5) == -128
            } else if x > 0xFE0 {
                x = 0xFE0; // +4064 -> (+4064 >> 5) == +127
            }
            out_dma[i] = (x >> 5) as i8;
        }
    }

    /// Mix active voices from a slice into stereo 8-bit DMA ring buffers using borrowed sample views.
    ///
    /// Completely zero-allocation and suitable for `#![no_std]` bare-metal ROM playback.
    pub fn mix_voices_with_views(
        voices: &mut [GaxVoice],
        samples: &[GaxSampleView<'_>],
        out_left: &mut [i8],
        out_right: &mut [i8],
    ) {
        for voice in voices.iter_mut() {
            if !voice.is_active {
                continue;
            }
            if let Some(sample) = samples.iter().find(|s| s.id == voice.sample_id) {
                Self::mix_voice_view(sample, voice, out_left, out_right);
            }
        }
    }

    /// Mix all active voices into the destination stereo buffers.
    #[cfg(feature = "alloc")]
    pub fn mix_buffer(
        &mut self,
        samples: &[GaxSample],
        out_left: &mut [i8],
        out_right: &mut [i8],
    ) {
        for voice in &mut self.voices {
            if !voice.is_active {
                continue;
            }
            if let Some(sample) = samples.iter().find(|s| s.id == voice.sample_id) {
                Self::mix_voice_pcm8(&sample.pcm_data, voice, out_left, out_right);
            }
        }
    }

    /// Mix all active voices into destination stereo buffers using borrowed sample views.
    #[cfg(feature = "alloc")]
    pub fn mix_buffer_views(
        &mut self,
        samples: &[GaxSampleView<'_>],
        out_left: &mut [i8],
        out_right: &mut [i8],
    ) {
        Self::mix_voices_with_views(&mut self.voices, samples, out_left, out_right);
    }

    /// Compute Pearson correlation coefficient between two audio streams.
    pub fn correlation(a: &[i8], b: &[i8]) -> f64 {
        let n = a.len().min(b.len());
        if n == 0 {
            return 0.0;
        }

        let mut sum_a = 0.0;
        let mut sum_b = 0.0;
        for i in 0..n {
            sum_a += a[i] as f64;
            sum_b += b[i] as f64;
        }
        let mean_a = sum_a / n as f64;
        let mean_b = sum_b / n as f64;

        let mut num = 0.0;
        let mut den_a = 0.0;
        let mut den_b = 0.0;

        for i in 0..n {
            let da = a[i] as f64 - mean_a;
            let db = b[i] as f64 - mean_b;
            num += da * db;
            den_a += da * da;
            den_b += db * db;
        }

        let denom = sqrt_f64(den_a * den_b);
        if denom < 1e-12 {
            if den_a < 1e-12 && den_b < 1e-12 {
                1.0
            } else {
                0.0
            }
        } else {
            num / denom
        }
    }

    /// Root Mean Square Error (RMSE) between two audio signals.
    pub fn rmse(a: &[i8], b: &[i8]) -> f64 {
        let n = a.len().min(b.len());
        if n == 0 {
            return 0.0;
        }
        let sum_sq: f64 = a[..n].iter().zip(&b[..n]).map(|(&x, &y)| {
            let diff = x as f64 - y as f64;
            diff * diff
        }).sum();
        sqrt_f64(sum_sq / n as f64)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StudioSynth {
    pub sample_rate: u32,
    pub sinc_quality: u8,
}

impl StudioSynth {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            sinc_quality: 64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gax_freq_table_lookup() {
        assert_eq!(AccuracySynth::timer_reload_for_rate(10513), (65536 - 1596) as u16);
        assert_eq!(AccuracySynth::timer_reload_for_rate(13380), (65536 - 1254) as u16);
        assert_eq!(AccuracySynth::timer_reload_for_rate(18158), (65536 - 924) as u16);
        assert_eq!(AccuracySynth::timer_reload_for_rate(26760), (65536 - 627) as u16);
    }

    #[test]
    fn test_gax_rate_from_reload() {
        let reload = (65536 - 1254) as u16;
        assert_eq!(AccuracySynth::rate_from_timer_reload(reload), 13380);
    }

    #[test]
    fn test_mix_voice_pcm8() {
        let pcm = [64i8, 32i8, -32i8];
        let mut voice = GaxVoice::new(0, 0x1000, 128, 64); // Center pan, full volume
        let mut left = [0i8; 3];
        let mut right = [0i8; 3];

        AccuracySynth::mix_voice_pcm8(&pcm, &mut voice, &mut left, &mut right);

        assert_eq!(left[0], 64);
        assert_eq!(right[0], 64);
        assert_eq!(left[1], 32);
        assert_eq!(right[1], 32);
        assert!(!voice.is_active);
    }

    #[test]
    fn test_correlation_identical() {
        let sig = [10i8, -20, 30, -40, 50];
        let corr = AccuracySynth::correlation(&sig, &sig);
        assert!((corr - 1.0).abs() < 1e-6);
        assert_eq!(AccuracySynth::rmse(&sig, &sig), 0.0);
    }

    #[test]
    fn test_sqrt_f64_accuracy() {
        assert_eq!(sqrt_f64(0.0), 0.0);
        assert!(sqrt_f64(-5.0).is_nan());
        assert!((sqrt_f64(1.0) - 1.0).abs() < 1e-12);
        assert!((sqrt_f64(4.0) - 2.0).abs() < 1e-12);
        assert!((sqrt_f64(9.0) - 3.0).abs() < 1e-12);
        assert!((sqrt_f64(16.0) - 4.0).abs() < 1e-12);
        assert!((sqrt_f64(100.0) - 10.0).abs() < 1e-12);
        assert!((sqrt_f64(2.0) - core::f64::consts::SQRT_2).abs() < 1e-12);
        assert!((sqrt_f64(0.25) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn test_mix_voices_with_views() {
        let pcm = [40i8, -40i8];
        let view = GaxSampleView::new(1, 13380, &pcm);
        let mut voices = [GaxVoice::new(1, 0x1000, 128, 64)];
        let samples = [view];
        let mut left = [0i8; 2];
        let mut right = [0i8; 2];

        AccuracySynth::mix_voices_with_views(&mut voices, &samples, &mut left, &mut right);

        assert_eq!(left[0], 40);
        assert_eq!(right[0], 40);
        assert_eq!(left[1], -40);
        assert_eq!(right[1], -40);
        assert!(!voices[0].is_active);
    }

    #[test]
    fn test_malformed_inputs_do_not_panic() {
        assert_eq!(AccuracySynth::timer_reload_for_rate(0), 0);
        let _ = AccuracySynth::timer_reload_for_rate(u32::MAX);

        let pcm_i8 = [10i8, 20, 30, 40];
        let mut neg_pos = -4096i32;
        let mut acc_i16 = [0i16; 8];
        AccuracySynth::mix_v1_voice_i16(&pcm_i8, &mut neg_pos, 0x0800, i32::MAX, &mut acc_i16, false);

        let pcm_u8 = [0x80u8, 0x90, 0xA0, 0xB0];
        let mut oob_pos = 100u32 << 11;
        let mut acc_u16 = [0u16; 8];
        AccuracySynth::mix_v3_voice_u16(&pcm_u8, &mut oob_pos, 0x0800, u32::MAX, 0, &mut acc_u16);

        let mut oneshot_pos = 0u32;
        AccuracySynth::mix_v3_voice_u16(&pcm_u8, &mut oneshot_pos, 0x0800, 255, 0, &mut acc_u16);

        let mut dma = [0i8; 8];
        AccuracySynth::clamp_v3_u16_to_dma8(&acc_u16, &mut dma, u32::MAX);
    }
}
