//! Up / Down stepping through the slides tab: the slide click that hands
//! the list the arrows, which slide a press lands on, the clamped ends,
//! and the row scrolled into view.

use super::*;
use op_editor_core::scene_template_catalog::TemplateScene;

const PANEL: Rect = Rect {
    origin: Point2D { x: 0.0, y: 48.0 },
    size: Point2D { x: 240.0, y: 700.0 },
};

/// `count` 16:9 boards side by side, with the slides tab showing and a
/// slide already clicked (the list owns the arrows).
fn deck(count: usize) -> EditorState {
    let boards = (0..count)
        .map(|i| {
            format!(
                r#"{{"type":"frame","id":"slide-{i}","name":"Slide {i}","x":{},"y":0,"width":1920,"height":1080}}"#,
                i * 2100
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(r#"{{"version":"1.0.0","children":[{boards}]}}"#);
    let document = jian_ops_schema::load_str(&source)
        .expect("fixture parses")
        .value;
    let mut state = EditorState::from_document(document);
    state.editor_ui.scenario = Some(TemplateScene::Slides);
    state.editor_ui.slides_panel.tab = LeftPanelTab::Slides;
    state.editor_ui.slides_panel.owns_arrow_keys = true;
    state
}

/// The list as laid out now. An empty scene: every board falls back to
/// 16:9, which is all the row geometry needs.
fn laid_out(state: &EditorState) -> SlidesPanelLayout {
    let scene = LayoutScene {
        pages: Vec::new(),
        active_page_index: 0,
    };
    layout(state, &slides(state), &scene, PANEL).expect("the rail has room")
}

/// Whether row `index`, with the gap on either side of its card, sits
/// wholly inside the list band.
fn row_in_view(layout: &SlidesPanelLayout, index: usize) -> bool {
    let row = layout.row_rect(index);
    let gap = layout.row_rect(1).origin.y - (layout.row_rect(0).origin.y + layout.row_height());
    row.origin.y - gap >= layout.list.origin.y - 0.01
        && row.origin.y + row.size.y + gap <= layout.list.origin.y + layout.list.size.y + 0.01
}

#[test]
fn only_a_slide_click_hands_the_list_the_arrows() {
    let mut state = deck(3);
    state.editor_ui.slides_panel.owns_arrow_keys = false;
    let layout = laid_out(&state);
    assert_eq!(
        step(&mut state, &layout, Some(0), true),
        None,
        "before a slide click the arrows stay with the selection"
    );

    // A click on the action bar does not take them…
    let present = layout.actions.present;
    let point = Point2D::new(
        present.origin.x + present.size.x / 2.0,
        present.origin.y + present.size.y / 2.0,
    );
    let _ = press(&mut state, &layout, point);
    assert_eq!(release(&mut state, &layout), SlidesRelease::Present);
    assert!(!state.editor_ui.slides_panel.owns_arrow_keys);

    // …a click on a slide row does.
    let row = layout.row_rect(1);
    let point = Point2D::new(
        row.origin.x + row.size.x / 2.0,
        row.origin.y + row.size.y / 2.0,
    );
    let _ = press(&mut state, &layout, point);
    assert_eq!(release(&mut state, &layout), SlidesRelease::Activate(1));
    assert!(state.editor_ui.slides_panel.owns_arrow_keys);
    assert_eq!(step(&mut state, &layout, Some(1), true), Some(2));

    // Leaving the panel's reach hands them back.
    state.editor_ui.slides_panel.clear_pointer();
    assert_eq!(step(&mut state, &layout, Some(1), true), None);
}

#[test]
fn down_and_up_move_one_slide_and_stop_at_the_ends() {
    let mut state = deck(3);
    let layout = laid_out(&state);
    assert_eq!(step(&mut state, &layout, Some(0), true), Some(1));
    assert_eq!(step(&mut state, &layout, Some(1), false), Some(0));
    assert_eq!(step(&mut state, &layout, Some(2), true), Some(2), "last");
    assert_eq!(step(&mut state, &layout, Some(0), false), Some(0), "first");
}

#[test]
fn with_no_active_slide_either_arrow_starts_at_the_first() {
    let mut state = deck(3);
    let layout = laid_out(&state);
    assert_eq!(step(&mut state, &layout, None, true), Some(0));
    assert_eq!(step(&mut state, &layout, None, false), Some(0));
}

#[test]
fn an_empty_list_has_nothing_to_step_to() {
    let mut state = deck(3);
    let mut layout = laid_out(&state);
    layout.count = 0;
    assert_eq!(step(&mut state, &layout, None, true), None);
}

#[test]
fn stepping_scrolls_the_new_row_into_view_both_ways() {
    let mut state = deck(12);
    let layout = laid_out(&state);
    assert!(layout.max_scroll() > 0.0, "twelve slides overflow the rail");
    let last = layout.count - 1;
    assert!(!row_in_view(&layout, last));

    // Down onto the last row: scrolled just far enough to show it whole.
    assert_eq!(step(&mut state, &layout, Some(last - 1), true), Some(last));
    let layout = laid_out(&state);
    assert!(row_in_view(&layout, last));
    assert_eq!(layout.offset, layout.max_scroll(), "the list's very bottom");

    // Up onto the first row from the bottom: back to the top.
    assert_eq!(step(&mut state, &layout, Some(1), false), Some(0));
    let layout = laid_out(&state);
    assert!(row_in_view(&layout, 0));
    assert_eq!(layout.offset, 0.0, "the list's very top");
}

#[test]
fn a_row_already_in_view_leaves_the_scroll_alone() {
    let mut state = deck(12);
    state.editor_ui.slides_panel.scroll.offset = 40.0;
    let layout = laid_out(&state);
    let visible = (0..layout.count)
        .find(|&index| index > 0 && row_in_view(&layout, index + 1))
        .expect("some row has a visible neighbour below it");
    assert_eq!(
        step(&mut state, &layout, Some(visible), true),
        Some(visible + 1)
    );
    assert_eq!(state.editor_ui.slides_panel.scroll.offset, 40.0);
}
