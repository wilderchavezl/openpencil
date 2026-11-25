//! The LayerPanel Pages / Layers split survives whole-state replacement
//! (New / Open / templates): it is the user's chrome preference.

use super::*;

#[test]
fn a_chosen_pages_height_survives_a_document_swap() {
    let mut previous = EditorState::new();
    previous.editor_ui.layer_split.pages_height = Some(180.0);
    previous
        .editor_ui
        .layer_split
        .begin_drag(10.0, 180.0, 32.0, 400.0);
    let mut next = EditorState::starter();

    preserve_app_preferences(&previous, &mut next);

    assert_eq!(next.editor_ui.layer_split.pages_height, Some(180.0));
    assert!(
        next.editor_ui.layer_split.drag.is_none(),
        "a gesture in flight belongs to the replaced document's frame"
    );
}
