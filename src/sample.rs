//! GAX Sample structures and PCM decoders.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use crate::version::GaxVersion;

/// Borrowed zero-allocation view of a GAX sample.
///
/// Enables `#![no_std]` applications to play samples directly from ROM or static buffers
/// without requiring a heap allocator (`feature = "alloc"`).
///
/// Tracks whether the underlying ROM bytes are unsigned 8-bit PCM (GAX v3.x, `0x80` center)
/// or signed 8-bit PCM (GAX v1.x/v2.x, `0x00` center), decoding `b ^ 0x80` transparently
/// when accessing centered signed sample values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GaxSampleView<'a> {
    pub id: u32,
    pub sample_rate: u32,
    pub loop_start: Option<u32>,
    pub loop_end: Option<u32>,
    pub pcm_data: &'a [i8],
    /// `true` when `pcm_data` backs raw unsigned 8-bit ROM bytes (`0x80` center, GAX v3.x).
    pub is_unsigned: bool,
}

impl<'a> GaxSampleView<'a> {
    /// Construct a new sample view from a borrowed slice of signed 8-bit PCM data.
    pub const fn new(id: u32, sample_rate: u32, pcm_data: &'a [i8]) -> Self {
        Self {
            id,
            sample_rate,
            loop_start: None,
            loop_end: None,
            pcm_data,
            is_unsigned: false,
        }
    }

    /// Construct a `GaxSampleView` from a raw byte slice (interpreting bytes as signed 8-bit PCM).
    #[allow(unsafe_code)]
    pub const fn from_raw_bytes(id: u32, sample_rate: u32, data: &'a [u8]) -> Self {
        // SAFETY:
        // `u8` and `i8` have identical memory layout, size (1 byte), and alignment (1 byte).
        // Every bit pattern in `[0x00..=0xFF]` represents a valid value for both `u8` and `i8`.
        // The slice reference `data` is valid, non-null, properly aligned, and points to
        // `data.len()` continuous initialized bytes that outlive lifetime `'a`.
        let pcm_data = unsafe {
            core::slice::from_raw_parts(data.as_ptr().cast::<i8>(), data.len())
        };
        Self {
            id,
            sample_rate,
            loop_start: None,
            loop_end: None,
            pcm_data,
            is_unsigned: false,
        }
    }

    /// Construct a version-aware `GaxSampleView` from raw ROM bytes without allocating.
    ///
    /// For GAX v3.x (`version.is_unsigned_pcm() == true`), the view records `is_unsigned = true`
    /// and transparently decodes `b ^ 0x80` into centered signed 8-bit PCM on access.
    #[allow(unsafe_code)]
    pub const fn from_rom_bytes(
        id: u32,
        sample_rate: u32,
        version: GaxVersion,
        data: &'a [u8],
    ) -> Self {
        // SAFETY: Same layout/validity invariants as `from_raw_bytes`.
        let pcm_data = unsafe {
            core::slice::from_raw_parts(data.as_ptr().cast::<i8>(), data.len())
        };
        Self {
            id,
            sample_rate,
            loop_start: None,
            loop_end: None,
            pcm_data,
            is_unsigned: version.is_unsigned_pcm(),
        }
    }

    /// Read sample at `idx` as centered signed 8-bit PCM (`0` = silence).
    #[inline]
    pub const fn sample_i8(&self, idx: usize) -> i8 {
        let raw = self.pcm_data[idx] as u8;
        if self.is_unsigned {
            (raw ^ 0x80) as i8
        } else {
            raw as i8
        }
    }

    /// Read sample at `idx` as unsigned 8-bit PCM (`0x80` = silence, as used by GAX v3 ARM mixer).
    #[inline]
    pub const fn sample_u8(&self, idx: usize) -> u8 {
        let raw = self.pcm_data[idx] as u8;
        if self.is_unsigned {
            raw
        } else {
            raw ^ 0x80
        }
    }

    /// Number of audio samples.
    pub const fn len(&self) -> usize {
        self.pcm_data.len()
    }

    /// Whether the sample contains zero frames.
    pub const fn is_empty(&self) -> bool {
        self.pcm_data.is_empty()
    }

    /// Return min and max centered signed 8-bit sample values.
    pub fn min_max(&self) -> (i8, i8) {
        if self.pcm_data.is_empty() {
            return (0, 0);
        }
        let mut min_val = self.sample_i8(0);
        let mut max_val = min_val;
        for i in 1..self.pcm_data.len() {
            let s = self.sample_i8(i);
            if s < min_val {
                min_val = s;
            }
            if s > max_val {
                max_val = s;
            }
        }
        (min_val, max_val)
    }

