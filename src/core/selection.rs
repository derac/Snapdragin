use super::{grid::rect_from_bounds, GridCell, GridSpec, ScreenPoint, ScreenRect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSelection {
    pub anchor: GridCell,
    pub focus: GridCell,
}

impl GridSelection {
    #[must_use]
    pub const fn new(anchor: GridCell, focus: GridCell) -> Self {
        Self { anchor, focus }
    }

    #[must_use]
    pub fn area(self) -> GridArea {
        GridArea::from_selection(self)
    }

    #[must_use]
    pub fn screen_rect(self, grid: GridSpec, monitor: ScreenRect) -> Option<ScreenRect> {
        self.area().screen_rect(grid, monitor)
    }
}

/// A normalized grid area using exclusive right/bottom bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridArea {
    pub left: u16,
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
}

impl GridArea {
    #[must_use]
    pub fn from_selection(selection: GridSelection) -> Self {
        let left = selection.anchor.column.min(selection.focus.column);
        let top = selection.anchor.row.min(selection.focus.row);
        let right = selection
            .anchor
            .column
            .max(selection.focus.column)
            .saturating_add(1);
        let bottom = selection
            .anchor
            .row
            .max(selection.focus.row)
            .saturating_add(1);

        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    #[must_use]
    pub fn screen_rect(self, grid: GridSpec, monitor: ScreenRect) -> Option<ScreenRect> {
        if monitor.is_empty()
            || self.left >= self.right
            || self.top >= self.bottom
            || self.right > grid.columns()
            || self.bottom > grid.rows()
        {
            return None;
        }

        let left = grid.column_boundary(monitor, self.left);
        let right = grid.column_boundary(monitor, self.right);
        let top = grid.row_boundary(monitor, self.top);
        let bottom = grid.row_boundary(monitor, self.bottom);

        rect_from_bounds(left, top, right, bottom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionTracker {
    grid: GridSpec,
    monitor: ScreenRect,
    anchor: Option<GridCell>,
    focus: Option<GridCell>,
}

impl SelectionTracker {
    #[must_use]
    pub const fn new(grid: GridSpec, monitor: ScreenRect) -> Self {
        Self {
            grid,
            monitor,
            anchor: None,
            focus: None,
        }
    }

    #[must_use]
    pub fn current_selection(self) -> Option<GridSelection> {
        Some(GridSelection::new(self.anchor?, self.focus?))
    }

    pub fn begin(&mut self, point: ScreenPoint) -> Option<GridSelection> {
        let cell = self.grid.cell_at_clamped(self.monitor, point)?;
        self.anchor = Some(cell);
        self.focus = Some(cell);
        self.current_selection()
    }

    pub fn update(&mut self, point: ScreenPoint) -> Option<GridSelection> {
        let cell = self.grid.cell_at_clamped(self.monitor, point)?;

        self.anchor?;
        self.focus = Some(cell);
        self.current_selection()
    }

    #[cfg(test)]
    pub fn finish(&mut self) -> Option<GridSelection> {
        let selection = self.current_selection();
        self.clear();
        selection
    }

    #[cfg(test)]
    pub fn clear(&mut self) {
        self.anchor = None;
        self.focus = None;
    }
}

#[cfg(test)]
mod tests {
    use super::{GridArea, GridSelection, SelectionTracker};
    use crate::core::{GridCell, GridSpec, ScreenPoint, ScreenRect};

    #[test]
    fn normalizes_selection_from_any_direction() {
        let selection = GridSelection::new(GridCell::new(3, 2), GridCell::new(1, 0));

        assert_eq!(
            selection.area(),
            GridArea {
                left: 1,
                top: 0,
                right: 4,
                bottom: 3
            }
        );
    }

    #[test]
    fn converts_selection_to_screen_rect() {
        let grid = GridSpec::new(4, 4).unwrap();
        let monitor = ScreenRect::new(0, 0, 800, 400);
        let selection = GridSelection::new(GridCell::new(1, 1), GridCell::new(2, 3));

        assert_eq!(
            selection.screen_rect(grid, monitor),
            Some(ScreenRect::new(200, 100, 400, 300))
        );
    }

    #[test]
    fn rejects_area_outside_grid() {
        let grid = GridSpec::new(2, 2).unwrap();
        let monitor = ScreenRect::new(0, 0, 200, 200);
        let selection = GridSelection::new(GridCell::new(1, 1), GridCell::new(2, 2));

        assert_eq!(selection.screen_rect(grid, monitor), None);
    }

    #[test]
    fn tracker_begins_updates_and_finishes_selection() {
        let grid = GridSpec::new(4, 4).unwrap();
        let monitor = ScreenRect::new(0, 0, 400, 400);
        let mut tracker = SelectionTracker::new(grid, monitor);

        assert_eq!(
            tracker.begin(ScreenPoint::new(25, 25)),
            Some(GridSelection::new(GridCell::new(0, 0), GridCell::new(0, 0)))
        );
        assert_eq!(
            tracker.update(ScreenPoint::new(399, 399)),
            Some(GridSelection::new(GridCell::new(0, 0), GridCell::new(3, 3)))
        );
        assert_eq!(
            tracker.finish(),
            Some(GridSelection::new(GridCell::new(0, 0), GridCell::new(3, 3)))
        );
        assert_eq!(tracker.current_selection(), None);
    }

    #[test]
    fn tracker_ignores_updates_before_begin() {
        let grid = GridSpec::new(4, 4).unwrap();
        let monitor = ScreenRect::new(0, 0, 400, 400);
        let mut tracker = SelectionTracker::new(grid, monitor);

        assert_eq!(tracker.update(ScreenPoint::new(300, 300)), None);
        assert_eq!(tracker.current_selection(), None);
    }

    #[test]
    fn tracker_clamps_outside_points() {
        let grid = GridSpec::new(4, 4).unwrap();
        let monitor = ScreenRect::new(0, 0, 400, 400);
        let mut tracker = SelectionTracker::new(grid, monitor);

        tracker.begin(ScreenPoint::new(-100, -100));

        assert_eq!(
            tracker.update(ScreenPoint::new(800, 800)),
            Some(GridSelection::new(GridCell::new(0, 0), GridCell::new(3, 3)))
        );
    }
}
