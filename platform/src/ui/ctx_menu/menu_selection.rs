use super::selection_enum::SelectionEnum;
use tmui::{
    tlib::figure::Color,
    views::{
        cell::{cell_render::TextCellRender, Cell, CellStringBuilder},
        list_view::list_view_object::ListViewObject,
        node::node_render::NodeRender,
    },
};

pub struct MenuSelection {
    val: SelectionEnum,
}

impl ListViewObject for MenuSelection {
    #[inline]
    fn cells(&self) -> Vec<Cell> {
        vec![CellStringBuilder::default()
            .value(self.val.to_string())
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
    pub fn new(val: SelectionEnum) -> Self {
        Self { val }
    }
}

pub trait CtxMenuSelectionCreator {
    fn create_selections() -> Vec<MenuSelection>;
}
