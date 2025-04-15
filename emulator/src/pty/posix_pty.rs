use super::{Pty, PtySignals};
use cli::session::SessionPropsId;
use pty::prelude::Fork;
use std::{
    os::fd::{AsRawFd, RawFd},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tmui::{prelude::*, tlib::object::ObjectSubclass};

#[extends(Object)]
pub struct PosixPty {
    cols: i32,
    rows: i32,
    #[derivative(Default(value = "std::env::current_dir().unwrap()"))]
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
        id: SessionPropsId,
        program: &str,
        arguments: Vec<&str>,
        enviroment: Vec<&str>,
    ) -> bool{
        // Generate the program arguments.
        let mut args = String::new();
        arguments.iter().for_each(|arg| {
            args.push_str(arg);
            args.push(' ');
        });

        // Generate the program envs.
        let mut envs = String::new();
        enviroment.iter().for_each(|env| {
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

    #[inline]
    fn send_data(&mut self, data: String) {
        todo!()
    }

    #[inline]
    fn read_data(&mut self) -> Vec<u8> {
        unreachable!()
    }

    #[inline]
    fn on_window_closed(&mut self) {
        
    }
}

impl PtySignals for PosixPty {}

impl PosixPty {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}
