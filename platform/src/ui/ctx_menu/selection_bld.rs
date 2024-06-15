use tmui::tree_view::tree_node::TreeNode;
use super::menu_selection::MenuSelection;

#[derive(Default)]
pub enum CtxMenuLoc {
    #[default]
    Unknown,
    SessionCredentialTree,
}

impl CtxMenuLoc {
    pub(super) fn bld_selections(&self, root: &mut TreeNode) {
        match self {
            Self::SessionCredentialTree => {
                root.add_node(&MenuSelection::new("New Session"));
            },
            _ => {},
        }
    }
}
