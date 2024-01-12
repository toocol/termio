use tmui::{
   prelude::*,
   tlib::object::{ObjectImpl, ObjectSubclass},
   widget::WidgetImpl,
};

#[extends(Widget)]
pub struct SessionBar {}

impl ObjectSubclass for SessionBar {
   const NAME: &'static str = "SessionBar";
}

impl ObjectImpl for SessionBar {}

impl WidgetImpl for SessionBar {}