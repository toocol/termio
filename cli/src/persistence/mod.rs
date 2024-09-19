pub mod mgr;

use libs::{
    err,
    error::Error,
};
use serde::Serialize;
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

use crate::constant::paths::PERSISTENCE_PATH;

pub trait Persistence: Sized + Serialize {
    const EXTENSION: &'static str;

    fn name() -> &'static str;

    fn parse(data: &str) -> Result<Self, Error>;

    fn path() -> PathBuf {
        let mut path = PathBuf::from(PERSISTENCE_PATH);
        if Self::name() != "*" {
            path = path.join(Self::name());
            path = path.with_extension(Self::EXTENSION);
        }
        path
    }

    fn loads() -> Result<Vec<Self>, Error> {
        if Self::name() == "*" {
            let mut res = vec![];
            for entry in err!(fs::read_dir(Self::path()))? {
                let entry = err!(entry)?;
                let path = entry.path();

                if path.is_file()
                    && path.extension().and_then(|ext| ext.to_str()) == Some(Self::EXTENSION)
                {
                    let mut file = err!(File::open(&path))?;
                    let mut buf = String::new();
                    err!(file.read_to_string(&mut buf))?;
                    if buf.is_empty() {
                        continue;
                    }
                    res.push(err!(Self::parse(&buf))?);
                }
            }
            Ok(res)
        } else {
            let mut file = err!(File::open(Self::path()))?;
            let mut buf = String::new();
            err!(file.read_to_string(&mut buf))?;
            if buf.is_empty() {
                return Ok(vec![])
            }
            Ok(vec![err!(Self::parse(&buf))?])
        }
    }

    #[inline]
    fn sep_name(&self) -> &str {
        ""
    }

    #[inline]
    fn sep_path(&self) -> PathBuf {
        let mut path = PathBuf::from(PERSISTENCE_PATH);

        let sep_name = self.sep_name();
        if sep_name.is_empty() {
            path = path.join(Self::name());
        } else {
            path = path.join(sep_name);
        }

        path = path.with_extension(Self::EXTENSION);
        path
    }

    fn persistence(&self) -> Result<(), Error> {
        let parse = err!(serde_json::to_string(self))?;
        let path = self.sep_path();
        if let Some(parent) = path.parent() {
            err!(fs::create_dir_all(parent))?;
        }

        let mut file = err!(File::create(path))?;
        err!(file.write_all(parse.as_bytes()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};
    use crate::{
        auth::{connect_info::ConnectInfo, credential::Credential}, constant::{paths::PERSISTENCE_PATH, ProtocolType}, persistence::Persistence, prelude::*, session::{
            cfg::SessionCfg, session_grp::SessionGroup, session_grp_pers::SessionGrpPers,
        }
    };

    #[test]
    fn test_persistence() {
        let session = SessionCfg::new(
            Credential::new(
                None,
                ProtocolType::Ssh,
                ConnectInfo::LocalShell("~".to_string())
            ),
            "group".to_string(),
        );

        let session_grp = SessionGrpPers::new("group");

        session.persistence().unwrap();
        session_grp.persistence().unwrap();

        let mut sessions = SessionCfg::loads().unwrap();
        let mut grps = SessionGrpPers::loads().unwrap();
        assert!(sessions.len() == 1);
        assert!(grps.len() == 1);
        assert_eq!(session, sessions.pop().unwrap());
        assert_eq!(session_grp, grps.pop().unwrap());

        let path = PathBuf::from(PERSISTENCE_PATH);
        err!(fs::remove_dir_all(path)).unwrap();
    }
}
