#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataSender {
    LocalDisplay,
    Pty,
}
