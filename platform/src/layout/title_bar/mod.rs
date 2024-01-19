use tmui::{prelude::*, tlib::object::ObjectSubclass};

use crate::ui::{session_bar::SessionBar, win_control_buttons::WinControlButtons};

#[extends(Widget, Layout(HBox))]
#[derive(Childrenable)]
pub struct TitleBar {
    #[children]
    app_icon: Box<Widget>,

    #[children]
    session_bar: Box<SessionBar>,

    #[children]
    win_control_buttons: Box<WinControlButtons>,
}

impl ObjectSubclass for TitleBar {
    const NAME: &'static str = "TitleBar";
}

impl ObjectImpl for TitleBar {
    fn initialize(&mut self) {
        self.set_hexpand(true);
        self.height_request(30);

        self.app_icon.width_request(30);
        self.app_icon.set_vexpand(true);
    }
}

impl WidgetImpl for TitleBar {}
