use std::ops::Range;

/// A drag-to-select text range; `selection_reversed` tracks which edge is moving 
/// so `selected_range` stays normalized (start <= end).
pub struct Selection {
    pub selected_range: Range<usize>,
    pub selection_reversed: bool,
    is_selecting: bool,
}

impl Selection {
    pub fn new(selected_range: Range<usize>) -> Self {
        Self {
            selected_range,
            selection_reversed: false,
            is_selecting: false,
        }
    }

    pub fn select_to(&mut self, offset: usize) {
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
    }

    pub fn move_to(&mut self, offset: usize) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;
    }

    pub fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    pub fn start_selecting(&mut self) {
        self.is_selecting = true;
    }

    pub fn stop_selecting(&mut self) {
        self.is_selecting = false;
    }

    pub fn is_selecting(&self) -> bool {
        self.is_selecting
    }
}

#[cfg(test)]
mod tests {
    use crate::selection::Selection;

    #[test]
    fn should_create_new_selection() {
        let selection = Selection::new(0..0);
        assert_eq!(selection.selected_range, 0..0);
        assert!(!selection.is_selecting);
        assert!(!selection.selection_reversed)
    }

    #[test]
    fn should_flip_reversed_when_dragged_past_anchor() {
        let mut selection = Selection::new(5..5);
        selection.select_to(2);

        assert_eq!(selection.selected_range, 2..5);
        assert!(selection.selection_reversed);
    }

    #[test]
    fn should_flip_reversed_back_when_dragged_past_anchor_from_reversed_state() {
        let mut selection = Selection::new(2..5);
        selection.selection_reversed = true;
        selection.select_to(8);

        assert_eq!(selection.selected_range, 5..8);
        assert!(!selection.selection_reversed);
    }

    #[test]
    fn should_collapse_range_and_reset_reversed_on_move_to() {
        let mut selection = Selection::new(5..5);
        selection.selection_reversed = true;
        selection.move_to(0);

        assert_eq!(selection.selected_range, 0..0);
        assert!(!selection.selection_reversed);
    }

    #[test]
    fn should_return_end_as_cursor_offset_when_not_reversed() {
        let selection = Selection::new(3..7);
        assert_eq!(selection.cursor_offset(), 7);
    }

    #[test]
    fn should_return_start_as_cursor_offset_when_reversed() {
        let mut selection = Selection::new(3..7);
        selection.selection_reversed = true;
        assert_eq!(selection.cursor_offset(), 3);
    }

    #[test]
    fn should_extend_range_without_flipping_when_not_reversed() {
        let mut selection = Selection::new(5..5);
        selection.select_to(8);

        assert_eq!(selection.selected_range, 5..8);
        assert!(!selection.selection_reversed);
    }

    #[test]
    fn should_extend_range_without_flipping_when_reversed() {
        let mut selection = Selection::new(2..5);
        selection.selection_reversed = true;
        selection.select_to(0);

        assert_eq!(selection.selected_range, 0..5);
        assert!(selection.selection_reversed);
    }

    #[test]
    fn should_start_selecting() {
        let mut selection = Selection::new(3..7);
        selection.start_selecting();
        assert!(selection.is_selecting())
    }

    #[test]
    fn should_stop_selecting() {
        let mut selection = Selection::new(3..7);
        selection.start_selecting();
        selection.stop_selecting();
        assert!(!selection.is_selecting())
    }
}
