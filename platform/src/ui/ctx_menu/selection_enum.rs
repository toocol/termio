use crate::ui::sessions::session_credential_tree;
use std::fmt::Display;
use tmui::{tlib::events::MouseEvent, views::list_view::list_node::ListNode};

use super::CtxMenu;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionEnum {
    NewSession,
    NewGroup,
}

impl Display for SelectionEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NewSession => f.write_str(STR_NEW_SESSION),
            Self::NewGroup => f.write_str(STR_NEW_GROUP),
        }
    }
}

#[allow(clippy::should_implement_trait)]
impl SelectionEnum {
    #[inline]
    pub fn from_str(str: &str) -> Self {
        match str {
            STR_NEW_SESSION => Self::NewSession,
            STR_NEW_GROUP => Self::NewGroup,
            _ => panic!("Unknown selection."),
        }
    }

    pub fn handle_mouse_pressed(
        &self,
        ctx_menu: &mut CtxMenu,
        node: &mut ListNode,
        _evt: &MouseEvent,
    ) {
        match self {
            Self::NewSession => {
                session_credential_tree::service::SessionCredentialService::new_session_pressed(
                    ctx_menu, node,
                )
            }
            Self::NewGroup => {
                session_credential_tree::service::SessionCredentialService::new_group_pressed(
                    ctx_menu, node,
                )
            }
        }
    }
}

/// Constants:
pub const STR_NEW_SESSION: &str = "New Session";
pub const STR_NEW_GROUP: &str = "New Group";
