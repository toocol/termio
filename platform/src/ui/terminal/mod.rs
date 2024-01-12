use tmui::{prelude::*, tlib::object::ObjectSubclass};

#[extends(SharedWidget, id = "terminal")]
pub struct Terminal {}

impl Terminal {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}

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

impl SharedWidgetImpl for Terminal {}
