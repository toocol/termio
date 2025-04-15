use super::{
    ffi::winconpty::{
        close_conpty, open_conpty, resize_conpty, set_utf8_mode, start_read_listener,
        start_sub_process, write_data,
    },
    Pty, PtySignals,
};
use cli::session::SessionPropsId;
use derivative::Derivative;
use log::warn;
use std::{path::PathBuf, sync::Arc, thread};
use tmui::{prelude::*, tipc::parking_lot::Mutex, tlib::object::ObjectSubclass};

#[extends(Object)]
pub struct ConPty {
    cols: i32,
    rows: i32,
    #[derivative(Default(value = "std::env::current_dir().unwrap()"))]
    working_directory: PathBuf,
    #[derivative(Default(value = "true"))]
    writeable: bool,
    utf8_mode: bool,
    timeout: u32,
    /// Xon/Xoff flow control.
    xon_xoff: bool,
    running: bool,
    data_buffer: Arc<Mutex<Vec<u8>>>,
    fd: i32,
}

impl ObjectSubclass for ConPty {
    const NAME: &'static str = "ConPty";
}

impl ObjectImpl for ConPty {}

impl Pty for ConPty {
    fn start(
        &mut self,
        _id: SessionPropsId,
        program: &str,
        arguments: Vec<&str>,
        enviroments: Vec<&str>,
    ) -> bool {
        set_utf8_mode(self.utf8_mode);

        let mut cmd = program.to_string();

        let fd = open_conpty(self.cols, self.rows);
        if fd == 0 {
            return false;
        }

        let data_buffer = self.data_buffer.clone();
        start_read_listener(fd, move |data| {
            data_buffer.lock().extend_from_slice(&data.as_bytes());
        });

        // Generate the program arguments.
        if !arguments.is_empty() {
            arguments.iter().for_each(|arg| {
                cmd.push_str(" ");
                cmd.push_str(arg);
            });
        };

        // Generate the program envs.
        if !enviroments.is_empty() {
            enviroments.iter().for_each(|env| {
                cmd.push_str(" ");
                cmd.push_str(env);
            });
        };

        thread::spawn(move || {
            start_sub_process(fd, &cmd);
        });
        self.running = true;
        self.fd = fd;
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
        if self.fd == 0 {
            return;
        }
        resize_conpty(self.fd, cols, rows);
    }

    #[inline]
    fn window_size(&self) -> Size {
        Size::new(self.cols, self.rows)
    }

    #[inline]
    fn set_working_directory(&mut self, directory: PathBuf) {
        self.working_directory = directory;
    }

    #[inline]
    fn is_running(&self) -> bool {
        self.running
    }

    #[inline]
    fn set_utf8_mode(&mut self, on: bool) {
        self.utf8_mode = on;
    }

    #[inline]
    fn set_timeout(&mut self, timeout: u32) {
        self.timeout = timeout
    }

    #[inline]
    fn send_data(&mut self, data: String) {
        if !self.writeable {
            warn!("The `ConPTY` is not writeable.");
            return;
        }
        write_data(self.fd, &data);
    }

    #[inline]
    fn read_data(&mut self) -> Vec<u8> {
        let mut data = vec![];

        let mut guard = self.data_buffer.lock();
        data.extend_from_slice(guard.as_slice());
        guard.clear();

        data
    }

    #[inline]
    fn on_window_closed(&mut self) {
        close_conpty(self.fd);
    }
}

impl PtySignals for ConPty {}

impl ConPty {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}
