use libs::{err, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tmui::{tlib::utils::Timestamp, views::tree_view::tree_node::TreeNode};

use crate::{auth::credential::Credential, persistence::Persistence};

use super::{session_grp::SessionGroup, ROOT_SESSION};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionGrpPers {
    name: String,
    expand: bool,
    children: HashMap<String, SessionGrpPers>,
    timestamp: u64,
}

impl SessionGrpPers {
    #[inline]
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            expand: true,
            children: HashMap::new(),
            timestamp: Timestamp::now().as_millis()
        }
    }

    #[inline]
    pub fn to_view_obj(&self) -> SessionGroup {
        SessionGroup::new_with_time(self.name.as_str(), self.timestamp)
    }

    #[inline]
    pub fn children(&self) -> &HashMap<String, SessionGrpPers> {
        &self.children
    }

    #[inline]
    pub fn set_expand(&mut self, expand: bool) {
        self.expand = expand
    }

    #[inline]
    pub fn is_expand(&self) -> bool {
        self.expand
    }

    #[inline]
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn build_node(
        &self,
        grp_credential_map: &HashMap<String, Vec<Credential>>,
        mut root: &mut TreeNode,
    ) {
        if self.name != ROOT_SESSION {
            root = root.add_node(&self.to_view_obj()).unwrap();
        };

        for (_, child) in self.children.iter() {
            child.build_node(grp_credential_map, root)
        }

        if let Some(cs) = grp_credential_map.get(&self.name) {
            for c in cs.iter() {
                root.add_node(c);
            }
        }

        if !self.expand {
            root.shuffle_expand();
        }
    }

    pub fn add_group(&mut self, parent: &str, group: &str) -> bool {
        if self.name == parent {
            let grp = SessionGrpPers::new(group);
            self.children.insert(group.to_string(), grp);
            return true
        }

        for (_, c) in self.children.iter_mut() {
            if c.add_group(parent, group) {
                return true
            }
        }

        return false
    }
}

impl Persistence for SessionGrpPers {
    const EXTENSION: &'static str = "";

    #[inline]
    fn name() -> &'static str {
        "grps"
    }

    #[inline]
    fn parse(data: &str) -> Result<Self, Error> {
        err!(serde_json::from_str(data))
    }
}
