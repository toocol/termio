use crate::ui::{ctx_menu::CtxMenu, edit_window::EditWindow};
use log::debug;
use tmui::{
    tlib::{
        events::MouseEvent,
        figure::{point, Point},
    }, views::{list_view::list_node::ListNode, tree_view::TreeView}, widget::{widget_ext::WidgetExt, ChildOp, WidgetImpl, WindowAcquire}, window::{win_builder::WindowBuilder, win_config::WindowConfig}
};

pub struct SessionCredentialService;

impl SessionCredentialService {
    pub fn new_session_pressed(
        ctx_menu: &mut CtxMenu,
        node: &mut ListNode,
    ) {
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

    pub fn add_new_session_credential(tree: &mut TreeView) {

    }
}
