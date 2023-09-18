#![allow(dead_code)]
#[cfg(target_os = "windows")]
pub mod con_pty;
#[cfg(not(target_os = "windows"))]
pub mod posix_pty;

use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, Once},
    thread,
    time::Duration,
};
use tmui::{
    prelude::*,
    tlib::{emit, signals},
};
#[cfg(target_os = "windows")]
use winptyrs::PTY;
#[cfg(not(target_os = "windows"))]
use pty::prelude::Fork;
#[cfg(not(target_os = "windows"))]
use std::io::Read;

#[repr(u8)]
#[derive(Default)]
pub enum ProtocolType {
    #[default]
    None = 0,
    Ssh,
    Mosh,
    Telnet,
    Rsh,
    LocalShell,
}

impl AsMutPtr for dyn Pty {}

pub trait Pty: PtySignals {
    /// Start the terminal process.
    ///
    /// Return true if the process was started successfully or non-zero otherwise.
    fn start(
        &mut self,
        program: &str,
        arguments: Vec<&str>,
        enviroments: Vec<&str>,
        protocol_type: ProtocolType,
    ) -> bool;

    /// Set the terminal process was writeable or not.
    fn set_writeable(&mut self, writeable: bool);

    /// Get the terminal process was writeable or not.
    fn writeable(&self) -> bool;

    /// Enables or disables Xon/Xoff flow control. <br>
    /// The flow control setting may be changed later by a terminal application, so flow_control_enabled()
    /// may not equal the value of @p on in the previous call to set_flow_control_enabled().
    fn set_flow_control_enable(&mut self, on: bool);

    /// Queries the terminal state and returns true if Xon/Xoff flow control is enabled.
    fn flow_control_enable(&self) -> bool;

    /// Sets the size of the window (in lines and columns of characters) used by the teletype.
    fn set_window_size(&mut self, cols: i32, rows: i32);

    /// Returns the size of the window used by this teletype.
    fn window_size(&self) -> Size;

    /// Set the working directory of terminal process.
    fn set_working_directory(&mut self, directory: PathBuf);

    /// Get the status that terminal process is running or not.
    fn is_running(&self) -> bool;

    /// Put the pty into UTF-8 mode on systems which support it.
    fn set_utf8_mode(&mut self, on: bool);

    /// Set the timeout of pty.
    fn set_timeout(&mut self, timeout: u32);

    /// Sends data to the process currently controlling the teletype.
    ///
    /// @param data: the data to send.
    fn send_data(&mut self, data: String);
}

pub trait PtySignals: ActionExt {
    signals! {
        PtySignals: 

        /// Emitted when a new block of data was received from the teletype.
        ///
        /// @param data [`String`] the data received.
        receive_data();

        /// Emitted when terminal process was finished. <br>
        ///
        /// @param exit_code [`i32`] <br>
        /// @param exit_status [`ExitStatus`](tmui::tlib::namespace::ExitStatus)
        finished();
    }
}

#[derive(Default)]
pub struct PtyReceivePool {
    #[cfg(target_os = "windows")]
    ptys: Arc<Mutex<HashMap<u16, (Arc<Mutex<PTY>>, Signal)>>>,

    #[cfg(not(target_os = "windows"))]
    ptys: Arc<Mutex<HashMap<u16, (Arc<Mutex<Fork>>, Signal)>>>,
}

#[inline]
pub fn pty_receive_pool() -> &'static mut PtyReceivePool {
    static mut PTY_RECEIVE_POOL: Lazy<PtyReceivePool> = Lazy::new(|| PtyReceivePool::default());
    unsafe { &mut PTY_RECEIVE_POOL }
}

/// Make sure PtyReceivePool::start() only execute once.
static ONCE: Once = Once::new();
impl PtyReceivePool {
    pub fn start(&self) {
        ONCE.call_once(|| {
            let ptys = self.ptys.clone();

            thread::spawn(move || loop {

                #[cfg(target_os = "windows")]
                ptys.lock().unwrap().iter().for_each(|(_, (pty, signal))| {
                    if let Ok(data) = pty.lock().unwrap().read(u32::MAX, false) {
                        if data.len() > 0 {
                            emit!(signal.clone(), data.to_str().unwrap())
                        }
                    }
                });

                #[cfg(not(target_os = "windows"))]
                {
                    let ptys = ptys.clone();
                    tasync!(move {
                        ptys.lock().unwrap().iter().for_each(|(_, (pty, signal))| {
                            if let Some(mut master) = pty.lock().unwrap().is_parent().ok() {
                                let mut data = String::new();
                                // Is that blocked read?
                                master.read_to_string(&mut data).unwrap();
                                if data.len() > 0 {
                                    emit!(signal.clone(), data);
                                }
                            }
                        });
                        ()
                    });
                }

                std::thread::park_timeout(Duration::from_millis(10));
            });
        });
    }

    #[inline]
    #[cfg(target_os = "windows")]
    pub fn add_pty(&mut self, id: u16, pty: Arc<Mutex<PTY>>, signal: Signal) {
        self.ptys.lock().unwrap().insert(id, (pty, signal));
    }

    #[inline]
    #[cfg(not(target_os = "windows"))]
    pub fn add_pty(&mut self, id: u16, pty: Arc<Mutex<Fork>>, signal: Signal) {
        self.ptys.lock().unwrap().insert(id, (pty, signal));
    }

    #[inline]
    pub fn remove_pty(&mut self, id: u16) {
        self.ptys.lock().unwrap().remove(&id);
    }
}

#[macro_export]
macro_rules! pty_ref {
    ( $obj:ident ) => {
        $obj.pty.as_ref().unwrap()
    };
}
#[macro_export]
macro_rules! pty_mut {
    ( $obj:ident ) => {
        $obj.pty.as_mut().unwrap()
    };
}
