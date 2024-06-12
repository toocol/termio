pub mod menu_selection;
pub mod selection_bld;

use tmui::{
    prelude::*,
    scroll_area::LayoutMode,
    tlib::{
        global_watch,
        object::{ObjectImpl, ObjectSubclass},
    },
    tree_view::TreeView,
    widget::WidgetImpl,
};
use self::selection_bld::CtxMenuLoc;

#[extends(Popup)]
#[derive(Childable)]
#[global_watch(MouseReleased)]
pub struct CtxMenu {
    loc: CtxMenuLoc,

    #[child]
    selection_list: Box<TreeView>,
}

impl ObjectSubclass for CtxMenu {
    const NAME: &'static str = "CtxMenu";
}

impl ObjectImpl for CtxMenu {
    fn initialize(&mut self) {
        self.selection_list.set_layout_mode(LayoutMode::Overlay);

        self.loc
            .bld_selections(self.selection_list.get_store_mut().root_mut())
    }
}

impl WidgetImpl for CtxMenu {}

impl GlobalWatchImpl for CtxMenu {
    fn on_global_mouse_released(&mut self, evt: &tlib::events::MouseEvent) -> bool {
        if !self.visible() {
            return false;
        }
        let pos: Point = evt.position().into();
        if !self.rect().contains(&pos) {
            self.hide();
        }

        true
    }
}

impl PopupImpl for CtxMenu {
    #[inline]
    fn calculate_position(&self, _: Rect, point: Point) -> Point {
        point
    }

    #[inline]
    fn is_modal(&self) -> bool {
        true
    }
}

impl CtxMenu {
    #[inline]
    pub fn new(loc: CtxMenuLoc) -> Box<Self> {
        let mut menu: Box<Self> = Object::new(&[]);
        menu.loc = loc;
        menu
    }
}
