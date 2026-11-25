//! The draggable divider between the LayerPanel's Pages and Layers
//! sections: its paint, its grab band, and the Pages height bounds a drag
//! resolves. The press / drag / release / cursor dispatch both hosts share
//! lives in [`super::layer_split_flow`].

use super::layer_panel::LayerPanel;
use super::layer_panel_walkers::{pages_split_bounds, LayerRegions};
use super::PaintCx;
use crate::{Point2D, Rect};

/// Half-height of the divider's grab band around the hairline (px). The
/// band spans the gap between the two sections, so it never covers a row.
const SPLIT_GRAB_HALF: f32 = 4.0;

impl LayerPanel {
    /// y of the hairline between the Pages and Layers sections.
    fn split_line_y(&self, r: &LayerRegions) -> f32 {
        r.layers_header_y - self.metrics.section_gap / 2.0
    }

    /// The divider's grab band, or `None` on touch chrome (the touch
    /// sheets keep their own gestures and have no divider).
    pub(crate) fn split_handle_rect(&self, rect: Rect, r: &LayerRegions) -> Option<Rect> {
        if self.metrics.touch {
            return None;
        }
        let y = self.split_line_y(r);
        Some(Rect {
            origin: Point2D::new(rect.origin.x, y - SPLIT_GRAB_HALF),
            size: Point2D::new(rect.size.x, SPLIT_GRAB_HALF * 2.0),
        })
    }

    /// Bounds a divider drag may give the Pages viewport in `rect`.
    pub(crate) fn pages_height_bounds(&self, rect: Rect) -> (f32, f32) {
        pages_split_bounds(rect, self.metrics)
    }

    /// Paint the hairline between the sections — mirrors the TS
    /// LayerPanel's `border-t border-border`; a live divider drag tints it
    /// with the primary colour.
    pub(super) fn paint_split_divider(&self, cx: &mut PaintCx<'_>, rect: Rect, r: &LayerRegions) {
        let (thickness, color) = if self.split_dragging {
            (2.0, self.theme.primary)
        } else {
            (1.0, self.theme.border)
        };
        cx.backend.fill_rect(
            Rect {
                origin: Point2D::new(
                    rect.origin.x + self.metrics.row_pad_x,
                    self.split_line_y(r) - (thickness - 1.0) / 2.0,
                ),
                size: Point2D::new(rect.size.x - self.metrics.row_pad_x * 2.0, thickness),
            },
            color,
        );
    }
}
