//! Shared input flow for the LayerPanel's Pages / Layers divider. Both
//! hosts reach [`press`] through `press_flow::layer_panel_press` (ahead of
//! the layer rows), feed pointer moves to [`drag`] while a drag is in
//! flight and end it with [`release`]; the hosts keep only their platform
//! tails (dirty marking, cursor plumbing).

use crate::widgets::layer_panel::LayerPanel;
use crate::{Point2D, Rect};
use jian_core::CursorHint;
use op_editor_core::EditorState;

/// Whether `point` is on the divider of `panel` painted into `rect`.
fn over_divider(panel: &LayerPanel, rect: Rect, point: Point2D) -> bool {
    panel
        .split_handle_rect(rect, &panel.regions(rect))
        .is_some_and(|handle| handle.contains(point))
}

/// A press on the divider starts a drag from the Pages height on screen.
/// Returns whether the press was taken.
pub fn press(state: &mut EditorState, panel: &LayerPanel, rect: Rect, point: Point2D) -> bool {
    if !over_divider(panel, rect, point) {
        return false;
    }
    let start_height = panel.regions(rect).pages_view_h;
    let (min, max) = panel.pages_height_bounds(rect);
    state
        .editor_ui
        .layer_split
        .begin_drag(point.y, start_height, min, max);
    true
}

/// Follow a pointer move. Returns whether a divider drag owns the move
/// (the host repaints); `false` when no drag is in flight.
pub fn drag(state: &mut EditorState, point: Point2D) -> bool {
    if state.editor_ui.layer_split.drag.is_none() {
        return false;
    }
    state.editor_ui.layer_split.drag_to(point.y);
    true
}

/// End a divider drag. Returns whether one was in flight.
pub fn release(state: &mut EditorState) -> bool {
    state.editor_ui.layer_split.end_drag()
}

/// The north-south cursor a divider drag holds wherever the pointer goes.
pub fn drag_cursor(state: &EditorState) -> Option<CursorHint> {
    state
        .editor_ui
        .layer_split
        .drag
        .map(|_| CursorHint::ResizeNs)
}

/// [`drag_cursor`], else the resize cursor while hovering the divider.
pub fn cursor(
    state: &EditorState,
    panel: &LayerPanel,
    rect: Rect,
    point: Point2D,
) -> Option<CursorHint> {
    drag_cursor(state).or_else(|| over_divider(panel, rect, point).then_some(CursorHint::ResizeNs))
}
