use tmui::{
   prelude::*,
   tlib::object::{ObjectImpl, ObjectSubclass},
   widget::WidgetImpl,
};

#[extends(Widget, Layout(HBox))]
pub struct ActivityBar {}

impl ObjectSubclass for ActivityBar {
   const NAME: &'static str = "ActivityBar";
}

impl ObjectImpl for ActivityBar {}

impl WidgetImpl for ActivityBar {}