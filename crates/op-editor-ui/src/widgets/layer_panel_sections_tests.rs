//! Collapsible Pages / Layers section regressions: header hit targets,
//! folded geometry, the folded header paint, the history-free toggle, and
//! that a folded section neither scrolls nor auto-reveals while hidden.

use super::layer_panel::{LayerPanel, LayerPanelHit, LAYER_PANEL_WIDTH};
use super::press_flow::{apply_layer_panel_click, LayerPanelClick};
use super::scroll_flow::{reveal_layer_panel_selection, scroll_layer_panel};
use super::test_capture_backend::CaptureBackend;
use super::PaintCx;
use super::Widget;
use crate::{Point2D, Rect};
use op_editor_core::editor_ui_state::LayerPanelSection;
use op_editor_core::{EditorState, NodeId};

const HEADER: f32 = 28.0;
const PAGE_ROW: f32 = 32.0;
const LAYER_ROW: f32 = 28.0;
const GAP: f32 = 8.0;

/// `pages` pages named `Page N`, each with three rectangles.
fn paged_state(pages: usize) -> EditorState {
    let page_json = (0..pages)
        .map(|page| {
            let children = (0..3)
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
        .expect("section fixture parses")
        .value;
    EditorState::from_document(doc)
}

fn rail(height: f32) -> Rect {
    Rect {
        origin: Point2D::new(0.0, 0.0),
        size: Point2D::new(LAYER_PANEL_WIDTH, height),
    }
}

fn click(state: &mut EditorState, rect: Rect, point: Point2D) -> Option<LayerPanelClick> {
    let hit = LayerPanel::from_editor(state).hit_test(rect, point)?;
    Some(apply_layer_panel_click(state, hit, 0, false))
}

#[test]
fn header_rows_toggle_their_section_and_plus_still_adds_a_page() {
    let state = paged_state(3);
    let panel = LayerPanel::from_editor(&state);
    let rect = rail(600.0);
    let r = panel.regions(rect);
    let pages_header = Point2D::new(60.0, r.pages_header_y + HEADER / 2.0);
    let layers_header = Point2D::new(60.0, r.layers_header_y + HEADER / 2.0);
    assert_eq!(
        panel.hit_test(rect, pages_header),
        Some(LayerPanelHit::ToggleSection(LayerPanelSection::Pages))
    );
    assert_eq!(
        panel.hit_test(rect, layers_header),
        Some(LayerPanelHit::ToggleSection(LayerPanelSection::Layers))
    );
    let plus = Point2D::new(rect.size.x - 18.0, r.pages_header_y + HEADER / 2.0);
    assert_eq!(panel.hit_test(rect, plus), Some(LayerPanelHit::AddPage));
}

#[test]
fn toggling_a_section_is_view_state_outside_history() {
    let mut state = paged_state(3);
    let rect = rail(600.0);
    let header = Point2D::new(60.0, 8.0 + HEADER / 2.0);
    assert_eq!(
        click(&mut state, rect, header),
        Some(LayerPanelClick::Dirty)
    );
    assert!(state.editor_ui.layer_sections_collapsed.pages);
    assert!(!state.history.can_undo());
    // Folding survives a document replacement — it is a chrome preference.
    state.editor_ui.clear_document_derived();
    assert!(state.editor_ui.layer_sections_collapsed.pages);
    assert_eq!(
        click(&mut state, rect, header),
        Some(LayerPanelClick::Dirty)
    );
    assert!(!state.editor_ui.layer_sections_collapsed.pages);
}

#[test]
fn folded_pages_lift_layers_under_its_header() {
    let mut state = paged_state(3);
    state.editor_ui.layer_sections_collapsed.pages = true;
    let panel = LayerPanel::from_editor(&state);
    let rect = rail(600.0);
    let r = panel.regions(rect);
    assert_eq!(r.pages_view_h, 0.0);
    assert_eq!(r.layers_header_y, 8.0 + HEADER + GAP);
    // Where page 0 used to sit is now the Layers header…
    let old_page_row = Point2D::new(60.0, 8.0 + HEADER + PAGE_ROW / 2.0);
    assert_eq!(
        panel.hit_test(rect, old_page_row),
        Some(LayerPanelHit::ToggleSection(LayerPanelSection::Layers))
    );
    // …and the first layer row follows it directly.
    let first_layer = Point2D::new(60.0, r.layers_rows_top + LAYER_ROW / 2.0);
    assert_eq!(
        panel.hit_test(rect, first_layer),
        Some(LayerPanelHit::Layer(panel.items[0].node_id.clone()))
    );
}

#[test]
fn folded_layers_give_the_page_list_the_rest_of_the_rail() {
    let mut state = paged_state(12);
    let rect = rail(600.0);
    let expanded = LayerPanel::from_editor(&state).regions(rect);
    assert_eq!(expanded.pages_view_h, 6.0 * PAGE_ROW, "6-row cap when open");

    state.editor_ui.layer_sections_collapsed.layers = true;
    let panel = LayerPanel::from_editor(&state);
    let r = panel.regions(rect);
    assert_eq!(
        r.pages_view_h,
        12.0 * PAGE_ROW,
        "all 12 pages fit in 600 px"
    );
    assert_eq!(r.layers_view_h, 0.0);
    assert_eq!(r.layers_header_y, r.pages_rows_top + r.pages_view_h + GAP);
    let last_page = Point2D::new(60.0, r.pages_rows_top + 11.5 * PAGE_ROW);
    assert_eq!(
        panel.hit_test(rect, last_page),
        Some(LayerPanelHit::Page(11))
    );
    // No layer row is reachable below the folded Layers header.
    let below = Point2D::new(60.0, r.layers_rows_top + LAYER_ROW / 2.0);
    assert_eq!(panel.hit_test(rect, below), None);

    // A short rail clamps the unfolded page list to the room left.
    let short = rail(300.0);
    let r = panel.regions(short);
    assert_eq!(r.pages_view_h, 300.0 - (8.0 + HEADER + GAP + HEADER + 8.0));
    assert!(r.pages.max_offset > 0.0, "the rest scrolls");
}

#[test]
fn folded_pages_header_shows_only_its_label() {
    let mut state = paged_state(3);
    let _ = state.set_active_page(2);
    state.editor_ui.layer_sections_collapsed.pages = true;
    let panel = LayerPanel::from_editor(&state);
    let rect = rail(600.0);
    let mut backend = CaptureBackend::default();
    panel.paint(
        &mut PaintCx {
            backend: &mut backend,
        },
        rect,
    );
    let header: Vec<_> = backend
        .texts
        .iter()
        .filter(|(_, p)| p.y < 8.0 + HEADER)
        .map(|(t, _)| t.as_str())
        .collect();
    assert_eq!(header, vec![panel.pages_label]);
    // Folded: no page row labels are painted at all.
    assert!(!backend.texts.iter().any(|(t, _)| t.starts_with("Page ")));
}

#[test]
fn wheel_over_a_folded_section_leaves_its_hidden_rows_alone() {
    let mut state = paged_state(12);
    let rect = rail(600.0);
    let header = Point2D::new(60.0, 8.0 + HEADER / 2.0);

    // Sanity: the open page list (12 rows, 6 visible) scrolls.
    let panel = LayerPanel::from_editor(&state);
    assert_eq!(
        scroll_layer_panel(&mut state, &panel, rect, header, 0.0, -40.0),
        Some(true)
    );
    let open_offset = state.editor_ui.layer_pages_scroll.offset;
    assert!(open_offset > 0.0);

    state.editor_ui.layer_sections_collapsed.pages = true;
    let panel = LayerPanel::from_editor(&state);
    assert_eq!(panel.regions(rect).pages.max_offset, 0.0);
    let _ = scroll_layer_panel(&mut state, &panel, rect, header, 0.0, -40.0);
    assert!(state.editor_ui.layer_pages_scroll.offset <= open_offset);

    // Folded Layers: the empty rail below its header scrolls nothing.
    state.editor_ui.layer_sections_collapsed = Default::default();
    state.editor_ui.layer_sections_collapsed.layers = true;
    let panel = LayerPanel::from_editor(&state);
    let r = panel.regions(rect);
    assert_eq!(r.layers.max_offset, 0.0);
    let below = Point2D::new(60.0, r.layers_rows_top + 40.0);
    let _ = scroll_layer_panel(&mut state, &panel, rect, below, 0.0, -40.0);
    assert_eq!(state.editor_ui.layer_layers_scroll.offset, 0.0);
}

#[test]
fn selection_reveal_waits_until_layers_unfold() {
    let src = r#"{"version":"1.0.0","children":[
        {"type":"frame","id":"f","name":"Frame","width":100,"height":100,"children":[
            {"type":"rectangle","id":"r","name":"Inner","width":10,"height":10}
        ]}
    ]}"#;
    let doc = jian_ops_schema::load_str(src)
        .expect("reveal fixture parses")
        .value;
    let mut state = EditorState::from_document(doc);
    let rect = rail(600.0);
    let frame = NodeId::new("f".to_string());
    let inner = NodeId::new("r".to_string());
    state.toggle_node_collapsed(&frame);
    state.set_single_selection(inner.clone());
    state.editor_ui.layer_sections_collapsed.layers = true;

    // Folded: no tree walk, no ancestor expansion, anchor stays pending
    // so the host retries once the section opens.
    let panel = LayerPanel::from_editor(&state);
    assert!(!reveal_layer_panel_selection(&mut state, &panel, rect));
    assert!(state.editor_ui.collapsed_layers.contains(&frame));
    assert_eq!(state.editor_ui.last_revealed_layer_anchor, None);

    state.editor_ui.layer_sections_collapsed.layers = false;
    let panel = LayerPanel::from_editor(&state);
    let _ = reveal_layer_panel_selection(&mut state, &panel, rect);
    assert!(!state.editor_ui.collapsed_layers.contains(&frame));
    assert_eq!(state.editor_ui.last_revealed_layer_anchor, Some(inner));
}
