use std::vec;

use nmos_schema::is_04::{v1_0_x, v1_3_x};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    resource::{Device, Flow, Transport},
    version::{
        is_04::{V1_0, V1_3},
        APIVersion,
    },
};

use super::{Registerable, ResourceCore, ResourceCoreBuilder};

macro_rules! registration_request {
    ($value:expr, $version:ident) => {
        json!($version::RegistrationapiResourcePostRequest::Variant2(
            $version::RegistrationapiResourcePostRequestHealthVariant2 {
                data: Some($value),
                type_: Some(String::from("sender")),
            }
        ))
    };
}

#[must_use]
pub struct SenderBuilder {
    core: ResourceCoreBuilder,
    flow_id: Uuid,
    transport: Transport,
    device_id: Uuid,
    manifest_href: Option<String>,
}

impl SenderBuilder {
    pub fn new<S: Into<String>>(
        label: S,
        device: &Device,
        flow: &Flow,
        transport: Transport,
    ) -> Self {
        SenderBuilder {
            core: ResourceCoreBuilder::new(label),
            flow_id: flow.core.id,
            transport,
            device_id: device.core.id,
            manifest_href: None,
        }
    }

    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.core = self.core.description(description);
        self
    }

    pub fn tag<S, V>(mut self, key: S, values: V) -> Self
    where
        S: Into<String>,
        V: IntoIterator<Item = S>,
    {
        self.core = self.core.tag(key, values);
        self
    }

    pub fn manifest<S: Into<String>>(mut self, manifest: S) -> Self {
        // TODO: Store manifest and generate href
        self.manifest_href = Some(manifest.into());
        self
    }

    #[must_use]
    pub fn build(self) -> Sender {
        Sender {
            core: self.core.build(),
            flow_id: self.flow_id,
            transport: self.transport,
            device_id: self.device_id,
            manifest_href: self.manifest_href.unwrap_or_default(),
        }
    }
}

#[derive(Debug)]
pub struct Sender {
    pub core: ResourceCore,
    pub flow_id: Uuid,
    pub transport: Transport,
    pub device_id: Uuid,
    pub manifest_href: String,
}

impl Sender {
    pub fn builder<S: Into<String>>(
        label: S,
        device: &Device,
        flow: &Flow,
        transport: Transport,
    ) -> SenderBuilder {
        SenderBuilder::new(label, device, flow, transport)
    }

    #[must_use]
    pub fn to_json(&self, api: &APIVersion) -> SenderJson {
        match *api {
            V1_0 => SenderJson::V1_0(v1_0_x::Sender {
                id: self.core.id.to_string(),
                version: self.core.version.to_string(),
                label: self.core.label.clone(),
                description: self.core.description.clone(),
                flow_id: self.flow_id.to_string(),
                transport: self.transport.to_string(),
                tags: (!self.core.tags.is_empty()).then_some(self.core.tags_json()),
                device_id: self.device_id.to_string(),
                manifest_href: self.manifest_href.clone(),
            }),
            V1_3 => {
                SenderJson::V1_3(v1_3_x::Sender {
                    interface_bindings: vec![],
                    // TODO: implement caps
                    caps: None,
                    id: self.core.id.to_string(),
                    version: self.core.version.to_string(),
                    label: self.core.label.clone(),
                    description: self.core.description.clone(),
                    flow_id: Some(self.flow_id.to_string()),
                    tags: self.core.tags_json(),
                    device_id: self.device_id.to_string(),
                    manifest_href: None,
                    subscription: v1_3_x::SenderSubscription {
                        active: false,
                        receiver_id: None,
                    },
                    transport: v1_3_x::SenderTransport::Variant0(self.transport.to_string().into()),
                })
            }
            _ => panic!("Unsupported API"),
        }
    }
}

impl Registerable for Sender {
    fn registry_path(&self) -> String {
        format!("senders/{}", self.core.id)
    }

    fn registration_request(&self, api: &APIVersion) -> serde_json::Value {
        match self.to_json(api) {
            SenderJson::V1_0(json) => registration_request!(json, v1_0_x),
            SenderJson::V1_3(json) => registration_request!(json, v1_3_x),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum SenderJson {
    V1_0(v1_0_x::Sender),
    V1_3(v1_3_x::Sender),
}
