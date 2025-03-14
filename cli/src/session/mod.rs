pub mod cfg;
pub mod session_grp;
pub mod session_grp_pers;

use crate::{auth::credential::Credential, constant::ProtocolType};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::collections::HashMap;
use tmui::tlib::utils::{SnowflakeGuidGenerator, Timestamp};

pub type SessionPropsId = u64;

pub const ROOT_SESSION: &'static str = "Sessions";

lazy_static! {
    pub static ref SESSION_MAP: Mutex<HashMap<u64, SessionProps>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Clone)]
pub struct SessionProps {
    id: u64,
    establish_time: Timestamp,
    credential: Credential,
}

pub trait SessionExt {
    /// Create a new session.
    fn create(credential: Credential) -> SessionPropsId;

    /// Get the copy of session.
    fn get(id: SessionPropsId) -> Option<SessionProps> {
        SESSION_MAP.lock().get(&id).map(|s| s.clone())
    }

    /// Remove the session.
    fn remove(id: SessionPropsId) {
        SESSION_MAP.lock().remove(&id);
    }

    fn command(id: SessionPropsId) -> Option<String> {
        let guard = SESSION_MAP.lock();
        let session = guard.get(&id)?;
        let credential = &session.credential;
        let connect_info = credential.connect_info();

        match session.protocol() {
            ProtocolType::Ssh => Some(format!(
                "ssh {}@{} -p {}",
                connect_info.user(),
                connect_info.host(),
                connect_info.port()
            )),
            _ => None,
        }
    }

    /// Get the global unique id.
    fn id(&self) -> SessionPropsId;

    /// Get the connection protocol of the session.
    fn protocol(&self) -> ProtocolType;

    /// Get the timestamp of session establishment.
    fn establish_time(&self) -> Timestamp;
}

impl SessionExt for SessionProps {
    fn create(credential: Credential) -> SessionPropsId {
        let props = SessionProps {
            id: gen_id(),
            establish_time: Timestamp::now(),
            credential: credential,
        };

        let id = props.id();

        SESSION_MAP.lock().insert(id, props);

        id
    }

    #[inline]
    fn id(&self) -> SessionPropsId {
        self.id
    }

    #[inline]
    fn protocol(&self) -> ProtocolType {
        self.credential.protocol_type()
    }

    #[inline]
    fn establish_time(&self) -> Timestamp {
        self.establish_time
    }
}

/// Generate global unique u64 id.
fn gen_id() -> SessionPropsId {
    SnowflakeGuidGenerator::next_id().expect("`SnowflakeGuidGenerator` generate id failed.")
}
