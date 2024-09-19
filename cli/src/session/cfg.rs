use crate::{auth::credential::Credential, persistence::Persistence};
use libs::{err, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionCfg {
    credential: Credential,
    group: String,
}

impl SessionCfg {
    #[inline]
    pub fn new(credential: Credential, group: String) -> Self {
        Self {
            credential,
            group,
        }
    }

    #[inline]
    pub fn credential(&self) -> &Credential {
        &self.credential
    }

    #[inline]
    pub fn group(&self) -> &String {
        &self.group
    }
}

impl Persistence for SessionCfg {
    const EXTENSION: &'static str = "tsc";

    #[inline]
    fn name() -> &'static str {
        "*"
    }

    #[inline]
    fn parse(data: &str) -> Result<Self, Error> {
        err!(serde_json::from_str(data))
    }

    #[inline]
    fn sep_name(&self) -> &str {
        self.credential.shown_name()
    }
}
