use std::collections::BTreeMap;

use nmos_schema::is_04::{v1_0_x, v1_3_x};
use serde::Serialize;
use serde_json::json;

use crate::version::{is_04::V1_0, is_04::V1_3, APIVersion};

use super::{Registerable, ResourceCore, ResourceCoreBuilder};

macro_rules! registration_request {
    ($value:expr, $version:ident) => {
        json!($version::RegistrationapiResourcePostRequest::Variant0(
            $version::RegistrationapiResourcePostRequestHealthVariant0 {
                data: Some($value),
                type_: Some(String::from("node")),
            }
        ))
    };
}

#[derive(Debug)]
pub struct NodeService {
    pub href: String,
    pub type_: String,
}

#[must_use]
pub struct NodeBuilder {
    core: ResourceCoreBuilder,
    href: String,
    hostname: Option<String>,
    services: Vec<NodeService>,
}

impl NodeBuilder {
    pub fn new<S: Into<String>>(label: S, href: S) -> Self {
        NodeBuilder {
            core: ResourceCoreBuilder::new(label),
            href: href.into(),
            hostname: None,
            services: Vec::new(),
        }
    }

    pub fn with_service(mut self, service: NodeService) -> Self {
        self.services.push(service);
        self
    }

    #[must_use]
    pub fn build(self) -> Node {
        Node {
            core: self.core.build(),
            href: self.href.parse().unwrap(),
            hostname: self.hostname,
            services: self.services,
        }
    }
}

#[derive(Debug)]
pub struct Node {
    pub core: ResourceCore,
    pub href: url::Url,
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
                    .map(|service| v1_0_x::NodeItemServices {
                        href: service.href.clone(),
                        type_: service.type_.clone(),
                    })
                    .collect();

                NodeJson::V1_0(v1_0_x::Node {
                    id: self.core.id.to_string(),
                    version: self.core.version.to_string(),
                    label: self.core.label.clone(),
                    href: self.href.to_string(),
                    hostname: self.hostname.clone(),
                    caps: BTreeMap::default(),
                    services,
                })
            }
            V1_3 => {
                let services = self
                    .services
                    .iter()
                    .map(|service| v1_3_x::NodeItemServices {
                        authorization: None,
                        href: service.href.clone(),
                        type_: service.type_.clone(),
                    })
                    .collect();

                NodeJson::V1_3(v1_3_x::Node {
                    description: self.core.description.to_string(),
                    id: self.core.id.to_string(),
                    version: self.core.version.to_string(),
                    label: self.core.label.clone(),
                    href: self.href.to_string(),
                    hostname: self.hostname.clone(),
                    caps: BTreeMap::default(),
                    clocks: vec![serde_json::json!({"name": "clk0", "ref_type": "internal"})],
                    interfaces: vec![],
                    api: v1_3_x::NodeApi {
                        versions: vec![V1_3.to_string()],
                        endpoints: vec![v1_3_x::NodeApiItemEndpoints {
                            host: serde_json::json!(self.href.host_str().unwrap()),
                            port: self.href.port().unwrap() as i64,
                            protocol: "http".into(),
                            authorization: Some(false),
                        }],
                    },
                    tags: self.core.tags_json(),
                    services,
                })
            }
            _ => panic!("Unsupported API"),
        }
    }
}

impl Registerable for Node {
    fn registry_path(&self) -> String {
        format!("/nodes/{}", self.core.id)
    }

    fn registration_request(&self, api: &APIVersion) -> serde_json::Value {
        match self.to_json(api) {
            NodeJson::V1_0(json) => registration_request!(json, v1_0_x),
            NodeJson::V1_3(json) => registration_request!(json, v1_3_x),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum NodeJson {
    V1_0(v1_0_x::Node),
    V1_3(v1_3_x::Node),
}
