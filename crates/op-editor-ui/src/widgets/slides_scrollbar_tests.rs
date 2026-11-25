//! The slides tab's scrollbar through the shared flow: a press in the
//! gutter of an overflowing list beats the row under it and drags the
//! list, a press there with slides that fit stays a row press, and touch
//! chrome has no gutter.

use super::*;
use crate::widgets::layer_scrollbar_flow;
use op_editor_core::scene_template_catalog::TemplateScene;

const PANEL: Rect = Rect {
    origin: Point2D { x: 0.0, y: 48.0 },
    size: Point2D { x: 240.0, y: 700.0 },
};

/// A deck of `count` 16:9 boards, the slides tab showing.
fn deck(count: usize) -> EditorState {
    let boards = (0..count)
        .map(|i| {
            format!(
                r#"{{"type":"frame","id":"slide-{i}","x":{},"y":0,"width":1920,"height":1080}}"#,
                i * 2100
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let src = format!(r#"{{"version":"1.0.0","children":[{boards}]}}"#);
    let doc = jian_ops_schema::load_str(&src)
        .expect("fixture parses")
        .value;
    let mut state = EditorState::from_document(doc);
    state.editor_ui.scenario = Some(TemplateScene::Slides);
    state.editor_ui.slides_panel.tab = LeftPanelTab::Slides;
    state
}

fn laid_out(state: &EditorState) -> SlidesPanelLayout {
    let scene = LayoutScene {
        pages: Vec::new(),
        active_page_index: 0,
    };
    layout(state, &slides(state), &scene, PANEL).expect("the rail has room")
}

fn gutter_point(layout: &SlidesPanelLayout, y: f32) -> Point2D {
    let gutter = layout.scrollbar().gutter;
    Point2D::new(gutter.origin.x + gutter.size.x / 2.0, y)
}

#[test]
fn dragging_the_thumb_scrolls_the_slide_list() {
    let mut state = deck(12);
    let l = laid_out(&state);
    let thumb = l.scrollbar().thumb.expect("twelve slides overflow");
    let grab = gutter_point(&l, thumb.origin.y + 4.0);

    assert_eq!(press(&mut state, &l, grab), SlidesPress::Claimed(None));
    let ui = &state.editor_ui;
    assert_eq!(ui.slides_panel.pressed, None, "not a slide press");
    assert_eq!(ui.slides_panel.drag, None, "not a reorder");
    assert_eq!(ui.slides_panel.scroll.offset, 0.0, "grabbing does not jump");

    // Far below the rail: the list pins to its end.
    assert!(layer_scrollbar_flow::drag(
        &mut state,
        Point2D::new(500.0, 10_000.0)
    ));
    assert_eq!(state.editor_ui.slides_panel.scroll.offset, l.max_scroll());
    assert!(layer_scrollbar_flow::release(&mut state));
    assert_eq!(laid_out(&state).offset, l.max_scroll());
}

#[test]
fn a_track_press_pages_toward_the_pointer() {
    let mut state = deck(12);
    let l = laid_out(&state);
    let track = l.scrollbar().track;
    let end = gutter_point(&l, track.origin.y + track.size.y - 1.0);
    assert_eq!(press(&mut state, &l, end), SlidesPress::Claimed(None));
    assert_eq!(state.editor_ui.slides_panel.scroll.offset, l.max_scroll());
}

#[test]
fn the_gutter_stays_a_row_when_the_slides_fit() {
    let mut state = deck(2);
    let l = laid_out(&state);
    assert!(l.scrollbar().thumb.is_none());
    let row = l.row_rect(0);
    let point = gutter_point(&l, row.origin.y + row.size.y / 2.0);
    assert_eq!(
        press(&mut state, &l, point),
        SlidesPress::Claimed(Some(SlidesPanelTarget::Slide(0)))
    );
    assert!(state.editor_ui.layer_scrollbar_drag.is_none());
}

#[test]
fn touch_chrome_has_no_gutter() {
    let mut state = deck(12);
    state.editor_ui.touch = true;
    let l = laid_out(&state);
    let thumb = l.scrollbar().thumb.expect("the thumb still paints");
    let point = gutter_point(&l, thumb.origin.y + 4.0);
    press(&mut state, &l, point);
    assert!(state.editor_ui.layer_scrollbar_drag.is_none());
}
