use crate::ui::sessions::session_credential_tree;
use tmui::{tlib::events::MouseEvent, views::list_view::list_node::ListNode};

use super::CtxMenu;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionEnum {
    NewSession,
    NewGroup,
}

impl ToString for SelectionEnum {
    #[inline]
    fn to_string(&self) -> String {
        match self {
            Self::NewSession => STR_NEW_SESSION.to_string(),
            Self::NewGroup => STR_NEW_GROUP.to_string(),
        }
    }
}

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
        evt: &MouseEvent,
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
pub const STR_NEW_SESSION: &'static str = "New Session";
pub const STR_NEW_GROUP: &'static str = "New Group";
