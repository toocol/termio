pub mod menu_selection;
pub mod selection_bld;

use self::selection_bld::CtxMenuLoc;
use tlib::connect;
use tmui::{
    graphics::box_shadow::{BoxShadow, ShadowSide},
    prelude::*,
    scroll_area::LayoutMode,
    tlib::{
        global_watch,
        object::{ObjectImpl, ObjectSubclass},
    },
    views::list_view::ListView,
    widget::WidgetImpl,
};

#[extends(Popup)]
#[derive(Childable)]
#[global_watch(MouseReleased)]
pub struct CtxMenu {
    loc: CtxMenuLoc,

    #[child]
    selection_list: Box<ListView>,
}

impl ObjectSubclass for CtxMenu {
    const NAME: &'static str = "CtxMenu";
}

impl ObjectImpl for CtxMenu {
    fn initialize(&mut self) {
        self.width_request(200);
        self.height_request(400);

        self.set_borders(1., 1., 1., 1.);
        self.set_border_color(Color::GREY_LIGHT);
        self.set_box_shadow(BoxShadow::new(
            8.,
            Color::BLACK,
            None,
            Some(ShadowSide::new(&[ShadowSide::RIGHT, ShadowSide::BOTTOM])),
            None,
        ));

        self.selection_list.set_vexpand(true);
        self.selection_list.set_hexpand(true);
        self.selection_list.set_layout_mode(LayoutMode::Overlay);
        self.selection_list.set_mouse_tracking(true);

        let scroll_bar = self.selection_list.scroll_bar_mut();
        scroll_bar.set_slider_radius(5.);
        scroll_bar.set_background(Color::TRANSPARENT);
        scroll_bar.set_color(Color::GREY_LIGHT.with_a(155));
        scroll_bar.set_active_color(Some(Color::GREY_MEDIUM.with_a(155)));
        scroll_bar.set_visible_in_valid(true);

        self.loc.bld_selections(&mut self.selection_list);
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
