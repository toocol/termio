use tmui::views::list_view::ListView;
use super::menu_selection::MenuSelection;

#[derive(Default)]
pub enum CtxMenuLoc {
    #[default]
    Unknown,
    SessionCredentialTree,
}

impl CtxMenuLoc {
    pub(super) fn bld_selections(&self, view: &mut ListView) {
        match self {
            Self::SessionCredentialTree => {
                view.add_node(&MenuSelection::new("New Session"));
            },
            _ => {},
        }
    }
}
