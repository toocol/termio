use tmui::{prelude::*, tlib::object::ObjectSubclass};

#[extends(Widget, Layout(HBox))]
#[derive(Childrenable)]
pub struct TitleBar {}

impl ObjectSubclass for TitleBar {
    const NAME: &'static str = "TitleBar";
}

impl ObjectImpl for TitleBar {
    fn construct(&mut self) {
        self.parent_construct();

        self.set_hexpand(true);
        self.height_request(30);
    }
}

impl WidgetImpl for TitleBar {}
