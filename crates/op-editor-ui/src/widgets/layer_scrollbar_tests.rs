//! LayerPanel scrollbar regressions: the thumb is painted over each
//! overflowing Pages / Layers viewport, and the shared press / drag /
//! release flow scrolls the region it was pressed in.

use super::layer_panel::{LayerPanel, LayerPanelHit, LAYER_PANEL_WIDTH};
use super::layer_scrollbar_flow as flow;
use super::press_flow;
use super::test_capture_backend::CaptureBackend;
use super::{PaintCx, Widget};
use crate::{Point2D, Rect};
use op_editor_core::editor_ui_state::LayerScrollRegion;
use op_editor_core::EditorState;

/// `pages` pages named `Page N`; the first holds `layers` rectangles.
fn state_with(pages: usize, layers: usize) -> EditorState {
    let page_json = (0..pages)
        .map(|page| {
            let count = if page == 0 { layers } else { 1 };
            let children = (0..count)
                .map(|row| {
                    format!(
                        r#"{{"type":"rectangle","id":"n-{page}-{row}","name":"Layer {row}","width":10,"height":10}}"#
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(r#"{{"id":"p-{page}","name":"Page {page}","children":[{children}]}}"#)
        })
        .collect::<Vec<_>>()
        .join(",");
    let src = format!(r#"{{"version":"1.0.0","children":[],"pages":[{page_json}]}}"#);
    let doc = jian_ops_schema::load_str(&src)
        .expect("scrollbar fixture parses")
        .value;
    EditorState::from_document(doc)
}

fn rail() -> Rect {
    Rect {
        origin: Point2D::new(0.0, 0.0),
        size: Point2D::new(LAYER_PANEL_WIDTH, 600.0),
    }
}

fn thumb(state: &EditorState, region: LayerScrollRegion) -> Option<Rect> {
    let panel = LayerPanel::from_editor(state);
    let r = panel.regions(rail());
    panel.scrollbar(rail(), &r, region).thumb
}

fn center(rect: Rect) -> Point2D {
    Point2D::new(
        rect.origin.x + rect.size.x / 2.0,
        rect.origin.y + rect.size.y / 2.0,
    )
}

/// The thin thumbs `paint` draws in the scrollbar gutter.
fn painted_thumbs(state: &EditorState) -> Vec<Rect> {
    let panel = LayerPanel::from_editor(state);
    let mut backend = CaptureBackend::default();
    panel.paint(
        &mut PaintCx {
            backend: &mut backend,
        },
        rail(),
    );
    let gutter_left = LAYER_PANEL_WIDTH - 10.0;
    backend
        .round_fills
        .iter()
        .filter(|(r, _, _)| r.origin.x >= gutter_left && r.size.x <= 2.0)
        .map(|(r, _, _)| *r)
        .collect()
}

#[test]
fn thumbs_paint_only_over_overflowing_regions() {
    // 3 pages and 3 layers fit: no scrollbar at all.
    assert!(painted_thumbs(&state_with(3, 3)).is_empty());
    assert_eq!(thumb(&state_with(3, 3), LayerScrollRegion::Pages), None);

    // 12 pages overflow the six-row Pages viewport, 60 layers the rest.
    let state = state_with(12, 60);
    let pages = thumb(&state, LayerScrollRegion::Pages).expect("pages overflow");
    let layers = thumb(&state, LayerScrollRegion::Layers).expect("layers overflow");
    let painted = painted_thumbs(&state);
    assert_eq!(painted.len(), 2, "one thumb per region: {painted:?}");
    for (bar, thumb) in painted.iter().zip([pages, layers]) {
        assert_eq!((bar.origin.y, bar.size.y), (thumb.origin.y, thumb.size.y));
    }
    let r = LayerPanel::from_editor(&state).regions(rail());
    assert!(pages.origin.y >= r.pages_rows_top);
    assert!(pages.origin.y + pages.size.y <= r.pages_rows_top + r.pages_view_h);
    assert!(layers.origin.y >= r.layers_rows_top);
}

#[test]
fn thumb_tracks_the_scroll_offset() {
    let mut state = state_with(3, 60);
    let top = thumb(&state, LayerScrollRegion::Layers).unwrap();
    state.editor_ui.layer_layers_scroll.offset = 1_000_000.0;
    let bottom = thumb(&state, LayerScrollRegion::Layers).unwrap();
    let r = LayerPanel::from_editor(&state).regions(rail());
    assert!(bottom.origin.y > top.origin.y);
    assert_eq!(
        bottom.origin.y + bottom.size.y,
        r.layers_rows_top + r.layers_view_h - 2.0,
        "a clamped offset pins the thumb to the track end"
    );
}

#[test]
fn gutter_stays_clear_of_the_panel_width_resize_grip() {
    let state = state_with(12, 60);
    let panel = LayerPanel::from_editor(&state);
    let r = panel.regions(rail());
    for region in [LayerScrollRegion::Pages, LayerScrollRegion::Layers] {
        let gutter = panel.scrollbar(rail(), &r, region).gutter;
        // The host's width-resize grip owns |x - right edge| <= 4.
        assert!(gutter.origin.x + gutter.size.x <= LAYER_PANEL_WIDTH - 4.0);
    }
}

#[test]
fn dragging_the_thumb_scrolls_only_its_region() {
    let mut state = state_with(12, 60);
    let rect = rail();
    let grab = center(thumb(&state, LayerScrollRegion::Layers).unwrap());
    let panel = LayerPanel::from_editor(&state);
    assert!(flow::press(&mut state, &panel, rect, grab));
    assert_eq!(
        state.editor_ui.layer_layers_scroll.offset, 0.0,
        "grabbing the thumb does not jump"
    );

    assert!(flow::drag(&mut state, Point2D::new(grab.x, grab.y + 50.0)));
    let moved = state.editor_ui.layer_layers_scroll.offset;
    assert!(moved > 0.0);
    assert_eq!(state.editor_ui.layer_pages_scroll.offset, 0.0);
    // The pointer may leave the gutter: the drag follows its y alone.
    flow::drag(&mut state, Point2D::new(500.0, 10_000.0));
    let max = LayerPanel::from_editor(&state)
        .regions(rect)
        .layers
        .max_offset;
    assert_eq!(state.editor_ui.layer_layers_scroll.offset, max);

    assert!(flow::release(&mut state));
    assert!(!flow::release(&mut state));
    assert!(!flow::drag(&mut state, grab), "no drag after release");
    assert_eq!(state.editor_ui.layer_layers_scroll.offset, max);
}

#[test]
fn pressing_the_track_centres_the_thumb_there() {
    let mut state = state_with(12, 3);
    let rect = rail();
    let panel = LayerPanel::from_editor(&state);
    let r = panel.regions(rect);
    let bar = panel.scrollbar(rect, &r, LayerScrollRegion::Pages);
    let track_end = Point2D::new(
        center(bar.gutter).x,
        bar.track.origin.y + bar.track.size.y - 1.0,
    );
    assert!(flow::press(&mut state, &panel, rect, track_end));
    let after = thumb(&state, LayerScrollRegion::Pages).unwrap();
    assert_eq!(
        after.origin.y + after.size.y,
        bar.track.origin.y + bar.track.size.y,
        "a press near the end pages to the end"
    );
    assert!(flow::release(&mut state));
}

#[test]
fn gutter_press_falls_through_to_the_rows_when_they_fit() {
    let mut state = state_with(3, 3);
    let rect = rail();
    let panel = LayerPanel::from_editor(&state);
    let r = panel.regions(rect);
    let gutter = panel.scrollbar(rect, &r, LayerScrollRegion::Layers).gutter;
    let point = Point2D::new(center(gutter).x, r.layers_rows_top + 14.0);
    let mut layer_drag = None;
    assert!(!press_flow::layer_panel_press(
        &panel,
        rect,
        point,
        &mut state,
        &mut layer_drag
    ));
    assert!(state.editor_ui.layer_scrollbar_drag.is_none());
    assert!(matches!(
        panel.hit_test(rect, point),
        Some(LayerPanelHit::Layer(_))
    ));
    assert!(layer_drag.is_some(), "the row still seeds a reorder");
}

#[test]
fn shared_press_takes_the_gutter_before_the_rows() {
    let mut state = state_with(3, 60);
    let rect = rail();
    let grab = center(thumb(&state, LayerScrollRegion::Layers).unwrap());
    let mut layer_drag = None;
    assert!(press_flow::layer_panel_press(
        &LayerPanel::from_editor(&state),
        rect,
        grab,
        &mut state,
        &mut layer_drag
    ));
    assert!(layer_drag.is_none(), "no reorder candidate under the thumb");
    assert_eq!(
        state.editor_ui.layer_scrollbar_drag.map(|d| d.region),
        Some(LayerScrollRegion::Layers)
    );
}

#[test]
fn touch_chrome_paints_the_thumb_but_has_no_gutter() {
    let mut state = state_with(3, 60);
    state.editor_ui.touch = true;
    let rect = rail();
    let grab = center(thumb(&state, LayerScrollRegion::Layers).expect("thumb on touch"));
    let panel = LayerPanel::from_editor(&state);
    assert!(!flow::press(&mut state, &panel, rect, grab));
}

#[test]
fn replacing_the_document_drops_a_drag_in_flight() {
    let mut state = state_with(3, 60);
    let grab = center(thumb(&state, LayerScrollRegion::Layers).unwrap());
    let panel = LayerPanel::from_editor(&state);
    flow::press(&mut state, &panel, rail(), grab);
    state.editor_ui.clear_document_derived();
    assert!(!flow::drag(&mut state, grab));
}
