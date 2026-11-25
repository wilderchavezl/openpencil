//! Collapse state of the LayerPanel's Pages / Layers sections.
//!
//! View-only chrome preference: never part of the document, the undo
//! history or `clear_document_derived` — a collapsed section stays
//! collapsed across document switches (desktop also persists it).

/// One of the two collapsible LayerPanel sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerPanelSection {
    Pages,
    Layers,
}

/// Which LayerPanel sections are collapsed to their header row.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LayerSectionsCollapsed {
    pub pages: bool,
    pub layers: bool,
}

impl LayerSectionsCollapsed {
    /// Whether `section` is collapsed.
    pub fn is_collapsed(self, section: LayerPanelSection) -> bool {
        match section {
            LayerPanelSection::Pages => self.pages,
            LayerPanelSection::Layers => self.layers,
        }
    }

    /// Flip `section` between collapsed and expanded.
    pub fn toggle(&mut self, section: LayerPanelSection) {
        match section {
            LayerPanelSection::Pages => self.pages = !self.pages,
            LayerPanelSection::Layers => self.layers = !self.layers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_toggle_independently() {
        let mut s = LayerSectionsCollapsed::default();
        assert!(!s.is_collapsed(LayerPanelSection::Pages));
        s.toggle(LayerPanelSection::Pages);
        assert!(s.is_collapsed(LayerPanelSection::Pages));
        assert!(!s.is_collapsed(LayerPanelSection::Layers));
        s.toggle(LayerPanelSection::Layers);
        s.toggle(LayerPanelSection::Pages);
        assert_eq!(
            s,
            LayerSectionsCollapsed {
                pages: false,
                layers: true
            }
        );
    }
}
