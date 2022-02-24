use std::{cell::Cell, fmt::Display, ops::Div, rc::Rc};

use arrayvec::ArrayVec;
use glam::UVec2;

pub const INFINITY_SYMBOL: &str = "∞";

pub fn color_to_string(color: &ArrayVec<u8, 3>) -> String {
    format!("rgb({}, {}, {})", color[0], color[1], color[2])
}

pub fn delimit_string<'a>(lines: &[String], delimiter: &str) -> String {
    let mut result = String::new();
    for (i, line) in lines.iter().enumerate() {
        result += line;
        if i != lines.len() - 1 {
            result += delimiter;
        }
    }
    result
}

pub fn merge_lines(lines: &[String]) -> String {
    delimit_string(lines, "\n")
}

/// Gets the definite article to use for the given noun.
pub fn article(noun: &str) -> &'static str {
    let first_char = noun.chars().next().unwrap().to_ascii_lowercase();
    if matches!(first_char, 'a' | 'e' | 'i' | 'o' | 'u') {
        "an"
    } else {
        "a"
    }
}

#[derive(Debug, Default)]
struct VersionState {
    version: Cell<u64>,
}

/// A version counter that can be used to track updates to some piece of data.
#[derive(Debug, Default)]
pub struct Version {
    state: Rc<VersionState>,
}

impl Version {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a snapshot of the version at the current time.
    pub fn snapshot(&self) -> VersionSnapshot {
        VersionSnapshot {
            state: Rc::clone(&self.state),
            snapshot: self.state.version.clone(),
        }
    }

    /// Increments the version counter, causing any
    /// snapshots to return `true` from `is_outdated`.
    pub fn update(&self) {
        self.state
            .version
            .set(self.state.version.get().wrapping_add(1));
    }
}

/// Contains a snapshot of a [`Version`]. Can be used
/// to check whether the version has changed since the snapshot
/// was taken.
#[derive(Debug)]
pub struct VersionSnapshot {
    state: Rc<VersionState>,
    snapshot: Cell<u64>,
}

impl VersionSnapshot {
    /// Returns whether the snapshot is outdated
    /// because [`Version::update`] was called.
    pub fn is_outdated(&self) -> bool {
        self.snapshot != self.state.version
    }

    /// Updates to the latest version, causing `is_outdated` to return
    /// `false` again.
    pub fn update(&self) {
        self.snapshot.set(self.state.version.get());
    }
}

/// A wrapper over a `u32` that can be either finite or infinite.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MaybeInfinityU32 {
    Finite(u32),
    Infinite,
}

impl MaybeInfinityU32 {
    pub fn new(x: u32) -> Self {
        Self::Finite(x)
    }
}

impl Div<u32> for MaybeInfinityU32 {
    type Output = Self;

    fn div(self, rhs: u32) -> Self::Output {
        if rhs == 0 {
            Self::Infinite
        } else {
            match self {
                Self::Finite(x) => Self::Finite(x / rhs),
                Self::Infinite => Self::Infinite,
            }
        }
    }
}

impl Display for MaybeInfinityU32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaybeInfinityU32::Finite(x) => x.fmt(f),
            MaybeInfinityU32::Infinite => write!(f, "∞"),
        }
    }
}

pub trait UVecExt {
    fn distance_squared(self, other: Self) -> u32;
}

impl UVecExt for UVec2 {
    fn distance_squared(self, other: Self) -> u32 {
        abs_diff(self.x, other.x).pow(2) + abs_diff(self.y, other.y).pow(2)
    }
}

fn abs_diff(a: u32, b: u32) -> u32 {
    if a < b {
        b - a
    } else {
        // a >= b
        a - b
    }
}
