//! LayerPanel Pages / Layers divider regressions: the chosen Pages height
//! and its bounds, the shared press / drag / release flow, the cursor, and
//! that rows keep hit-testing where they paint after a resize.

use super::layer_panel::{LayerPanel, LayerPanelHit, LAYER_PANEL_WIDTH};
use super::layer_split_flow as flow;
use super::test_capture_backend::CaptureBackend;
use super::{PaintCx, Widget};
use crate::{Point2D, Rect};
use jian_core::CursorHint;
use op_editor_core::EditorState;

const HEADER: f32 = 28.0;
const PAGE_ROW: f32 = 32.0;
const LAYER_ROW: f32 = 28.0;
const GAP: f32 = 8.0;
/// Default Pages height in a 600 px rail: half of the 520 px rows share.
const DEFAULT_PAGES_H: f32 = 260.0;

/// `pages` pages named `Page N`, each with four rectangles.
fn paged_state(pages: usize) -> EditorState {
    let page_json = (0..pages)
        .map(|page| {
            let children = (0..4)
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
        .expect("split fixture parses")
        .value;
    EditorState::from_document(doc)
}

fn rail(height: f32) -> Rect {
    Rect {
        origin: Point2D::new(0.0, 0.0),
        size: Point2D::new(LAYER_PANEL_WIDTH, height),
    }
}

/// y of the divider hairline for `panel` in `rect`.
fn divider_y(panel: &LayerPanel, rect: Rect) -> f32 {
    panel.regions(rect).layers_header_y - GAP / 2.0
}

#[test]
fn default_split_is_half_and_half_on_desktop() {
    // Rows share 600 - (8 + header + gap + header + 8) = 520 px.
    let rows_room = 600.0 - (8.0 + HEADER + GAP + HEADER + 8.0);
    for pages in [1, 2, 12] {
        // A blank canvas (one page) splits evenly too: the page list
        // simply leaves blank room under its rows.
        let state = paged_state(pages);
        let r = LayerPanel::from_editor(&state).regions(rail(600.0));
        assert_eq!(r.pages_view_h, rows_room / 2.0, "{pages} pages");
        assert_eq!(r.layers_view_h, rows_room / 2.0, "{pages} pages");
    }

    // Half of a 140 px rail's 60 px rows share is 30 px: still one full
    // page row (the one layer row Layers keeps fits below it).
    let r = LayerPanel::from_editor(&paged_state(12)).regions(rail(140.0));
    assert_eq!(r.pages_view_h, PAGE_ROW);
}

#[test]
fn chosen_height_is_honoured_within_its_bounds() {
    let mut state = paged_state(20);
    let rect = rail(600.0);
    // Room: 600 - (8 + header + gap + header + one layer row + 8).
    let room = 600.0 - (8.0 + HEADER + GAP + HEADER + LAYER_ROW + 8.0);

    for (chosen, expected) in [
        (250.0, 250.0),   // free height, not a row multiple
        (5.0, PAGE_ROW),  // at least one page row
        (10_000.0, room), // never past the rail's room
    ] {
        state.editor_ui.layer_split.pages_height = Some(chosen);
        let panel = LayerPanel::from_editor(&state);
        assert_eq!(panel.regions(rect).pages_view_h, expected, "chose {chosen}");
        assert_eq!(panel.pages_height_bounds(rect), (PAGE_ROW, room));
    }

    // A short page list may sit in a taller viewport: the split is the
    // user's, the blank room stays under its rows.
    let mut short = paged_state(2);
    short.editor_ui.layer_split.pages_height = Some(400.0);
    let r = LayerPanel::from_editor(&short).regions(rect);
    assert_eq!(r.pages_view_h, 400.0);
    assert_eq!(r.pages.max_offset, 0.0, "nothing to scroll");
}

#[test]
fn dragging_the_divider_resizes_pages_and_moves_layers() {
    let mut state = paged_state(12);
    let rect = rail(600.0);
    let panel = LayerPanel::from_editor(&state);
    let y = divider_y(&panel, rect);
    let off = Point2D::new(60.0, y + 20.0);
    assert!(
        !flow::press(&mut state, &panel, rect, off),
        "rows are not the divider"
    );
    assert!(!flow::drag(&mut state, off), "no drag in flight");

    let grab = Point2D::new(60.0, y);
    assert!(flow::press(&mut state, &panel, rect, grab));
    assert_eq!(flow::drag_cursor(&state), Some(CursorHint::ResizeNs));
    assert!(flow::drag(&mut state, Point2D::new(60.0, y + 100.0)));
    assert_eq!(
        state.editor_ui.layer_split.pages_height,
        Some(DEFAULT_PAGES_H + 100.0)
    );
    assert!(flow::release(&mut state));
    assert!(!flow::release(&mut state));
    assert_eq!(flow::drag_cursor(&state), None);

    // Rows follow the divider: page 7 is now on screen and the first
    // layer row sits right under the moved Layers header.
    let panel = LayerPanel::from_editor(&state);
    let r = panel.regions(rect);
    assert_eq!(r.pages_view_h, DEFAULT_PAGES_H + 100.0);
    let page_7 = Point2D::new(60.0, r.pages_rows_top + 7.5 * PAGE_ROW);
    assert_eq!(panel.hit_test(rect, page_7), Some(LayerPanelHit::Page(7)));
    let first_layer = Point2D::new(60.0, r.layers_rows_top + LAYER_ROW / 2.0);
    assert_eq!(
        panel.hit_test(rect, first_layer),
        Some(LayerPanelHit::Layer(panel.items[0].node_id.clone()))
    );
}

#[test]
fn divider_hover_cursor_and_drag_tint() {
    let mut state = paged_state(3);
    let rect = rail(600.0);
    let panel = LayerPanel::from_editor(&state);
    let y = divider_y(&panel, rect);
    assert_eq!(
        flow::cursor(&state, &panel, rect, Point2D::new(60.0, y)),
        Some(CursorHint::ResizeNs)
    );
    assert_eq!(
        flow::cursor(&state, &panel, rect, Point2D::new(60.0, y + 30.0)),
        None
    );

    let paint_line = |state: &EditorState| {
        let panel = LayerPanel::from_editor(state);
        let mut backend = CaptureBackend::default();
        panel.paint(
            &mut PaintCx {
                backend: &mut backend,
            },
            rect,
        );
        let y = divider_y(&panel, rect);
        backend
            .rect_fills
            .iter()
            .find(|(r, _)| r.origin.y <= y && r.origin.y + r.size.y >= y && r.size.y <= 2.0)
            .copied()
            .expect("divider hairline painted")
    };
    let (idle, idle_color) = paint_line(&state);
    assert_eq!(idle.size.y, 1.0);
    assert!(flow::press(&mut state, &panel, rect, Point2D::new(60.0, y)));
    let (dragging, drag_color) = paint_line(&state);
    assert_eq!(dragging.size.y, 2.0);
    assert_ne!(drag_color, idle_color);
}

#[test]
fn touch_chrome_has_no_divider() {
    let mut state = paged_state(3);
    state.editor_ui.touch = true;
    let rect = rail(600.0);
    let panel = LayerPanel::from_editor(&state);
    let y = divider_y(&panel, rect);
    assert!(panel
        .split_handle_rect(rect, &panel.regions(rect))
        .is_none());
    assert!(!flow::press(
        &mut state,
        &panel,
        rect,
        Point2D::new(60.0, y)
    ));
}
