use std::fmt;

use nmos_schema::is_04::{v1_0_x, v1_3_x};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    resource::Node,
    version::{is_04::V1_0, is_04::V1_3, APIVersion},
};

use super::{Registerable, ResourceCore, ResourceCoreBuilder};

macro_rules! registration_request {
    ($value:expr, $version:ident) => {
        json!($version::RegistrationapiResourcePostRequest::Variant1(
            $version::RegistrationapiResourcePostRequestHealthVariant1 {
                data: Some($value),
                type_: Some(String::from("device")),
            }
        ))
    };
}

#[derive(Debug, Clone, Copy)]
pub enum DeviceType {
    Generic,
    Pipeline,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::Generic => write!(f, "urn:x-nmos:device:generic"),
            DeviceType::Pipeline => write!(f, "urn:x-nmos:device:pipeline"),
        }
    }
}

#[must_use]
pub struct DeviceBuilder {
    core: ResourceCoreBuilder,
    type_: DeviceType,
    node_id: Uuid,
}

impl DeviceBuilder {
    pub fn new<S: Into<String>>(label: S, node: &Node, device_type: DeviceType) -> Self {
        DeviceBuilder {
            core: ResourceCoreBuilder::new(label),
            type_: device_type,
            node_id: node.core.id,
        }
    }

    #[must_use]
    pub fn build(self) -> Device {
        Device {
            core: self.core.build(),
            type_: self.type_,
            node_id: self.node_id,
            senders: Vec::new(),
            receivers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Device {
    pub core: ResourceCore,
    pub type_: DeviceType,
    pub node_id: Uuid,
    pub senders: Vec<Uuid>,
    pub receivers: Vec<Uuid>,
}

impl Device {
    pub fn builder<S: Into<String>>(
        label: S,
        node: &Node,
        device_type: DeviceType,
    ) -> DeviceBuilder {
        DeviceBuilder::new(label, node, device_type)
    }

    #[must_use]
    pub fn to_json(&self, api: &APIVersion) -> DeviceJson {
        match *api {
            V1_0 => DeviceJson::V1_0(self.clone().into()),
            V1_3 => DeviceJson::V1_3(self.clone().into()),
            _ => panic!("Unsupported API"),
        }
    }
}

impl Registerable for Device {
    fn registry_path(&self) -> String {
        format!("devices/{}", self.core.id)
    }

    fn registration_request(&self, api: &APIVersion) -> serde_json::Value {
        match self.to_json(api) {
            DeviceJson::V1_0(json) => registration_request!(json, v1_0_x),
            DeviceJson::V1_3(json) => registration_request!(json, v1_3_x),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum DeviceJson {
    V1_0(v1_0_x::Device),
    V1_3(v1_3_x::Device),
}

impl Into<v1_0_x::Device> for Device {
    fn into(self) -> v1_0_x::Device {
        let senders = self.senders.iter().map(ToString::to_string).collect();
        let receivers = self.receivers.iter().map(ToString::to_string).collect();
        v1_0_x::Device {
            id: self.core.id.to_string(),
            version: self.core.version.to_string(),
            label: self.core.label.clone(),
            type_: self.type_.to_string(),
            node_id: self.node_id.to_string(),
            senders,
            receivers,
        }
    }
}

impl Into<v1_3_x::Device> for Device {
    fn into(self) -> v1_3_x::Device {
        let senders = self.senders.iter().map(ToString::to_string).collect();
        let receivers = self.receivers.iter().map(ToString::to_string).collect();
        v1_3_x::Device {
            id: self.core.id.to_string(),
            version: self.core.version.to_string(),
            label: self.core.label.clone(),
            type_: v1_3_x::DeviceType::Variant0(self.type_.to_string().into()),
            node_id: self.node_id.to_string(),
            senders,
            receivers,
            tags: self.core.tags_json(),
            description: "".to_string(),
            controls: vec![],
        }
    }
}
