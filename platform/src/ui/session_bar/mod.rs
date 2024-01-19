use tmui::{
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    widget::WidgetImpl,
};

#[extends(Widget, Layout(HBox))]
#[derive(Childrenable)]
pub struct SessionBar {}

impl ObjectSubclass for SessionBar {
    const NAME: &'static str = "SessionBar";
}

impl ObjectImpl for SessionBar {
    fn initialize(&mut self) {
        self.set_background(Color::CYAN);

        self.set_vexpand(true);
        self.set_hexpand(true);
        self.set_hscale(0.7);
    }
}

impl WidgetImpl for SessionBar {}
