pub mod load;
pub mod service;

use crate::ui::ctx_menu::{menu_selection::{CtxMenuSelectionCreator, MenuSelection}, selection_bld::CtxMenuLoc, selection_enum::SelectionEnum, CtxMenu};
use tmui::{
    popup::Popupable,
    tlib::{events::MouseEvent, figure::Color, namespace::MouseButton, Object, prelude::*},
    views::tree_view::{tree_node::TreeNode, TreeView},
    widget::widget_ext::WidgetExt,
};

pub const SESSION_CREDENTIAL_TREE: &'static str = "SessionCredentialTree";

pub struct SessionCredentialTree;

impl SessionCredentialTree {
    #[inline]
    pub fn view() -> Box<TreeView> {
        let mut view: Box<TreeView> = Object::new(&[]);
        view.set_name(SESSION_CREDENTIAL_TREE);
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
                let node_id = node.id();
                let view = node.get_view();
                let view_id = view.id();
                view.show_popup(view.map_to_global(&evt.position().into()));
                view.get_popup_mut().unwrap().set_property(PROP_TREE_NODE_ID, node_id.to_value());
                view.get_popup_mut().unwrap().set_property(&PROP_TREE_VIEW_ID, view_id.to_value());
            }
        }
        _ => {}
    }
}

impl CtxMenuSelectionCreator for SessionCredentialTree {
    #[inline]
    fn create_selections() -> Vec<MenuSelection> {
        vec![
            MenuSelection::new(SelectionEnum::NewSession),
            MenuSelection::new(SelectionEnum::NewGroup),
        ]
    }
}

// Constants: 
pub const PROP_TREE_NODE_ID: &'static str = "tree_node_id";
pub const PROP_TREE_VIEW_ID: &'static str = "tree_view_id";