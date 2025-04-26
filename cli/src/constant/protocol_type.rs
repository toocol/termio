use serde::{Deserialize, Serialize};
use tmui::prelude::*;
use tmui::tlib::{implements_enum_value, namespace::AsNumeric, values::FromValue};

const NUM_CMD: u8 = 0;
const NUM_POWER_SHELL: u8 = 1;
const NUM_SSH: u8 = 2;
const NUM_MOSH: u8 = 3;
const NUM_TELNET: u8 = 4;
const NUM_RSH: u8 = 5;
const NUM_CUSTOM: u8 = 6;

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProtocolType {
    #[default]
    Cmd = 0,
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
            ProtocolType::Cmd => "Command Prompt",
            ProtocolType::PowerShell => "PowerShell",
            ProtocolType::Ssh => "SSH",
            ProtocolType::Mosh => "Mosh",
            ProtocolType::Telnet => "Telnet",
            ProtocolType::Rsh => "Rsh",
            ProtocolType::Custom => "Custom",
        }
    }
}

impl AsNumeric<u8> for ProtocolType {
    #[inline]
    fn as_numeric(&self) -> u8 {
        *self as u8
    }
}
impl From<u8> for ProtocolType {
    #[inline]
    fn from(value: u8) -> Self {
        match value {
            NUM_CMD => ProtocolType::Cmd,
            NUM_POWER_SHELL => ProtocolType::PowerShell,
            NUM_SSH => ProtocolType::Ssh,
            NUM_MOSH => ProtocolType::Mosh,
            NUM_TELNET => ProtocolType::Telnet,
            NUM_RSH => ProtocolType::Rsh,
            NUM_CUSTOM => ProtocolType::Custom,
            _ => unreachable!(),
        }
    }
}
implements_enum_value!(ProtocolType, u8);

#[cfg(test)]
mod tests {
    use super::ProtocolType;
    use tmui::prelude::ToValue;

    #[test]
    fn test_protocol_type_value() {
        let val = ProtocolType::PowerShell.to_value();
        assert_eq!(val.get::<ProtocolType>(), ProtocolType::PowerShell);
    }
}
