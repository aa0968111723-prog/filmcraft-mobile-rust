//! Small, validated editing model for the downloadable procedural-demo preview.
use crate::{TICKS_PER_SECOND, Tick, TrimRange};
use serde::{Deserialize, Serialize};
use std::fmt;

pub const FPS: i64 = 30;
pub const FRAME_TICKS: i64 = TICKS_PER_SECOND / FPS;
pub const MAX_CLIPS: usize = 64;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DemoSource {
    Dawn,
    Waves,
    Night,
}

impl DemoSource {
    pub fn title(self) -> &'static str {
        match self {
            Self::Dawn => "Morning light",
            Self::Waves => "Ocean study",
            Self::Night => "After dark",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Clip {
    pub source: DemoSource,
    pub start: Tick,
    pub end: Tick,
}

impl Clip {
    pub fn frames(&self) -> u32 {
        u32::try_from((self.end.0.saturating_sub(self.start.0)) / FRAME_TICKS).unwrap_or(0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub version: u32,
    pub clips: Vec<Clip>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectError {
    InvalidProject,
    InvalidTrim,
    InvalidSelection,
    SplitAtEdge,
    ClipLimit,
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidProject => "The saved demo project is invalid or unsupported.",
            Self::InvalidTrim => "Trim requires 0 <= in < out <= 300 frames.",
            Self::InvalidSelection => "Select a clip first.",
            Self::SplitAtEdge => "Move the playhead inside the selected clip to split.",
            Self::ClipLimit => "This preview supports up to 64 clips.",
        })
    }
}

impl std::error::Error for ProjectError {}

impl Default for Project {
    fn default() -> Self {
        Self {
            version: 1,
            clips: [DemoSource::Dawn, DemoSource::Waves, DemoSource::Night]
                .into_iter()
                .map(|source| Clip {
                    source,
                    start: Tick::ZERO,
                    end: Tick(4 * TICKS_PER_SECOND),
                })
                .collect(),
        }
    }
}

impl Project {
    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.version != 1 || self.clips.is_empty() || self.clips.len() > MAX_CLIPS {
            return Err(ProjectError::InvalidProject);
        }
        for clip in &self.clips {
            if TrimRange::new(clip.start, clip.end, Tick(10 * TICKS_PER_SECOND)).is_err()
                || clip.start.0 % FRAME_TICKS != 0
                || clip.end.0 % FRAME_TICKS != 0
            {
                return Err(ProjectError::InvalidProject);
            }
        }
        Ok(())
    }

    pub fn total_frames(&self) -> u32 {
        self.clips
            .iter()
            .fold(0_u32, |sum, clip| sum.saturating_add(clip.frames()))
    }

    pub fn clip_at(&self, frame: u32) -> Option<(usize, u32)> {
        let mut offset = 0_u32;
        for (index, clip) in self.clips.iter().enumerate() {
            let end = offset.saturating_add(clip.frames());
            if frame < end {
                return Some((index, frame.saturating_sub(offset)));
            }
            offset = end;
        }
        None
    }

    pub fn split(&mut self, selected: usize, timeline_frame: u32) -> Result<(), ProjectError> {
        self.validate()?;
        if self.clips.len() >= MAX_CLIPS {
            return Err(ProjectError::ClipLimit);
        }
        let (index, local) = self
            .clip_at(timeline_frame)
            .ok_or(ProjectError::SplitAtEdge)?;
        if index != selected || local == 0 {
            return Err(ProjectError::SplitAtEdge);
        }
        let clip = self
            .clips
            .get_mut(index)
            .ok_or(ProjectError::InvalidSelection)?;
        let split = Tick(clip.start.0 + i64::from(local) * FRAME_TICKS);
        let right = Clip {
            source: clip.source,
            start: split,
            end: clip.end,
        };
        clip.end = split;
        self.clips.insert(index.saturating_add(1), right);
        Ok(())
    }

    pub fn trim(
        &mut self,
        selected: usize,
        in_frame: u32,
        out_frame: u32,
    ) -> Result<(), ProjectError> {
        self.validate()?;
        if in_frame >= out_frame || out_frame > 300 {
            return Err(ProjectError::InvalidTrim);
        }
        let start = Tick(i64::from(in_frame) * FRAME_TICKS);
        let end = Tick(i64::from(out_frame) * FRAME_TICKS);
        TrimRange::new(start, end, Tick(10 * TICKS_PER_SECOND))
            .map_err(|_| ProjectError::InvalidTrim)?;
        let clip = self
            .clips
            .get_mut(selected)
            .ok_or(ProjectError::InvalidSelection)?;
        clip.start = start;
        clip.end = end;
        Ok(())
    }

    pub fn move_clip(&mut self, selected: usize, destination: usize) -> Result<(), ProjectError> {
        self.validate()?;
        if selected >= self.clips.len() || destination >= self.clips.len() {
            return Err(ProjectError::InvalidSelection);
        }
        self.clips.swap(selected, destination);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_preserves_duration_and_source_continuity() -> Result<(), ProjectError> {
        let mut project = Project::default();
        let duration = project.total_frames();
        project.split(0, 45)?;
        assert_eq!(project.total_frames(), duration);
        assert_eq!(
            project.clips.first().map(|c| c.end),
            project.clips.get(1).map(|c| c.start)
        );
        assert_eq!(project.clip_at(45), Some((1, 0)));
        project.validate()
    }

    #[test]
    fn rejected_edits_preserve_project() {
        let mut project = Project::default();
        let original = project.clone();
        assert!(project.split(0, 0).is_err());
        assert!(project.split(0, 120).is_err());
        assert_eq!(project.trim(0, 20, 10), Err(ProjectError::InvalidTrim));
        assert!(project.trim(0, 0, u32::MAX).is_err());
        assert!(project.move_clip(0, usize::MAX).is_err());
        assert_eq!(project, original);
    }

    #[test]
    fn corrupt_saved_ticks_and_versions_are_rejected() {
        let mut project = Project::default();
        if let Some(clip) = project.clips.first_mut() {
            clip.end = Tick(i64::MAX);
        }
        assert!(project.validate().is_err());
        project = Project::default();
        project.version = 2;
        assert!(project.validate().is_err());
    }
}
