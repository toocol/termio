use tmui::{
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    tree_view::TreeView,
    widget::WidgetImpl,
};
use crate::ui::session_credential_tree::SessionCredentialTree;

#[extends(Widget, Layout(Stack))]
#[derive(Childrenable)]
pub struct WorkspacePanel {
    #[derivative(Default(value = "SessionCredentialTree::view()"))]
    #[children]
    session_tree: Box<TreeView>,
}

impl ObjectSubclass for WorkspacePanel {
    const NAME: &'static str = "WorkspacePanel";
}

impl ObjectImpl for WorkspacePanel {
    fn initialize(&mut self) {
        self.set_vexpand(true);
        self.set_hexpand(true);

        self.session_tree.set_hexpand(true);
        self.session_tree.set_vexpand(true);
    }
}

impl WidgetImpl for WorkspacePanel {}

impl WorkspacePanel {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}
