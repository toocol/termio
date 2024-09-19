use serde::{Deserialize, Serialize};
use tmui::{
    tlib::{figure::Color, utils::Timestamp},
    views::{
        cell::{cell_index::CellIndex, cell_render::TextCellRender, Cell},
        node::node_render::NodeRender,
        tree_view::{tree_node::TreeNode, tree_view_object::TreeViewObject},
    },
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionGrpIdx {
    Name = 0,
    Timestamp,
}
impl CellIndex for SessionGrpIdx {
    #[inline]
    fn index(&self) -> usize {
        *self as usize
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionGroup {
    name: String,
    timestamp: u64,
}

impl TreeViewObject for SessionGroup {
    #[inline]
    fn cells(&self) -> Vec<Cell> {
        vec![
            Cell::string()
                .value(self.name.clone())
                .cell_render(TextCellRender::builder().color(Color::BLACK).build())
                .build(),
            Cell::value_cell().value(self.timestamp).build(),
        ]
    }

    #[inline]
    fn extensible(&self) -> bool {
        true
    }

    #[inline]
    fn node_render(&self) -> NodeRender {
        NodeRender::builder().build()
    }
}

impl SessionGroup {
    #[inline]
    pub fn new(name: impl ToString) -> Self {
        Self::new_with_time(name, Timestamp::now().as_millis())
    }

    #[inline]
    pub fn new_with_time(name: impl ToString, timestamp: u64) -> Self {
        Self {
            name: name.to_string(),
            timestamp,
        }
    }

    #[inline]
    pub fn from_tree_node(node: &TreeNode) -> Self {
        use SessionGrpIdx::*;
        let name = node.get_value::<String>(Name).unwrap();
        let timestamp = node.get_value::<u64>(Timestamp).unwrap();

        Self { name, timestamp }
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}
