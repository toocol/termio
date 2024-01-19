pub mod session_credential;
pub mod session_group;

use tmui::{tlib::Object, tree_view::TreeView};

pub struct SessionCredentialTree;

impl SessionCredentialTree {
    #[inline]
    pub fn view() -> Box<TreeView> {
        Object::new(&[])
    }
}
