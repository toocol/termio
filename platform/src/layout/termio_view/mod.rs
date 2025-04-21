use super::{central_panel::CentralPanel, status_bar::StatusBar, title_bar::TitleBar};
use tmui::{
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    widget::WidgetImpl,
};

#[extends(Widget, Layout(VBox))]
#[derive(Childrenable)]
pub struct TermioView {
    #[children]
    title_bar: Tr<TitleBar>,

    #[children]
    central_panel: Tr<CentralPanel>,

    #[children]
    status_bar: Tr<StatusBar>,
}

impl ObjectSubclass for TermioView {
    const NAME: &'static str = "TermioView";
}

impl ObjectImpl for TermioView {
    fn initialize(&mut self) {
        self.set_hexpand(true);
        self.set_vexpand(true);
    }
}

impl WidgetImpl for TermioView {}

impl TermioView {
    #[inline]
    pub fn new() -> Tr<Self> {
        Self::new_alloc()
    }
}
