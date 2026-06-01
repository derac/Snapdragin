//! OS-free grid and selection logic for Snapdragin.
//!
//! This module intentionally contains no operating system calls. The Windows app
//! translates native window/input events into these geometry and selection types.

mod geometry;
mod grid;
mod selection;

pub use geometry::{ScreenPoint, ScreenRect};
pub use grid::{GridCell, GridSpec};
pub use selection::{GridSelection, SelectionTracker};
