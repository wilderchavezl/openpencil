//! The LayerPanel's Pages / Layers scrollbars, laid out from
//! [`LayerRegions`], the same source paint and hit-test use.

use crate::widgets::layer_panel::LayerPanel;
use crate::widgets::layer_panel_walkers::LayerRegions;
use crate::widgets::rail_scrollbar::{ScrollbarGeometry, MIN_THUMB, MIN_THUMB_TOUCH};
use crate::widgets::PaintCx;
use crate::Rect;
use op_editor_core::editor_ui_state::LayerScrollRegion;

impl LayerPanel {
    /// Scrollbar of the Pages or Layers viewport of the panel painted
    /// into `rect`.
    pub(crate) fn scrollbar(
        &self,
        rect: Rect,
        r: &LayerRegions,
        region: LayerScrollRegion,
    ) -> ScrollbarGeometry {
        let (top, view, scroll) = match region {
            LayerScrollRegion::Pages => (r.pages_rows_top, r.pages_view_h, r.pages),
            // The panel has no slides list; only the Layers rows remain.
            LayerScrollRegion::Layers | LayerScrollRegion::Slides => {
                (r.layers_rows_top, r.layers_view_h, r.layers)
            }
        };
        ScrollbarGeometry::new(
            rect.origin.x + rect.size.x,
            top,
            view,
            view + scroll.max_offset,
            scroll.offset,
            if self.metrics.touch {
                MIN_THUMB_TOUCH
            } else {
                MIN_THUMB
            },
        )
    }

    /// Paint the thumb of every overflowing region.
    pub(crate) fn paint_scrollbars(&self, cx: &mut PaintCx<'_>, rect: Rect, r: &LayerRegions) {
        for region in [LayerScrollRegion::Pages, LayerScrollRegion::Layers] {
            self.scrollbar(rect, r, region).paint(cx, &self.theme);
        }
    }
}
