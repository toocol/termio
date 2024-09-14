pub mod menu_selection;
pub mod selection_bld;
pub mod selection_enum;

use self::selection_bld::CtxMenuLoc;
use selection_enum::SelectionEnum;
use tmui::{
    graphics::box_shadow::{BoxShadow, ShadowSide},
    prelude::*,
    scroll_area::LayoutMode,
    tlib::object::{ObjectImpl, ObjectSubclass},
    views::list_view::ListView,
    widget::WidgetImpl,
};

#[extends(Popup)]
#[derive(Childable)]
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

        let ctx_menu_id = self.id();
        self.selection_list.register_node_pressed(move |node, _, evt| {
            let selection_str = node.get_value::<String>(0).unwrap();
            let view = node.get_view();
            let ctx_menu = ApplicationWindow::window_of(view.window_id())
                .find_id_mut(ctx_menu_id)
                .unwrap()
                .downcast_mut::<CtxMenu>()
                .unwrap();

            SelectionEnum::from_str(&selection_str).handle_mouse_pressed(ctx_menu, node, evt);
        });

        self.loc.bld_selections(&mut self.selection_list);
    }
}

impl WidgetImpl for CtxMenu {}

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
