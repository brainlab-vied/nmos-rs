use std::collections::BTreeMap;

use nmos_schema::is_04;
use serde::Serialize;
use uuid::Uuid;

use crate::version::{is_04::V1_0, APIVersion};

use super::{ResourceCore, ResourceCoreBuilder};

#[derive(Debug)]
pub struct NodeService {
    pub href: String,
    pub type_: String,
}

#[must_use]
pub struct NodeBuilder {
    id: Option<Uuid>,
    core: ResourceCoreBuilder,
    href: String,
    hostname: Option<String>,
    services: Vec<NodeService>,
}

impl NodeBuilder {
    pub fn new<S: Into<String>>(label: S, href: S) -> Self {
        NodeBuilder {
            id: None,
            core: ResourceCoreBuilder::new(label),
            href: href.into(),
            hostname: None,
            services: Vec::new(),
        }
    }

    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_service(mut self, service: NodeService) -> Self {
        self.services.push(service);
        self
    }

    #[must_use]
    pub fn build(self) -> Node {
        let mut core = self.core.build();

        if let Some(id) = self.id{
            core.id = id;
        }

        Node {
            core,
            href: self.href,
            hostname: self.hostname,
            services: self.services,
        }
    }
}

#[derive(Debug)]
pub struct Node {
    pub core: ResourceCore,
    pub href: String,
    pub hostname: Option<String>,
    pub services: Vec<NodeService>,
}

impl Node {
    pub fn builder<S: Into<String>>(label: S, href: S) -> NodeBuilder {
        NodeBuilder::new(label, href)
    }

    #[must_use]
    pub fn to_json(&self, api: &APIVersion) -> NodeJson {
        match *api {
            V1_0 => {
                let services = self
                    .services
                    .iter()
                    .map(|service| is_04::v1_0_x::NodeItemServices {
                        href: service.href.clone(),
                        type_: service.type_.clone(),
                    })
                    .collect();

                NodeJson::V1_0(is_04::v1_0_x::Node {
                    id: self.core.id.to_string(),
                    version: self.core.version.to_string(),
                    label: self.core.label.clone(),
                    href: self.href.clone(),
                    hostname: self.hostname.clone(),
                    caps: BTreeMap::default(),
                    services,
                })
            }
            _ => panic!("Unsupported API"),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum NodeJson {
    V1_0(is_04::v1_0_x::Node),
}
