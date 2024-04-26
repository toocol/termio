use tmui::{
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    widget::WidgetImpl,
};

use crate::ui::activity_bar::ActivityBar;

use super::workspace_panel::WorkspacePanel;

#[extends(Widget, Layout(HBox))]
#[derive(Childrenable)]
pub struct LeftPanel {
    #[children]
    activity_bar: Box<ActivityBar>,

    #[children]
    workspace: Box<WorkspacePanel>
}

impl ObjectSubclass for LeftPanel {
    const NAME: &'static str = "LeftPanel";
}

impl ObjectImpl for LeftPanel {
    fn initialize(&mut self) {
        self.set_vexpand(true);
        self.width_request(300);
        self.set_background(Color::GREY_LIGHT);
        self.set_size_hint(SizeHint::new().with_min_width(300).with_max_width(900));
    }
}

impl WidgetImpl for LeftPanel {}

impl LeftPanel {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}