    /// Check if all centered samples are zero (silence).
    pub fn is_silent(&self) -> bool {
        let silence_raw = if self.is_unsigned { 0x80u8 as i8 } else { 0i8 };
        self.pcm_data.iter().all(|&s| s == silence_raw)
    }

    /// View underlying PCM storage as raw byte slice.
    #[allow(unsafe_code)]
    pub const fn as_bytes(&self) -> &'a [u8] {
        // SAFETY:
        // `i8` and `u8` have identical memory layout, size (1 byte), and alignment (1 byte).
        // Every bit pattern in `[0x00..=0xFF]` is valid for `u8`. The returned slice borrows
        // `self.pcm_data` for lifetime `'a`.
        let ptr = self.pcm_data.as_ptr().cast::<u8>();
        unsafe { core::slice::from_raw_parts(ptr, self.pcm_data.len()) }
    }

    /// Calculate root-mean-square amplitude over centered signed samples.
    pub fn rms(&self) -> f64 {
        if self.pcm_data.is_empty() {
            return 0.0;
        }
        let mut sum_sq = 0.0f64;
        for i in 0..self.pcm_data.len() {
            let val = self.sample_i8(i) as f64;
            sum_sq += val * val;
        }
        crate::synth::sqrt_f64(sum_sq / (self.pcm_data.len() as f64))
    }

    /// Convert this borrowed view into an owned `GaxSample` with properly centered signed `i8` PCM.
    #[cfg(feature = "alloc")]
    pub fn to_owned_sample(&self) -> GaxSample {
        let pcm_data = if self.is_unsigned {
            self.pcm_data
                .iter()
                .map(|&s| ((s as u8) ^ 0x80) as i8)
                .collect()
        } else {
            self.pcm_data.to_vec()
        };
        GaxSample {
            id: self.id,
            sample_rate: self.sample_rate,
            loop_start: self.loop_start,
            loop_end: self.loop_end,
            pcm_data,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GaxSample {
    pub id: u32,
    pub sample_rate: u32,
    pub loop_start: Option<u32>,
    pub loop_end: Option<u32>,
    #[cfg(feature = "alloc")]
    pub pcm_data: Vec<i8>,
}

impl GaxSample {
    #[cfg(feature = "alloc")]
    pub fn new(id: u32, sample_rate: u32, pcm_data: Vec<i8>) -> Self {
        Self {
            id,
            sample_rate,
            loop_start: None,
            loop_end: None,
            pcm_data,
        }
    }

    /// Construct a `GaxSample` from raw byte slice (interpreting bytes as signed 8-bit PCM).
    #[cfg(feature = "alloc")]
    pub fn from_raw_bytes(id: u32, sample_rate: u32, data: &[u8]) -> Self {
        let pcm_data: Vec<i8> = data.iter().map(|&b| b as i8).collect();
        Self::new(id, sample_rate, pcm_data)
    }

    /// Construct a version-aware `GaxSample` from raw ROM bytes, storing centered signed `i8` PCM.
    ///
    /// - GAX v1.x / v2.x: interprets bytes directly as signed `i8`.
    /// - GAX v3.x: converts unsigned `0x80`-centered PCM to signed `i8` via `b ^ 0x80`.
    #[cfg(feature = "alloc")]
    pub fn from_rom_bytes(id: u32, sample_rate: u32, version: GaxVersion, data: &[u8]) -> Self {
        let pcm_data: Vec<i8> = if version.is_unsigned_pcm() {
            data.iter().map(|&b| (b ^ 0x80) as i8).collect()
        } else {
            data.iter().map(|&b| b as i8).collect()
        };
        Self::new(id, sample_rate, pcm_data)
    }

    /// View this sample as a borrowed zero-allocation `GaxSampleView`.
    #[cfg(feature = "alloc")]
    pub fn as_view(&self) -> GaxSampleView<'_> {
        GaxSampleView {
            id: self.id,
            sample_rate: self.sample_rate,
            loop_start: self.loop_start,
            loop_end: self.loop_end,
            pcm_data: &self.pcm_data,
            is_unsigned: false,
        }
    }

    /// Number of audio samples.
    #[cfg(feature = "alloc")]
    pub fn len(&self) -> usize {
        self.pcm_data.len()
    }

    /// Whether the sample contains zero frames.
    #[cfg(feature = "alloc")]
    pub fn is_empty(&self) -> bool {
        self.pcm_data.is_empty()
    }

    /// Return min and max signed 8-bit sample values.
    #[cfg(feature = "alloc")]
    pub fn min_max(&self) -> (i8, i8) {
        self.as_view().min_max()
    }

    /// Check if all samples are zero.
    #[cfg(feature = "alloc")]
    pub fn is_silent(&self) -> bool {
        self.as_view().is_silent()
    }

    /// View signed PCM data as raw byte slice.
    #[cfg(feature = "alloc")]
    #[allow(unsafe_code)]
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY:
        // `i8` and `u8` have identical memory layout, size (1 byte), and alignment (1 byte).
        // Every bit pattern in `[0x00..=0xFF]` represents a valid value for both `i8` and `u8`.
        // The pointer is derived from `self.pcm_data` (a valid initialized `Vec<i8>`), is non-null,
        // and points to `self.pcm_data.len()` continuous valid initialized bytes.
        // The returned slice borrows `self.pcm_data` for the lifetime of `&self`, ensuring
        // no concurrent mutation or aliasing violations occur.
        let ptr = self.pcm_data.as_ptr().cast::<u8>();
        unsafe { core::slice::from_raw_parts(ptr, self.pcm_data.len()) }
    }

    /// Calculate root-mean-square amplitude.
    #[cfg(feature = "alloc")]
    pub fn rms(&self) -> f64 {
        self.as_view().rms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_view_no_alloc() {
        let raw: [u8; 4] = [0x00, 0x7F, 0x80, 0xFF];
        let view = GaxSampleView::from_raw_bytes(1, 42000, &raw);
        assert_eq!(view.len(), 4);
        assert!(!view.is_empty());
        assert!(!view.is_unsigned);
        assert_eq!(view.pcm_data, &[0, 127, -128, -1]);
        assert_eq!(view.min_max(), (-128, 127));
        assert!(!view.is_silent());
        assert_eq!(view.as_bytes(), &raw);

        let silent_raw: [u8; 4] = [0, 0, 0, 0];
        let silent_view = GaxSampleView::from_raw_bytes(2, 42000, &silent_raw);
        assert!(silent_view.is_silent());
        assert_eq!(silent_view.rms(), 0.0);
    }

    #[test]
    fn test_sample_view_from_rom_bytes_v3_unsigned() {
        let raw: [u8; 4] = [0x80, 0xFF, 0x00, 0x7F];
        let view = GaxSampleView::from_rom_bytes(1, 42000, GaxVersion::V3, &raw);
        assert!(view.is_unsigned);
        assert_eq!(view.sample_i8(0), 0);
        assert_eq!(view.sample_i8(1), 127);
        assert_eq!(view.sample_i8(2), -128);
        assert_eq!(view.sample_i8(3), -1);
        assert_eq!(view.sample_u8(0), 0x80);
        assert_eq!(view.min_max(), (-128, 127));
        assert!(!view.is_silent());

        let silent_u8: [u8; 4] = [0x80, 0x80, 0x80, 0x80];
        let silent_view = GaxSampleView::from_rom_bytes(2, 42000, GaxVersion::V3, &silent_u8);
        assert!(silent_view.is_silent());
        assert_eq!(silent_view.rms(), 0.0);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_sample_construction() {
        let raw: [u8; 4] = [0x00, 0x7F, 0x80, 0xFF];
        let sample = GaxSample::from_raw_bytes(1, 42000, &raw);
        assert_eq!(sample.len(), 4);
        assert!(!sample.is_empty());
        assert_eq!(sample.pcm_data, alloc::vec![0, 127, -128, -1]);
        assert_eq!(sample.min_max(), (-128, 127));
        assert!(!sample.is_silent());
        assert_eq!(sample.as_bytes(), &raw);

        let view = sample.as_view();
        assert_eq!(view.pcm_data, sample.pcm_data.as_slice());
        let roundtrip = view.to_owned_sample();
        assert_eq!(roundtrip, sample);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_sample_from_rom_bytes_version_aware() {
        let raw: [u8; 4] = [0x80, 0xFF, 0x00, 0x7F];
        let v1 = GaxSample::from_rom_bytes(1, 42260, GaxVersion::V1, &raw);
        assert_eq!(v1.pcm_data, alloc::vec![-128, -1, 0, 127]);

        let v3 = GaxSample::from_rom_bytes(2, 82241, GaxVersion::V3, &raw);
        assert_eq!(v3.pcm_data, alloc::vec![0, 127, -128, -1]);

        let v3_view = GaxSampleView::from_rom_bytes(2, 82241, GaxVersion::V3, &raw);
        assert_eq!(v3_view.to_owned_sample(), v3);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_silent_sample() {
        let raw: [u8; 4] = [0, 0, 0, 0];
        let sample = GaxSample::from_raw_bytes(2, 42000, &raw);
        assert!(sample.is_silent());
        assert_eq!(sample.rms(), 0.0);
    }
}
