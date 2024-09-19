use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ConnectInfo {
    /// (Path)
    LocalShell(String),

    /// (Host, User, Password, Port)
    Ssh(String, String, String, u32),

    Mosh,
    Telnet,
    Rsh,
}

impl ConnectInfo {
    #[inline]
    pub fn path(&self) -> &str {
        match self {
            Self::LocalShell(path) => path.as_str(),
            _ => "",
        }
    }

    #[inline]
    pub fn host(&self) -> &str {
        match self {
            Self::Ssh(host, _, _, _) => host.as_str(),
            _ => "",
        }
    }

    #[inline]
    pub fn user(&self) -> &str {
        match self {
            Self::Ssh(_, user, _, _) => user.as_str(),
            _ => "",
        }
    }

    #[inline]
    pub fn password(&self) -> &str {
        match self {
            Self::Ssh(_, _, password, _) => password.as_str(),
            _ => "",
        }
    }

    #[inline]
    pub fn port(&self) -> u32 {
        match self {
            Self::Ssh(_, _, _, port) => *port,
            _ => 0,
        }
    }
}
