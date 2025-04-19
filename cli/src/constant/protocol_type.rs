use serde::{Deserialize, Serialize};

#[repr(i32)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProtocolType {
    #[default]
    Cmd = 1,
    PowerShell,
    Ssh,
    Mosh,
    Telnet,
    Rsh,
    Custom,
}

impl ProtocolType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProtocolType::Cmd => "Cmd",
            ProtocolType::PowerShell => "PowerShell",
            ProtocolType::Ssh => "SSH",
            ProtocolType::Mosh => "Mosh",
            ProtocolType::Telnet => "Telnet",
            ProtocolType::Rsh => "Rsh",
            ProtocolType::Custom => "Custom",
        }
    }
}
