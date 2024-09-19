pub mod paths;
use serde::{Deserialize, Serialize};

#[repr(i32)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolType {
    LocalShell = 1,
    Ssh,
    Mosh,
    Telnet,
    Rsh,
}

impl ProtocolType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProtocolType::Ssh => "SSH",
            ProtocolType::Mosh => "Mosh",
            ProtocolType::Telnet => "Telnet",
            ProtocolType::Rsh => "Rsh",
            ProtocolType::LocalShell => "Local Shell"
        }
    }
}
