use cli::ProtocolType;
use tmui::{
    tlib::{figure::Color, utils::Timestamp},
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
    create_time: Timestamp,
}

impl TreeViewObject for SessionCredential {
    #[inline]
    fn cells(&self) -> Vec<Cell> {
        vec![
            Cell::string()
                .value(self.protocol.as_str().to_string())
                .cell_render(TextCellRender::builder().color(Color::BLACK).build())
                .build(),
            Cell::string()
                .value(self.name.clone())
                .cell_render(TextCellRender::builder().color(Color::BLACK).build())
                .build(),
            Cell::string()
                .value(self.address.clone())
                .cell_render(TextCellRender::builder().color(Color::BLACK).build())
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
    pub fn new(
        protocol: ProtocolType,
        address: impl ToString,
        name: Option<impl ToString>,
    ) -> Self {
        let address = address.to_string();
        let name = name.map(|n| n.to_string()).unwrap_or(address.clone());
        Self {
            protocol,
            name,
            address,
            create_time: Timestamp::now(),
        }
    }
}
