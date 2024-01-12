use tmui::{
   prelude::*,
   tlib::object::{ObjectImpl, ObjectSubclass},
   widget::WidgetImpl,
};

use super::terminal::Terminal;

#[extends(Widget, Layout(HBox))]
#[derive(Childrenable)]
pub struct CentralPanel {
    #[children]
    terminal: Box<Terminal>,
}

impl ObjectSubclass for CentralPanel {
   const NAME: &'static str = "CentralPanel";
}

impl ObjectImpl for CentralPanel {
    fn construct(&mut self) {
        self.parent_construct();

        self.set_hexpand(true);
        self.set_vexpand(true);
        self.set_homogeneous(true);
    }
}

impl WidgetImpl for CentralPanel {}