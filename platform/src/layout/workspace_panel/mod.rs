use crate::ui::sessions::{load::load_data, SessionCredentialTree};
use tlib::run_after;
use tmui::{
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    views::tree_view::{tree_node::TreeNode, TreeView},
    widget::WidgetImpl,
};

#[extends(Widget, Layout(Stack))]
#[derive(Childrenable)]
#[run_after]
#[async_task(name = "BuildSessionTree", value = "Box<TreeNode>")]
pub struct WorkspacePanel {
    #[derivative(Default(value = "SessionCredentialTree::view()"))]
    #[children]
    session_tree: Tr<TreeView>,
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

impl WidgetImpl for WorkspacePanel {
    fn run_after(&mut self) {
        let store = self.session_tree.get_store();
        let store_id = store.id();
        let level = store.root().level() + 1;

        self.session_tree.start_loading();
        self.build_session_tree(load_data(store_id, level), |w: &mut WorkspacePanel, val| {
            w.session_tree.stop_loading();
            w.session_tree
                .get_store_mut()
                .root_mut()
                .add_node_directly(val);
        })
    }
}

impl WorkspacePanel {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}
