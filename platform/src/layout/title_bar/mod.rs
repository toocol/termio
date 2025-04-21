use crate::ui::{app_icon::AppIcon, sessions::SessionBar, win_control_buttons::WinControlButtons};
use tmui::{prelude::*, tlib::object::ObjectSubclass};

#[extends(Widget, Layout(HBox))]
#[derive(Childrenable)]
pub struct TitleBar {
    #[children]
    app_icon: Tr<AppIcon>,

    #[children]
    session_bar: Tr<SessionBar>,

    #[children]
    win_control_buttons: Tr<WinControlButtons>,
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
