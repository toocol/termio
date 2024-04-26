pub mod session_credential;
pub mod session_group;

use tmui::{tlib::Object, tree_view::TreeView};

use self::session_group::SessionGroup;

pub struct SessionCredentialTree;

impl SessionCredentialTree {
    #[inline]
    pub fn view() -> Box<TreeView> {
        let mut view: Box<TreeView> = Object::new(&[]);
        view.get_store_mut()
            .root_mut()
            .add_node(&SessionGroup::new(" Sessions"));
        view
    }
}
