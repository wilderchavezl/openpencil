//! The LayerPanel Pages / Layers divider through the native ladders: a
//! press on the divider wins over the rows, the drag holds the pointer
//! even over the rows, the cursor reads north-south, and the release ends
//! the drag without touching the document history.

use crate::widget_host::{CursorHint, WidgetHostNative};
use op_editor_core::EditorState;

const VW: f32 = 1440.0;
const VH: f32 = 1000.0;
const PAGE_ROW: f32 = 32.0;

fn host() -> WidgetHostNative {
    let pages = (0..12)
        .map(|page| {
            format!(
                r#"{{"id":"p-{page}","name":"Page {page}","children":[
                    {{"type":"rectangle","id":"r-{page}","width":10,"height":10}}
                ]}}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let src = format!(r#"{{"version":"1.0.0","children":[],"pages":[{pages}]}}"#);
    let doc = jian_ops_schema::load_str(&src)
        .expect("fixture parses")
        .value;
    let mut host = WidgetHostNative::new();
    host.install_imported_state(EditorState::from_document(doc));
    host.last_viewport_w = VW;
    host.last_viewport_h = VH;
    host
}

/// Screen point on the divider hairline.
fn divider(host: &WidgetHostNative) -> (f32, f32) {
    let rect = host.layers_content_rect(VW, VH);
    let r = host.layer_panel().regions(rect);
    (rect.origin.x + rect.size.x / 2.0, r.layers_header_y - 4.0)
}

#[test]
fn dragging_the_divider_resizes_the_pages_list() {
    let mut host = host();
    assert!(host.layers_panel_visible());
    let (x, y) = divider(&host);
    let rect = host.layers_content_rect(VW, VH);
    let start = host.layer_panel().regions(rect).pages_view_h;
    let expected = start - 2.0 * PAGE_ROW;
    assert_eq!(host.cursor_hint(x, y, VW, VH), CursorHint::ResizeNs);

    assert!(host.apply_press(x, y, VW, VH));
    assert!(host.layer_drag.is_none(), "not a row reorder");
    // Back up over the page rows: the drag keeps the pointer.
    host.apply_cursor_move(x, y - 2.0 * PAGE_ROW);
    assert_eq!(
        host.editor_state().editor_ui.layer_split.pages_height,
        Some(expected)
    );
    assert_eq!(host.cursor_hint(x, y + 200.0, VW, VH), CursorHint::ResizeNs);
    host.apply_release_with_viewport(VW, VH);

    let ui = &host.editor_state().editor_ui;
    assert!(ui.layer_split.drag.is_none());
    assert_eq!(ui.layer_split.pages_height, Some(expected));
    assert!(!host.editor_state().history.can_undo());
    assert_eq!(host.layer_panel().regions(rect).pages_view_h, expected);
}
