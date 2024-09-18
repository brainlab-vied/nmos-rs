use std::{collections::BTreeMap, vec};

use nmos_schema::is_04;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    resource::{Device, Format, Transport},
    version::{is_04::V1_0, is_04::V1_3, APIVersion},
};

use super::{ResourceCore, ResourceCoreBuilder};

#[must_use]
pub struct ReceiverBuilder {
    core: ResourceCoreBuilder,
    format: Format,
    device_id: Uuid,
    transport: Transport,
    subscription: Option<Uuid>,
}

impl ReceiverBuilder {
    pub fn new<S: Into<String>>(
        label: S,
        device: &Device,
        format: Format,
        transport: Transport,
    ) -> Self {
        ReceiverBuilder {
            core: ResourceCoreBuilder::new(label),
            format,
            device_id: device.core.id,
            transport,
            subscription: None,
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

    #[must_use]
    pub fn build(self) -> Receiver {
        Receiver {
            core: self.core.build(),
            format: self.format,
            device_id: self.device_id,
            transport: self.transport,
            subscription: self.subscription,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Receiver {
    pub core: ResourceCore,
    pub format: Format,
    pub device_id: Uuid,
    pub transport: Transport,
    pub subscription: Option<Uuid>,
}

impl Receiver {
    pub fn builder<S: Into<String>>(
        label: S,
        device: &Device,
        format: Format,
        transport: Transport,
    ) -> ReceiverBuilder {
        ReceiverBuilder::new(label, device, format, transport)
    }

    #[must_use]
    pub fn to_json(&self, api: &APIVersion) -> ReceiverJson {
        match *api {
            V1_0 => {
                let subscription = is_04::v1_0_x::ReceiverSubscription {
                    sender_id: self.subscription.map(|s| s.to_string()),
                };

                ReceiverJson::V1_0(is_04::v1_0_x::Receiver {
                    id: self.core.id.to_string(),
                    version: self.core.version.to_string(),
                    label: self.core.label.clone(),
                    description: self.core.description.clone(),
                    format: self.format.to_string(),
                    caps: BTreeMap::default(),
                    tags: self.core.tags_json(),
                    device_id: self.device_id.to_string(),
                    transport: self.transport.to_string(),
                    subscription,
                })
            }
            V1_3 => ReceiverJson::V1_3((*self).clone().into()),
            _ => panic!("Unsupported API"),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ReceiverJson {
    V1_0(is_04::v1_0_x::Receiver),
    V1_3(is_04::v1_3_x::Receiver),
}

impl Into<nmos_schema::is_04::v1_3_x::Receiver> for Receiver {
    fn into(self) -> nmos_schema::is_04::v1_3_x::Receiver {
        let tags = self.core.tags_json();
        let interface_bindings: Vec<std::string::String> = vec![];
        let id = self.core.id.to_string();
        let version = self.core.version.to_string();
        let label = self.core.label.clone();
        let description = self.core.description.clone();
        let format = self.format.to_string();
        let device_id = self.device_id.to_string();

        match self.format {
            Format::Video => {
                let subscription = is_04::v1_3_x::ReceiverVideoSubscription {
                    active: false,
                    sender_id: self.subscription.map(|s| s.to_string()),
                };
                let caps = is_04::v1_3_x::ReceiverVideoCaps { media_types: None };
                // TODO: figure out which one is correct
                let transport = nmos_schema::is_04::v1_3_x::ReceiverVideoTransport::Variant0(
                    self.transport.to_string().into(),
                );
                is_04::v1_3_x::Receiver::Variant0(is_04::v1_3_x::ReceiverVideo {
                    interface_bindings,
                    id,
                    version,
                    tags,
                    label,
                    description,
                    format,
                    device_id,
                    caps,
                    subscription,
                    transport,
                })
            }
            Format::Audio => {
                let subscription = is_04::v1_3_x::ReceiverAudioSubscription {
                    active: false,
                    sender_id: self.subscription.map(|s| s.to_string()),
                };
                let caps = is_04::v1_3_x::ReceiverAudioCaps {
                    // TODO: implement caps
                    media_types: None,
                };
                let transport = nmos_schema::is_04::v1_3_x::ReceiverAudioTransport::Variant0(
                    self.transport.to_string().into(),
                );
                is_04::v1_3_x::Receiver::Variant1(is_04::v1_3_x::ReceiverAudio {
                    interface_bindings,
                    id,
                    version,
                    tags,
                    label,
                    description,
                    format,
                    device_id,
                    caps,
                    subscription,
                    transport,
                })
            }
            Format::Data => {
                let subscription = is_04::v1_3_x::ReceiverDataSubscription {
                    active: false,
                    sender_id: self.subscription.map(|s| s.to_string()),
                };
                let transport = nmos_schema::is_04::v1_3_x::ReceiverDataTransport::Variant0(
                    self.transport.to_string().into(),
                );
                let caps = is_04::v1_3_x::ReceiverDataCaps {
                    // TODO: implement caps
                    media_types: None,
                    event_types: None,
                };
                is_04::v1_3_x::Receiver::Variant2(is_04::v1_3_x::ReceiverData {
                    interface_bindings,
                    id,
                    version,
                    tags,
                    label,
                    description,
                    format,
                    device_id,
                    subscription,
                    transport,
                    caps,
                })
            }
        }
    }
}
