use super::left_panel::LeftPanel;
use crate::ui::terminal::Terminal;
use tmui::{
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    widget::WidgetImpl,
};

#[extends(Widget, Layout(Pane))]
#[derive(Childrenable)]
pub struct CentralPanel {
    #[children]
    left_panel: Box<LeftPanel>,

    #[children]
    terminal: Box<Terminal>,
}

impl ObjectSubclass for CentralPanel {
    const NAME: &'static str = "CentralPanel";
}

impl ObjectImpl for CentralPanel {
    fn initialize(&mut self) {
        self.set_hexpand(true);
        self.set_vexpand(true);
    }
}

impl WidgetImpl for CentralPanel {}
