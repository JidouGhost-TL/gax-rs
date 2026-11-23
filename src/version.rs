//! GAX Engine Version enumeration and detection.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GaxVersion {
    /// GAX v1.x (used in early titles such as Iridion 3D)
    V1,
    /// GAX v2.x (transitional engine)
    V2,
    /// GAX v3.01 / v3.05 (most widespread across commercial GBA catalog)
    V3,
}

impl GaxVersion {
    /// Returns `true` if this GAX engine version stores sample PCM in ROM as unsigned 8-bit (`0x80` center).
    ///
    /// - GAX v1.x and v2.x store signed 8-bit PCM (`0x00` center, fetched via `ldrsb`).
    /// - GAX v3.x stores unsigned 8-bit PCM (`0x80` center, fetched via `ldrb` for packed SIMD mixing
    ///   with DC bias subtraction during DirectSound DMA clamping).
    #[inline]
    pub const fn is_unsigned_pcm(&self) -> bool {
        matches!(self, Self::V3)
    }

    /// Detect GAX version from engine signature strings or header markers.
    pub fn detect_from_bytes(data: &[u8]) -> Option<Self> {
        // Primary check: locate "GAX Sound Engine" and inspect the version token immediately following it
        const PREFIX: &[u8] = b"GAX Sound Engine";
        for (idx, w) in data.windows(PREFIX.len()).enumerate() {
            if w == PREFIX {
                let tail_start = idx + PREFIX.len();
                let tail_end = (tail_start + 24).min(data.len());
                let tail = &data[tail_start..tail_end];
                if tail.windows(2).any(|v| v == b"3.") {
                    return Some(Self::V3);
                }
                if tail.windows(2).any(|v| v == b"2.") {
                    return Some(Self::V2);
                }
                if tail.windows(2).any(|v| v == b"1.") {
                    return Some(Self::V1);
                }
            }
        }

        // Secondary direct short markers (exact window lengths)
        if data.windows(7).any(|w| w == b"GAX 3.0" || w == b"GAX v3.") {
            return Some(Self::V3);
        }
        if data.windows(6).any(|w| w == b"GAX v2" || w == b"GAX 2.")
            || data.windows(9).any(|w| w == b"GAX2_INIT")
        {
            return Some(Self::V2);
        }
        if data.windows(6).any(|w| w == b"GAX v1" || w == b"GAX 1.") {
            return Some(Self::V1);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_v1() {
        let dummy = b"Binary header GAX Sound Engine v1.99d (c) Shin'en Multimedia 3.0 elsewhere";
        assert_eq!(GaxVersion::detect_from_bytes(dummy), Some(GaxVersion::V1));
    }

    #[test]
    fn test_detect_v2() {
        let dummy = b"Driver GAX Sound Engine 2.31 (Apr 19 2002) Shin'en";
        assert_eq!(GaxVersion::detect_from_bytes(dummy), Some(GaxVersion::V2));
    }

    #[test]
    fn test_detect_v3() {
        let dummy = b"Driver header GAX Sound Engine 3.01 Shin'en";
        assert_eq!(GaxVersion::detect_from_bytes(dummy), Some(GaxVersion::V3));
    }

    #[test]
    fn test_detect_none() {
        let dummy = b"Random binary data without Shin'en audio tags";
        assert_eq!(GaxVersion::detect_from_bytes(dummy), None);
    }

    #[test]
    fn test_is_unsigned_pcm() {
        assert!(!GaxVersion::V1.is_unsigned_pcm());
        assert!(!GaxVersion::V2.is_unsigned_pcm());
        assert!(GaxVersion::V3.is_unsigned_pcm());
    }
}
