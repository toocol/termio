#![cfg(target_os = "windows")]
use derivative::Derivative;

use super::{Pty, ProtocolType, PtySignals};
use tmui::{prelude::*, tlib::object::ObjectSubclass};

#[extends(Object)]
#[derive(Derivative)]
#[derivative(Default)]
pub struct ConPty {}

impl ObjectSubclass for ConPty {
    const NAME: &'static str = "ConPty";
}

impl ObjectImpl for ConPty {}

impl Pty for ConPty {
    fn start(
        &mut self,
        program: &str,
        arguments: Vec<&str>,
        enviroments: Vec<&str>,
        protocol_type: ProtocolType,
    ) -> bool {
        todo!()
    }

    fn set_writeable(&mut self, writeable: bool) {
        todo!()
    }

    fn writeable(&self) -> bool {
        todo!()
    }

    fn set_flow_control_enable(&mut self, on: bool) {
        todo!()
    }

    fn flow_control_enable(&self) -> bool {
        todo!()
    }

    fn set_window_size(&mut self, lines: i32, cols: i32) {
        todo!()
    }

    fn window_size(&self) -> tmui::prelude::Size {
        todo!()
    }

    fn set_working_directory(&mut self, directory: std::path::PathBuf) {
        todo!()
    }

    fn is_running(&self) -> bool {
        todo!()
    }

    fn set_utf8_mode(&mut self, on: bool) {
        todo!()
    }

    fn lock_pty(&mut self, lock: bool) {
        todo!()
    }

    fn send_data(&mut self, data: &str) {
        todo!()
    }
}

impl PtySignals for ConPty {}

impl ConPty {
    #[inline]
    pub fn new() -> Self {
        Object::new(&[])
    }
}