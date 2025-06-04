#![allow(dead_code)]
#[cfg(target_os = "windows")]
pub mod con_pty;
pub mod ffi;
#[cfg(not(target_os = "windows"))]
pub mod posix_pty;

use cli::session::SessionPropsId;
#[cfg(not(target_os = "windows"))]
use pty::prelude::Fork;
use std::path::PathBuf;
use tlib::namespace::ExitStatus;
use tmui::{prelude::*, tlib::signals};

impl AsMutPtr for dyn Pty {}

pub trait Pty: PtySignals {
    /// Start the terminal process.
    ///
    /// Return true if the process was started successfully or non-zero otherwise.
    fn start(
        &mut self,
        id: SessionPropsId,
        program: &str,
        arguments: Vec<&str>,
        enviroment: Vec<&str>,
    ) -> bool;

    /// Close the terminal process.
    fn close(&mut self);

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

    /// Read data from the process.
    fn read_data(&mut self) -> Vec<u8>;

    /// Execute when app exited.
    fn on_window_closed(&mut self);
}

pub trait PtySignals: ActionExt {
    signals! {
        PtySignals:
        /// Emitted when terminal process was finished. <br>
        ///
        /// @param session id <br>
        /// @param exit_status [`ExitStatus`](tmui::tlib::namespace::ExitStatus)
        finished(SessionPropsId, ExitStatus);
    }
}
