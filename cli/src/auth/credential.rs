use crate::{constant::ProtocolType, persistence::mgr::PersistenceMgr};
use serde::{Deserialize, Serialize};
use tmui::{
    tlib::{
        figure::Color,
        utils::{SnowflakeGuidGenerator, Timestamp},
    },
    views::{
        cell::{cell_index::CellIndex, cell_render::TextCellRender, Cell},
        node::node_render::NodeRender,
        tree_view::{tree_node::TreeNode, tree_view_object::TreeViewObject},
    },
};

use super::connect_info::ConnectInfo;

pub type CredentialId = u64;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialIdx {
    Protocol = 0,
    ShowName,
    Id,
}
impl CellIndex for CredentialIdx {
    #[inline]
    fn index(&self) -> usize {
        *self as usize
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Credential {
    id: CredentialId,
    shown_name: String,
    protocol: ProtocolType,
    connect_info: ConnectInfo,
    timestamp: u64,
}

impl Credential {
    #[inline]
    pub fn new(
        shown_name: Option<String>,
        protocol: ProtocolType,
        connect_info: ConnectInfo,
    ) -> Self {
        let shown_name = shown_name.unwrap_or_else(|| match connect_info {
            ConnectInfo::LocalShell(ref path) => {
                format!("LocalShell({})", path)
            }
            ConnectInfo::Ssh(ref host, _, _, _) => host.clone(),
            _ => String::new(),
        });
        Credential {
            id: SnowflakeGuidGenerator::next_id().expect("Generate uid failed."),
            shown_name,
            protocol,
            connect_info,
            timestamp: Timestamp::now().as_millis(),
        }
    }

    #[inline]
    pub fn from_tree_node(node: &TreeNode) -> Option<Self> {
        PersistenceMgr::get_credential(node.get_value(CredentialIdx::Id)?)
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
    pub fn protocol_type(&self) -> ProtocolType {
        self.protocol
    }

    #[inline]
    pub fn connect_info(&self) -> &ConnectInfo {
        &self.connect_info
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
            Cell::string()
                .value(self.shown_name.clone())
                .cell_render(TextCellRender::builder().color(Color::BLACK).build())
                .build(),
            Cell::value_cell().value(self.id).build(),
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
