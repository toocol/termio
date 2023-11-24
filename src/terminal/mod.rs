pub mod imp;

use tmui::prelude::*;

#[extends(SharedWidget, id = "terminal")]
pub struct Terminal {}

impl Terminal {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}