use tmui::{
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    widget::WidgetImpl,
};

#[extends(Widget, Layout(VBox))]
pub struct ActivityBar {}

impl ObjectSubclass for ActivityBar {
    const NAME: &'static str = "ActivityBar";
}

impl ObjectImpl for ActivityBar {
    fn initialize(&mut self) {
        self.set_background(Color::GREY_DARK);

        self.set_vexpand(true);
        self.width_request(50);

        self.set_borders(0., 1., 0., 0.);
    }
}

impl WidgetImpl for ActivityBar {}
