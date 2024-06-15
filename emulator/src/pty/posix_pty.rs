use crate::{pty_mut, pty_ref};

use super::{ProtocolType, Pty, PtySignals, pty_receive_pool};
use pty::prelude::Fork;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex}, os::fd::{RawFd, AsRawFd},
};
use tmui::{prelude::*, tlib::object::ObjectSubclass};

#[extends(Object)]
pub struct PosixPty {
    cols: i32,
    rows: i32,
    working_directory: PathBuf,
    writeable: bool,
    utf8_mode: bool,
    timeout: u32,
    /// Xon/Xoff flow control.
    xon_xoff: bool,
    running: bool,
    pty: Option<Arc<Mutex<Fork>>>,
    pid: Option<RawFd>,
}

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
        // Generate the program arguments.
        let mut args = String::new();
        arguments.iter().for_each(|arg| {
            args.push_str(arg);
            args.push(' ');
        });

        // Generate the program envs.
        let mut envs = String::new();
        enviroments.iter().for_each(|env| {
            envs.push_str("export ");
            envs.push_str(env);
            envs.push('\n');
        });

        let mut program = program.to_string();
        program.push_str(&args);
        program.push(' ');
        envs.push_str(&program);

        let static_program = Box::leak(envs.into_boxed_str());
        self.pty = Some(Arc::new(Mutex::new(Fork::new(static_program).unwrap())));

        if let Some(master) = pty_mut!(self).lock().unwrap().is_parent().ok() {
            self.pid = Some(master.as_raw_fd())
        }

        pty_receive_pool().add_pty(self.id(), pty_ref!(self).clone(), self.receive_data());

        true
    }

    #[inline]
    fn set_writeable(&mut self, writeable: bool) {
        self.writeable = writeable
    }

    #[inline]
    fn writeable(&self) -> bool {
        self.writeable
    }

    #[inline]
    fn set_flow_control_enable(&mut self, on: bool) {
        self.xon_xoff = on
    }

    #[inline]
    fn flow_control_enable(&self) -> bool {
        self.xon_xoff
    }

    #[inline]
    fn set_window_size(&mut self, cols: i32, rows: i32) {
        self.cols = cols;
        self.rows = rows;
    }

    #[inline]
    fn window_size(&self) -> Size {
        Size::new(self.cols, self.rows)
    }

    #[inline]
    fn set_working_directory(&mut self, directory: std::path::PathBuf) {
        self.working_directory = directory
    }

    #[inline]
    fn is_running(&self) -> bool {
        self.running
    }

    #[inline]
    fn set_utf8_mode(&mut self, on: bool) {
        self.utf8_mode = on
    }

    #[inline]
    fn set_timeout(&mut self, timeout: u32) {
        self.timeout = timeout
    }

    fn send_data(&mut self, data: String) {
        todo!()
    }
}

impl PtySignals for PosixPty {}

impl PosixPty {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}
