use crate::ui::sessions::SessionCredentialTree;
use tmui::views::list_view::ListView;

use super::menu_selection::CtxMenuSelectionCreator;

#[derive(Default)]
pub enum CtxMenuLoc {
    #[default]
    Unknown,
    SessionCredentialTree,
}

impl CtxMenuLoc {
    pub(super) fn bld_selections(&self, view: &mut ListView) {
        let selections = match self {
            Self::SessionCredentialTree => SessionCredentialTree::create_selections(),
            _ => vec![],
        };

        for obj in selections.iter() {
            view.add_node(obj);
        }
    }
}
