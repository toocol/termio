use tmui::tree_view::{
    cell::{cell_render::TextCellRender, Cell, CellStringBuilder},
    node_render::NodeRender,
    tree_view_object::TreeViewObject,
};

pub struct MenuSelection {
    val: String,
}

impl TreeViewObject for MenuSelection {
    #[inline]
    fn cells(&self) -> Vec<Cell> {
        vec![CellStringBuilder::default()
            .value(self.val.clone())
            .cell_render(TextCellRender::builder().build())
            .build()]
    }

    #[inline]
    fn extensible(&self) -> bool {
        false
    }

    #[inline]
    fn node_render(&self) -> NodeRender {
        NodeRender::builder().build()
    }
}

impl MenuSelection {
    #[inline]
    pub fn new(val: impl ToString) -> Self {
        Self {
            val: val.to_string(),
        }
    }
}
