#![cfg(not(target_os = "windows"))]
use super::{ProtocolType, Pty, PtySignals};
use derivative::Derivative;
use tmui::{prelude::*, tlib::object::ObjectSubclass};

#[extends(Object)]
#[derive(Derivative)]
#[derivative(Default)]
pub struct PosixPty {}

impl ObjectSubclass for PosixPty {
    const NAME: &'static str = "PosixPty";
}

impl ObjectImpl for PosixPty {}

impl Pty for PosixPty {
    fn start(
        &mut self,
        program: &str,
        arguments: Vec<&str>,
        enviroments: Vec<&str>,
        _protocol_type: ProtocolType,
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

    fn set_window_size(&mut self, cols: i32, rows: i32) {
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

    fn set_timeout(&mut self, timeout: u32) {
        todo!()
    }

    fn send_data(&mut self, data: &str) {
        todo!()
    }
}

impl PtySignals for PosixPty {}

impl PosixPty {
    #[inline]
    pub fn new() -> Self {
        Object::new(&[])
    }
}
