//! Scrollbars of the left rail's scrolling lists (the LayerPanel's Pages /
//! Layers viewports and the slides tab's list): a thin thumb over each
//! overflowing list, plus the desktop gutter a press grabs to drag it.
//! One geometry feeds paint, the press and the drag.

use crate::theme::Theme;
use crate::widgets::PaintCx;
use crate::{Color, Point2D, Rect};
use jian_core::scroll::ScrollState;
use op_editor_core::editor_ui_state::{LayerScrollRegion, LayerScrollbarDrag};
use op_editor_core::EditorState;

/// Painted thumb width, matching the editor's other list scrollbars.
const THUMB_W: f32 = 2.0;
/// Gutter right edge, inset from the rail edge so it stays clear of the
/// rail's ±4 px width-resize gutter (an earlier press tier).
const GUTTER_RIGHT_INSET: f32 = 4.0;
/// Pressable gutter width. LayerPanel row actions end at least 8 px from
/// the rail edge, so they overlap only the gutter's leftmost 2 px; slide
/// cards end 10 px in. Whole-row click targets do run under the gutter,
/// which is why it only takes a press while its list overflows.
const GUTTER_W: f32 = 6.0;
/// Track inset from the viewport's top and bottom.
const TRACK_PAD: f32 = 2.0;
pub(crate) const MIN_THUMB: f32 = 24.0;
pub(crate) const MIN_THUMB_TOUCH: f32 = 44.0;

/// A list's scrollbar, in screen space.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ScrollbarGeometry {
    /// Pressable gutter: the viewport's full height.
    pub gutter: Rect,
    /// Track the thumb travels along.
    pub track: Rect,
    /// Thumb, or `None` when the rows fit.
    pub thumb: Option<Rect>,
    pub content: f32,
    pub view: f32,
    pub min_thumb: f32,
}

impl ScrollbarGeometry {
    /// Scrollbar of a list whose viewport spans `top..top + view` and ends
    /// at `right_x`, scrolled `offset` into `content` px of rows.
    pub(crate) fn new(
        right_x: f32,
        top: f32,
        view: f32,
        content: f32,
        offset: f32,
        min_thumb: f32,
    ) -> Self {
        let gutter = Rect {
            origin: Point2D::new(right_x - GUTTER_RIGHT_INSET - GUTTER_W, top),
            size: Point2D::new(GUTTER_W, view.max(0.0)),
        };
        let track = Rect {
            origin: Point2D::new(gutter.origin.x, top + TRACK_PAD),
            size: Point2D::new(GUTTER_W, (view - TRACK_PAD * 2.0).max(0.0)),
        };
        let thumb = ScrollState { offset }
            .thumb(track.size.y, content, view, min_thumb)
            .map(|thumb| Rect {
                origin: Point2D::new(track.origin.x, track.origin.y + thumb.offset),
                size: Point2D::new(GUTTER_W, thumb.len),
            });
        Self {
            gutter,
            track,
            thumb,
            content,
            view,
            min_thumb,
        }
    }

    /// Paint the thumb, if the rows overflow.
    pub(crate) fn paint(&self, cx: &mut PaintCx<'_>, theme: &Theme) {
        let Some(thumb) = self.thumb else {
            return;
        };
        let bar = Rect {
            origin: Point2D::new(
                thumb.origin.x + (thumb.size.x - THUMB_W) / 2.0,
                thumb.origin.y,
            ),
            size: Point2D::new(THUMB_W, thumb.size.y),
        };
        cx.backend.fill_round_rect(
            bar,
            THUMB_W / 2.0,
            Color {
                a: 0.55,
                ..theme.muted_foreground
            },
        );
    }

    /// A press in the gutter of an overflowing list grabs the thumb; a
    /// press on the track beside it first centres the thumb on the
    /// pointer. Returns whether the press was taken: outside the gutter,
    /// or with rows that fit, it belongs to the rows.
    pub(crate) fn press(
        &self,
        state: &mut EditorState,
        region: LayerScrollRegion,
        point: Point2D,
    ) -> bool {
        if !self.gutter.contains(point) {
            return false;
        }
        let Some(thumb) = self.thumb else {
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
            track_top: self.track.origin.y,
            track_len: self.track.size.y,
            content: self.content,
            view: self.view,
            min_thumb: self.min_thumb,
        };
        drag.scroll_to(state.editor_ui.layer_scroll_mut(region), point.y);
        state.editor_ui.layer_scrollbar_drag = Some(drag);
        true
    }
}
