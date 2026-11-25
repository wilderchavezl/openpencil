//! The LayerPanel scrollbar through the native ladders: a press on the
//! Layers thumb wins over the rows, the drag holds the pointer wherever it
//! goes, and the release ends it without touching the document history.

use crate::widget_host::WidgetHostNative as Host;
use op_editor_core::EditorState;

const VW: f32 = 1440.0;
const VH: f32 = 1000.0;

/// One page holding 80 rectangles: far more rows than the rail shows.
fn host() -> Host {
    let rows = (0..80)
        .map(|row| format!(r#"{{"type":"rectangle","id":"r-{row}","width":10,"height":10}}"#))
        .collect::<Vec<_>>()
        .join(",");
    let src = format!(
        r#"{{"version":"1.0.0","children":[],"pages":[{{"id":"p","name":"Page","children":[{rows}]}}]}}"#
    );
    let doc = jian_ops_schema::load_str(&src)
        .expect("fixture parses")
        .value;
    let mut host = Host::new();
    host.install_imported_state(EditorState::from_document(doc));
    host.last_viewport_w = VW;
    host.last_viewport_h = VH;
    host
}

#[test]
fn dragging_the_layers_thumb_scrolls_the_rows() {
    let mut host = host();
    let rect = host.layers_content_rect(VW, VH);
    let r = host.layer_panel().regions(rect);
    assert!(r.layers.max_offset > 0.0, "the rows overflow");
    // At offset 0 the thumb starts at the track top, 2 px under the rows'
    // top, in the gutter 4..10 px in from the panel's right edge.
    let (x, y) = (rect.origin.x + rect.size.x - 7.0, r.layers_rows_top + 12.0);

    assert!(host.apply_press(x, y, VW, VH));
    assert!(host.layer_drag.is_none(), "not a row reorder");
    // Down past the rail's bottom and off to the side: the drag keeps the
    // pointer and pins the rows to their end.
    host.apply_cursor_move(x + 300.0, y + 5_000.0);
    let ui = &host.editor_state().editor_ui;
    assert_eq!(ui.layer_layers_scroll.offset, r.layers.max_offset);
    assert_eq!(ui.layer_pages_scroll.offset, 0.0);
    host.apply_release_with_viewport(VW, VH);

    let state = host.editor_state();
    assert!(state.editor_ui.layer_scrollbar_drag.is_none());
    assert_eq!(
        state.editor_ui.layer_layers_scroll.offset,
        r.layers.max_offset
    );
    assert!(!state.history.can_undo());
}
