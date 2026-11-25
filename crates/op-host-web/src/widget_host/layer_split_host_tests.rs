//! The LayerPanel Pages / Layers divider through the web ladders: the
//! canvas cursor reads `ns-resize` over it, a press wins over the rows,
//! the drag holds the pointer and the release ends it.

use super::WidgetHost;
use op_editor_core::EditorState;

const VW: f32 = 1440.0;
const VH: f32 = 1000.0;
const PAGE_ROW: f32 = 32.0;

fn host() -> WidgetHost {
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
    let mut host = WidgetHost::new();
    host.editor_state = EditorState::from_document(doc);
    host.last_viewport_w = VW;
    host.last_viewport_h = VH;
    host
}

#[test]
fn dragging_the_divider_resizes_the_pages_list() {
    let mut host = host();
    let rect = host.layer_panel_rect(VH);
    let r = host.layer_panel().regions(rect);
    let (x, y) = (rect.origin.x + rect.size.x / 2.0, r.layers_header_y - 4.0);
    let expected = r.pages_view_h - 2.0 * PAGE_ROW;
    assert_eq!(host.resize_cursor_css(x, y), "ns-resize");
    assert_eq!(host.resize_cursor_css(x, y + 40.0), "");

    assert!(host.apply_press(x, y, VW, VH));
    assert!(host.layer_drag.is_none(), "not a row reorder");
    host.apply_cursor_move(x, y - 2.0 * PAGE_ROW);
    assert_eq!(
        host.editor_state.editor_ui.layer_split.pages_height,
        Some(expected)
    );
    assert_eq!(host.resize_cursor_css(x, y + 200.0), "ns-resize");
    host.apply_release();
    assert!(host.editor_state.editor_ui.layer_split.drag.is_none());
    assert_eq!(host.resize_cursor_css(x, y + 200.0), "");
    assert!(!host.editor_state.history.can_undo());
}
