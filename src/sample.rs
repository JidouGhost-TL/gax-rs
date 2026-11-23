//! GAX Sample structures and PCM decoders.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Borrowed zero-allocation view of a GAX sample.
///
/// Enables `#![no_std]` applications to play samples directly from ROM or static buffers
/// without requiring a heap allocator (`feature = "alloc"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GaxSampleView<'a> {
    pub id: u32,
    pub sample_rate: u32,
    pub loop_start: Option<u32>,
    pub loop_end: Option<u32>,
    pub pcm_data: &'a [i8],
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

    /// Return min and max signed 8-bit sample values.
    pub fn min_max(&self) -> (i8, i8) {
        if self.pcm_data.is_empty() {
            return (0, 0);
        }
        let mut min_val = self.pcm_data[0];
        let mut max_val = self.pcm_data[0];
        for &s in self.pcm_data {
            if s < min_val {
                min_val = s;
            }
            if s > max_val {
                max_val = s;
            }
        }
        (min_val, max_val)
    }

    /// Check if all samples are zero.
    pub fn is_silent(&self) -> bool {
        self.pcm_data.iter().all(|&s| s == 0)
    }

    /// View signed PCM data as raw byte slice.
    #[allow(unsafe_code)]
    pub const fn as_bytes(&self) -> &'a [u8] {
        // SAFETY:
        // `i8` and `u8` have identical memory layout, size (1 byte), and alignment (1 byte).
        // Every bit pattern in `[0x00..=0xFF]` is valid for `u8`. The returned slice borrows
        // `self.pcm_data` for lifetime `'a`.
        let ptr = self.pcm_data.as_ptr().cast::<u8>();
        unsafe { core::slice::from_raw_parts(ptr, self.pcm_data.len()) }
    }

    /// Calculate root-mean-square amplitude.
    pub fn rms(&self) -> f64 {
        if self.pcm_data.is_empty() {
            return 0.0;
        }
        let sum_sq: f64 = self.pcm_data.iter().map(|&s| {
            let val = s as f64;
            val * val
        }).sum();
        crate::synth::sqrt_f64(sum_sq / (self.pcm_data.len() as f64))
    }

    /// Convert this borrowed view into an owned `GaxSample`.
    #[cfg(feature = "alloc")]
    pub fn to_owned_sample(&self) -> GaxSample {
        GaxSample {
            id: self.id,
            sample_rate: self.sample_rate,
            loop_start: self.loop_start,
            loop_end: self.loop_end,
            pcm_data: self.pcm_data.to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// Construct a GaxSample from raw byte slice (interpreting bytes as signed 8-bit PCM).
    #[cfg(feature = "alloc")]
    pub fn from_raw_bytes(id: u32, sample_rate: u32, data: &[u8]) -> Self {
        let pcm_data: Vec<i8> = data.iter().map(|&b| b as i8).collect();
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
    fn test_silent_sample() {
        let raw: [u8; 4] = [0, 0, 0, 0];
        let sample = GaxSample::from_raw_bytes(2, 42000, &raw);
        assert!(sample.is_silent());
        assert_eq!(sample.rms(), 0.0);
    }
}
