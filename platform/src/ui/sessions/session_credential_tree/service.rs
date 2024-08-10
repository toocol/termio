use crate::ui::{
    ctx_menu::CtxMenu,
    edit_window::EditWindow,
    sessions::{PROP_TREE_NODE_ID, PROP_TREE_VIEW_ID},
};
use cli::session::session_grp::SessionGroup;
use log::{debug, warn};
use tmui::{
    input::{dialog::InputDialog, text::Text, Input},
    prelude::{ApplicationWindow, Coordinate, ObjectId, ObjectOperation},
    tlib::{
        events::MouseEvent,
        figure::{point, Point},
        namespace::KeyCode,
    },
    views::{
        list_view::list_node::ListNode,
        tree_view::{self, TreeView},
    },
    widget::{
        callbacks::CallbacksRegister, widget_ext::WidgetExt, ChildOp, WidgetFinder, WidgetImpl,
        WindowAcquire,
    },
    window::{win_builder::WindowBuilder, win_config::WindowConfig},
};

pub struct SessionCredentialService;

impl SessionCredentialService {
    pub fn new_session_pressed(ctx_menu: &mut CtxMenu, node: &mut ListNode) {
        ctx_menu.hide();

        const EDIT_WIN_WIDTH: u32 = 600;
        const EDIT_WIN_HEIGHT: u32 = 400;

        let win = node.get_view().window();
        let win_size = win.size();
        let win_pos = win.outer_position();
        let pos = Point::new(
            win_pos.x() + (win_size.width() - EDIT_WIN_WIDTH as i32) / 2,
            win_pos.y() + (win_size.height() - EDIT_WIN_HEIGHT as i32) / 2,
        );

        win.create_window(
            WindowBuilder::default()
                .config(
                    WindowConfig::builder()
                        .width(EDIT_WIN_WIDTH)
                        .height(EDIT_WIN_HEIGHT)
                        .title("New Session".to_string())
                        .position(pos)
                        .build(),
                )
                .modal(true)
                .on_activate(|win| win.child(EditWindow::new())),
        );
    }

    pub fn new_group_pressed(ctx_menu: &mut CtxMenu, node: &mut ListNode) {
        println!("new group pressed.");
        ctx_menu.hide();

        let tree_node_id = ctx_menu
            .get_property(PROP_TREE_NODE_ID)
            .unwrap()
            .get::<ObjectId>();
        let tree_view_id = ctx_menu
            .get_property(PROP_TREE_VIEW_ID)
            .unwrap()
            .get::<ObjectId>();

        let tree_view = ctx_menu.find_id_mut::<TreeView>(tree_view_id).unwrap();
        let line_spacing = tree_view.line_spacing();
        let tree_node = tree_view
            .get_store_mut()
            .get_node_mut(tree_node_id)
            .unwrap();

        if let Some(new_group) = tree_node.add_node(&SessionGroup::new("Test")) {
            let mut rect = new_group.rect(Coordinate::World).unwrap();
            rect.set_height(rect.height() - line_spacing);
            let new_group_id = new_group.id();

            let input_dialog = InputDialog::text(rect, None);
            input_dialog
                .input_mut::<Text>()
                .unwrap()
                .register_key_released(|w, evt| {
                    if evt.key_code() == KeyCode::KeyEnter {
                        w.hide();
                        return;
                    }
                });
            input_dialog.register_visibility_changed(move |w, visible| {
                println!("{} visible has changed {}", w.name(), visible);
                if !visible {
                    let text = w
                        .downcast_mut::<InputDialog>()
                        .unwrap()
                        .input_mut::<Text>()
                        .unwrap();
                    let group_name = text.value();

                    let tree_view = ApplicationWindow::window()
                        .find_id_mut(tree_view_id)
                        .unwrap()
                        .downcast_mut::<TreeView>()
                        .unwrap();
                    let new_group = tree_view
                        .get_store_mut()
                        .get_node_mut(new_group_id)
                        .unwrap();

                    if group_name.is_empty() {
                        new_group.remove();
                    } else {
                        new_group.set_value(0, group_name)
                    }
                }
            });
        } else {
            warn!(
                "{}({}:{}) Add new session group failed.",
                file!(),
                line!(),
                column!()
            )
        }
    }
}