//! Shared, platform-independent logic for Snapdragin'.
//!
//! This crate intentionally contains no operating system calls. Platform crates
//! translate native window/input events into these geometry and selection types.

mod geometry;
mod grid;
mod selection;

pub use geometry::{ScreenPoint, ScreenRect};
pub use grid::{GridCell, GridError, GridSpec};
pub use selection::{GridArea, GridSelection, SelectionTracker};
