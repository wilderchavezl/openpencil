//! Cross-launch UI preferences (`~/.openpencil/ui.json`) — small,
//! non-secret editor chrome choices that should survive a restart.

use op_editor_core::editor_ui_state::LayerSectionsCollapsed;
use op_editor_core::PencilCursorStyle;

const FILE: &str = "ui.json";

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct UiPrefs {
    #[serde(default)]
    pencil_cursor: Option<String>,
    #[serde(default)]
    agent_swarm: Option<bool>,
    #[serde(default)]
    layer_pages_collapsed: Option<bool>,
    #[serde(default)]
    layer_layers_collapsed: Option<bool>,
}

/// Every persisted choice. `ui.json` is rewritten whole on each save, so a
/// save must carry all of them or it silently resets the others.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UiPrefsSnapshot {
    pub(crate) pencil_cursor: PencilCursorStyle,
    pub(crate) agent_swarm: bool,
    pub(crate) layer_sections: LayerSectionsCollapsed,
}

pub(crate) fn save(snapshot: UiPrefsSnapshot) {
    crate::test_config_root::guard_user_config();
    let value = UiPrefs {
        pencil_cursor: Some(snapshot.pencil_cursor.id().to_string()),
        agent_swarm: Some(snapshot.agent_swarm),
        layer_pages_collapsed: Some(snapshot.layer_sections.pages),
        layer_layers_collapsed: Some(snapshot.layer_sections.layers),
    };
    if let Err(err) = op_config_store::write_json(FILE, &value) {
        eprintln!("[ui-prefs] write failed: {err}");
    }
}

/// Stored choices; a missing or unknown entry is `None` (keep the default).
pub(crate) fn load() -> (
    Option<PencilCursorStyle>,
    Option<bool>,
    LayerSectionsCollapsed,
) {
    crate::test_config_root::guard_user_config();
    let value: UiPrefs = op_config_store::read_json(FILE)
        .ok()
        .flatten()
        .unwrap_or_default();
    (
        value
            .pencil_cursor
            .as_deref()
            .and_then(PencilCursorStyle::from_id),
        value.agent_swarm,
        LayerSectionsCollapsed {
            pages: value.layer_pages_collapsed.unwrap_or(false),
            layers: value.layer_layers_collapsed.unwrap_or(false),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_json_without_layer_sections_keeps_them_open() {
        let older: UiPrefs =
            serde_json::from_str(r#"{"pencil_cursor":"default","agent_swarm":true}"#)
                .expect("pre-sections ui.json parses");
        assert_eq!(older.layer_pages_collapsed, None);
        assert_eq!(older.layer_layers_collapsed, None);

        let folded = UiPrefs {
            layer_pages_collapsed: Some(true),
            layer_layers_collapsed: Some(false),
            ..UiPrefs::default()
        };
        let back: UiPrefs =
            serde_json::from_str(&serde_json::to_string(&folded).expect("serializes"))
                .expect("round-trips");
        assert_eq!(back.layer_pages_collapsed, Some(true));
        assert_eq!(back.layer_layers_collapsed, Some(false));
    }

    #[test]
    fn style_ids_round_trip() {
        for style in PencilCursorStyle::ALL {
            assert_eq!(PencilCursorStyle::from_id(style.id()), Some(style));
        }
    }
}
