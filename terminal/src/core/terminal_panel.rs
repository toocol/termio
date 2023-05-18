use super::session::Session;
use derivative::Derivative;
use tmui::{prelude::*, tlib::object::ObjectSubclass};

#[repr(u8)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum SplitState {
    #[default]
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 4,
    Four = 8,
}

#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ViewLocation {
    #[default]
    OneCenter,

    // Split screen for two
    TwoLeft,
    TwoRight,

    // Split screen for three
    ThreeLeft,
    ThreeRightTop,
    ThreeRightBottom,

    // Split screen for four
    FourLeftTop,
    FourLeftBottom,
    FourRightTop,
    FourRightBottom,
}

/// TerminalPanel was built to manage the terminal view, it holds all the terminal session,
/// and each session has a binded TerminalView.
#[extends(Widget)]
#[derive(Derivative)]
#[derivative(Default)]
pub struct TerminalPanel {
    /// All the terminal sessions.
    sessions: Vec<Session>,
}
impl ObjectSubclass for TerminalPanel {
    const NAME: &'static str = "TerminalPanel";
}
impl ObjectImpl for TerminalPanel {}
impl WidgetImpl for TerminalPanel {}
