use crate::{pty_mut, pty_ref};
use cli::session::SessionPropsId;
use derivative::Derivative;
use log::warn;
use std::{
    ffi::OsString,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use winptyrs::{AgentConfig, MouseMode, PTYArgs, PTYBackend, PTY};

use super::{pty_receive_pool, Pty, PtySignals};
use tmui::{prelude::*, tlib::object::ObjectSubclass};

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
    pty: Option<Arc<Mutex<PTY>>>,
}

impl ObjectSubclass for ConPty {
    const NAME: &'static str = "ConPty";
}

impl ObjectImpl for ConPty {}

impl Pty for ConPty {
    fn start(
        &mut self,
        id: SessionPropsId,
        program: &str,
        arguments: Vec<&str>,
        enviroments: Vec<&str>,
    ) -> bool {
        let cmd = OsString::from(program);

        let pty_args = PTYArgs {
            cols: self.cols,
            rows: self.rows,
            mouse_mode: MouseMode::WINPTY_MOUSE_MODE_NONE,
            timeout: self.timeout,
            agent_config: AgentConfig::WINPTY_FLAG_COLOR_ESCAPES,
        };

        self.pty = Some(Arc::new(Mutex::new(
            PTY::new_with_backend(&pty_args, PTYBackend::ConPTY).unwrap(),
        )));

        // Generate the program arguments.
        let args = if arguments.is_empty() {
            None
        } else {
            let mut args = OsString::new();
            arguments.iter().for_each(|arg| {
                args.push(arg);
                args.push(" ");
            });
            Some(args)
        };

        // Generate the program envs.
        let envs = if enviroments.is_empty() {
            None
        } else {
            let mut envs = OsString::new();
            enviroments.iter().for_each(|env| {
                envs.push(env);
                envs.push(" ");
            });
            Some(envs)
        };

        pty_mut!(self)
            .lock()
            .unwrap()
            .spawn(
                cmd,
                args,
                Some(self.working_directory.as_os_str().to_os_string()),
                envs,
            )
            .unwrap();

        pty_receive_pool().add_pty(id, pty_ref!(self).clone());

        self.running = true;

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
        if let Some(ref pty) = self.pty {
            pty.lock().unwrap().set_size(cols, rows).unwrap()
        }
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
        pty_mut!(self)
            .lock()
            .unwrap()
            .write(OsString::from(data))
            .unwrap();
    }

    #[inline]
    fn read_data(&mut self) -> Vec<u8> {
        unreachable!()
    }

    #[inline]
    fn on_window_closed(&mut self) {}
}

impl PtySignals for ConPty {}

impl ConPty {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}
