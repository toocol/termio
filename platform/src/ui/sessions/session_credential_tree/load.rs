use cli::{
    persistence::mgr::PersistenceMgr,
    session::{session_grp_pers::SessionGrpPers, ROOT_SESSION},
};
use tmui::views::tree_view::tree_node::TreeNode;

pub async fn load_data(store_id: u32, level: i32) -> Box<TreeNode> {
    PersistenceMgr::load_data();

    if PersistenceMgr::empty_pesistence() {
        let root_grp = SessionGrpPers::new(ROOT_SESSION);
        let view_obj = root_grp.to_view_obj();
        PersistenceMgr::set_root_group(root_grp);
        TreeNode::create(store_id, level, &view_obj)
    } else {
        PersistenceMgr::with_guard(move |guard| {
            let root = guard.root_group();
            let mut root_node = TreeNode::create(store_id, level, &root.to_view_obj());
            root.build_node(guard.grp_credential_map(), &mut root_node);
            root_node.sort(true);
            root_node
        })
    }
}
