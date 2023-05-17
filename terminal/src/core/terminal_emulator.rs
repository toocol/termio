#![allow(dead_code)]
use tmui::{prelude::*, tlib::object::ObjectSubclass};
/*
                                            |- Session/Tab/Emulation |- ScreenWidow/Screens
          - SessionGroup/TerminalView/TabBar|
          |                                 |- Session/Tab/Emulation |- ScreenWidow/Screens
 Terminal-|
          |                                 |- Session/Tab/Emulation |- ScreenWidow/Screens
          - SessionGroup/TerminalView/TabBar|
                                            |- Session/Tab/Emulation |- ScreenWidow/Screens
*/

/// The terminal's main widget. Responsible for all layouts management of `TerminalView`,
/// forward the client's input information from the ipc channel.
#[extends(Widget, Layout(VBox))]
#[derive(Default, Childrenable)]
pub struct TerminalEmulator {

}
impl ObjectSubclass for TerminalEmulator {
    const NAME: &'static str = "TerminalEmulator";
}
impl ObjectImpl for TerminalEmulator {
    fn initialize(&mut self) {
        
    }
}
impl WidgetImpl for TerminalEmulator {}