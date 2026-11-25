//! Collapsible Pages / Layers section headers for `LayerPanel`.
//!
//! Each header carries a leading chevron; a press anywhere on the header
//! row (outside the Pages `+`) folds or unfolds that section. Paint and
//! hit-test share the geometry below.

use super::icons::{draw_icon, Icon};
use super::layer_panel::{LayerPanel, LayerPanelHit};
use super::layer_panel_metrics::{add_page_target, glyph_rect_in, LayerPanelMetrics};
use super::layer_panel_walkers::LayerRegions;
use super::PaintCx;
use crate::{Point2D, Rect, TextLayout};
use op_editor_core::editor_ui_state::LayerPanelSection;

/// Space between the chevron glyph and the header label.
const CHEVRON_GAP: f32 = 4.0;

/// Full-width header row — the section's toggle target.
fn section_header_rect(rect: Rect, header_y: f32, m: LayerPanelMetrics) -> Rect {
    Rect {
        origin: Point2D::new(rect.origin.x, header_y),
        size: Point2D::new(rect.size.x, m.section_header_height),
    }
}

/// Leading chevron glyph box, vertically centred on the header row.
fn chevron_glyph(rect: Rect, header_y: f32, m: LayerPanelMetrics) -> Rect {
    let size = m.trailing_glyph_size;
    Rect {
        origin: Point2D::new(
            rect.origin.x + m.row_pad_x - 2.0,
            header_y + (m.section_header_height - size) / 2.0,
        ),
        size: Point2D::new(size, size),
    }
}

/// x where the header label starts (after the chevron).
fn section_label_x(rect: Rect, m: LayerPanelMetrics) -> f32 {
    rect.origin.x + m.row_pad_x - 2.0 + m.trailing_glyph_size + CHEVRON_GAP
}

impl LayerPanel {
    /// Whether `section` is folded to its header row.
    pub(crate) fn is_section_collapsed(&self, section: LayerPanelSection) -> bool {
        self.sections.is_collapsed(section)
    }

    /// Section whose header row contains `point`. Callers test the
    /// Pages `+` first so it keeps its own action.
    pub(super) fn section_header_at(
        &self,
        rect: Rect,
        r: &LayerRegions,
        point: Point2D,
    ) -> Option<LayerPanelHit> {
        [
            (r.pages_header_y, LayerPanelSection::Pages),
            (r.layers_header_y, LayerPanelSection::Layers),
        ]
        .into_iter()
        .find(|(y, _)| section_header_rect(rect, *y, self.metrics).contains(point))
        .map(|(_, section)| LayerPanelHit::ToggleSection(section))
    }

    /// Paint both section headers plus the Pages `+` affordance.
    pub(super) fn paint_section_headers(&self, cx: &mut PaintCx<'_>, rect: Rect, r: &LayerRegions) {
        self.paint_section_header(cx, rect, r.pages_header_y, LayerPanelSection::Pages);
        let plus = glyph_rect_in(
            add_page_target(rect, r.pages_header_y, self.metrics),
            self.metrics.glyph_size,
        );
        draw_icon(
            cx.backend,
            Icon::Plus,
            plus.origin,
            self.metrics.glyph_size,
            self.theme.muted_foreground,
            1.4,
        );
        self.paint_section_header(cx, rect, r.layers_header_y, LayerPanelSection::Layers);
    }

    fn paint_section_header(
        &self,
        cx: &mut PaintCx<'_>,
        rect: Rect,
        y: f32,
        section: LayerPanelSection,
    ) {
        let m = self.metrics;
        let chevron = chevron_glyph(rect, y, m);
        draw_icon(
            cx.backend,
            if self.is_section_collapsed(section) {
                Icon::ChevronRight
            } else {
                Icon::ChevronDown
            },
            chevron.origin,
            chevron.size.x,
            self.theme.muted_foreground,
            1.4,
        );

        let header = section_header_rect(rect, y, m);
        let baseline = if m.touch {
            jian_widgets::centered_text_baseline_y(header, m.header_font)
        } else {
            y + 19.0
        };
        let label = match section {
            LayerPanelSection::Pages => self.pages_label,
            LayerPanelSection::Layers => self.layers_label,
        };
        let text = TextLayout::single_run(
            label,
            "system-ui",
            m.header_font,
            self.theme.muted_foreground.to_jian(),
            Point2D::new(0.0, 0.0),
        );
        cx.backend
            .draw_text(&text, Point2D::new(section_label_x(rect, m), baseline));
    }
}
