use serde::{Deserialize, Serialize};
use tmui::{
    tlib::{
        figure::Color,
        utils::{SnowflakeGuidGenerator, Timestamp},
    },
    views::{
        cell::{cell_render::TextCellRender, Cell},
        node::node_render::NodeRender,
        tree_view::tree_view_object::TreeViewObject,
    },
};

use crate::constant::ProtocolType;
pub type CredentialId = u64;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Credential {
    id: CredentialId,
    shown_name: String,
    host: String,
    user: String,
    password: String,
    port: u32,
    protocol: ProtocolType,
    timestamp: u64,
}

impl Credential {
    #[inline]
    pub fn new(
        shown_name: Option<String>,
        host: String,
        user: String,
        password: String,
        port: u32,
        protocol: ProtocolType,
    ) -> Self {
        let shown_name = shown_name.unwrap_or(host.clone());
        Credential {
            id: SnowflakeGuidGenerator::next_id().expect("Generate uid failed."),
            shown_name,
            host,
            user,
            password,
            port,
            protocol,
            timestamp: Timestamp::now().as_millis(),
        }
    }

    #[inline]
    pub fn id(&self) -> CredentialId {
        self.id
    }

    #[inline]
    pub fn shown_name(&self) -> &str {
        &self.shown_name
    }

    #[inline]
    pub fn host(&self) -> &str {
        &self.host
    }

    #[inline]
    pub fn user(&self) -> &str {
        &self.user
    }

    #[inline]
    pub fn password(&self) -> &str {
        &self.password
    }
    
    #[inline]
    pub fn port(&self) -> u32 {
        self.port
    }

    #[inline]
    pub fn protocol_type(&self) -> ProtocolType {
        self.protocol
    }

    #[inline]
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl TreeViewObject for Credential {
    #[inline]
    fn cells(&self) -> Vec<Cell> {
        vec![
            Cell::string()
                .value(self.protocol.as_str().to_string())
                .cell_render(TextCellRender::builder().color(Color::BLACK).build())
                .build(),
            Cell::value_cell().value(self.timestamp).build(),
            Cell::string()
                .value(self.shown_name.clone())
                .cell_render(TextCellRender::builder().color(Color::BLACK).build())
                .build(),
            Cell::value_cell().value(self.port).build(),
            Cell::value_cell().value(self.password.clone()).build(),
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
