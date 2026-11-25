//! Scrollbars of the LayerPanel's bounded Pages / Layers row viewports: a
//! thin thumb over each overflowing region, plus the desktop gutter a
//! press grabs to drag it. Geometry comes from [`LayerRegions`], the same
//! source paint and hit-test use.

use crate::widgets::layer_panel::LayerPanel;
use crate::widgets::layer_panel_walkers::{LayerRegions, LayerResolvedScroll};
use crate::widgets::PaintCx;
use crate::{Color, Point2D, Rect};
use jian_core::scroll::ScrollState;
use op_editor_core::editor_ui_state::LayerScrollRegion;

/// Painted thumb width, matching the editor's other list scrollbars.
const THUMB_W: f32 = 2.0;
/// Gutter right edge, inset from the panel edge so it stays clear of the
/// panel's ±4 px width-resize gutter (an earlier press tier).
const GUTTER_RIGHT_INSET: f32 = 4.0;
/// Pressable gutter width. Row actions end at least 8 px from the panel
/// edge, so they overlap only the gutter's leftmost 2 px. Whole-row click
/// targets do run under the gutter, which is why it only takes a press
/// while its region overflows.
const GUTTER_W: f32 = 6.0;
/// Track inset from the viewport's top and bottom.
const TRACK_PAD: f32 = 2.0;
const MIN_THUMB: f32 = 24.0;
const MIN_THUMB_TOUCH: f32 = 44.0;

/// One region's viewport and clamped scroll, as resolved by `regions`.
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

impl LayerPanel {
    /// Scrollbar geometry of `region` for the panel painted into `rect`.
    pub(crate) fn scrollbar(
        &self,
        rect: Rect,
        r: &LayerRegions,
        region: LayerScrollRegion,
    ) -> ScrollbarGeometry {
        let (top, view, scroll): (f32, f32, LayerResolvedScroll) = match region {
            LayerScrollRegion::Pages => (r.pages_rows_top, r.pages_view_h, r.pages),
            LayerScrollRegion::Layers => (r.layers_rows_top, r.layers_view_h, r.layers),
        };
        let gutter = Rect {
            origin: Point2D::new(
                rect.origin.x + rect.size.x - GUTTER_RIGHT_INSET - GUTTER_W,
                top,
            ),
            size: Point2D::new(GUTTER_W, view),
        };
        let track = Rect {
            origin: Point2D::new(gutter.origin.x, top + TRACK_PAD),
            size: Point2D::new(GUTTER_W, (view - TRACK_PAD * 2.0).max(0.0)),
        };
        let content = view + scroll.max_offset;
        let min_thumb = if self.metrics.touch {
            MIN_THUMB_TOUCH
        } else {
            MIN_THUMB
        };
        let thumb = ScrollState {
            offset: scroll.offset,
        }
        .thumb(track.size.y, content, view, min_thumb)
        .map(|thumb| Rect {
            origin: Point2D::new(track.origin.x, track.origin.y + thumb.offset),
            size: Point2D::new(GUTTER_W, thumb.len),
        });
        ScrollbarGeometry {
            gutter,
            track,
            thumb,
            content,
            view,
            min_thumb,
        }
    }

    /// Paint the thumb of every overflowing region.
    pub(crate) fn paint_scrollbars(&self, cx: &mut PaintCx<'_>, rect: Rect, r: &LayerRegions) {
        for region in [LayerScrollRegion::Pages, LayerScrollRegion::Layers] {
            let Some(thumb) = self.scrollbar(rect, r, region).thumb else {
                continue;
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
                    ..self.theme.muted_foreground
                },
            );
        }
    }
}
