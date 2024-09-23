use std::collections::BTreeMap;

use nmos_schema::is_04::{v1_0_x, v1_3_x};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    resource::{Device, Format},
    version::{is_04::V1_0, is_04::V1_3, APIVersion},
};

use super::{ResourceCore, ResourceCoreBuilder};

macro_rules! registration_request {
    ($value:expr, $version:ident) => {
        json!($version::RegistrationapiResourcePostRequest::Variant4(
            $version::RegistrationapiResourcePostRequestHealthVariant4 {
                data: Some($value),
                type_: Some(String::from("source")),
            }
        ))
    };
}

#[must_use]
pub struct SourceBuilder {
    core: ResourceCoreBuilder,
    format: Format,
    device_id: Uuid,
    parents: Vec<Uuid>,
}

impl SourceBuilder {
    pub fn new<S: Into<String>>(label: S, device: &Device, format: Format) -> Self {
        SourceBuilder {
            core: ResourceCoreBuilder::new(label),
            format,
            device_id: device.core.id,
            parents: Vec::new(),
        }
    }

    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.core = self.core.description(description);
        self
    }

    #[must_use]
    pub fn build(self) -> Source {
        Source {
            core: self.core.build(),
            format: self.format,
            device_id: self.device_id,
            parents: self.parents,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Source {
    pub core: ResourceCore,
    pub format: Format,
    pub device_id: Uuid,
    pub parents: Vec<Uuid>,
}

impl Source {
    pub fn builder<S: Into<String>>(label: S, device: &Device, format: Format) -> SourceBuilder {
        SourceBuilder::new(label, device, format)
    }

    #[must_use]
    pub fn to_json(&self, api: &APIVersion) -> SourceJson {
        match *api {
            V1_0 => {
                let tags = self.core.tags_json();
                let parents = self.parents.iter().map(ToString::to_string).collect();

                SourceJson::V1_0(v1_0_x::Source {
                    id: self.core.id.to_string(),
                    version: self.core.version.to_string(),
                    label: self.core.label.clone(),
                    description: self.core.description.clone(),
                    format: self.format.to_string(),
                    caps: BTreeMap::default(),
                    tags,
                    device_id: self.device_id.to_string(),
                    parents,
                })
            }
            V1_3 => SourceJson::V1_3((*self).clone().into()),
            _ => panic!("Unsupported API"),
        }
    }

    pub fn registration_request(&self, api: &APIVersion) -> serde_json::Value {
        match self.to_json(api) {
            SourceJson::V1_0(json) => registration_request!(json, v1_0_x),
            SourceJson::V1_3(json) => registration_request!(json, v1_3_x),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum SourceJson {
    V1_0(v1_0_x::Source),
    V1_3(v1_3_x::Source),
}

impl Into<v1_3_x::Source> for Source {
    fn into(self) -> v1_3_x::Source {
        let id = self.core.id.to_string();
        let label = self.core.label.clone();
        let description = self.core.description.clone();
        let tags = self.core.tags_json();
        let parents = self.parents.iter().map(ToString::to_string).collect();
        let device_id = self.device_id.to_string();
        let clock_name: Option<std::string::String> = None;
        let version = self.core.version.to_string();
        let format = self.format.to_string();
        let caps = BTreeMap::default();

        match self.format {
            Format::Data | Format::Video => v1_3_x::Source::Variant0(v1_3_x::SourceGeneric {
                clock_name,
                id,
                version,
                label,
                description,
                format,
                tags,
                caps,
                parents,
                device_id,
                grain_rate: None,
            }),
            Format::Audio => v1_3_x::Source::Variant1(v1_3_x::SourceAudio {
                clock_name,
                id,
                version,
                label,
                description,
                format,
                tags,
                caps,
                parents,
                device_id,
                channels: vec![
                    v1_3_x::SourceAudioItemChannels {
                        label: "L".into(),
                        symbol: None,
                    },
                    v1_3_x::SourceAudioItemChannels {
                        label: "R".into(),
                        symbol: None,
                    },
                ],
                grain_rate: None,
            }),
        }
    }
}
