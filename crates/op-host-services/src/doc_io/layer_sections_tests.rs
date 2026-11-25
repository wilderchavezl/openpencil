//! LayerPanel section folding survives whole-state replacement (New /
//! Open / templates): it is the user's chrome preference.

use super::*;
use op_editor_core::editor_ui_state::LayerSectionsCollapsed;

#[test]
fn folded_sections_survive_a_document_swap() {
    let mut previous = EditorState::new();
    previous.editor_ui.layer_sections_collapsed = LayerSectionsCollapsed {
        pages: true,
        layers: false,
    };
    let mut next = EditorState::starter();

    preserve_app_preferences(&previous, &mut next);

    assert_eq!(
        next.editor_ui.layer_sections_collapsed,
        LayerSectionsCollapsed {
            pages: true,
            layers: false
        }
    );
}
