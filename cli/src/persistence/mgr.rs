use crate::{
    auth::credential::Credential,
    session::{cfg::SessionCfg, session_grp_pers::SessionGrpPers},
};
use lazy_static::lazy_static;
use libs::prelude::*;
use parking_lot::{Mutex, RawMutex};
use std::collections::HashMap;

use super::Persistence;

lazy_static! {
    static ref INSTANCE: Mutex<Box<PersistenceMgr>> =
        Mutex::new(Box::new(PersistenceMgr::default()));
}

#[derive(Default)]
pub struct PersistenceMgr {
    sessions: Vec<SessionCfg>,
    root_group: Option<SessionGrpPers>,
    grp_credential_map: HashMap<String, Vec<Credential>>,
}

impl PersistenceMgr {
    #[inline]
    pub fn add_session(session: SessionCfg) {
        let mut guard = INSTANCE.lock();
        guard.sessions.push(session);
    }

    pub fn set_root_group(root_group: SessionGrpPers) {
        let mut guard = INSTANCE.lock();
        guard.root_group = Some(root_group);

        if let Err(e) = err!(guard.root_group.as_ref().unwrap().persistence()) {
            e.handle()
        }
    }

    #[inline]
    pub fn empty_pesistence() -> bool {
        let guard = INSTANCE.lock();
        guard.root_group.is_none()
    }

    #[inline]
    pub fn with_guard<F, R>(f: F) -> R
    where
        F: Fn(parking_lot::lock_api::MutexGuard<RawMutex, Box<PersistenceMgr>>) -> R,
    {
        let guard = INSTANCE.lock();
        f(guard)
    }

    #[inline]
    pub fn load_data() {
        let sessions = SessionCfg::loads();
        let root_group = SessionGrpPers::loads();

        let mut guard = INSTANCE.lock();
        match sessions {
            Ok(sessions) => {
                for session in sessions.iter() {
                    guard.grp_credential_map
                        .entry(session.group().name().to_string())
                        .or_default()
                        .push(session.credential().clone());
                }
                guard.sessions = sessions;
            }
            Err(e) => e.handle(),
        }

        match root_group {
            Ok(mut root_group) => {
                guard.root_group = root_group.pop();
            }
            Err(e) => e.handle(),
        }
    }

    #[inline]
    pub fn root_group(&self) -> &SessionGrpPers {
        self.root_group.as_ref().unwrap()
    }

    #[inline]
    pub fn grp_credential_map(&self) -> &HashMap<String, Vec<Credential>> {
        &self.grp_credential_map
    }
}
