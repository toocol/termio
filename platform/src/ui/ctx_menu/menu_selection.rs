use tmui::{
    tlib::figure::Color,
    views::{
        cell::{cell_render::TextCellRender, Cell, CellStringBuilder},
        list_view::list_view_object::ListViewObject,
        node::node_render::NodeRender,
    },
};

pub struct MenuSelection {
    val: String,
}

impl ListViewObject for MenuSelection {
    #[inline]
    fn cells(&self) -> Vec<Cell> {
        vec![CellStringBuilder::default()
            .value(self.val.clone())
            .cell_render(TextCellRender::builder().color(Color::BLACK).build())
            .build()]
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
