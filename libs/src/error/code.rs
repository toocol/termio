use std::fmt::Debug;

pub const SYS_IO_ERROR: u32 = 0x00000001;
pub const SYS_SERDE_JSON_ERROR: u32 = 0x00000002;
pub const SYS_FMT_ERROR: u32 = 0x00000003;
pub const SYS_SERDE_ERROR: u32 = 0x00000004;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    IoError = SYS_IO_ERROR,
    SerdeJsonError = SYS_SERDE_JSON_ERROR,
    FmtError = SYS_FMT_ERROR,
    SerdeError = SYS_SERDE_ERROR,
}

impl Debug for ErrorCode {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("0x{:08X}", *self as u32))
    }
}
