use tmui::{
    button::Button, prelude::*, tlib::object::{ObjectImpl, ObjectSubclass}, widget::WidgetImpl
};

use crate::components::{password_bundle::PasswordBundle, text_bundle::TextBundle};

#[extends(Widget, Layout(VBox))]
#[derive(Childrenable)]
pub struct EditWindow {
    #[children]
    #[derivative(Default(value = "TextBundle::new(\"Remote Host:\")"))]
    remote_host: Box<TextBundle>,

    #[children]
    #[derivative(Default(value = "TextBundle::new(\"Specifiy User:\")"))]
    user: Box<TextBundle>,

    #[children]
    #[derivative(Default(value = "PasswordBundle::new(\"Password:\")"))]
    password: Box<PasswordBundle>,

    #[children]
    #[derivative(Default(value = "Button::new(Some(\"Submit\"))"))]
    submit_btn: Box<Button>,
}

impl ObjectSubclass for EditWindow {
    const NAME: &'static str = "EditWindow";
}

impl ObjectImpl for EditWindow {}

impl WidgetImpl for EditWindow {}

impl EditWindow {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}