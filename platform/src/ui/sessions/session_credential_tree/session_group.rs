use tmui::{
    tlib::figure::Color,
    views::{
        cell::{cell_render::TextCellRender, Cell},
        node::node_render::NodeRender,
        tree_view::tree_view_object::TreeViewObject,
    },
};

pub struct SessionGroup {
    name: String,
}

impl TreeViewObject for SessionGroup {
    #[inline]
    fn cells(&self) -> Vec<Cell> {
        vec![Cell::string()
            .value(self.name.clone())
            .cell_render(TextCellRender::builder().color(Color::BLACK).build())
            .build()]
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
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}
