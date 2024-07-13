use crate::ui::sessions::session_credential_tree;
use tmui::{tlib::events::MouseEvent, widget::WidgetImpl};

use super::CtxMenu;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionEnum {
    NewSession,
}

impl ToString for SelectionEnum {
    #[inline]
    fn to_string(&self) -> String {
        match self {
            Self::NewSession => STR_NEW_SESSION.to_string(),
        }
    }
}

impl SelectionEnum {
    #[inline]
    pub fn from_str(str: &str) -> Self {
        match str {
            STR_NEW_SESSION => Self::NewSession,
            _ => panic!("Unknown selection."),
        }
    }

    pub fn handle_mouse_pressed(
        &self,
        ctx_menu: &mut CtxMenu,
        widget: &mut dyn WidgetImpl,
        evt: &MouseEvent,
    ) {
        match self {
            Self::NewSession => {
                session_credential_tree::service::SessionCredentialService::new_session_pressed(
                    ctx_menu, widget,
                )
            }
        }
    }
}

/// Constants:
pub const STR_NEW_SESSION: &'static str = "New Session";
