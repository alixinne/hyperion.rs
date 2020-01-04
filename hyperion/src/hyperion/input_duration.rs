//! Definition of the InputDuration type

use std::cmp::Ordering;
use std::time::{Duration, Instant};

/// Duration type of an input
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InputDurationKind {
    /// Apply the command once and forget
    OneShot,
    /// Apply the command forever
    Endless,
    /// Apply the command for a specific duration
    Limited {
        /// Duration of the period
        duration: Duration,
    },
}

/// Duration of an input
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct InputDuration {
    /// Start of the period
    start: Instant,
    /// Kind of duration
    kind: InputDurationKind,
}

impl InputDuration {
    /// Create a one-shot duration
    pub fn oneshot() -> Self {
        Self {
            start: Instant::now(),
            kind: InputDurationKind::OneShot,
        }
    }

    /// Return true if this duration is a one-shot duration
    pub fn is_oneshot(&self) -> bool {
        match self.kind {
            InputDurationKind::OneShot => true,
            _ => false,
        }
    }

    /// Return the start point of this duration
    pub fn start(&self) -> Instant {
        self.start
    }

    /// Return the deadline of this duration, if it exists
    ///
    /// # Returns
    ///
    /// `None` if this duration is not limited in time or is a one-shot event
    pub fn deadline(&self) -> Option<Instant> {
        match self.kind {
            InputDurationKind::OneShot | InputDurationKind::Endless => None,
            InputDurationKind::Limited { duration } => Some(self.start + duration),
        }
    }

    /// Return a value indicating if this input will be expired at the given time
    pub fn is_expired(&self, when: Instant) -> bool {
        match self.kind {
            InputDurationKind::OneShot | InputDurationKind::Endless => false,
            InputDurationKind::Limited { duration } => self.start + duration < when,
        }
    }
}

impl Ord for InputDuration {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.kind {
            InputDurationKind::OneShot => match other.kind {
                InputDurationKind::OneShot => other.start.cmp(&self.start),
                InputDurationKind::Endless | InputDurationKind::Limited { .. } => Ordering::Less,
            },
            InputDurationKind::Endless => match other.kind {
                InputDurationKind::Endless => other.start.cmp(&self.start),
                InputDurationKind::OneShot | InputDurationKind::Limited { .. } => Ordering::Greater,
            },
            InputDurationKind::Limited { .. } => match other.kind {
                InputDurationKind::OneShot => Ordering::Greater,
                InputDurationKind::Endless => Ordering::Less,
                InputDurationKind::Limited { .. } => other.start.cmp(&self.start),
            },
        }
    }
}

impl PartialOrd for InputDuration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<(Instant, Option<i32>)> for InputDuration {
    fn from(d: (Instant, Option<i32>)) -> Self {
        let start = d.0;
        match d.1 {
            Some(duration) if duration > 0 => Self {
                start,
                kind: InputDurationKind::Limited {
                    duration: Duration::from_millis(duration as u64),
                },
            },
            Some(duration) if duration <= 0 => Self {
                start,
                kind: InputDurationKind::Endless,
            },
            _ => Self {
                start,
                kind: InputDurationKind::OneShot,
            },
        }
    }
}
