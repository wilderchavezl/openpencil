//! Shared input flow for the LayerPanel scrollbars. Both hosts reach
//! [`press`] through `press_flow::layer_panel_press` (ahead of the layer
//! rows), feed pointer moves to [`drag`] while a drag is in flight and end
//! it with [`release`]; the hosts keep only their platform tails.

use crate::widgets::layer_panel::LayerPanel;
use crate::{Point2D, Rect};
use op_editor_core::editor_ui_state::{LayerScrollRegion, LayerScrollbarDrag};
use op_editor_core::EditorState;

/// A press in an overflowing region's scrollbar gutter grabs the thumb; a
/// press on the track beside it first centres the thumb on the pointer.
/// Returns whether the press was taken. Touch chrome scrolls by dragging
/// the rows, so it has no grabbable gutter.
pub fn press(state: &mut EditorState, panel: &LayerPanel, rect: Rect, point: Point2D) -> bool {
    if panel.metrics.touch {
        return false;
    }
    let r = panel.regions(rect);
    for region in [LayerScrollRegion::Pages, LayerScrollRegion::Layers] {
        let bar = panel.scrollbar(rect, &r, region);
        if !bar.gutter.contains(point) {
            continue;
        }
        let Some(thumb) = bar.thumb else {
            return false;
        };
        let grab_y = if thumb.contains(point) {
            point.y - thumb.origin.y
        } else {
            thumb.size.y / 2.0
        };
        let drag = LayerScrollbarDrag {
            region,
            grab_y,
            track_top: bar.track.origin.y,
            track_len: bar.track.size.y,
            content: bar.content,
            view: bar.view,
            min_thumb: bar.min_thumb,
        };
        drag.scroll_to(state.editor_ui.layer_scroll_mut(region), point.y);
        state.editor_ui.layer_scrollbar_drag = Some(drag);
        return true;
    }
    false
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
