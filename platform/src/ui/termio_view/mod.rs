use tmui::{
   prelude::*,
   tlib::object::{ObjectImpl, ObjectSubclass},
   widget::WidgetImpl,
};

use super::{title_bar::TitleBar, central_panel::CentralPanel};

#[extends(Widget, Layout(VBox))]
#[derive(Childrenable)]
pub struct TermioView {
    #[children]
    title_bar: Box<TitleBar>,

    #[children]
    central_panel: Box<CentralPanel>,
}

impl ObjectSubclass for TermioView {
   const NAME: &'static str = "TermioView";
}

impl ObjectImpl for TermioView {
    fn construct(&mut self) {
        self.parent_construct();

        self.set_hexpand(true);
        self.set_vexpand(true);
    }
}

impl WidgetImpl for TermioView {}

impl TermioView {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}