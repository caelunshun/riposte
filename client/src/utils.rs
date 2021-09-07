use std::{cell::Cell, rc::Rc};

use arrayvec::ArrayVec;

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
