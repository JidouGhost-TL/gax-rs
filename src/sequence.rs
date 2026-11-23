//! GAX Song sequences, pattern tables, and event representations.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GaxEvent {
    NoteOn { channel: u8, note: u8, velocity: u8 },
    NoteOff { channel: u8 },
    PitchBend { channel: u8, bend: i16 },
    Volume { channel: u8, volume: u8 },
    WaitTicks(u16),
    LoopPoint,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GaxPattern {
    pub id: u32,
    #[cfg(feature = "alloc")]
    pub events: Vec<GaxEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GaxSong {
    pub id: u32,
    pub title: &'static str,
    pub tempo: u16,
    #[cfg(feature = "alloc")]
    pub patterns: Vec<GaxPattern>,
}
