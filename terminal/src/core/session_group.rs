#![allow(dead_code)]
use tmui::{prelude::*, tlib::object::{ObjectSubclass, ObjectImpl}};
use super::{session::Session, terminal_view::TerminalView};
use lazy_static::lazy_static;

#[extends(Object)]
#[derive(Default)]
pub struct SessionGroup {
    group_id: u16,
    sessions: Vec<Box<Session>>,
    view: Box<TerminalView>,
}
impl ObjectSubclass for SessionGroup {
    const NAME: &'static str = "SessionGroup";
}
impl ObjectImpl for SessionGroup {}

lazy_static! {
    
}