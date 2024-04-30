use tmui::{
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    widget::WidgetImpl,
};

#[extends(Popup)]
pub struct SessionCtxMenu {}

impl ObjectSubclass for SessionCtxMenu {
    const NAME: &'static str = "SessionCtxMenu";
}

impl ObjectImpl for SessionCtxMenu {}

impl WidgetImpl for SessionCtxMenu {}

impl PopupImpl for SessionCtxMenu {}

impl SessionCtxMenu {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}