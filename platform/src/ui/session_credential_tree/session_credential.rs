use cli::ProtocolType;
use tmui::{
    tlib::utils::TimeStamp,
    tree_view::{
        cell::{cell_render::TextCellRender, Cell},
        node_render::NodeRender,
        tree_view_object::TreeViewObject,
    },
};

pub struct SessionCredential {
    protocol: ProtocolType,
    name: String,
    address: String,
    create_time: u64,
}

impl TreeViewObject for SessionCredential {
    #[inline]
    fn cells(&self) -> Vec<Cell> {
        vec![
            Cell::string()
                .value(self.protocol.as_str().to_string())
                .cell_render(TextCellRender::builder().build())
                .build(),
            Cell::string()
                .value(self.name.as_str().to_string())
                .cell_render(TextCellRender::builder().build())
                .build(),
            Cell::string()
                .value(self.address.as_str().to_string())
                .cell_render(TextCellRender::builder().build())
                .build(),
        ]
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

impl SessionCredential {
    #[inline]
    pub fn new(protocol: ProtocolType, address: String, name: Option<String>) -> Self {
        let name = name.or(Some(address.clone())).unwrap();
        Self {
            protocol,
            name,
            address,
            create_time: TimeStamp::timestamp(),
        }
    }
}
