//! Shared input flow for the left rail's scrollbars. Both hosts reach
//! [`press`] through `press_flow::layer_panel_press` (ahead of the layer
//! rows) and the slides tab's through `slides_panel_flow::press`; they feed
//! pointer moves to [`drag`] while a drag is in flight and end it with
//! [`release`], keeping only their platform tails.

use crate::widgets::layer_panel::LayerPanel;
use crate::{Point2D, Rect};
use op_editor_core::editor_ui_state::LayerScrollRegion;
use op_editor_core::EditorState;

/// A press in the scrollbar gutter of an overflowing Pages or Layers
/// viewport grabs the thumb (see `rail_scrollbar`). Returns whether the
/// press was taken. Touch chrome scrolls by dragging the rows, so it has no
/// grabbable gutter.
pub fn press(state: &mut EditorState, panel: &LayerPanel, rect: Rect, point: Point2D) -> bool {
    if panel.metrics.touch {
        return false;
    }
    let r = panel.regions(rect);
    [LayerScrollRegion::Pages, LayerScrollRegion::Layers]
        .into_iter()
        .any(|region| {
            panel
                .scrollbar(rect, &r, region)
                .press(state, region, point)
        })
}

/// Follow a pointer move. Returns whether a scrollbar drag owns the move
/// (the host repaints); `false` when no drag is in flight.
pub fn drag(state: &mut EditorState, point: Point2D) -> bool {
    state.editor_ui.drag_layer_scrollbar(point.y)
}

/// End a scrollbar drag. Returns whether one was in flight.
pub fn release(state: &mut EditorState) -> bool {
    state.editor_ui.layer_scrollbar_drag.take().is_some()
}
