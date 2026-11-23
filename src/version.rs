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
    /// Detect GAX version from engine signature strings or header markers.
    pub fn detect_from_bytes(data: &[u8]) -> Option<Self> {
        // Check for GAX 3.x signatures
        if data.windows(8).any(|w| {
            w == b"GAX 3.01"
                || w == b"GAX 3.03"
                || w == b"GAX 3.05"
                || w == b"GAX 3.0"
                || w == b"GAX2_INI"
        }) || data.windows(16).any(|w| w == b"GAX Sound Engine") && data.windows(3).any(|w| w == b"3.0") {
            return Some(Self::V3);
        }

        // Check for GAX 2.x
        if data.windows(6).any(|w| w == b"GAX v2" || w == b"GAX 2.")
            || (data.windows(16).any(|w| w == b"GAX Sound Engine") && data.windows(3).any(|w| w == b"2.0" || w == b"2.1"))
        {
            return Some(Self::V2);
        }

        // Check for GAX 1.x (e.g., v1.99d in Iridion 3D)
        if data.windows(6).any(|w| w == b"GAX v1" || w == b"GAX 1." || w == b"v1.99")
            || (data.windows(16).any(|w| w == b"GAX Sound Engine") && data.windows(4).any(|w| w == b"1.99"))
        {
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
        let dummy = b"Binary header GAX Sound Engine v1.99d (c) Shin'en Multimedia";
        assert_eq!(GaxVersion::detect_from_bytes(dummy), Some(GaxVersion::V1));
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
}
