use super::Terminal;
use tmui::{
    tlib::object::{ObjectImpl, ObjectSubclass, ObjectImplExt},
    widget::{WidgetImpl, WidgetExt},
};

impl ObjectSubclass for Terminal {
    const NAME: &'static str = "Terminal";
}

impl ObjectImpl for Terminal {
    fn construct(&mut self) {
        self.parent_construct();

        self.set_vexpand(true);
        self.set_hexpand(true);

        self.set_focus(true);
        self.set_mouse_tracking(true);
        self.set_rerender_difference(true);
    }
}

impl WidgetImpl for Terminal {}
