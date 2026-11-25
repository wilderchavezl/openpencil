//! The LayerPanel Pages / Layers split: the Pages height the user dragged
//! the divider to, and the in-flight divider drag.
//!
//! View-only chrome state: never part of the document, the undo history
//! or `clear_document_derived` (desktop also persists the height).

/// Pages / Layers divider state.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LayerSplitState {
    /// Height (logical px) of the Pages row viewport chosen by dragging
    /// the divider. `None` keeps the default: the page list, capped at the
    /// panel's row limit. The panel re-clamps it to the room it has.
    pub pages_height: Option<f32>,
    /// Divider drag in flight, if any.
    pub drag: Option<LayerSplitDrag>,
}

/// A divider drag: the press anchor plus the height bounds resolved at
/// press time, so moves need no panel geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerSplitDrag {
    pub start_y: f32,
    pub start_height: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl LayerSplitState {
    /// Start a divider drag from `start_y` with the Pages viewport at
    /// `start_height`, bounded to `min_height..=max_height`.
    pub fn begin_drag(
        &mut self,
        start_y: f32,
        start_height: f32,
        min_height: f32,
        max_height: f32,
    ) {
        self.drag = Some(LayerSplitDrag {
            start_y,
            start_height,
            min_height,
            max_height: max_height.max(min_height),
        });
    }

    /// Follow the pointer to `y`. Returns whether the Pages height changed;
    /// `false` with no drag in flight.
    pub fn drag_to(&mut self, y: f32) -> bool {
        let Some(drag) = self.drag else {
            return false;
        };
        let height = (drag.start_height + y - drag.start_y).clamp(drag.min_height, drag.max_height);
        if self.pages_height == Some(height) {
            return false;
        }
        self.pages_height = Some(height);
        true
    }

    /// End the drag. Returns whether one was in flight.
    pub fn end_drag(&mut self) -> bool {
        self.drag.take().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_follows_the_pointer_within_its_bounds() {
        let mut split = LayerSplitState::default();
        assert!(!split.drag_to(50.0), "no drag in flight");
        split.begin_drag(100.0, 96.0, 32.0, 200.0);
        assert!(split.drag_to(140.0));
        assert_eq!(split.pages_height, Some(136.0));
        assert!(!split.drag_to(140.0), "same height is no change");
        split.drag_to(900.0);
        assert_eq!(split.pages_height, Some(200.0));
        split.drag_to(-900.0);
        assert_eq!(split.pages_height, Some(32.0));
        assert!(split.end_drag());
        assert!(!split.end_drag());
        assert_eq!(
            split.pages_height,
            Some(32.0),
            "the height outlives the drag"
        );
    }

    #[test]
    fn inverted_bounds_pin_to_the_minimum() {
        let mut split = LayerSplitState::default();
        split.begin_drag(0.0, 40.0, 32.0, 10.0);
        split.drag_to(500.0);
        assert_eq!(split.pages_height, Some(32.0));
    }
}
