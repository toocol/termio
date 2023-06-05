#![allow(dead_code)]
#[cfg(target_os = "windows")]
pub mod con_pty;
#[cfg(not(target_os = "windows"))]
pub mod posix_pty;

use std::path::PathBuf;
use tmui::{prelude::*, tlib::signals};

#[repr(u8)]
#[derive(Default)]
pub enum ProtocolType {
    #[default]
    None = 0,
    Ssh,
    Mosh,
    Telnet,
    Rsh,
    LocalShell
}

pub trait Pty: PtySignals {
    /// Start the terminal process.
    /// 
    /// Return true if the process was started successfully or non-zero otherwise.
    fn start(&mut self, program: &str, arguments: Vec<&str>, enviroments: Vec<&str>, protocol_type: ProtocolType) -> bool;

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
    fn set_window_size(&mut self, lines: i32, cols: i32);

    /// Returns the size of the window used by this teletype.
    fn window_size(&self) -> Size;

    /// Set the working directory of terminal process.
    fn set_working_directory(&mut self, directory: PathBuf);

    /// Get the status that terminal process is running or not.
    fn is_running(&self) -> bool;

    /// Put the pty into UTF-8 mode on systems which support it.
    fn set_utf8_mode(&mut self, on: bool);

    /// Suspend or resume processing of data from the standard output of the terminal process.
    /// 
    /// @param lock: If true, processing of output is suspended, otherwise processing is resumed.
    fn lock_pty(&mut self, lock: bool);

    /// Sends data to the process currently controlling the teletype.
    /// 
    /// @param data: the data to send.
    fn send_data(&mut self, data: &str);

    /// Send the heart beat message to the terminal process to maintain connection.
    fn heart_beat(&mut self) {
        self.send_data("");
    }
}

pub trait PtySignals: ActionExt {
    signals! {
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