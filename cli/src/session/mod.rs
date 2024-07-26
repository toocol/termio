use lazy_static::lazy_static;
use parking_lot::Mutex;
use tmui::tlib::utils::{SnowflakeGuidGenerator, Timestamp};
use std::collections::HashMap;
use crate::constant::ProtocolType;

pub type SessionId = u64;

#[derive(Debug, Clone, Copy)]
pub enum Session {
    Ssh(SessionProps),
    Mosh(SessionProps),
    Telnet(SessionProps),
    Rsh(SessionProps),
    LocalShell(SessionProps),
}

#[derive(Debug, Clone, Copy)]
pub struct SessionProps {
    id: u64,
    establish_time: Timestamp,
}

pub trait SessionExt {
    /// Create a new session.
    fn create(protocol: ProtocolType) -> SessionId;

    /// Get the copy of session.
    fn get(id: SessionId) -> Option<Session> {
        SESSION_MAP.lock().get(&id).map(|s| *s)
    }

    /// Remove the session.
    fn remove(id: SessionId) {
        SESSION_MAP.lock().remove(&id);
    }

    /// Get the property of session.
    fn props(&self) -> &SessionProps;

    /// Get the global unique id.
    fn id(&self) -> SessionId;

    /// Get the connection protocol of the session.
    fn protocol(&self) -> ProtocolType;

    /// Get the timestamp of session establishment.
    fn establish_time(&self) -> Timestamp;
}

impl SessionExt for Session {
    fn create(protocol: ProtocolType) -> SessionId {
        let props = SessionProps {
            id: gen_id(),
            establish_time: Timestamp::now(),
        };

        let session = match protocol {
            ProtocolType::Ssh => Self::Ssh(props),
            ProtocolType::Mosh => Self::Mosh(props),
            ProtocolType::Telnet => Self::Telnet(props),
            ProtocolType::Rsh => Self::Rsh(props),
            ProtocolType::LocalShell => Self::LocalShell(props),
        };

        let id = session.id();

        SESSION_MAP.lock().insert(id, session);

        id
    }

    #[inline]
    fn props(&self) -> &SessionProps {
        match self {
            Self::Ssh(props) => props,
            Self::Mosh(props) => props,
            Self::Telnet(props) => props,
            Self::Rsh(props) => props,
            Self::LocalShell(props) => props,
        }
    }

    #[inline]
    fn id(&self) -> SessionId {
        self.props().id
    }

    #[inline]
    fn protocol(&self) -> ProtocolType {
        match self {
            Self::Ssh(_) => ProtocolType::Ssh,
            Self::Mosh(_) => ProtocolType::Mosh,
            Self::Telnet(_) => ProtocolType::Telnet,
            Self::Rsh(_) => ProtocolType::Rsh,
            Self::LocalShell(_) => ProtocolType::LocalShell,
        }
    }

    #[inline]
    fn establish_time(&self) -> Timestamp {
        self.props().establish_time
    }
}

/// Generate global unique u64 id.
fn gen_id() -> SessionId {
    SnowflakeGuidGenerator::next_id().expect("`SnowflakeGuidGenerator` generate id failed.")
}

lazy_static! {
    pub static ref SESSION_MAP: Mutex<HashMap<u64, Session>> = Mutex::new(HashMap::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constant::ProtocolType;

    #[test]
    fn test_sessions() {
        let session = Session::create(ProtocolType::Ssh);
        assert_eq!(session, Session::get(session).unwrap().id())
    }
}
