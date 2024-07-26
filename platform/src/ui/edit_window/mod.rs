use cli::{auth::credential::Credential, constant::ProtocolType};
use log::debug;
use tlib::{connect, events::MouseEvent};
use tmui::{
    button::Button,
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    views::tree_view::TreeView,
    widget::WidgetImpl,
};

use crate::{
    components::{password_bundle::PasswordBundle, text_bundle::TextBundle},
};

use super::sessions::SESSION_CREDENTIAL_TREE;

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

impl ObjectImpl for EditWindow {
    #[inline]
    fn initialize(&mut self) {
        self.set_spacing(10);
        self.set_hexpand(true);
        self.set_vexpand(true);
        self.set_homogeneous(false);

        self.remote_host.set_required(true);
        self.user.set_required(true);
        self.password.set_required(true);

        self.submit_btn.width_request(50);
        self.submit_btn.height_request(20);
        self.submit_btn.set_halign(Align::Center);

        connect!(self.submit_btn, mouse_pressed(), self, submit(MouseEvent));
    }
}

impl WidgetImpl for EditWindow {}

impl EditWindow {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }

    pub fn submit(&mut self, _: MouseEvent) {
        debug!("Submit button pressed.");

        let host_check = self.remote_host.check_required();
        let user_check = self.user.check_required();
        let password_check = self.password.check_required();

        if !host_check || !user_check || !password_check {
            return;
        }

        let host = self.remote_host.value();
        let user = self.user.value();
        let password = self.password.value();

        self.window().call_response(move |win| {
            let sct = win
                .find_name_mut(&SESSION_CREDENTIAL_TREE)
                .unwrap()
                .downcast_mut::<TreeView>()
                .unwrap();

            let node_session = sct
                .get_store_mut()
                .root_mut()
                .children_mut()
                .first_mut()
                .unwrap();
            node_session.add_node(&Credential::new(
                None,
                host,
                user,
                password,
                "".to_string(),
                22,
                ProtocolType::Ssh,
            ));
        });

        self.window().close();
    }
}
