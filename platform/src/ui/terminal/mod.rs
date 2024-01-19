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
    fn initialize(&mut self) {
        self.set_vexpand(true);
        self.set_hexpand(true);

        self.set_focus(true);
        self.set_mouse_tracking(true);
    }
}

impl WidgetImpl for Terminal {}

impl SharedWidgetImpl for Terminal {}
