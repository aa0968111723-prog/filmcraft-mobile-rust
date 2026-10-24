//! Portable mobile policies. No UI, media decoder, or operating system bridge yet.

pub use filmcraft_time::{TICKS_PER_SECOND, Tick};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// One main panel, with tools in a bottom sheet.
    Compact,
    /// Preview and inspector side by side, timeline below.
    Expanded,
}

/// Choose from available logical width after safe areas, rather than device names.
pub fn layout_for_width(logical_width: u32) -> Layout {
    if logical_width >= 840 {
        Layout::Expanded
    } else {
        Layout::Compact
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditError {
    InvalidRange,
    TimeOverflow,
    EmptyFrame,
    FrameTooLarge,
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidRange => "trim requires 0 <= start < end <= source duration",
            Self::TimeOverflow => "edit time exceeds the supported tick range",
            Self::EmptyFrame => "frame dimensions must be greater than zero",
            Self::FrameTooLarge => "frame exceeds the mobile preview cache budget",
        })
    }
}

impl std::error::Error for EditError {}

/// Validated half-open source range. Fields are private to preserve invariants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimRange {
    start: Tick,
    end: Tick,
}

impl TrimRange {
    pub fn new(start: Tick, end: Tick, source_duration: Tick) -> Result<Self, EditError> {
        if start < Tick::ZERO || end <= start || end > source_duration {
            return Err(EditError::InvalidRange);
        }
        Ok(Self { start, end })
    }

    pub fn start(self) -> Tick {
        self.start
    }

    pub fn end(self) -> Tick {
        self.end
    }

    pub fn duration(self) -> Tick {
        // Constructor guarantees nonnegative operands and end > start.
        Tick(self.end.0 - self.start.0)
    }

    /// Translate source time into timeline time, rejecting unrepresentable results.
    pub fn timeline_end(self, timeline_start: Tick) -> Result<Tick, EditError> {
        if timeline_start < Tick::ZERO {
            return Err(EditError::InvalidRange);
        }
        timeline_start
            .0
            .checked_add(self.duration().0)
            .map(Tick)
            .ok_or(EditError::TimeOverflow)
    }
}

/// Bound CPU RGBA preview cache accounting before allocating frame buffers.
/// This budget excludes GPU textures, decoder buffers, audio and project data.
#[derive(Debug, Clone, Copy)]
pub struct PreviewBudget {
    bytes: u64,
}

impl PreviewBudget {
    pub const fn new(bytes: u64) -> Self {
        Self { bytes }
    }

    pub fn cached_rgba_frames(self, width: u32, height: u32) -> Result<u64, EditError> {
        if width == 0 || height == 0 {
            return Err(EditError::EmptyFrame);
        }
        let frame_bytes = u64::from(width)
            .checked_mul(u64::from(height))
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(EditError::FrameTooLarge)?;
        let count = self.bytes / frame_bytes;
        if count == 0 {
            return Err(EditError::FrameTooLarge);
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_trim_inputs_return_errors() {
        for (start, end, duration) in [(-1, 2, 3), (2, 2, 3), (3, 2, 4), (0, 4, 3), (0, 1, -1)] {
            assert_eq!(
                TrimRange::new(Tick(start), Tick(end), Tick(duration)),
                Err(EditError::InvalidRange)
            );
        }
    }

    #[test]
    fn fractional_frame_edits_retain_upstream_exact_time() -> Result<(), EditError> {
        // One 29.97 fps frame: 1001 / 30000 seconds, exactly representable.
        let frame = Tick::from_rational(1, 1001, 30_000);
        let range = TrimRange::new(frame, frame + frame, frame + frame)?;
        assert_eq!(range.duration().0, 8_475_667_200);
        assert_eq!(range.timeline_end(Tick::ZERO)?, frame);
        Ok(())
    }

    #[test]
    fn timeline_overflow_does_not_silently_saturate() -> Result<(), EditError> {
        let range = TrimRange::new(Tick::ZERO, Tick(1), Tick(1))?;
        assert_eq!(
            range.timeline_end(Tick(i64::MAX)),
            Err(EditError::TimeOverflow)
        );
        assert_eq!(range.timeline_end(Tick(-1)), Err(EditError::InvalidRange));
        Ok(())
    }

    #[test]
    fn hostile_frame_sizes_are_rejected() {
        let budget = PreviewBudget::new(64 * 1024 * 1024);
        assert_eq!(
            budget.cached_rgba_frames(0, 1080),
            Err(EditError::EmptyFrame)
        );
        assert_eq!(
            budget.cached_rgba_frames(u32::MAX, u32::MAX),
            Err(EditError::FrameTooLarge)
        );
        assert_eq!(
            PreviewBudget::new(0).cached_rgba_frames(1, 1),
            Err(EditError::FrameTooLarge)
        );
        assert_eq!(budget.cached_rgba_frames(1920, 1080), Ok(8));
    }
}
