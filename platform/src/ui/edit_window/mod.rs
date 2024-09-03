use super::sessions::SESSION_CREDENTIAL_TREE;
use crate::components::{
    number_bundle::NumberBundle, password_bundle::PasswordBundle, text_bundle::TextBundle,
};
use cli::{auth::credential::Credential, constant::ProtocolType};
use log::debug;
use tlib::{connect, events::MouseEvent};
use tmui::{
    button::Button,
    input::number::Number,
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    views::tree_view::TreeView,
    widget::WidgetImpl,
};

#[extends(Widget, Layout(VBox))]
#[derive(Childrenable)]
pub struct EditWindow {
    #[children]
    #[derivative(Default(value = r#"TextBundle::new("Remote Host:")"#))]
    remote_host: Box<TextBundle>,

    #[children]
    #[derivative(Default(value = r#"TextBundle::new("Specifiy User:")"#))]
    user: Box<TextBundle>,

    #[children]
    #[derivative(Default(value = r#"PasswordBundle::new("Password:")"#))]
    password: Box<PasswordBundle>,

    #[children]
    #[derivative(Default(value = r#"NumberBundle::new("Port:")"#))]
    port: Box<NumberBundle>,

    #[children]
    #[derivative(Default(value = r#"Button::new(Some("Submit"))"#))]
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
        self.port.set_required(true);

        self.port.set_val(22.);
        self.port.set_min(0.);
        self.port.set_max(65535.);

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

        if !self.remote_host.check_required()
            || !self.user.check_required()
            || !self.password.check_required()
            || !self.port.check_required()
        {
            return;
        }

        let host = self.remote_host.value();
        let user = self.user.value();
        let password = self.password.value();
        let port = self.port.val().unwrap() as u32;
        let group_id = ApplicationWindow::window().get_param::<u32>("group_id").unwrap();

        self.window().call_response(move |win| {
            let sct = win
                .find_name_mut(&SESSION_CREDENTIAL_TREE)
                .unwrap()
                .downcast_mut::<TreeView>()
                .unwrap();

            let group = sct.get_store_mut().get_node_mut(group_id).unwrap();
            group.add_node(&Credential::new(
                None,
                host,
                user,
                password,
                port,
                ProtocolType::Ssh,
            ));
        });

        self.window().close();
    }
}
