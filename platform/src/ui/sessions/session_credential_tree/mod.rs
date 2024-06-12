pub mod session_credential;
pub mod session_group;

use self::session_group::SessionGroup;
use crate::ui::ctx_menu::{selection_bld::CtxMenuLoc, CtxMenu};
use tmui::{
    popup::Popupable,
    tlib::{events::MouseEvent, figure::Color, namespace::MouseButton, Object},
    tree_view::{tree_node::TreeNode, TreeView},
    widget::widget_ext::WidgetExt,
};

pub struct SessionCredentialTree;

impl SessionCredentialTree {
    #[inline]
    pub fn view() -> Box<TreeView> {
        let mut view: Box<TreeView> = Object::new(&[]);
        view.get_store_mut()
            .root_mut()
            .add_node(&SessionGroup::new(" Sessions"));
        view.set_background(Color::WHITE);
        view.add_popup(CtxMenu::new(CtxMenuLoc::SessionCredentialTree));
        view.register_node_pressed(node_pressed);
        view.register_node_released(node_released);
        view.register_free_area_released(node_released);
        view
    }
}

fn node_pressed(node: &mut TreeNode, evt: &MouseEvent) {
    match evt.mouse_button() {
        MouseButton::LeftButton => {}
        MouseButton::RightButton => {}
        _ => {}
    }
}

fn node_released(node: &mut TreeNode, evt: &MouseEvent) {
    match evt.mouse_button() {
        MouseButton::LeftButton => {}
        MouseButton::RightButton => {
            let node = if node.is_root() {
                // Get the `Sessions` node.
                node.children_mut().first_mut().unwrap().as_mut()
            } else {
                node
            };

            if node.is_extensible() {
                let view = node.get_view();
                // TODO: endless loop happens here:
                view.show_popup(view.map_to_global(&evt.position().into()));
            }
        }
        _ => {}
    }
}
