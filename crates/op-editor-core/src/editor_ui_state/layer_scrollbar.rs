//! The left rail's scrollbar thumb drag: the LayerPanel's Pages or Layers
//! row viewport, or the slides tab's list.
//!
//! View-only chrome state, never part of the document or the undo history.
//! `clear_document_derived` drops an in-flight drag: its captured content
//! height belongs to the replaced document.

use jian_core::scroll::ScrollState;

/// Which left-rail list a scrollbar scrolls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerScrollRegion {
    Pages,
    Layers,
    Slides,
}

/// A scrollbar thumb drag in flight. The track geometry is captured at
/// press time so pointer moves need no panel geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerScrollbarDrag {
    pub region: LayerScrollRegion,
    /// Pointer y minus the thumb top at press time.
    pub grab_y: f32,
    /// Screen y of the track top.
    pub track_top: f32,
    pub track_len: f32,
    /// Row content height and viewport height of the region.
    pub content: f32,
    pub view: f32,
    pub min_thumb: f32,
}

impl LayerScrollbarDrag {
    /// Scroll `scroll` so the thumb follows a pointer at `y`. Returns
    /// whether the offset changed.
    pub fn scroll_to(&self, scroll: &mut ScrollState, y: f32) -> bool {
        let before = scroll.offset;
        scroll.set_from_thumb(
            y - self.track_top - self.grab_y,
            self.track_len,
            self.content,
            self.view,
            self.min_thumb,
        );
        scroll.offset != before
    }
}

impl super::EditorUiState {
    /// Vertical scroll state of a left-rail list.
    pub fn layer_scroll_mut(&mut self, region: LayerScrollRegion) -> &mut ScrollState {
        match region {
            LayerScrollRegion::Pages => &mut self.layer_pages_scroll,
            LayerScrollRegion::Layers => &mut self.layer_layers_scroll,
            LayerScrollRegion::Slides => &mut self.slides_panel.scroll,
        }
    }

    /// Follow the in-flight scrollbar drag to pointer `y`. Returns whether
    /// a drag is in flight (the host repaints).
    pub fn drag_layer_scrollbar(&mut self, y: f32) -> bool {
        let Some(drag) = self.layer_scrollbar_drag else {
            return false;
        };
        drag.scroll_to(self.layer_scroll_mut(drag.region), y);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor_ui_state::EditorUiState;

    fn drag(region: LayerScrollRegion, grab_y: f32) -> LayerScrollbarDrag {
        // 100 px track over a 100 px viewport of 400 px content: the
        // thumb is 25 px long and travels 75 px for 300 px of scroll.
        LayerScrollbarDrag {
            region,
            grab_y,
            track_top: 50.0,
            track_len: 100.0,
            content: 400.0,
            view: 100.0,
            min_thumb: 24.0,
        }
    }

    #[test]
    fn thumb_follows_the_pointer_and_clamps_to_the_content() {
        let d = drag(LayerScrollRegion::Layers, 10.0);
        let mut scroll = ScrollState::default();
        assert!(d.scroll_to(&mut scroll, 50.0 + 10.0 + 37.5));
        assert!((scroll.offset - 150.0).abs() < 0.01);
        d.scroll_to(&mut scroll, 1_000.0);
        assert_eq!(scroll.offset, 300.0);
        d.scroll_to(&mut scroll, -1_000.0);
        assert_eq!(scroll.offset, 0.0);
        assert!(!d.scroll_to(&mut scroll, 0.0), "no change reports false");
    }

    #[test]
    fn drag_moves_only_its_own_region() {
        let mut ui = EditorUiState::default();
        assert!(!ui.drag_layer_scrollbar(120.0), "no drag in flight");
        ui.layer_scrollbar_drag = Some(drag(LayerScrollRegion::Pages, 0.0));
        assert!(ui.drag_layer_scrollbar(50.0 + 75.0));
        assert_eq!(ui.layer_pages_scroll.offset, 300.0);
        assert_eq!(ui.layer_layers_scroll.offset, 0.0);
    }
}
