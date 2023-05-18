use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use super::session::Session;
use lazy_static::lazy_static;
use tmui::{
    prelude::*,
    tlib::{object::{ObjectImpl, ObjectSubclass}, signals},
};

lazy_static! {
    static ref ACTIVE_SESSION: AtomicPtr<Session> = AtomicPtr::new(null_mut());
}

/// Session group manage all the sessions, and it brige view and emulation.
#[extends(Widget)]
#[derive(Default)]
pub struct TabPage {}
impl ObjectSubclass for TabPage {
    const NAME: &'static str = "SessionGroup";
}
impl ObjectImpl for TabPage {}
impl WidgetImpl for TabPage {}

pub trait TabPageSignals: ActionExt {
    signals!{
        /// Emited when tab page closed.
        tab_page_closed();
    }
}
impl TabPageSignals for TabPage {}

impl TabPage {
    #[inline]
    pub fn active_sesion() -> &'static mut Session {
        unsafe { ACTIVE_SESSION.load(Ordering::Relaxed).as_mut().unwrap() }
    }

    #[inline]
    pub fn set_active_session(session: &mut Session) {
        ACTIVE_SESSION.store(session, Ordering::Relaxed);
    }

    pub fn unbind_view_emulation(&mut self) {

    }

    pub fn bind_view_to_emulation(&mut self) {

    }
}
