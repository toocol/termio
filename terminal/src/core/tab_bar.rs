use tmui::{
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    widget::WidgetImpl,
};

#[extends(Widget)]
pub struct PageBar {}

impl ObjectSubclass for PageBar {
    const NAME: &'static str = "PageBar";
}
impl ObjectImpl for PageBar {}
impl WidgetImpl for PageBar {}
