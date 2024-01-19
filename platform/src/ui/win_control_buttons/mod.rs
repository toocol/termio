use tmui::{
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    widget::WidgetImpl,
};

#[extends(Widget, Layout(HBox))]
pub struct WinControlButtons {}

impl ObjectSubclass for WinControlButtons {
    const NAME: &'static str = "WinControlButtons";
}

impl ObjectImpl for WinControlButtons {
    fn initialize(&mut self) {
        self.set_background(Color::RED);

        self.set_vexpand(true);
        self.width_request(136);

        self.set_halign(Align::End);
        self.set_mouse_tracking(true);
    }
}

impl WidgetImpl for WinControlButtons {}

impl WinControlButtons {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}
