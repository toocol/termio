pub mod session_credential;
pub mod session_group;

use tmui::{popup::Popupable, tlib::{events::MouseEvent, namespace::MouseButton, Object}, tree_view::{tree_node::TreeNode, TreeView}};

use self::session_group::SessionGroup;

use super::SessionCtxMenu;

pub struct SessionCredentialTree;

impl SessionCredentialTree {
    #[inline]
    pub fn view() -> Box<TreeView> {
        let mut view: Box<TreeView> = Object::new(&[]);
        view.get_store_mut()
            .root_mut()
            .add_node(&SessionGroup::new(" Sessions"));
        view.add_popup(SessionCtxMenu::new());
        view.register_node_pressed(node_pressed);
        view
    }
}

fn node_pressed(node: &mut TreeNode, evt: &MouseEvent) {
    match evt.mouse_button() {
        MouseButton::LeftButton => {},
        MouseButton::RightButton => {},
        _ => {}
    }
}