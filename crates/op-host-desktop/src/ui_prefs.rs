//! Cross-launch UI preferences (`~/.openpencil/ui.json`) — small,
//! non-secret editor chrome choices that should survive a restart.

use op_editor_core::PencilCursorStyle;

const FILE: &str = "ui.json";

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct UiPrefs {
    #[serde(default)]
    pencil_cursor: Option<String>,
    #[serde(default)]
    agent_swarm: Option<bool>,
    /// LayerPanel Pages height chosen with the divider (logical px).
    #[serde(default)]
    layer_pages_height: Option<f32>,
}

/// Every persisted choice. `ui.json` is rewritten whole on each save, so a
/// save must carry all of them or it silently resets the others.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct UiPrefsSnapshot {
    pub(crate) pencil_cursor: PencilCursorStyle,
    pub(crate) agent_swarm: bool,
    pub(crate) layer_pages_height: Option<f32>,
}

pub(crate) fn save(snapshot: UiPrefsSnapshot) {
    crate::test_config_root::guard_user_config();
    let value = UiPrefs {
        pencil_cursor: Some(snapshot.pencil_cursor.id().to_string()),
        agent_swarm: Some(snapshot.agent_swarm),
        layer_pages_height: snapshot.layer_pages_height,
    };
    if let Err(err) = op_config_store::write_json(FILE, &value) {
        eprintln!("[ui-prefs] write failed: {err}");
    }
}

/// Stored choices; a missing or unknown entry is `None` (keep the default).
pub(crate) fn load() -> (Option<PencilCursorStyle>, Option<bool>, Option<f32>) {
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
        value
            .layer_pages_height
            .filter(|height| height.is_finite() && *height >= 0.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_json_without_a_pages_height_keeps_the_default_split() {
        let older: UiPrefs =
            serde_json::from_str(r#"{"pencil_cursor":"default","agent_swarm":true}"#)
                .expect("pre-split ui.json parses");
        assert_eq!(older.layer_pages_height, None);

        let chosen = UiPrefs {
            layer_pages_height: Some(160.0),
            ..UiPrefs::default()
        };
        let back: UiPrefs =
            serde_json::from_str(&serde_json::to_string(&chosen).expect("serializes"))
                .expect("round-trips");
        assert_eq!(back.layer_pages_height, Some(160.0));
    }

    #[test]
    fn style_ids_round_trip() {
        for style in PencilCursorStyle::ALL {
            assert_eq!(PencilCursorStyle::from_id(style.id()), Some(style));
        }
    }
}
