use tmui::{tree_view::TreeView, tlib::Object};

pub struct SessionCredentialTree;

impl SessionCredentialTree {
    #[inline]
    pub fn view() -> Box<TreeView> {
        Object::new(&[])
    }
}

pub struct SessionGroup {
}

pub struct SessionCredential {
}