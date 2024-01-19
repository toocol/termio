use tmui::{
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    widget::WidgetImpl,
};

#[extends(Widget, Layout(HBox))]
pub struct StatusBar {}

impl ObjectSubclass for StatusBar {
    const NAME: &'static str = "StatusBar";
}

impl ObjectImpl for StatusBar {
    fn initialize(&mut self) {
        self.set_background(Color::GREY);

        self.set_hexpand(true);
        self.height_request(20);

        self.set_borders(1., 0., 0., 0.);
    }
}

impl WidgetImpl for StatusBar {}

impl StatusBar {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}
