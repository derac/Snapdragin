use std::{error::Error, fmt};

use crate::{ScreenPoint, ScreenRect};

pub const MIN_GRID_DIMENSION: u16 = 1;
pub const MAX_GRID_DIMENSION: u16 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSpec {
    columns: u16,
    rows: u16,
}

impl GridSpec {
    /// Creates a validated grid specification.
    ///
    /// The upper bound keeps accidental settings corruption from creating an
    /// unusably dense overlay or overflowing intermediate selection math.
    pub const fn new(columns: u16, rows: u16) -> Result<Self, GridError> {
        if columns < MIN_GRID_DIMENSION {
            return Err(GridError::ZeroColumns);
        }

        if rows < MIN_GRID_DIMENSION {
            return Err(GridError::ZeroRows);
        }

        if columns > MAX_GRID_DIMENSION {
            return Err(GridError::TooManyColumns {
                actual: columns,
                max: MAX_GRID_DIMENSION,
            });
        }

        if rows > MAX_GRID_DIMENSION {
            return Err(GridError::TooManyRows {
                actual: rows,
                max: MAX_GRID_DIMENSION,
            });
        }

        Ok(Self { columns, rows })
    }

    #[must_use]
    pub const fn clamped(columns: u16, rows: u16) -> Self {
        Self {
            columns: clamp_dimension(columns),
            rows: clamp_dimension(rows),
        }
    }

    #[must_use]
    pub const fn columns(self) -> u16 {
        self.columns
    }

    #[must_use]
    pub const fn rows(self) -> u16 {
        self.rows
    }

    #[must_use]
    pub fn cell_at(self, monitor: ScreenRect, point: ScreenPoint) -> Option<GridCell> {
        if !monitor.contains(point) {
            return None;
        }

        Some(self.cell_at_unchecked(monitor, point))
    }

    /// Returns the nearest cell for a point, clamping points outside the monitor
    /// to the closest monitor edge.
    #[must_use]
    pub fn cell_at_clamped(self, monitor: ScreenRect, point: ScreenPoint) -> Option<GridCell> {
        if monitor.is_empty() {
            return None;
        }

        let left = i64::from(monitor.x);
        let top = i64::from(monitor.y);
        let right = monitor.right();
        let bottom = monitor.bottom();

        let x = i64::from(point.x).clamp(left, right - 1);
        let y = i64::from(point.y).clamp(top, bottom - 1);

        Some(self.cell_at_unchecked(
            monitor,
            ScreenPoint::new(i32::try_from(x).ok()?, i32::try_from(y).ok()?),
        ))
    }

    #[must_use]
    pub fn cell_rect(self, monitor: ScreenRect, cell: GridCell) -> Option<ScreenRect> {
        if !self.contains_cell(cell) || monitor.is_empty() {
            return None;
        }

        let left = self.column_boundary(monitor, cell.column);
        let right = self.column_boundary(monitor, cell.column + 1);
        let top = self.row_boundary(monitor, cell.row);
        let bottom = self.row_boundary(monitor, cell.row + 1);

        rect_from_bounds(left, top, right, bottom)
    }

    #[must_use]
    pub const fn contains_cell(self, cell: GridCell) -> bool {
        cell.column < self.columns && cell.row < self.rows
    }

    pub(crate) fn column_boundary(self, monitor: ScreenRect, column: u16) -> i64 {
        i64::from(monitor.x)
            + (i64::from(monitor.width) * i64::from(column)) / i64::from(self.columns)
    }

    pub(crate) fn row_boundary(self, monitor: ScreenRect, row: u16) -> i64 {
        i64::from(monitor.y) + (i64::from(monitor.height) * i64::from(row)) / i64::from(self.rows)
    }

    fn cell_at_unchecked(self, monitor: ScreenRect, point: ScreenPoint) -> GridCell {
        let relative_x = i64::from(point.x) - i64::from(monitor.x);
        let relative_y = i64::from(point.y) - i64::from(monitor.y);

        let column = relative_x * i64::from(self.columns) / i64::from(monitor.width);
        let row = relative_y * i64::from(self.rows) / i64::from(monitor.height);

        GridCell::new(
            u16::try_from(column).expect("grid column should fit u16"),
            u16::try_from(row).expect("grid row should fit u16"),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCell {
    pub column: u16,
    pub row: u16,
}

impl GridCell {
    #[must_use]
    pub const fn new(column: u16, row: u16) -> Self {
        Self { column, row }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridError {
    ZeroColumns,
    ZeroRows,
    TooManyColumns { actual: u16, max: u16 },
    TooManyRows { actual: u16, max: u16 },
}

impl fmt::Display for GridError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroColumns => write!(f, "grid must have at least one column"),
            Self::ZeroRows => write!(f, "grid must have at least one row"),
            Self::TooManyColumns { actual, max } => {
                write!(f, "grid has {actual} columns, but the maximum is {max}")
            }
            Self::TooManyRows { actual, max } => {
                write!(f, "grid has {actual} rows, but the maximum is {max}")
            }
        }
    }
}

impl Error for GridError {}

const fn clamp_dimension(value: u16) -> u16 {
    if value < MIN_GRID_DIMENSION {
        MIN_GRID_DIMENSION
    } else if value > MAX_GRID_DIMENSION {
        MAX_GRID_DIMENSION
    } else {
        value
    }
}

pub(crate) fn rect_from_bounds(left: i64, top: i64, right: i64, bottom: i64) -> Option<ScreenRect> {
    let width = right.checked_sub(left)?;
    let height = bottom.checked_sub(top)?;

    Some(ScreenRect::new(
        i32::try_from(left).ok()?,
        i32::try_from(top).ok()?,
        u32::try_from(width).ok()?,
        u32::try_from(height).ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::{GridCell, GridError, GridSpec};
    use crate::{ScreenPoint, ScreenRect};

    #[test]
    fn rejects_invalid_grid_dimensions() {
        assert_eq!(GridSpec::new(0, 4), Err(GridError::ZeroColumns));
        assert_eq!(GridSpec::new(4, 0), Err(GridError::ZeroRows));
        assert!(matches!(
            GridSpec::new(65, 4),
            Err(GridError::TooManyColumns {
                actual: 65,
                max: 64
            })
        ));
        assert!(matches!(
            GridSpec::new(4, 65),
            Err(GridError::TooManyRows {
                actual: 65,
                max: 64
            })
        ));
    }

    #[test]
    fn clamps_grid_dimensions() {
        assert_eq!(GridSpec::clamped(0, 99), GridSpec::new(1, 64).unwrap());
    }

    #[test]
    fn maps_points_to_cells() {
        let grid = GridSpec::new(4, 2).unwrap();
        let monitor = ScreenRect::new(0, 0, 400, 200);

        assert_eq!(
            grid.cell_at(monitor, ScreenPoint::new(0, 0)),
            Some(GridCell::new(0, 0))
        );
        assert_eq!(
            grid.cell_at(monitor, ScreenPoint::new(399, 199)),
            Some(GridCell::new(3, 1))
        );
        assert_eq!(grid.cell_at(monitor, ScreenPoint::new(400, 199)), None);
    }

    #[test]
    fn handles_negative_monitor_coordinates() {
        let grid = GridSpec::new(2, 2).unwrap();
        let monitor = ScreenRect::new(-800, -200, 800, 600);

        assert_eq!(
            grid.cell_at(monitor, ScreenPoint::new(-1, 399)),
            Some(GridCell::new(1, 1))
        );
    }

    #[test]
    fn cell_rects_cover_uneven_monitor_width_without_gaps() {
        let grid = GridSpec::new(3, 1).unwrap();
        let monitor = ScreenRect::new(0, 0, 10, 10);

        assert_eq!(
            grid.cell_rect(monitor, GridCell::new(0, 0)),
            Some(ScreenRect::new(0, 0, 3, 10))
        );
        assert_eq!(
            grid.cell_rect(monitor, GridCell::new(1, 0)),
            Some(ScreenRect::new(3, 0, 3, 10))
        );
        assert_eq!(
            grid.cell_rect(monitor, GridCell::new(2, 0)),
            Some(ScreenRect::new(6, 0, 4, 10))
        );
    }

    #[test]
    fn clamped_cell_at_keeps_selection_on_monitor() {
        let grid = GridSpec::new(4, 4).unwrap();
        let monitor = ScreenRect::new(100, 100, 400, 400);

        assert_eq!(
            grid.cell_at_clamped(monitor, ScreenPoint::new(900, -500)),
            Some(GridCell::new(3, 0))
        );
    }
}
